use ringbuf::traits::{Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasapi::*;

pub struct CaptureHandle {
    pub running: Arc<AtomicBool>,
    pub thread: Option<std::thread::JoinHandle<()>>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CaptureHandle {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_mic_capture(
    device_id: &str,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    start_capture(device_id.to_owned(), false)
}

pub fn start_loopback_capture(
    device_id: &str,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    start_capture(device_id.to_owned(), true)
}

/// Probe the device to get format info before spawning the thread.
/// Returns (sample_rate, channels, bits_per_sample, block_align).
fn probe_format(device_id: &str, loopback: bool) -> anyhow::Result<(u32, u16, u16, u32)> {
    let direction = if loopback {
        Direction::Render
    } else {
        Direction::Capture
    };
    let device = crate::audio::devices::get_device_by_id(device_id, &direction)?;
    let audio_client = device.get_iaudioclient()?;
    let format = audio_client.get_mixformat()?;
    Ok((
        format.get_samplespersec(),
        format.get_nchannels(),
        format.get_bitspersample(),
        format.get_blockalign(),
    ))
}

fn start_capture(
    device_id: String,
    loopback: bool,
) -> anyhow::Result<(CaptureHandle, ringbuf::HeapCons<f32>)> {
    let (sample_rate, channels, bits_per_sample, block_align) =
        probe_format(&device_id, loopback)?;

    log::info!(
        "Capture format: {}Hz, {}ch, {}bit, loopback={}",
        sample_rate,
        channels,
        bits_per_sample,
        loopback
    );

    // Ring buffer: ~200ms of f32 samples
    let rb_size = (sample_rate as usize) * (channels as usize) / 5;
    let rb = HeapRb::<f32>::new(rb_size);
    let (producer, consumer) = rb.split();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let thread = std::thread::Builder::new()
        .name(if loopback {
            "capture-loopback".into()
        } else {
            "capture-mic".into()
        })
        .spawn(move || {
            // Initialize COM for this thread
            let _ = initialize_mta();
            capture_thread(
                device_id,
                loopback,
                producer,
                running_clone,
                bits_per_sample,
                block_align,
            );
        })?;

    let handle = CaptureHandle {
        running,
        thread: Some(thread),
        sample_rate,
        channels,
    };

    Ok((handle, consumer))
}

fn capture_thread(
    device_id: String,
    loopback: bool,
    mut producer: ringbuf::HeapProd<f32>,
    running: Arc<AtomicBool>,
    bits_per_sample: u16,
    block_align: u32,
) {
    let direction = if loopback {
        Direction::Render
    } else {
        Direction::Capture
    };

    let device = match crate::audio::devices::get_device_by_id(&device_id, &direction) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Capture thread: failed to get device: {}", e);
            return;
        }
    };

    let mut audio_client = match device.get_iaudioclient() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Capture thread: failed to get audio client: {}", e);
            return;
        }
    };

    let format = match audio_client.get_mixformat() {
        Ok(f) => f,
        Err(e) => {
            log::error!("Capture thread: failed to get mix format: {}", e);
            return;
        }
    };

    let (_, min_period) = match audio_client.get_device_period() {
        Ok(p) => p,
        Err(e) => {
            log::error!("Capture thread: failed to get periods: {}", e);
            return;
        }
    };

    let stream_mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: min_period,
    };

    if let Err(e) = audio_client.initialize_client(&format, &Direction::Capture, &stream_mode) {
        log::error!("Capture thread: failed to initialize client: {}", e);
        return;
    }

    let capture_client = match audio_client.get_audiocaptureclient() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Capture thread: failed to get capture client: {}", e);
            return;
        }
    };

    let event = match audio_client.set_get_eventhandle() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Capture thread: failed to get event handle: {}", e);
            return;
        }
    };

    capture_loop(
        audio_client,
        capture_client,
        event,
        &mut producer,
        running,
        bits_per_sample,
        block_align,
    );
}

fn capture_loop(
    audio_client: AudioClient,
    capture_client: AudioCaptureClient,
    event: Handle,
    producer: &mut ringbuf::HeapProd<f32>,
    running: Arc<AtomicBool>,
    bits_per_sample: u16,
    block_align: u32,
) {
    if let Err(e) = audio_client.start_stream() {
        log::error!("Failed to start capture stream: {}", e);
        return;
    }

    // Allocate a read buffer sized for a reasonable number of frames
    let buf_frames = 4096usize;
    let mut read_buf = vec![0u8; buf_frames * block_align as usize];

    while running.load(Ordering::Relaxed) {
        if event.wait_for_event(100).is_err() {
            continue;
        }

        // Drain all available packets
        loop {
            match capture_client.get_next_packet_size() {
                Ok(Some(nbr_frames)) if nbr_frames > 0 => {
                    let needed = nbr_frames as usize * block_align as usize;
                    if read_buf.len() < needed {
                        read_buf.resize(needed, 0u8);
                    }
                    match capture_client.read_from_device(&mut read_buf[..needed]) {
                        Ok((frames_read, _flags)) if frames_read > 0 => {
                            let byte_count = frames_read as usize * block_align as usize;
                            let samples = bytes_to_f32(&read_buf[..byte_count], bits_per_sample);
                            let pushed = producer.push_slice(&samples);
                            if pushed < samples.len() {
                                log::warn!(
                                    "Ring buffer overflow: dropped {} samples",
                                    samples.len() - pushed
                                );
                            }
                        }
                        Ok(_) => break,
                        Err(e) => {
                            log::error!("Capture read error: {}", e);
                            break;
                        }
                    }
                }
                Ok(Some(_)) | Ok(None) => break,
                Err(e) => {
                    log::error!("Capture frame count error: {}", e);
                    break;
                }
            }
        }
    }

    let _ = audio_client.stop_stream();
    log::info!("Capture thread stopped");
}

fn bytes_to_f32(data: &[u8], bits_per_sample: u16) -> Vec<f32> {
    match bits_per_sample {
        16 => data
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / i16::MAX as f32)
            .collect(),
        32 => data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
        24 => data
            .chunks_exact(3)
            .map(|c| {
                let sample = i32::from_le_bytes([c[0], c[1], c[2], 0]) >> 8;
                sample as f32 / (1 << 23) as f32
            })
            .collect(),
        _ => {
            log::warn!("Unsupported bit depth: {}", bits_per_sample);
            vec![]
        }
    }
}

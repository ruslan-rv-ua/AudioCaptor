use crate::audio::types::{AudioCommand, OutputMode};
use crate::audio::writer::OutputWriter;
use crossbeam_channel::Receiver;
use ringbuf::traits::{Consumer, Observer};
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MixerConfig {
    pub mic_consumer: Option<ringbuf::HeapCons<f32>>,
    pub loopback_consumer: Option<ringbuf::HeapCons<f32>>,
    pub mic_sample_rate: u32,
    pub loopback_sample_rate: u32,
    pub mic_channels: u16,
    pub loopback_channels: u16,
    pub target_sample_rate: u32,
    pub target_channels: u16,
    pub output_mode: OutputMode,
    pub mic_volume: f32,
    pub loopback_volume: f32,
    pub writer: Box<dyn OutputWriter>,
    /// Secondary writer for parallel modes (MixPlusMicrophone, MixPlusLoopback).
    pub secondary_writer: Option<Box<dyn OutputWriter>>,
    pub command_rx: Receiver<AudioCommand>,
    /// Discard this many milliseconds of audio at startup to avoid capturing
    /// residual notification sound from the WASAPI render pipeline.
    pub warmup_discard_ms: u64,
}

pub struct MixerHandle {
    pub running: Arc<AtomicBool>,
    pub thread: Option<std::thread::JoinHandle<()>>,
}

impl MixerHandle {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MixerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start the mixer thread. Returns a handle to control it.
pub fn start_mixer(config: MixerConfig) -> anyhow::Result<MixerHandle> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let thread = std::thread::Builder::new()
        .name("mixer".into())
        .spawn(move || {
            mixer_loop(config, running_clone);
        })?;

    Ok(MixerHandle {
        running,
        thread: Some(thread),
    })
}

/// Create a rubato resampler if source rate differs from target rate.
/// `chunk_frames` is the number of frames per channel (not total interleaved samples).
fn maybe_resampler(
    source_rate: u32,
    target_rate: u32,
    channels: usize,
    chunk_frames: usize,
) -> Option<FftFixedIn<f32>> {
    if source_rate == target_rate {
        return None;
    }
    FftFixedIn::new(source_rate as usize, target_rate as usize, chunk_frames, 1, channels).ok()
}

/// Convert mono samples to stereo by duplicating each sample.
fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for &s in mono {
        stereo.push(s);
        stereo.push(s);
    }
    stereo
}

/// Convert stereo samples to mono by averaging each pair.
fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    stereo.chunks(2).map(|c| (c[0] + c[1]) * 0.5).collect()
}

/// Deinterleave interleaved samples into per-channel vectors (for rubato).
fn deinterleave(interleaved: &[f32], channels: usize) -> Vec<Vec<f32>> {
    let mut result: Vec<Vec<f32>> = (0..channels).map(|_| Vec::new()).collect();
    for (i, &sample) in interleaved.iter().enumerate() {
        result[i % channels].push(sample);
    }
    result
}

/// Interleave per-channel vectors back into a single buffer.
fn interleave(channels: &[Vec<f32>]) -> Vec<f32> {
    if channels.is_empty() {
        return vec![];
    }
    let frames = channels[0].len();
    let num_ch = channels.len();
    let mut result = Vec::with_capacity(frames * num_ch);
    for frame in 0..frames {
        for ch in channels {
            result.push(ch[frame]);
        }
    }
    result
}

/// Resample exactly `chunk_frames` frames from the front of `buf` (interleaved).
/// Returns the resampled interleaved output.
fn resample_chunk(
    buf: &[f32],
    channels: usize,
    resampler: &mut FftFixedIn<f32>,
) -> Vec<f32> {
    let deint = deinterleave(buf, channels);
    match resampler.process(&deint, None) {
        Ok(resampled) => interleave(&resampled),
        Err(e) => {
            log::error!("Resample error: {}", e);
            buf.to_vec()
        }
    }
}

/// Convert audio between channel counts if needed.
fn convert_channels(samples: Vec<f32>, source_ch: usize, target_ch: usize) -> Vec<f32> {
    if source_ch == target_ch {
        samples
    } else if source_ch == 1 && target_ch == 2 {
        mono_to_stereo(&samples)
    } else if source_ch == 2 && target_ch == 1 {
        stereo_to_mono(&samples)
    } else {
        log::warn!("Unsupported channel conversion: {source_ch} → {target_ch}");
        samples
    }
}

fn mix_samples(
    mic: &[f32], loopback: &[f32],
    mic_vol: f32, loop_vol: f32, channels: usize,
) -> Vec<f32> {
    let mic_frames = mic.len() / channels;
    let loop_frames = loopback.len() / channels;
    let len_frames = mic_frames.max(loop_frames);
    let mut out = Vec::with_capacity(len_frames * channels);
    for frame in 0..len_frames {
        for ch in 0..channels {
            let idx = frame * channels + ch;
            let mic_s = mic.get(idx).copied().unwrap_or(0.0) * mic_vol;
            let loop_s = loopback.get(idx).copied().unwrap_or(0.0) * loop_vol;
            out.push((mic_s + loop_s).clamp(-1.0, 1.0));
        }
    }
    out
}

fn mixer_loop(mut config: MixerConfig, running: Arc<AtomicBool>) {
    let mut paused = false;

    // After Resume, keep discarding until the loopback ring buffer has been quiet
    // for 20 ms (pipeline drained) or until a 500 ms safety deadline is reached.
    let mut pending_resume = false;
    let mut resume_quiet_since: Option<std::time::Instant> = None;
    let mut resume_deadline: Option<std::time::Instant> = None;

    // Discard the first warmup_discard_ms of audio at startup so that any
    // residual notification sound left in the WASAPI render pipeline is flushed
    // before we write to the WAV file.
    let warmup_end = std::time::Instant::now()
        + std::time::Duration::from_millis(config.warmup_discard_ms);

    let mut mic_volume = config.mic_volume;
    let mut loopback_volume = config.loopback_volume;
    let target_channels = config.target_channels as usize;
    let mic_channels = config.mic_channels as usize;
    let loopback_channels = config.loopback_channels as usize;

    // Read buffer — sized for ~10ms at max source sample rate
    let max_sr = config.mic_sample_rate.max(config.loopback_sample_rate);
    let chunk_frames = (max_sr as usize) / 100; // 10ms worth of frames

    // Fix #3: use actual channel counts for buffer sizing
    let mic_chunk_size = chunk_frames * mic_channels;
    let loopback_chunk_size = chunk_frames * loopback_channels;
    let mut mic_buf = vec![0.0f32; mic_chunk_size];
    let mut loopback_buf = vec![0.0f32; loopback_chunk_size];

    // Fix #1: pass chunk_frames (frames per channel) to maybe_resampler
    let mut mic_resampler = maybe_resampler(
        config.mic_sample_rate,
        config.target_sample_rate,
        mic_channels,
        chunk_frames,
    );
    let mut loopback_resampler = maybe_resampler(
        config.loopback_sample_rate,
        config.target_sample_rate,
        loopback_channels,
        chunk_frames,
    );

    // Fix #2: per-source staging buffers to accumulate samples before resampling
    let mut mic_staging: Vec<f32> = Vec::new();
    let mut loopback_staging: Vec<f32> = Vec::new();

    log::info!(
        "Mixer thread started, mode={:?}, target={}Hz/{}ch, mic={}Hz/{}ch, loop={}Hz/{}ch",
        config.output_mode,
        config.target_sample_rate, target_channels,
        config.mic_sample_rate, config.mic_channels,
        config.loopback_sample_rate, config.loopback_channels,
    );

    while running.load(Ordering::Relaxed) {
        // Check for commands (non-blocking)
        while let Ok(cmd) = config.command_rx.try_recv() {
            match cmd {
                AudioCommand::Pause => {
                    paused = true;
                    log::info!("Mixer paused");
                }
                AudioCommand::Resume => {
                    // Don't write immediately — wait for the loopback ring buffer
                    // to drain so residual notification-sound data is discarded.
                    pending_resume = true;
                    resume_quiet_since = None;
                    resume_deadline = Some(
                        std::time::Instant::now()
                            + std::time::Duration::from_millis(500),
                    );
                    log::info!("Mixer: pending resume, waiting for pipeline drain");
                }
                AudioCommand::Stop => {
                    log::info!("Mixer received Stop");
                    running.store(false, Ordering::Relaxed);
                    break;
                }
                AudioCommand::SetMicVolume(v) => mic_volume = v,
                AudioCommand::SetLoopbackVolume(v) => loopback_volume = v,
            }
        }

        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Read available samples from ring buffers into staging
        let mic_read = if let Some(ref mut consumer) = config.mic_consumer {
            let n = consumer.pop_slice(&mut mic_buf);
            if n > 0 {
                mic_staging.extend_from_slice(&mic_buf[..n]);
            }
            n
        } else {
            0
        };

        let loopback_read = if let Some(ref mut consumer) = config.loopback_consumer {
            let n = consumer.pop_slice(&mut loopback_buf);
            if n > 0 {
                loopback_staging.extend_from_slice(&loopback_buf[..n]);
            }
            n
        } else {
            0
        };

        // Advance pending-resume drain detection using the loopback read count.
        // We track how long the loopback ring buffer has been continuously empty;
        // once it stays empty for 20 ms we know the notification sound has been
        // fully captured and discarded, and it is safe to start writing.
        if pending_resume {
            let loopback_empty = config
                .loopback_consumer
                .as_ref()
                .map(|c| c.occupied_len() == 0 && loopback_read == 0)
                .unwrap_or(true); // no loopback source → consider it drained

            if loopback_empty {
                resume_quiet_since.get_or_insert_with(std::time::Instant::now);
            } else {
                resume_quiet_since = None;
            }

            let drained = resume_quiet_since
                .map(|t| t.elapsed().as_millis() >= 20)
                .unwrap_or(false);
            let timed_out = resume_deadline
                .map(|d| std::time::Instant::now() >= d)
                .unwrap_or(false);

            if drained || timed_out {
                pending_resume = false;
                paused = false;
                resume_quiet_since = None;
                resume_deadline = None;
                mic_staging.clear();
                loopback_staging.clear();
                log::info!(
                    "Mixer resumed ({})",
                    if drained { "pipeline drained" } else { "timeout" }
                );
            }
        }

        // If no data available from either source, sleep briefly to avoid busy-waiting
        if mic_read == 0 && loopback_read == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        let in_warmup = std::time::Instant::now() < warmup_end;
        if paused || in_warmup || pending_resume {
            // Drain staging buffers but don't write — prevents desync
            mic_staging.clear();
            loopback_staging.clear();
            continue;
        }

        // In Mix mode: only process when both sources have staged data.
        // Without this, single-source batches alternate and double the output duration.
        if matches!(config.output_mode, OutputMode::Mix | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback) {
            let mic_min = if mic_resampler.is_some() { chunk_frames * mic_channels } else { mic_channels };
            let loop_min = if loopback_resampler.is_some() { chunk_frames * loopback_channels } else { loopback_channels };
            if mic_staging.len() < mic_min || loopback_staging.len() < loop_min {
                continue;
            }
        }

        // In Mix mode: drain equal frame counts from both sources to keep them in sync.
        // In single-source modes: drain all available (usize::MAX as sentinel).
        let (mic_drain_frames, loop_drain_frames) = if matches!(config.output_mode, OutputMode::Mix | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback) {
            let mic_avail = mic_staging.len() / mic_channels;
            let loop_avail = loopback_staging.len() / loopback_channels;
            let min_avail = mic_avail.min(loop_avail);
            (
                if mic_resampler.is_some() { (min_avail / chunk_frames) * chunk_frames } else { min_avail },
                if loopback_resampler.is_some() { (min_avail / chunk_frames) * chunk_frames } else { min_avail },
            )
        } else {
            (usize::MAX, usize::MAX)
        };

        // Process mic — only resample when we have a full chunk; otherwise pass through directly
        let mic_processed = if mic_resampler.is_some() {
            let required = chunk_frames * mic_channels;
            let max_drain = mic_drain_frames.saturating_mul(mic_channels);
            let mut output = Vec::new();
            let mut drained = 0;
            while mic_staging.len() >= required && drained + required <= max_drain {
                let chunk: Vec<f32> = mic_staging.drain(..required).collect();
                let resampled = resample_chunk(&chunk, mic_channels, mic_resampler.as_mut().expect("mic_resampler is Some when mic sample rate differs from target rate"));
                output.extend(resampled);
                drained += required;
            }
            convert_channels(output, mic_channels, target_channels)
        } else if !mic_staging.is_empty() {
            // No resampler — drain up to mic_drain_frames frames
            let frames = mic_drain_frames.min(mic_staging.len() / mic_channels);
            let available: Vec<f32> = mic_staging.drain(..frames * mic_channels).collect();
            convert_channels(available, mic_channels, target_channels)
        } else {
            vec![]
        };

        // Process loopback — same pattern
        let loop_processed = if loopback_resampler.is_some() {
            let required = chunk_frames * loopback_channels;
            let max_drain = loop_drain_frames.saturating_mul(loopback_channels);
            let mut output = Vec::new();
            let mut drained = 0;
            while loopback_staging.len() >= required && drained + required <= max_drain {
                let chunk: Vec<f32> = loopback_staging.drain(..required).collect();
                let resampled = resample_chunk(&chunk, loopback_channels, loopback_resampler.as_mut().expect("loopback_resampler is Some when loopback sample rate differs from target rate"));
                output.extend(resampled);
                drained += required;
            }
            convert_channels(output, loopback_channels, target_channels)
        } else if !loopback_staging.is_empty() {
            // No resampler — drain up to loop_drain_frames frames
            let frames = loop_drain_frames.min(loopback_staging.len() / loopback_channels);
            let available: Vec<f32> = loopback_staging.drain(..frames * loopback_channels).collect();
            convert_channels(available, loopback_channels, target_channels)
        } else {
            vec![]
        };

        // Apply volume, mix, clip, and write based on mode
        let output = match config.output_mode {
            OutputMode::Microphone => {
                mic_processed.iter().map(|&s| (s * mic_volume).clamp(-1.0, 1.0)).collect::<Vec<_>>()
            }
            OutputMode::Loopback => {
                loop_processed.iter().map(|&s| (s * loopback_volume).clamp(-1.0, 1.0)).collect::<Vec<_>>()
            }
            OutputMode::Mix => {
                mix_samples(&mic_processed, &loop_processed, mic_volume, loopback_volume, target_channels)
            }
            OutputMode::MixPlusMicrophone => {
                let mix = mix_samples(&mic_processed, &loop_processed, mic_volume, loopback_volume, target_channels);
                let mic_only: Vec<f32> = mic_processed.iter()
                    .map(|&s| (s * mic_volume).clamp(-1.0, 1.0)).collect();
                if !mic_only.is_empty() {
                    if let Some(ref mut sw) = config.secondary_writer {
                        if let Err(e) = sw.write_samples(&mic_only) {
                            log::error!("Secondary WAV write error: {}", e);
                        }
                    }
                }
                mix
            }
            OutputMode::MixPlusLoopback => {
                let mix = mix_samples(&mic_processed, &loop_processed, mic_volume, loopback_volume, target_channels);
                let loop_only: Vec<f32> = loop_processed.iter()
                    .map(|&s| (s * loopback_volume).clamp(-1.0, 1.0)).collect();
                if !loop_only.is_empty() {
                    if let Some(ref mut sw) = config.secondary_writer {
                        if let Err(e) = sw.write_samples(&loop_only) {
                            log::error!("Secondary WAV write error: {}", e);
                        }
                    }
                }
                mix
            }
        };

        if !output.is_empty() {
            if let Err(e) = config.writer.write_samples(&output) {
                log::error!("WAV write error: {}", e);
                break;
            }
        }
    }

    // Finalize WAV file
    match config.writer.finalize() {
        Ok(path) => log::info!("Recording saved: {}", path.display()),
        Err(e) => log::error!("Failed to finalize WAV: {}", e),
    }

    // Finalize secondary WAV file (parallel modes)
    if let Some(sw) = config.secondary_writer {
        match sw.finalize() {
            Ok(path) => log::info!("Secondary recording saved: {}", path.display()),
            Err(e) => log::error!("Failed to finalize secondary WAV: {}", e),
        }
    }

    log::info!("Mixer thread stopped");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_to_stereo_duplicates_samples() {
        let mono = vec![0.5, -0.3, 1.0];
        let stereo = mono_to_stereo(&mono);
        assert_eq!(stereo, vec![0.5, 0.5, -0.3, -0.3, 1.0, 1.0]);
    }

    #[test]
    fn mono_to_stereo_empty() {
        assert_eq!(mono_to_stereo(&[]), Vec::<f32>::new());
    }

    #[test]
    fn stereo_to_mono_averages_pairs() {
        let stereo = vec![0.6, 0.4, -0.2, -0.8];
        let mono = stereo_to_mono(&stereo);
        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.5).abs() < 0.001);
        assert!((mono[1] + 0.5).abs() < 0.001);
    }

    #[test]
    fn stereo_to_mono_empty() {
        assert_eq!(stereo_to_mono(&[]), Vec::<f32>::new());
    }

    #[test]
    fn deinterleave_splits_channels() {
        let interleaved = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let channels = deinterleave(&interleaved, 2);
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0], vec![1.0, 3.0, 5.0]);
        assert_eq!(channels[1], vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn interleave_merges_channels() {
        let channels = vec![vec![1.0, 3.0, 5.0], vec![2.0, 4.0, 6.0]];
        let result = interleave(&channels);
        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn interleave_deinterleave_roundtrip() {
        let original = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let deint = deinterleave(&original, 2);
        let reint = interleave(&deint);
        assert_eq!(original, reint);
    }

    #[test]
    fn interleave_empty() {
        assert_eq!(interleave(&[]), Vec::<f32>::new());
    }

    #[test]
    fn convert_channels_same_passthrough() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = convert_channels(data.clone(), 2, 2);
        assert_eq!(result, data);
    }

    #[test]
    fn convert_channels_mono_to_stereo() {
        let mono = vec![0.5, -0.3];
        let stereo = convert_channels(mono, 1, 2);
        assert_eq!(stereo, vec![0.5, 0.5, -0.3, -0.3]);
    }

    #[test]
    fn convert_channels_stereo_to_mono() {
        let stereo = vec![0.6, 0.4, -0.2, -0.8];
        let mono = convert_channels(stereo, 2, 1);
        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.5).abs() < 0.001);
        assert!((mono[1] + 0.5).abs() < 0.001);
    }

    #[test]
    fn convert_channels_unsupported_passthrough() {
        let data = vec![1.0, 2.0, 3.0];
        let result = convert_channels(data.clone(), 3, 5);
        assert_eq!(result, data);
    }

    #[test]
    fn mix_samples_combines_with_volume() {
        let mic = vec![0.5, 0.5];
        let loopback = vec![0.3, 0.3];
        let result = mix_samples(&mic, &loopback, 1.0, 1.0, 2);
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.8).abs() < 0.001);
        assert!((result[1] - 0.8).abs() < 0.001);
    }

    #[test]
    fn mix_samples_clamps_output() {
        let mic = vec![0.9];
        let loopback = vec![0.9];
        let result = mix_samples(&mic, &loopback, 1.0, 1.0, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 1.0); // clamped
    }

    #[test]
    fn mix_samples_handles_unequal_lengths() {
        let mic = vec![0.5, 0.5, 0.5, 0.5]; // 2 frames stereo
        let loopback = vec![0.3, 0.3]; // 1 frame stereo
        let result = mix_samples(&mic, &loopback, 1.0, 1.0, 2);
        assert_eq!(result.len(), 4); // max frames * channels
    }
}

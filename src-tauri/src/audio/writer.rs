use anyhow::Context;
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// How often the WAV header is brought up to date while recording, in seconds
/// of recorded sound. hound fills in the sizes of the file and of its sound only
/// when the file is finalized, so a crash or a killed process would leave zeros
/// there, and many players would show no length or not open the file. Kept up
/// to date, a file cut short that way opens, and its header leaves out at most
/// this much of the sound written to it.
const HEADER_UPDATE_SECONDS: u64 = 1;

/// A recording file the recorder has finished writing.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedFile {
    pub path: PathBuf,
    /// Frames (one sample per channel) written to the file, counted by the
    /// recorder itself. The conversion to a compressed format reads this many
    /// frames and never trusts the WAV header: past 4 GB the header's length has
    /// wrapped around, and after a failed finalize it can be a second short.
    pub frames: u64,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Trait for writing audio samples to an output file.
/// A recording is always written as WAV; a compressed format is made from the
/// WAV after the recording stops (`crate::convert`).
pub trait OutputWriter: Send {
    fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()>;
    /// Finishes the file. An error is logged, not returned: the samples that were
    /// written are still described by the returned `RecordedFile`.
    fn finalize(self: Box<Self>) -> RecordedFile;
}

pub struct WavOutputWriter {
    writer: WavWriter<BufWriter<File>>,
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
    samples_written: u64,
    /// `samples_written` when the header was last brought up to date.
    samples_in_header: u64,
    /// Samples (`HEADER_UPDATE_SECONDS` of sound) between two header updates.
    samples_per_header_update: u64,
}

impl WavOutputWriter {
    /// Create a new WAV writer at the given path.
    /// Channels: 1 (mono) or 2 (stereo). Sample rate in Hz.
    pub fn new(path: PathBuf, sample_rate: u32, channels: u16) -> anyhow::Result<Self> {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer = WavWriter::create(&path, spec)?;
        log::info!("WAV writer created: {}", path.display());
        Ok(Self {
            writer,
            path,
            sample_rate,
            channels,
            samples_written: 0,
            samples_in_header: 0,
            samples_per_header_update: u64::from(sample_rate)
                * u64::from(channels)
                * HEADER_UPDATE_SECONDS,
        })
    }

    /// Writes the sizes of what the file holds so far into its header and hands
    /// the buffered samples to Windows, so that the file can be read up to here
    /// even if the app never finalizes it. The file stays open for more samples.
    ///
    /// A failure is a write error like any other: after it, the writer may no
    /// longer stand at the end of the file.
    fn update_header(&mut self) -> anyhow::Result<()> {
        self.samples_in_header = self.samples_written;
        self.writer
            .flush()
            .context("Failed to update the WAV header")?;
        Ok(())
    }
}

impl OutputWriter for WavOutputWriter {
    /// Write a slice of f32 samples (interleaved if stereo).
    /// Converts f32 [-1.0, 1.0] to i16.
    ///
    /// Runs on the mixer thread, which owns the writer; the capture threads only
    /// fill the ring buffers, so the header updates never hold them up.
    fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            self.writer.write_sample(int_sample)?;
            self.samples_written += 1;
        }
        // Only after whole frames: in the middle of one, hound's `flush` stops
        // before it seeks back to the end of the file, and the next samples would
        // overwrite the start of the sound.
        let whole_frames = self
            .samples_written
            .is_multiple_of(u64::from(self.channels.max(1)));
        if whole_frames
            && self.samples_written - self.samples_in_header >= self.samples_per_header_update
        {
            self.update_header()?;
        }
        Ok(())
    }

    /// Finalize the WAV file (writes header with correct sizes).
    fn finalize(self: Box<Self>) -> RecordedFile {
        let this = *self;
        match this.writer.finalize() {
            Ok(()) => log::info!("WAV file finalized: {}", this.path.display()),
            Err(e) => log::error!("Failed to finalize WAV {}: {e}", this.path.display()),
        }
        RecordedFile {
            path: this.path,
            frames: this.samples_written / u64::from(this.channels.max(1)),
            sample_rate: this.sample_rate,
            channels: this.channels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_wav_path() -> PathBuf {
        // The clock alone can give two tests that start together the same name.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join("audiocaptor_tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!(
            "test_{}_{}.wav",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    }

    #[test]
    fn write_and_finalize_creates_valid_wav() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 44100, 1).unwrap());

        // Write 1 second of silence
        let samples = vec![0.0f32; 44100];
        writer.write_samples(&samples).unwrap();
        let recorded = writer.finalize();
        assert_eq!(
            recorded,
            RecordedFile {
                path: path.clone(),
                frames: 44100,
                sample_rate: 44100,
                channels: 1,
            }
        );

        // Verify file is a valid WAV
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.spec().sample_rate, 44100);
        assert_eq!(reader.len(), 44100);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn write_samples_clips_to_range() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 44100, 1).unwrap());

        // Values beyond [-1, 1] should be clamped
        let samples = vec![2.0, -2.0, 0.5, -0.5];
        writer.write_samples(&samples).unwrap();
        writer.finalize();

        let mut reader = hound::WavReader::open(&path).unwrap();
        let read_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), 4);
        assert_eq!(read_samples[0], i16::MAX); // 2.0 clamped to 1.0
        assert_eq!(read_samples[1], -i16::MAX); // -2.0 clamped to -1.0

        std::fs::remove_file(&path).ok();
    }

    /// Samples of a signal in which no two neighbours are equal, so that a
    /// sample out of place shows.
    fn signal(samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| (i % 1000) as f32 / 1000.0 - 0.5)
            .collect()
    }

    /// The samples as the writer stores them.
    fn as_stored(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect()
    }

    /// How many frames the header of the file at `path` says it holds.
    fn frames_in_header(path: &std::path::Path) -> u32 {
        hound::WavReader::open(path).unwrap().duration()
    }

    #[test]
    fn unfinalized_file_holds_every_sample_up_to_the_last_header_update() {
        let path = temp_wav_path();
        let mut writer = WavOutputWriter::new(path.clone(), 8000, 2).unwrap();
        // Two and a half seconds of stereo at 8 kHz, in halves of a second.
        let sound = signal(5 * 8000);
        let mut halves = sound.chunks(8000);

        writer.write_samples(halves.next().unwrap()).unwrap();
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert_eq!(frames_in_header(&path), 8000, "after the first second");
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert_eq!(frames_in_header(&path), 8000, "half a second waits");
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert_eq!(frames_in_header(&path), 16000, "after the second second");
        writer.write_samples(halves.next().unwrap()).unwrap();

        // The app is killed: neither finalize nor drop runs (hound's drop would
        // write the header). The file stays open until the tests end.
        std::mem::forget(writer);

        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().channels, 2);
        assert_eq!(reader.spec().sample_rate, 8000);
        assert_eq!(reader.duration(), 16000);
        let read: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
        assert_eq!(read, as_stored(&sound[..32000]));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn finalized_file_is_the_same_as_without_header_updates() {
        let sound = signal(5 * 8000);
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 8000, 2).unwrap());
        for tenth_of_a_second in sound.chunks(1600) {
            writer.write_samples(tenth_of_a_second).unwrap();
        }
        writer.finalize();

        // The same samples through hound alone, as AudioCaptor wrote them before
        // it updated the header while recording.
        let plain_path = temp_wav_path();
        let spec = WavSpec {
            channels: 2,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut plain = WavWriter::create(&plain_path, spec).unwrap();
        for sample in as_stored(&sound) {
            plain.write_sample(sample).unwrap();
        }
        plain.finalize().unwrap();

        assert_eq!(
            std::fs::read(&path).unwrap(),
            std::fs::read(&plain_path).unwrap()
        );

        std::fs::remove_file(&path).ok();
        std::fs::remove_file(&plain_path).ok();
    }

    #[test]
    fn header_update_waits_for_a_whole_frame() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 8000, 2).unwrap());
        // A second and half a frame, then the other half of that frame.
        let sound = signal(16002);
        writer.write_samples(&sound[..16001]).unwrap();
        writer.write_samples(&sound[16001..]).unwrap();
        assert_eq!(frames_in_header(&path), 8001);
        writer.finalize();

        let mut reader = hound::WavReader::open(&path).unwrap();
        let read: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
        assert_eq!(read, as_stored(&sound));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn finalize_counts_frames_not_samples() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 48000, 2).unwrap());
        writer.write_samples(&[0.1f32; 960]).unwrap();
        writer.write_samples(&[0.2f32; 40]).unwrap();
        let recorded = writer.finalize();
        assert_eq!(recorded.frames, 500);
        assert_eq!(recorded.channels, 2);
        assert_eq!(recorded.sample_rate, 48000);
        assert_eq!(hound::WavReader::open(&path).unwrap().duration(), 500);

        std::fs::remove_file(&path).ok();
    }
}

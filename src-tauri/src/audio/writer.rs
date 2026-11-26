use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// A recording file the recorder has finished writing.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedFile {
    pub path: PathBuf,
    /// Frames (one sample per channel) written to the file, counted by the
    /// recorder itself. The conversion to a compressed format reads this many
    /// frames and never trusts the WAV header: past 4 GB the header's length has
    /// wrapped around, and after a failed finalize it is zero.
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
        })
    }
}

impl OutputWriter for WavOutputWriter {
    /// Write a slice of f32 samples (interleaved if stereo).
    /// Converts f32 [-1.0, 1.0] to i16.
    fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            self.writer.write_sample(int_sample)?;
            self.samples_written += 1;
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

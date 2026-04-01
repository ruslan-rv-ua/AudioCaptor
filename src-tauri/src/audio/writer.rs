use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Trait for writing audio samples to an output file.
/// Implementations handle format-specific encoding (WAV, MP3, etc.).
pub trait OutputWriter: Send {
    fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()>;
    fn finalize(self: Box<Self>) -> anyhow::Result<PathBuf>;
}

pub struct WavOutputWriter {
    writer: WavWriter<BufWriter<File>>,
    path: PathBuf,
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
        Ok(Self { writer, path })
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
        }
        Ok(())
    }

    /// Finalize the WAV file (writes header with correct sizes).
    fn finalize(self: Box<Self>) -> anyhow::Result<PathBuf> {
        self.writer.finalize()?;
        log::info!("WAV file finalized: {}", self.path.display());
        Ok(self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_wav_path() -> PathBuf {
        let dir = std::env::temp_dir().join("audiocaptor_tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!("test_{}.wav", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
    }

    #[test]
    fn write_and_finalize_creates_valid_wav() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> = Box::new(
            WavOutputWriter::new(path.clone(), 44100, 1).unwrap()
        );

        // Write 1 second of silence
        let samples = vec![0.0f32; 44100];
        writer.write_samples(&samples).unwrap();
        let result_path = writer.finalize().unwrap();
        assert_eq!(result_path, path);

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
        let mut writer: Box<dyn OutputWriter> = Box::new(
            WavOutputWriter::new(path.clone(), 44100, 1).unwrap()
        );

        // Values beyond [-1, 1] should be clamped
        let samples = vec![2.0, -2.0, 0.5, -0.5];
        writer.write_samples(&samples).unwrap();
        writer.finalize().unwrap();

        let mut reader = hound::WavReader::open(&path).unwrap();
        let read_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), 4);
        assert_eq!(read_samples[0], i16::MAX); // 2.0 clamped to 1.0
        assert_eq!(read_samples[1], -i16::MAX); // -2.0 clamped to -1.0

        std::fs::remove_file(&path).ok();
    }
}

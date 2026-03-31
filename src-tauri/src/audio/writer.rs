use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub struct AudioWriter {
    writer: WavWriter<BufWriter<File>>,
    path: PathBuf,
}

impl AudioWriter {
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

    /// Write a slice of f32 samples (interleaved if stereo).
    /// Converts f32 [-1.0, 1.0] to i16.
    pub fn write_samples(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            self.writer.write_sample(int_sample)?;
        }
        Ok(())
    }

    /// Finalize the WAV file (writes header with correct sizes).
    pub fn finalize(self) -> anyhow::Result<PathBuf> {
        self.writer.finalize()?;
        log::info!("WAV file finalized: {}", self.path.display());
        Ok(self.path)
    }
}

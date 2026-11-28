//! Reads the samples of a WAV recording made by `audio::writer`: a plain WAV,
//! or RF64 past 4 GB (about 6 h 12 min at 48 kHz stereo).
//!
//! How many frames to read is given by the caller, who counted them while
//! recording. The length in the header is ignored: after a crash or a failed
//! finalize it can be a second short (`audio::writer` brings it up to date
//! every second), and a WAV past 4 GB from AudioCaptor 0.4.1 or older is not
//! RF64 but has its 32-bit length wrapped around, while the samples are all in
//! the file. Such a WAV can still wait in the conversion queue.

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// 16-bit PCM samples of a WAV file, read front to back.
pub struct WavPcm {
    reader: BufReader<File>,
    sample_rate: u32,
    channels: u16,
    /// Frames not yet read.
    remaining: u64,
    bytes: Vec<u8>,
}

impl WavPcm {
    /// Opens `path` and stops at its first sample. `frames` is how many frames
    /// the file holds.
    pub fn open(path: &Path, frames: u64) -> io::Result<Self> {
        let mut reader = BufReader::with_capacity(1 << 16, File::open(path)?);
        let (channels, sample_rate) = read_header(&mut reader)?;
        Ok(Self {
            reader,
            sample_rate,
            channels,
            remaining: frames,
            bytes: Vec::new(),
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Frames not yet read.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Reads up to `max_frames` frames into `out` (emptied first) as interleaved
    /// samples and returns how many frames it read: 0 once all are read. Fails if
    /// the file ends before the frames it should hold.
    pub fn read(&mut self, max_frames: usize, out: &mut Vec<i16>) -> io::Result<usize> {
        out.clear();
        let frames = (max_frames as u64).min(self.remaining) as usize;
        if frames == 0 {
            return Ok(0);
        }
        self.bytes
            .resize(frames * usize::from(self.channels) * 2, 0);
        self.reader.read_exact(&mut self.bytes).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                invalid("the WAV file ends before the end of the recording")
            } else {
                e
            }
        })?;
        out.extend(
            self.bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&pair| i16::from_le_bytes(pair)),
        );
        self.remaining -= frames as u64;
        Ok(frames)
    }
}

fn invalid(reason: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason.into())
}

/// Reads up to the first sample of the `data` chunk and returns the channel
/// count and the sample rate from the `fmt ` chunk. Other chunks before it,
/// such as `JUNK` and the `ds64` chunk of RF64, are skipped.
fn read_header(reader: &mut impl Read) -> io::Result<(u16, u32)> {
    let mut riff = [0u8; 12];
    reader.read_exact(&mut riff)?;
    if !matches!(&riff[0..4], b"RIFF" | b"RF64") || &riff[8..12] != b"WAVE" {
        return Err(invalid("not a WAV file"));
    }
    let mut format = None;
    loop {
        let mut head = [0u8; 8];
        reader.read_exact(&mut head)?;
        let len = u64::from(u32::from_le_bytes([head[4], head[5], head[6], head[7]]));
        match &head[0..4] {
            // The chunk's own length is not used: see the module comment.
            b"data" => return format.ok_or_else(|| invalid("WAV file without a format chunk")),
            b"fmt " => {
                if len < 16 {
                    return Err(invalid("WAV format chunk is too short"));
                }
                let mut fmt = [0u8; 16];
                reader.read_exact(&mut fmt)?;
                let tag = u16::from_le_bytes([fmt[0], fmt[1]]);
                let channels = u16::from_le_bytes([fmt[2], fmt[3]]);
                let sample_rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
                let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
                // 1 is PCM, 0xFFFE the extensible form of it.
                if !(tag == 1 || tag == 0xFFFE) || bits != 16 {
                    return Err(invalid("only 16-bit PCM WAV files can be converted"));
                }
                if !(1..=2).contains(&channels) || sample_rate == 0 {
                    return Err(invalid("only mono and stereo WAV files can be converted"));
                }
                format = Some((channels, sample_rate));
                skip(reader, len - 16 + (len & 1))?;
            }
            // Chunks are padded to an even length.
            _ => skip(reader, len + (len & 1))?,
        }
    }
}

fn skip(reader: &mut impl Read, bytes: u64) -> io::Result<()> {
    let skipped = io::copy(&mut reader.take(bytes), &mut io::sink())?;
    if skipped == bytes {
        Ok(())
    } else {
        Err(invalid("WAV file ends inside a chunk"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::test_util::test_dir;
    use std::io::{Seek, SeekFrom, Write};

    fn write_wav(path: &Path, sample_rate: u32, channels: u16, samples: &[i16]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for &sample in samples {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn reads_the_samples_the_recorder_wrote() {
        let dir = test_dir("wav-read");
        let path = dir.join("a.wav");
        let samples: Vec<i16> = (0..2000).map(|i| (i * 13 - 9000) as i16).collect();
        write_wav(&path, 44_100, 2, &samples);

        let mut wav = WavPcm::open(&path, 1000).unwrap();
        assert_eq!((wav.sample_rate(), wav.channels()), (44_100, 2));
        let mut all = Vec::new();
        let mut part = Vec::new();
        // Uneven requests, the last one larger than what is left.
        for max in [1, 300, 600, 600] {
            let frames = wav.read(max, &mut part).unwrap();
            assert_eq!(part.len(), frames * 2);
            all.extend_from_slice(&part);
        }
        assert_eq!(all, samples);
        assert_eq!(wav.remaining(), 0);
        assert_eq!(wav.read(10, &mut part).unwrap(), 0);
        assert!(part.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reads_recordings_on_both_sides_of_the_4_gb_mark() {
        use crate::audio::writer::{OutputWriter, WavOutputWriter};
        let dir = test_dir("wav-rf64");
        // The mark lowered to 100 000 bytes of samples: 25 000 stereo frames.
        for (name, frames, magic) in [
            ("below.wav", 20_000usize, b"RIFF"),
            ("past.wav", 30_000, b"RF64"),
        ] {
            let path = dir.join(name);
            let sound: Vec<f32> = (0..frames * 2)
                .map(|i| ((i * 7919) % 2001) as f32 / 1000.0 - 1.0)
                .collect();
            let mut writer: Box<dyn OutputWriter> = Box::new(
                WavOutputWriter::with_rf64_past(path.clone(), 48_000, 2, 100_000).unwrap(),
            );
            for part in sound.chunks(960) {
                writer.write_samples(part).unwrap();
            }
            writer.finalize();
            assert_eq!(&std::fs::read(&path).unwrap()[..4], magic, "{name}");

            let mut wav = WavPcm::open(&path, frames as u64).unwrap();
            assert_eq!((wav.sample_rate(), wav.channels()), (48_000, 2));
            let mut all = Vec::new();
            let mut part = Vec::new();
            while wav.read(4096, &mut part).unwrap() > 0 {
                all.extend_from_slice(&part);
            }
            let stored: Vec<i16> = sound
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .collect();
            assert!(all == stored, "{name}: samples differ");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn frame_count_wins_over_the_length_in_the_header() {
        let dir = test_dir("wav-header");
        let path = dir.join("a.wav");
        let samples: Vec<i16> = (0..960).map(|i| i as i16).collect();
        write_wav(&path, 48_000, 2, &samples);
        // What a WAV looks like after the app was killed while recording, and
        // (with another wrong number) past 4 GB from AudioCaptor 0.4.1 or
        // older: the lengths in the header do not match the samples that
        // follow.
        let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.seek(SeekFrom::Start(4)).unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();
        file.seek(SeekFrom::Start(40)).unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();
        drop(file);
        assert_eq!(hound::WavReader::open(&path).unwrap().len(), 0);

        let mut wav = WavPcm::open(&path, 480).unwrap();
        let mut out = Vec::new();
        assert_eq!(wav.read(480, &mut out).unwrap(), 480);
        assert_eq!(out, samples);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fails_when_the_file_is_shorter_than_counted() {
        let dir = test_dir("wav-short");
        let path = dir.join("a.wav");
        write_wav(&path, 48_000, 1, &[1, 2, 3, 4]);
        let mut wav = WavPcm::open(&path, 10).unwrap();
        let mut out = Vec::new();
        let error = wav.read(10, &mut out).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("ends"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_chunks_before_the_samples() {
        // RIFF, a LIST chunk of odd length (so it is padded), fmt, data.
        let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
        bytes.extend_from_slice(b"LIST");
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(b"abc\0");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&16000u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&[0x34, 0x12, 0xFE, 0xFF]);
        let dir = test_dir("wav-chunks");
        let path = dir.join("a.wav");
        std::fs::write(&path, &bytes).unwrap();

        let mut wav = WavPcm::open(&path, 2).unwrap();
        assert_eq!((wav.sample_rate(), wav.channels()), (8000, 1));
        let mut out = Vec::new();
        wav.read(2, &mut out).unwrap();
        assert_eq!(out, [0x1234, -2]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_files_that_are_not_16_bit_pcm_wav() {
        let dir = test_dir("wav-bad");
        let not_wav = dir.join("a.wav");
        std::fs::write(&not_wav, b"OggS this is not a WAV file at all").unwrap();
        assert!(WavPcm::open(&not_wav, 1).is_err());

        let float = dir.join("b.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&float, spec).unwrap();
        writer.write_sample(0.5f32).unwrap();
        writer.finalize().unwrap();
        assert!(WavPcm::open(&float, 1).is_err());
        assert!(WavPcm::open(&dir.join("missing.wav"), 1).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

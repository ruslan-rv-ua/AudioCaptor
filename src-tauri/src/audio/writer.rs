use anyhow::Context;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// How often the WAV header is brought up to date while recording, in seconds
/// of recorded sound. The sizes in the header cover the sound only up to the
/// last update, so a file cut short by a crash or a killed process opens, and
/// its header leaves out at most this much of the sound written to it.
const HEADER_UPDATE_SECONDS: u64 = 1;

/// How much sound the writer holds before it hands it to Windows, in seconds:
/// more than comes between two header updates, so that the samples reach the
/// file together with the header that describes them. The file then always
/// ends where its header says, also after a crash. Media Foundation, which the
/// players of Windows use, opens an RF64 file only when the size in its header
/// is the length of the file (TASK-50); a plain WAV may be longer.
const BUFFER_SECONDS: u64 = 2 * HEADER_UPDATE_SECONDS;

/// Bytes before the first sample. The layout is the one FFmpeg writes with
/// `-rf64 auto`, after EBU Tech 3306.
///
/// Up to the 4 GB mark the file is a plain WAV: the RIFF header, the `fmt `
/// chunk where every WAV has it, a `JUNK` chunk of 28 zero bytes that players
/// skip, and the head of the `data` chunk. Past the mark the same 80 bytes are
/// rewritten as RF64, the WAV for files over 4 GB: `RIFF` becomes `RF64`, a
/// `ds64` chunk with the 64-bit sizes comes first, `fmt ` moves into the room
/// that `JUNK` kept, and the 32-bit sizes read 0xFFFFFFFF. The samples stay
/// where they are.
const HEADER_LEN: u64 = 80;

/// The largest RIFF size (the length of the file less 8 bytes) that the 32-bit
/// field holds. A file at 48 kHz stereo passes it after about 6 h 12 min.
const RIFF_SIZE_LIMIT: u64 = u32::MAX as u64;

/// A recording file the recorder has finished writing.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedFile {
    pub path: PathBuf,
    /// Frames (one sample per channel) written to the file, counted by the
    /// recorder itself. The conversion to a compressed format reads this many
    /// frames and never trusts the WAV header: after a crash or a failed
    /// finalize the header can be a second short, and in a WAV past 4 GB from
    /// AudioCaptor 0.4.1 or older its length has wrapped around.
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

/// Writes 16-bit PCM WAV, and RF64 once the file passes 4 GB (`HEADER_LEN`).
pub struct WavOutputWriter {
    file: BufWriter<File>,
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
    samples_written: u64,
    /// `samples_written` when the header was last brought up to date.
    samples_in_header: u64,
    /// Samples (`HEADER_UPDATE_SECONDS` of sound) between two header updates.
    samples_per_header_update: u64,
    /// The RIFF size past which the file is RF64: `RIFF_SIZE_LIMIT`, lower in
    /// tests.
    riff_size_limit: u64,
    /// Whether the header written last is RF64.
    rf64: bool,
    /// Set when a header update could not get back to the end of the file: the
    /// next samples go there first or are not written.
    off_the_end: bool,
    finalized: bool,
}

impl WavOutputWriter {
    /// Create a new WAV writer at the given path.
    /// Channels: 1 (mono) or 2 (stereo). Sample rate in Hz.
    pub fn new(path: PathBuf, sample_rate: u32, channels: u16) -> anyhow::Result<Self> {
        Self::create(path, sample_rate, channels, RIFF_SIZE_LIMIT)
    }

    /// A writer whose file turns into RF64 once it holds more than `data_bytes`
    /// of samples, instead of at 4 GB.
    #[cfg(test)]
    pub(crate) fn with_rf64_past(
        path: PathBuf,
        sample_rate: u32,
        channels: u16,
        data_bytes: u64,
    ) -> anyhow::Result<Self> {
        Self::create(path, sample_rate, channels, HEADER_LEN - 8 + data_bytes)
    }

    fn create(
        path: PathBuf,
        sample_rate: u32,
        channels: u16,
        riff_size_limit: u64,
    ) -> anyhow::Result<Self> {
        let bytes_per_second = u64::from(sample_rate) * u64::from(channels.max(1)) * 2;
        let buffer = (bytes_per_second * BUFFER_SECONDS).max(8 * 1024) as usize;
        let mut writer = Self {
            file: BufWriter::with_capacity(buffer, File::create(&path)?),
            path,
            sample_rate,
            channels,
            samples_written: 0,
            samples_in_header: 0,
            samples_per_header_update: u64::from(sample_rate)
                * u64::from(channels)
                * HEADER_UPDATE_SECONDS,
            riff_size_limit,
            rf64: false,
            off_the_end: false,
            finalized: false,
        };
        // A file without samples is a valid WAV from the start.
        let header = writer.header(0);
        writer.file.write_all(&header)?;
        writer.file.flush()?;
        log::info!("WAV writer created: {}", writer.path.display());
        Ok(writer)
    }

    /// Bytes of one frame.
    fn block_align(&self) -> u16 {
        2 * self.channels.max(1)
    }

    fn is_rf64(&self, data_bytes: u64) -> bool {
        HEADER_LEN - 8 + data_bytes > self.riff_size_limit
    }

    /// The `HEADER_LEN` bytes before the samples when the file holds
    /// `data_bytes` of them.
    fn header(&self, data_bytes: u64) -> Vec<u8> {
        let riff_size = HEADER_LEN - 8 + data_bytes;
        let block_align = self.block_align();
        let mut fmt = Vec::with_capacity(24);
        fmt.extend_from_slice(b"fmt ");
        fmt.extend_from_slice(&16u32.to_le_bytes());
        fmt.extend_from_slice(&1u16.to_le_bytes()); // PCM
        fmt.extend_from_slice(&self.channels.to_le_bytes());
        fmt.extend_from_slice(&self.sample_rate.to_le_bytes());
        fmt.extend_from_slice(&(self.sample_rate * u32::from(block_align)).to_le_bytes());
        fmt.extend_from_slice(&block_align.to_le_bytes());
        fmt.extend_from_slice(&16u16.to_le_bytes());

        let mut header = Vec::with_capacity(HEADER_LEN as usize);
        if self.is_rf64(data_bytes) {
            header.extend_from_slice(b"RF64");
            header.extend_from_slice(&u32::MAX.to_le_bytes());
            header.extend_from_slice(b"WAVE");
            header.extend_from_slice(b"ds64");
            header.extend_from_slice(&28u32.to_le_bytes());
            header.extend_from_slice(&riff_size.to_le_bytes());
            header.extend_from_slice(&data_bytes.to_le_bytes());
            header.extend_from_slice(&(data_bytes / u64::from(block_align)).to_le_bytes());
            // No table of other chunks larger than 4 GB.
            header.extend_from_slice(&0u32.to_le_bytes());
            header.extend_from_slice(&fmt);
            header.extend_from_slice(b"data");
            header.extend_from_slice(&u32::MAX.to_le_bytes());
        } else {
            // Below the limit both sizes fit into 32 bits.
            header.extend_from_slice(b"RIFF");
            header.extend_from_slice(&(riff_size as u32).to_le_bytes());
            header.extend_from_slice(b"WAVE");
            header.extend_from_slice(&fmt);
            header.extend_from_slice(b"JUNK");
            header.extend_from_slice(&28u32.to_le_bytes());
            header.extend_from_slice(&[0; 28]);
            header.extend_from_slice(b"data");
            header.extend_from_slice(&(data_bytes as u32).to_le_bytes());
        }
        debug_assert_eq!(header.len() as u64, HEADER_LEN);
        header
    }

    /// Writes the sizes of the whole frames written so far into the header and
    /// hands every sample to Windows, so that the file can be read up to here
    /// even if the app never finalizes it. The file stays open for more samples.
    ///
    /// The header goes in one write, so the file always has either the header
    /// before or the one after, also at the switch to RF64. A failure is a
    /// write error like any other.
    fn write_header(&mut self) -> std::io::Result<()> {
        let block_align = u64::from(self.block_align());
        let data_bytes = self.samples_written * 2 / block_align * block_align;
        let header = self.header(data_bytes);
        // Seeking writes out the buffered samples first.
        self.file.seek(SeekFrom::Start(0))?;
        let written = self
            .file
            .write_all(&header)
            .and_then(|()| self.file.flush());
        // Back to the end even after a failed write. If even that fails, the
        // header stays in the buffer, and the next samples must not follow it
        // over the start of the sound (`write_samples`).
        let back = self.file.seek(SeekFrom::End(0));
        self.off_the_end = back.is_err();
        written?;
        back?;
        if self.is_rf64(data_bytes) && !self.rf64 {
            self.rf64 = true;
            log::info!(
                "The recording has passed 4 GB and goes on as RF64: {}",
                self.path.display()
            );
        }
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
        if self.off_the_end {
            // Writes out the rest of the header, then goes to the end.
            self.file.seek(SeekFrom::End(0))?;
            self.off_the_end = false;
        }
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            self.file.write_all(&int_sample.to_le_bytes())?;
            self.samples_written += 1;
        }
        if self.samples_written - self.samples_in_header >= self.samples_per_header_update {
            self.samples_in_header = self.samples_written;
            self.write_header()
                .context("Failed to update the WAV header")?;
        }
        Ok(())
    }

    /// Finalize the WAV file (writes header with correct sizes).
    fn finalize(mut self: Box<Self>) -> RecordedFile {
        self.finalized = true;
        match self.write_header() {
            Ok(()) if self.rf64 => log::info!("RF64 file finalized: {}", self.path.display()),
            Ok(()) => log::info!("WAV file finalized: {}", self.path.display()),
            Err(e) => log::error!("Failed to finalize WAV {}: {e}", self.path.display()),
        }
        RecordedFile {
            path: self.path.clone(),
            frames: self.samples_written / u64::from(self.channels.max(1)),
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }
}

impl Drop for WavOutputWriter {
    /// A writer dropped without `finalize` (a panic in a build that unwinds)
    /// still leaves a header that covers every frame written.
    fn drop(&mut self) {
        if !self.finalized {
            let _ = self.write_header();
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

    /// The bytes the samples take in the file.
    fn as_bytes(samples: &[i16]) -> Vec<u8> {
        samples.iter().flat_map(|s| s.to_le_bytes()).collect()
    }

    /// How many frames the header of the file at `path` says it holds.
    fn frames_in_header(path: &std::path::Path) -> u32 {
        hound::WavReader::open(path).unwrap().duration()
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    fn u64_at(bytes: &[u8], at: usize) -> u64 {
        u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
    }

    /// What the header of a file in either layout says.
    #[derive(Debug, PartialEq)]
    struct Header {
        rf64: bool,
        /// The length of the file less 8 bytes.
        riff_size: u64,
        data_size: u64,
        /// The frame count of `ds64`. A plain WAV has none.
        frames: Option<u64>,
        /// The `fmt ` chunk with its head.
        fmt: Vec<u8>,
    }

    /// Reads the header of the file at `path` and checks every byte that is the
    /// same in all files of its layout.
    fn header_of(path: &std::path::Path) -> Header {
        let bytes = std::fs::read(path).unwrap();
        let h = &bytes[..HEADER_LEN as usize];
        assert_eq!(&h[8..12], b"WAVE");
        assert_eq!(&h[72..76], b"data");
        match &h[0..4] {
            b"RIFF" => {
                assert_eq!(&h[12..16], b"fmt ");
                assert_eq!(&h[36..40], b"JUNK");
                assert_eq!(u32_at(h, 40), 28);
                assert_eq!(h[44..72], [0; 28]);
                Header {
                    rf64: false,
                    riff_size: u64::from(u32_at(h, 4)),
                    data_size: u64::from(u32_at(h, 76)),
                    frames: None,
                    fmt: h[12..36].to_vec(),
                }
            }
            b"RF64" => {
                assert_eq!(u32_at(h, 4), u32::MAX);
                assert_eq!(&h[12..16], b"ds64");
                assert_eq!(u32_at(h, 16), 28);
                assert_eq!(u32_at(h, 44), 0, "no table of other chunks");
                assert_eq!(&h[48..52], b"fmt ");
                assert_eq!(u32_at(h, 76), u32::MAX);
                Header {
                    rf64: true,
                    riff_size: u64_at(h, 20),
                    data_size: u64_at(h, 28),
                    frames: Some(u64_at(h, 36)),
                    fmt: h[48..72].to_vec(),
                }
            }
            other => panic!("neither RIFF nor RF64: {other:?}"),
        }
    }

    /// The `fmt ` chunk of a 16-bit stereo file at 8 kHz.
    const FMT_8K_STEREO: [u8; 24] = [
        b'f', b'm', b't', b' ', 16, 0, 0, 0, // chunk head
        1, 0, 2, 0, // PCM, stereo
        0x40, 0x1F, 0, 0, // 8000 Hz
        0x00, 0x7D, 0, 0, // 32 000 bytes per second
        4, 0, 16, 0, // 4 bytes per frame, 16 bits
    ];

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

        // The app is killed: neither finalize nor drop runs (drop would write
        // the header). The file stays open until the tests end.
        std::mem::forget(writer);

        // The last half second was still in the writer's buffer.
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 80 + 64_000);
        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().channels, 2);
        assert_eq!(reader.spec().sample_rate, 8000);
        assert_eq!(reader.duration(), 16000);
        let read: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
        assert_eq!(read, as_stored(&sound[..32000]));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn finalized_file_is_a_plain_wav_with_room_for_rf64() {
        let sound = signal(5 * 8000);
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 8000, 2).unwrap());
        // Two header updates on the way leave no trace.
        for tenth_of_a_second in sound.chunks(1600) {
            writer.write_samples(tenth_of_a_second).unwrap();
        }
        writer.finalize();

        let mut expected = b"RIFF".to_vec();
        expected.extend_from_slice(&(72u32 + 80_000).to_le_bytes());
        expected.extend_from_slice(b"WAVE");
        expected.extend_from_slice(&FMT_8K_STEREO);
        expected.extend_from_slice(b"JUNK");
        expected.extend_from_slice(&28u32.to_le_bytes());
        expected.extend_from_slice(&[0; 28]);
        expected.extend_from_slice(b"data");
        expected.extend_from_slice(&80_000u32.to_le_bytes());
        expected.extend_from_slice(&as_bytes(&as_stored(&sound)));
        let written = std::fs::read(&path).unwrap();
        assert_eq!(written, expected);

        // Against the WAV that AudioCaptor wrote through hound before: the same
        // `fmt ` chunk right after the RIFF header and the same samples; only
        // the `JUNK` chunk stands before the samples now.
        let hound_path = temp_wav_path();
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut plain = hound::WavWriter::create(&hound_path, spec).unwrap();
        for sample in as_stored(&sound) {
            plain.write_sample(sample).unwrap();
        }
        plain.finalize().unwrap();
        let before = std::fs::read(&hound_path).unwrap();
        assert_eq!(written[..4], before[..4], "RIFF");
        // Between them the RIFF size, 36 bytes larger now.
        assert_eq!(written[8..36], before[8..36], "WAVE and the fmt chunk");
        assert_eq!(written[80..], before[44..], "samples");
        assert_eq!(written.len(), before.len() + 36);

        std::fs::remove_file(&path).ok();
        std::fs::remove_file(&hound_path).ok();
    }

    #[test]
    fn header_holds_only_whole_frames() {
        let path = temp_wav_path();
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::new(path.clone(), 8000, 2).unwrap());
        // A second and half a frame, then the other half of that frame.
        let sound = signal(16002);
        writer.write_samples(&sound[..16001]).unwrap();
        assert_eq!(frames_in_header(&path), 8000, "the half frame is left out");
        writer.write_samples(&sound[16001..]).unwrap();
        writer.finalize();

        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.duration(), 8001);
        let read: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
        assert_eq!(read, as_stored(&sound));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn header_turns_into_rf64_past_the_4_gb_mark() {
        let path = temp_wav_path();
        // The mark lowered to a second and a half of stereo at 8 kHz.
        let mut writer: Box<dyn OutputWriter> =
            Box::new(WavOutputWriter::with_rf64_past(path.clone(), 8000, 2, 48_000).unwrap());
        let empty = header_of(&path);
        assert_eq!(
            (empty.rf64, empty.riff_size, empty.data_size),
            (false, 72, 0)
        );
        assert_eq!(empty.fmt, FMT_8K_STEREO);

        let sound = signal(5 * 8000);
        let mut halves = sound.chunks(8000);
        writer.write_samples(halves.next().unwrap()).unwrap();
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert_eq!(
            header_of(&path),
            Header {
                rf64: false,
                riff_size: 72 + 32_000,
                data_size: 32_000,
                frames: None,
                fmt: FMT_8K_STEREO.to_vec(),
            },
            "after a second, below the mark"
        );
        // Past the mark (1.5 s) the header waits for its next update (2 s).
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert!(!header_of(&path).rf64);
        writer.write_samples(halves.next().unwrap()).unwrap();
        assert_eq!(
            header_of(&path),
            Header {
                rf64: true,
                riff_size: 72 + 64_000,
                data_size: 64_000,
                frames: Some(16_000),
                fmt: FMT_8K_STEREO.to_vec(),
            },
            "after two seconds, past the mark"
        );
        writer.write_samples(halves.next().unwrap()).unwrap();
        writer.finalize();

        assert_eq!(
            header_of(&path),
            Header {
                rf64: true,
                riff_size: 72 + 80_000,
                data_size: 80_000,
                frames: Some(20_000),
                fmt: FMT_8K_STEREO.to_vec(),
            },
            "finalized"
        );
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, 8 + 72 + 80_000);
        assert_eq!(bytes[80..], as_bytes(&as_stored(&sound)));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn unfinalized_rf64_file_holds_every_sample_up_to_the_last_header_update() {
        let path = temp_wav_path();
        let mut writer = WavOutputWriter::with_rf64_past(path.clone(), 8000, 2, 48_000).unwrap();
        let sound = signal(5 * 8000);
        for half in sound.chunks(8000) {
            writer.write_samples(half).unwrap();
        }
        // Killed half a second after the update at two seconds.
        std::mem::forget(writer);

        let header = header_of(&path);
        assert_eq!(
            (header.rf64, header.data_size, header.frames),
            (true, 64_000, Some(16_000))
        );
        // The last half second never reached the file: it ends where its
        // header says, or Media Foundation would not open it.
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            header.riff_size + 8
        );
        // The samples the header covers, read as the conversion reads them.
        let mut wav = crate::convert::wav::WavPcm::open(&path, 16_000).unwrap();
        assert_eq!((wav.sample_rate(), wav.channels()), (8000, 2));
        let mut read = Vec::new();
        assert_eq!(wav.read(20_000, &mut read).unwrap(), 16_000);
        assert_eq!(read, as_stored(&sound[..32_000]));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn a_dropped_writer_leaves_a_header_for_every_frame() {
        let path = temp_wav_path();
        let mut writer = WavOutputWriter::new(path.clone(), 8000, 1).unwrap();
        writer.write_samples(&signal(500)).unwrap();
        drop(writer);
        assert_eq!(frames_in_header(&path), 500);
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

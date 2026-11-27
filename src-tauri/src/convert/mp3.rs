//! MP3 files (`.mp3`), written by the MP3 encoder that comes with Windows: the
//! "MP3 Audio Encoder" of Media Foundation (decision-5). The app carries no MP3
//! library of its own.
//!
//! The encoder takes 16-bit PCM, mono or stereo, at the rates of MPEG-1 (32,
//! 44.1 and 48 kHz), MPEG-2 (16, 22.05 and 24 kHz) and MPEG-2.5 (8, 11.025 and
//! 12 kHz), so every rate of a profile is encoded as it is. It writes bare
//! frames at a constant bitrate (no ID3 tag, no Xing or Info header), and which
//! bitrates it offers depends on the rate and the channels (`offered_kbps`).
//!
//! Windows N editions have no encoder until the Media Feature Pack is
//! installed; a conversion then fails with `ConvertError::NoEncoder`.

use super::media_foundation::{Session, Transform};
use super::wav::WavPcm;
use super::{Codec, Control, ConvertError, Job};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use windows::core::GUID;
use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFTransform, MFAudioFormat_PCM, MFMediaType_Audio, MP3ACMCodecWrapper,
    MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE, MF_MT_AUDIO_BLOCK_ALIGNMENT,
    MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
    MF_MT_USER_DATA,
};

/// The bitrates of the variants in kbit/s, where the sample rate allows them.
pub const SPEECH_KBPS: u32 = 64;
pub const STANDARD_KBPS: u32 = 128;
pub const HIGH_KBPS: u32 = 320;

/// The environment variable that makes the app act as if Windows had no MP3
/// encoder, as a Windows N edition without the Media Feature Pack does: set to
/// 1, MP3 conversions fail and the recordings stay WAV. The NVDA test
/// (`just smoke`) uses it.
pub const NO_ENCODER_SWITCH: &str = "AUDIOCAPTOR_NO_MP3_ENCODER";

/// Frames of the recording handed to the encoder at a time.
const CHUNK_FRAMES: usize = 4096;

/// The encoder's delay and its last frame make an MP3 longer than the
/// recording: by 1729 to 2880 samples per channel (MPEG-1) or 1729 to 2304
/// (MPEG-2 and 2.5), as measured on Windows 11 (TASK-29). The sound starts
/// about 1201 samples in, so a file that holds the whole recording is at least
/// `MIN_EXTRA` samples longer; `MAX_EXTRA` leaves room for two more frames.
const MIN_EXTRA: u64 = 1152;
const MAX_EXTRA: u64 = 1728 + 2 * 1152;

/// The three entries of the format list that write MP3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variant {
    /// "MP3 - speech": mono.
    Speech,
    /// "MP3 - standard": stereo, 128 kbit/s.
    Standard,
    /// "MP3 - high quality": stereo, for music.
    High,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    pub channels: u16,
    pub kbps: u32,
}

/// The MPEG version of the frames, which the sample rate decides.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mpeg {
    V1,
    V2,
    V2_5,
}

impl Mpeg {
    fn for_rate(rate: u32) -> Option<Mpeg> {
        match rate {
            32_000 | 44_100 | 48_000 => Some(Mpeg::V1),
            16_000 | 22_050 | 24_000 => Some(Mpeg::V2),
            8_000 | 11_025 | 12_000 => Some(Mpeg::V2_5),
            _ => None,
        }
    }

    /// The rates of the three sample-rate codes in a frame header.
    fn rates(self) -> [u32; 3] {
        match self {
            Mpeg::V1 => [44_100, 48_000, 32_000],
            Mpeg::V2 => [22_050, 24_000, 16_000],
            Mpeg::V2_5 => [11_025, 12_000, 8_000],
        }
    }

    /// The bitrates (kbit/s) of the bitrate codes of a Layer III frame header;
    /// 0 marks a code that is not a bitrate.
    fn bitrates(self) -> [u32; 16] {
        match self {
            Mpeg::V1 => [
                0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
            ],
            Mpeg::V2 | Mpeg::V2_5 => [
                0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
            ],
        }
    }

    /// Samples per channel in one frame.
    fn samples_per_frame(self) -> u64 {
        match self {
            Mpeg::V1 => 1152,
            Mpeg::V2 | Mpeg::V2_5 => 576,
        }
    }
}

/// The bitrates (kbit/s) the Windows encoder offers at a rate MP3 has, as it
/// listed them on Windows 11 (TASK-29). Stereo below 96 kbit/s needs a rate
/// under 32 kHz.
fn offered_kbps(rate: u32, channels: u16) -> &'static [u32] {
    let mono = channels == 1;
    match rate {
        32_000.. if mono => &[48, 56, 64, 80, 96, 112, 128],
        32_000.. => &[96, 112, 128, 160, 192, 224, 256, 320],
        22_050.. if mono => &[24, 32, 40, 48],
        16_000.. if mono => &[16, 24, 32, 40, 48],
        16_000.. => &[32, 40, 48, 56, 64, 80],
        _ if mono => &[8, 16],
        _ => &[24, 32],
    }
}

/// The settings of a variant for a recording at `sample_rate` with
/// `wav_channels` channels, or `None` for a rate MP3 does not have. A mono
/// recording stays mono. The bitrate is the variant's, or, where the rate does
/// not allow it (8 and 16 kHz), the highest the encoder offers (decision-6).
pub fn settings(variant: Variant, sample_rate: u32, wav_channels: u16) -> Option<Settings> {
    Mpeg::for_rate(sample_rate)?;
    let (channels, wanted) = match variant {
        Variant::Speech => (1, SPEECH_KBPS),
        Variant::Standard => (2, STANDARD_KBPS),
        Variant::High => (2, HIGH_KBPS),
    };
    let channels = channels.min(wav_channels.max(1));
    let offered = offered_kbps(sample_rate, channels);
    let kbps = offered
        .iter()
        .rev()
        .copied()
        .find(|&kbps| kbps <= wanted)
        .unwrap_or(offered[0]);
    Some(Settings { channels, kbps })
}

/// Whether the value of `NO_ENCODER_SWITCH` turns the encoder off.
fn switched_off(value: Option<&OsStr>) -> bool {
    value.is_some_and(|value| value == "1")
}

pub struct Mp3Codec {
    variant: Variant,
    /// False when the encoder counts as missing: `NO_ENCODER_SWITCH`, or a test.
    encoder_allowed: bool,
}

impl Mp3Codec {
    pub fn new(variant: Variant) -> Self {
        Self {
            variant,
            encoder_allowed: !switched_off(std::env::var_os(NO_ENCODER_SWITCH).as_deref()),
        }
    }

    fn settings_for(&self, job: &Job) -> Result<(Mpeg, Settings), ConvertError> {
        match (
            Mpeg::for_rate(job.sample_rate),
            settings(self.variant, job.sample_rate, job.channels),
        ) {
            (Some(mpeg), Some(settings)) => Ok((mpeg, settings)),
            _ => Err(ConvertError::failed(format!(
                "MP3 cannot be written at {} Hz",
                job.sample_rate
            ))),
        }
    }
}

impl Codec for Mp3Codec {
    fn extension(&self) -> &'static str {
        "mp3"
    }

    /// The check only reads the frame headers: an hour at 128 kbit/s takes
    /// a fraction of a second, encoding it about 13 s (TASK-29).
    fn encode_share(&self) -> f32 {
        0.98
    }

    fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        let (_, settings) = self.settings_for(job)?;
        if !self.encoder_allowed {
            log::warn!("The MP3 encoder is turned off by {NO_ENCODER_SWITCH}");
            return Err(ConvertError::NoEncoder);
        }
        let mut wav = WavPcm::open(&job.wav, job.frames)?;
        if wav.sample_rate() != job.sample_rate || wav.channels() != job.channels {
            return Err(ConvertError::failed(format!(
                "the WAV file is {} Hz with {} channels, the recording was {} Hz with {}",
                wav.sample_rate(),
                wav.channels(),
                job.sample_rate,
                job.channels
            )));
        }

        let session = Session::start()?;
        let mut encoder = open_encoder(&session, job.sample_rate, settings)?;
        let mut file = BufWriter::new(File::create(output)?);
        let mut sink = |bytes: &[u8]| file.write_all(bytes);

        let wav_channels = usize::from(job.channels);
        let channels = usize::from(settings.channels);
        let rate = u64::from(job.sample_rate);
        let mut raw = Vec::new();
        let mut pcm = Vec::with_capacity(CHUNK_FRAMES * channels * 2);
        let mut done = 0u64;
        loop {
            control.check()?;
            let frames = wav.read(CHUNK_FRAMES, &mut raw)?;
            if frames == 0 {
                break;
            }
            pcm.clear();
            append_pcm(&raw, wav_channels, channels, &mut pcm);
            let timing = (ticks(done, rate), ticks(frames as u64, rate));
            encoder.push(&pcm, Some(timing), &mut sink)?;
            done += frames as u64;
            control.progress(done as f32 / job.frames as f32);
        }
        if job.frames == 0 {
            // An empty recording still gives a file a player opens: one sample
            // of silence goes in, and the encoder's delay makes it 48 ms long.
            pcm.clear();
            pcm.resize(channels * 2, 0);
            encoder.push(&pcm, Some((0, ticks(1, rate))), &mut sink)?;
        }
        encoder.finish(&mut sink)?;
        drop(encoder);

        // On the disk before the WAV may be deleted.
        file.flush()?;
        file.get_ref().sync_all()?;
        Ok(())
    }

    fn verify(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        let (mpeg, settings) = self.settings_for(job)?;
        let expected = FrameHeader {
            mpeg,
            rate: job.sample_rate,
            kbps: settings.kbps,
            mono: settings.channels == 1,
            padding: false,
        };
        let file = File::open(output)?;
        let size = file.metadata()?.len();
        let frames = count_frames(BufReader::new(file), size, &expected, control)?;
        check_length(frames * mpeg.samples_per_frame(), job.frames)
    }
}

/// The encoder for `settings` at `rate`, ready for 16-bit PCM.
fn open_encoder(
    session: &Session,
    rate: u32,
    settings: Settings,
) -> Result<Transform<'_>, ConvertError> {
    let encoder: IMFTransform = session.create(&MP3ACMCodecWrapper).map_err(|e| {
        log::warn!("Windows has no MP3 encoder: {e}");
        ConvertError::NoEncoder
    })?;
    let channels = u32::from(settings.channels);
    // The encoder takes only an output type from its own list: one built with
    // the same values is refused.
    let wanted = (rate, channels, settings.kbps * 1000 / 8);
    let output = (0..)
        // SAFETY: a plain call on a live object.
        .map_while(|index| unsafe { encoder.GetOutputAvailableType(0, index) }.ok())
        .find(|kind| audio_format(kind) == wanted)
        .ok_or_else(|| {
            ConvertError::failed(format!(
                "the MP3 encoder of Windows offers no {} kbit/s at {rate} Hz with {channels} channels",
                settings.kbps
            ))
        })?;
    pad_frames(&output)?;
    let input = session.media_type()?;
    // SAFETY: plain calls on live objects.
    unsafe {
        encoder.SetOutputType(0, &output, 0)?;
        input.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        input.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM)?;
        input.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, rate)?;
        input.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, channels)?;
        input.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)?;
        input.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, channels * 2)?;
        input.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, rate * channels * 2)?;
        encoder.SetInputType(0, &input, 0)?;
    }
    Transform::start(session, encoder)
}

/// `MPEGLAYER3_FLAG_PADDING_ISO`: frames get the padding byte that the MP3
/// standard gives them, so that the bitrate comes out exactly.
const PADDING_ISO: u32 = 0;

/// The encoder's own output types turn the padding of frames off. At 44.1 kHz
/// a frame then holds less than its share of the bitrate (127.7 kbit/s for
/// 128), and players that work the length out from the bitrate, the standard
/// players of Windows among them, show an hour as 59:52. With ISO padding they
/// show the length of the file; at 48, 16 and 8 kHz no frame needs padding.
/// The flag is part of the MPEGLAYER3WAVEFORMAT that the type holds as its
/// user data: after `wID` (2 bytes) come the `fdwFlags` (4 bytes).
fn pad_frames(kind: &IMFMediaType) -> Result<(), ConvertError> {
    // SAFETY: plain calls on a live object; the buffer has the blob's size.
    unsafe {
        let size = kind.GetBlobSize(&MF_MT_USER_DATA)?;
        let mut format = vec![0u8; size as usize];
        kind.GetBlob(&MF_MT_USER_DATA, &mut format, None)?;
        if format.len() < 6 {
            return Err(ConvertError::failed(
                "the MP3 encoder of Windows describes its output without the MPEG Layer-3 format",
            ));
        }
        format[2..6].copy_from_slice(&PADDING_ISO.to_le_bytes());
        kind.SetBlob(&MF_MT_USER_DATA, &format)?;
    }
    Ok(())
}

/// The rate, channels and bytes per second of an audio media type.
fn audio_format(kind: &IMFMediaType) -> (u32, u32, u32) {
    // SAFETY: a plain call on a live object.
    let get = |key: &GUID| unsafe { kind.GetUINT32(key) }.unwrap_or(0);
    (
        get(&MF_MT_AUDIO_SAMPLES_PER_SECOND),
        get(&MF_MT_AUDIO_NUM_CHANNELS),
        get(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND),
    )
}

/// `frames` at `rate` in the 100 ns units of Media Foundation.
fn ticks(frames: u64, rate: u64) -> i64 {
    i64::try_from(frames * 10_000_000 / rate.max(1)).unwrap_or(i64::MAX)
}

/// Appends `raw` (interleaved, `from` channels) to `out` as 16-bit
/// little-endian samples with `to` channels: for mono out of stereo, the
/// average of the two sides.
fn append_pcm(raw: &[i16], from: usize, to: usize, out: &mut Vec<u8>) {
    if from == to {
        out.extend(raw.iter().flat_map(|sample| sample.to_le_bytes()));
    } else {
        out.extend(raw.chunks_exact(from).flat_map(|frame| {
            let sum: i32 = frame.iter().map(|&sample| i32::from(sample)).sum();
            ((sum / from as i32) as i16).to_le_bytes()
        }));
    }
}

/// What the header of a Layer III frame says.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FrameHeader {
    mpeg: Mpeg,
    rate: u32,
    kbps: u32,
    mono: bool,
    padding: bool,
}

impl FrameHeader {
    /// Reads the four bytes a frame starts with; `None` if they are not the
    /// header of a Layer III frame.
    fn parse(bytes: [u8; 4]) -> Option<Self> {
        let header = u32::from_be_bytes(bytes);
        // Eleven sync bits, then Layer III.
        if header >> 21 != 0x7ff || (header >> 17) & 3 != 1 {
            return None;
        }
        let mpeg = match (header >> 19) & 3 {
            3 => Mpeg::V1,
            2 => Mpeg::V2,
            0 => Mpeg::V2_5,
            _ => return None,
        };
        let rate = *mpeg.rates().get(((header >> 10) & 3) as usize)?;
        let kbps = mpeg.bitrates()[((header >> 12) & 15) as usize];
        if kbps == 0 {
            return None;
        }
        Some(Self {
            mpeg,
            rate,
            kbps,
            mono: (header >> 6) & 3 == 3,
            padding: (header >> 9) & 1 == 1,
        })
    }

    /// Bytes in the frame, its header included.
    fn len(&self) -> u64 {
        let per_kbps = match self.mpeg {
            Mpeg::V1 => 144_000,
            Mpeg::V2 | Mpeg::V2_5 => 72_000,
        };
        u64::from(per_kbps * self.kbps / self.rate) + u64::from(self.padding)
    }

    /// The same stream format; padding may differ from frame to frame.
    fn same_format(&self, other: &FrameHeader) -> bool {
        (self.mpeg, self.rate, self.kbps, self.mono)
            == (other.mpeg, other.rate, other.kbps, other.mono)
    }
}

/// Reads an MP3 file of `size` bytes frame by frame and returns how many
/// frames it holds. Fails unless the file is nothing but whole frames in the
/// format of `expected`.
fn count_frames(
    mut reader: impl BufRead,
    size: u64,
    expected: &FrameHeader,
    control: &Control,
) -> Result<u64, ConvertError> {
    let cut_short = || ConvertError::failed("the MP3 file is cut short");
    let mut frames = 0u64;
    let mut at = 0u64;
    while !reader.fill_buf()?.is_empty() {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                cut_short()
            } else {
                e.into()
            }
        })?;
        let header = FrameHeader::parse(bytes).ok_or_else(|| {
            ConvertError::failed(format!("the MP3 file has no frame at byte {at}"))
        })?;
        if !header.same_format(expected) {
            return Err(ConvertError::failed(format!(
                "the MP3 frame at byte {at} is {header:?}, not {expected:?}"
            )));
        }
        let rest = header.len() - 4;
        if io::copy(&mut reader.by_ref().take(rest), &mut io::sink())? < rest {
            return Err(cut_short());
        }
        at += header.len();
        frames += 1;
        if frames.is_multiple_of(1024) {
            control.check()?;
            control.progress(at as f32 / size.max(1) as f32);
        }
    }
    Ok(frames)
}

/// The file must hold every sample of the recording and the encoder's delay,
/// and no more than the frames the encoder adds after it.
fn check_length(samples: u64, recorded: u64) -> Result<(), ConvertError> {
    if samples < recorded + MIN_EXTRA || samples > recorded + MAX_EXTRA {
        return Err(ConvertError::failed(format!(
            "the MP3 file holds {samples} samples per channel, the recording {recorded}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::OutputFormat;
    use crate::convert::test_util::{free_run, test_dir, tone_wav};
    use crate::convert::{convert_file, temp_path};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows::Win32::Media::MediaFoundation::{CLSID_MP3DecMediaObject, MFAudioFormat_MP3};

    /// Samples at the start that the Windows MP3 decoder leaves out: its own
    /// delay. What is left starts this many samples before the encoder's delay
    /// of about 1201 samples has passed.
    const DECODER_SKIPS: usize = 528;
    /// Where the sound of the recording starts in what the decoder gives out,
    /// give or take a few samples.
    const SOUND_STARTS_AT: i64 = 1201 - DECODER_SKIPS as i64;

    /// Whether this Windows has the MP3 encoder. GitHub's Windows Server
    /// images have no Media Foundation, so CI skips the tests that need it;
    /// anywhere else a missing encoder fails them.
    fn have_encoder() -> bool {
        let found = Session::start()
            .is_ok_and(|session| session.create::<IMFTransform>(&MP3ACMCodecWrapper).is_ok());
        if !found {
            assert!(
                std::env::var_os("CI").is_some(),
                "Windows has no MP3 encoder; outside CI the tests need it"
            );
            eprintln!("Skipped: this Windows has no MP3 encoder");
        }
        found
    }

    fn format_of(variant: Variant) -> OutputFormat {
        match variant {
            Variant::Speech => OutputFormat::Mp3Speech,
            Variant::Standard => OutputFormat::Mp3Standard,
            Variant::High => OutputFormat::Mp3High,
        }
    }

    /// Encodes `job` and checks it as the conversion does; returns the file.
    fn encode(job: &Job, variant: Variant) -> PathBuf {
        let codec = Mp3Codec::new(variant);
        let output = job.wav.with_extension("mp3");
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        codec.encode(job, &output, &control).unwrap();
        codec.verify(job, &output, &control).unwrap();
        output
    }

    /// What the Windows MP3 decoder makes of a file: samples from -1 to 1,
    /// interleaved, `channels` per frame.
    fn decode(path: &Path, rate: u32, channels: u16) -> Vec<f32> {
        let session = Session::start().unwrap();
        let decoder: IMFTransform = session.create(&CLSID_MP3DecMediaObject).unwrap();
        let input = session.media_type().unwrap();
        // SAFETY: plain calls on live objects.
        unsafe {
            input
                .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)
                .unwrap();
            input.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_MP3).unwrap();
            input
                .SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, rate)
                .unwrap();
            input
                .SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, u32::from(channels))
                .unwrap();
            decoder.SetInputType(0, &input, 0).unwrap();
            let output = (0..)
                .map_while(|index| decoder.GetOutputAvailableType(0, index).ok())
                .find(|kind| {
                    kind.GetGUID(&MF_MT_SUBTYPE).ok() == Some(MFAudioFormat_PCM)
                        && kind.GetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE).ok() == Some(16)
                        && kind.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS).ok()
                            == Some(u32::from(channels))
                })
                .expect("the decoder gives 16-bit PCM");
            decoder.SetOutputType(0, &output, 0).unwrap();
        }
        let mut transform = Transform::start(&session, decoder).unwrap();
        let mut bytes = Vec::new();
        let mut sink = |data: &[u8]| -> io::Result<()> {
            bytes.extend_from_slice(data);
            Ok(())
        };
        for chunk in std::fs::read(path).unwrap().chunks(4096) {
            transform.push(chunk, None, &mut sink).unwrap();
        }
        transform.finish(&mut sink).unwrap();
        drop(transform);
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| f32::from(i16::from_le_bytes(pair)) / 32768.0)
            .collect()
    }

    fn channel(samples: &[f32], channels: usize, which: usize) -> Vec<f32> {
        samples
            .iter()
            .skip(which)
            .step_by(channels)
            .copied()
            .collect()
    }

    /// The frames of an MP3 file, read as the check reads them.
    fn frames_in(path: &Path) -> Vec<FrameHeader> {
        let bytes = std::fs::read(path).unwrap();
        let mut frames = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            let header =
                FrameHeader::parse(bytes[at..at + 4].try_into().unwrap()).expect("a frame");
            at += header.len() as usize;
            frames.push(header);
        }
        assert_eq!(at, bytes.len());
        frames
    }

    /// Upward zero crossings per second: the pitch of a pure tone.
    fn pitch(samples: &[f32], rate: u32) -> f32 {
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
            .count();
        crossings as f32 * rate as f32 / samples.len() as f32
    }

    /// A frame header as the encoder writes it (no CRC, no padding).
    fn header(mpeg: Mpeg, rate: u32, kbps: u32, mono: bool) -> [u8; 4] {
        let version = match mpeg {
            Mpeg::V1 => 3,
            Mpeg::V2 => 2,
            Mpeg::V2_5 => 0,
        };
        let rate_code = mpeg.rates().iter().position(|&r| r == rate).unwrap() as u32;
        let kbps_code = mpeg.bitrates().iter().position(|&k| k == kbps).unwrap() as u32;
        let mode = if mono { 3 } else { 1 };
        let value = (0x7ff << 21)
            | (version << 19)
            | (1 << 17)
            | (1 << 16)
            | (kbps_code << 12)
            | (rate_code << 10)
            | (mode << 6);
        value.to_be_bytes()
    }

    /// `count` frames of silence-sized payload with `head` as their header.
    fn fake_frames(head: [u8; 4], count: usize) -> Vec<u8> {
        let len = FrameHeader::parse(head).unwrap().len() as usize;
        let mut bytes = Vec::new();
        for _ in 0..count {
            bytes.extend_from_slice(&head);
            bytes.resize(bytes.len() + len - 4, 0);
        }
        bytes
    }

    fn job_for(path: &Path, frames: u64, sample_rate: u32, channels: u16) -> Job {
        Job {
            wav: path.with_extension("wav"),
            frames,
            sample_rate,
            channels,
            format: OutputFormat::Mp3Standard,
        }
    }

    #[test]
    fn variants_get_their_bitrates_and_the_highest_possible_at_low_rates() {
        let at = |variant, rate, channels| {
            let s = settings(variant, rate, channels).unwrap();
            (s.channels, s.kbps)
        };
        for rate in [44_100, 48_000] {
            assert_eq!(at(Variant::Speech, rate, 2), (1, SPEECH_KBPS));
            assert_eq!(at(Variant::Standard, rate, 2), (2, 128));
            assert_eq!(at(Variant::High, rate, 2), (2, HIGH_KBPS));
        }
        // 16 kHz: MP3 has at most 80 kbit/s in stereo and 48 in mono there.
        assert_eq!(at(Variant::Speech, 16_000, 2), (1, 48));
        assert_eq!(at(Variant::Standard, 16_000, 2), (2, 80));
        assert_eq!(at(Variant::High, 16_000, 2), (2, 80));
        // 8 kHz: at most 32 in stereo and 16 in mono.
        assert_eq!(at(Variant::Speech, 8_000, 2), (1, 16));
        assert_eq!(at(Variant::Standard, 8_000, 2), (2, 32));
        assert_eq!(at(Variant::High, 8_000, 2), (2, 32));
        // A mono recording stays mono, at the highest bitrate mono has.
        assert_eq!(at(Variant::Standard, 48_000, 1), (1, 128));
        assert_eq!(at(Variant::High, 48_000, 1), (1, 128));
        assert_eq!(at(Variant::High, 16_000, 1), (1, 48));
        assert_eq!(settings(Variant::Standard, 96_000, 2), None);
        assert_eq!(settings(Variant::Standard, 0, 2), None);
    }

    #[test]
    fn the_switch_turns_the_encoder_off_only_when_set_to_1() {
        assert!(switched_off(Some(OsStr::new("1"))));
        assert!(!switched_off(Some(OsStr::new("0"))));
        assert!(!switched_off(Some(OsStr::new(""))));
        assert!(!switched_off(None));
    }

    #[test]
    fn frame_headers_are_read_as_the_encoder_writes_them() {
        // 48 kHz, 128 kbit/s, joint stereo, as in a file of the encoder.
        let head = FrameHeader::parse([0xff, 0xfb, 0x94, 0x44]).unwrap();
        assert_eq!(
            head,
            FrameHeader {
                mpeg: Mpeg::V1,
                rate: 48_000,
                kbps: 128,
                mono: false,
                padding: false
            }
        );
        assert_eq!(head.len(), 384);
        // 44.1 kHz frames are 417 bytes, 418 with padding.
        let at_44 = FrameHeader {
            mpeg: Mpeg::V1,
            rate: 44_100,
            kbps: 128,
            mono: false,
            padding: false,
        };
        assert_eq!(at_44.len(), 417);
        assert_eq!(
            FrameHeader {
                padding: true,
                ..at_44
            }
            .len(),
            418
        );
        let low = FrameHeader::parse(header(Mpeg::V2_5, 8_000, 16, true)).unwrap();
        assert_eq!(
            (low.mpeg, low.rate, low.kbps, low.mono),
            (Mpeg::V2_5, 8_000, 16, true)
        );
        assert_eq!(low.len(), 144);
        assert_eq!(
            FrameHeader::parse(header(Mpeg::V2, 16_000, 80, false))
                .unwrap()
                .len(),
            360
        );
        // Not frames: no sync, Layer II, a bad bitrate code.
        assert_eq!(FrameHeader::parse(*b"ID3\x04"), None);
        assert_eq!(FrameHeader::parse([0xff, 0xfd, 0x94, 0x44]), None);
        assert_eq!(FrameHeader::parse([0xff, 0xfb, 0xf4, 0x44]), None);
    }

    #[test]
    fn check_accepts_only_whole_frames_in_the_right_format() {
        let dir = test_dir("mp3-frames");
        let path = dir.join("a.mp3");
        let codec = Mp3Codec::new(Variant::Standard);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        let good = fake_frames(header(Mpeg::V1, 48_000, 128, false), 10);
        // Ten frames hold 11520 samples: a recording of 7488 to 10368 frames.
        let job = job_for(&path, 9000, 48_000, 2);
        std::fs::write(&path, &good).unwrap();
        assert_eq!(codec.verify(&job, &path, &control), Ok(()));

        let mut bad: Vec<(&str, Vec<u8>)> = vec![
            ("cut inside a frame", good[..good.len() - 100].to_vec()),
            ("empty", Vec::new()),
        ];
        let mut tagged = b"ID3\x04\0\0\0\0\0\0".to_vec();
        tagged.extend_from_slice(&good);
        bad.push(("a tag in front", tagged));
        let mut trailing = good.clone();
        trailing.extend_from_slice(b"TAG");
        bad.push(("bytes after the last frame", trailing));
        let mut other_bitrate = good.clone();
        other_bitrate.extend(fake_frames(header(Mpeg::V1, 48_000, 320, false), 1));
        bad.push(("another bitrate", other_bitrate));
        let mut mono = good.clone();
        mono.extend(fake_frames(header(Mpeg::V1, 48_000, 128, true), 1));
        bad.push(("a mono frame", mono));
        let mut other_rate = good.clone();
        other_rate.extend(fake_frames(header(Mpeg::V1, 44_100, 128, false), 1));
        bad.push(("another rate", other_rate));
        for (what, bytes) in bad {
            std::fs::write(&path, &bytes).unwrap();
            let result = codec.verify(&job, &path, &control);
            assert!(
                matches!(result, Err(ConvertError::Failed(_))),
                "{what}: {result:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_holds_the_file_to_the_length_of_the_recording() {
        let dir = test_dir("mp3-length");
        let path = dir.join("a.mp3");
        let codec = Mp3Codec::new(Variant::Speech);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        // Ten MPEG-2 frames at 16 kHz: 5760 samples per channel.
        std::fs::write(&path, fake_frames(header(Mpeg::V2, 16_000, 48, true), 10)).unwrap();
        for (frames, fits) in [
            (5760 - 4032 - 1, false),
            (5760 - 4032, true),
            (4000, true),
            (5760 - 1152, true),
            (5760 - 1152 + 1, false),
            (5760, false),
        ] {
            let job = job_for(&path, frames, 16_000, 2);
            let result = codec.verify(&job, &path, &control);
            assert_eq!(result.is_ok(), fits, "{frames} frames: {result:?}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_encoder_keeps_the_wav_and_leaves_nothing_else() {
        let dir = test_dir("mp3-no-encoder");
        let job = tone_wav(
            &dir.join("mix_2026-11-26_14-03-09.wav"),
            48_000,
            2,
            0.5,
            0.0,
            (440.0, 440.0),
            OutputFormat::Mp3Standard,
        );
        let wav_before = std::fs::read(&job.wav).unwrap();
        // As on Windows N without the Media Feature Pack, or with the switch.
        let codec = Mp3Codec {
            variant: Variant::Standard,
            encoder_allowed: false,
        };
        let (cancel, report) = free_run();

        let result = convert_file(&job, &codec, &Control::new(cancel, report));

        assert_eq!(result, Err(ConvertError::NoEncoder));
        let names: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["mix_2026-11-26_14-03-09.wav"]);
        assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_encoder_offers_the_bitrates_the_table_lists() {
        if !have_encoder() {
            return;
        }
        let session = Session::start().unwrap();
        let encoder: IMFTransform = session.create(&MP3ACMCodecWrapper).unwrap();
        let types: Vec<(u32, u32, u32)> = (0..)
            .map_while(|index| unsafe { encoder.GetOutputAvailableType(0, index) }.ok())
            .map(|kind| audio_format(&kind))
            .collect();
        for rate in [
            8_000, 11_025, 12_000, 16_000, 22_050, 24_000, 32_000, 44_100, 48_000,
        ] {
            let mpeg = Mpeg::for_rate(rate).unwrap();
            for channels in [1u16, 2] {
                // The encoder also lists 18 and 20 kbit/s, which no MP3 frame
                // header can say; the table leaves them out.
                let mut offered: Vec<u32> = types
                    .iter()
                    .filter(|t| (t.0, t.1) == (rate, u32::from(channels)))
                    .map(|t| t.2 * 8 / 1000)
                    .filter(|kbps| mpeg.bitrates().contains(kbps))
                    .collect();
                offered.sort_unstable();
                assert_eq!(
                    offered,
                    offered_kbps(rate, channels),
                    "{rate} Hz, {channels} ch"
                );
            }
        }
        drop(encoder);
    }

    #[test]
    fn every_rate_and_variant_gives_a_file_as_long_as_the_recording() {
        if !have_encoder() {
            return;
        }
        let dir = test_dir("mp3-rates");
        for rate in [8_000u32, 16_000, 44_100, 48_000] {
            for variant in [Variant::Speech, Variant::Standard, Variant::High] {
                let what = format!("{rate} Hz {variant:?}");
                // Half a second of silence, then a 1 kHz tone: 2.3 s in all,
                // not a whole number of frames.
                let wav = dir.join(format!("{rate}-{variant:?}.wav"));
                let job = tone_wav(
                    &wav,
                    rate,
                    2,
                    2.3,
                    0.5,
                    (1000.0, 1000.0),
                    format_of(variant),
                );
                let output = encode(&job, variant);
                let expected = settings(variant, rate, 2).unwrap();

                let frames = frames_in(&output);
                assert!(
                    frames.iter().all(|frame| frame.kbps == expected.kbps
                        && frame.mono == (expected.channels == 1)
                        && frame.rate == rate),
                    "{what}"
                );
                let mpeg = Mpeg::for_rate(rate).unwrap();
                let samples = frames.len() as u64 * mpeg.samples_per_frame();
                assert!(
                    (job.frames + 1729..=job.frames + 2880).contains(&samples),
                    "{what}: {samples} samples for {}",
                    job.frames
                );
                // A player that works the length out from the size and the
                // bitrate (the file has no header with the length) gets the
                // length of the frames: padding makes the bitrate exact.
                let bytes = std::fs::metadata(&output).unwrap().len();
                let by_bitrate = bytes as f64 * 8.0 / f64::from(expected.kbps * 1000);
                let length = samples as f64 / f64::from(rate);
                assert!(
                    (by_bitrate - length).abs() < 0.001,
                    "{what}: {by_bitrate} s by the bitrate, {length} s of frames"
                );

                let channels = usize::from(expected.channels);
                let played = decode(&output, rate, expected.channels);
                assert_eq!(
                    played.len() / channels,
                    samples as usize - DECODER_SKIPS,
                    "{what}"
                );
                let sound = channel(&played, channels, 0);
                // The tone starts where it started in the recording, moved by
                // the encoder's delay only.
                let half_second = rate as i64 / 2;
                let start = sound.iter().position(|s| s.abs() >= 0.2).unwrap() as i64;
                assert!(
                    (start - half_second - SOUND_STARTS_AT).abs() <= 8,
                    "{what}: tone at {start}"
                );
                // The last of the recording is in the file: the tone lasts to
                // the end of the recording.
                let end = sound.iter().rposition(|s| s.abs() >= 0.2).unwrap() as i64;
                let recording_end = job.frames as i64 + SOUND_STARTS_AT;
                assert!(
                    (end - recording_end).abs() <= 8,
                    "{what}: tone ends at {end}"
                );
                // Still 1 kHz, so the rate was not mixed up.
                let tone = &sound[rate as usize..2 * rate as usize];
                let hz = pitch(tone, rate);
                assert!((hz - 1000.0).abs() < 10.0, "{what}: {hz} Hz");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn standard_keeps_the_two_channels_apart_and_speech_mixes_them() {
        if !have_encoder() {
            return;
        }
        let dir = test_dir("mp3-channels");
        let job = tone_wav(
            &dir.join("standard.wav"),
            48_000,
            2,
            2.0,
            0.0,
            (1000.0, 400.0),
            OutputFormat::Mp3Standard,
        );
        let played = decode(&encode(&job, Variant::Standard), 48_000, 2);
        let left = pitch(&channel(&played, 2, 0)[24_000..72_000], 48_000);
        let right = pitch(&channel(&played, 2, 1)[24_000..72_000], 48_000);
        assert!((left - 1000.0).abs() < 10.0, "left: {left} Hz");
        assert!((right - 400.0).abs() < 10.0, "right: {right} Hz");

        // Speech: one channel that holds both sides at half their level.
        let job = tone_wav(
            &dir.join("speech.wav"),
            48_000,
            2,
            2.0,
            0.0,
            (1000.0, 0.0),
            OutputFormat::Mp3Speech,
        );
        let output = encode(&job, Variant::Speech);
        assert!(frames_in(&output).iter().all(|frame| frame.mono));
        let mono = decode(&output, 48_000, 1);
        let middle = &mono[24_000..72_000];
        assert!((pitch(middle, 48_000) - 1000.0).abs() < 10.0);
        let peak = middle.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
        assert!((0.2..0.32).contains(&peak), "peak {peak}");

        // A mono recording stays mono, also in a stereo variant.
        let job = tone_wav(
            &dir.join("mono.wav"),
            16_000,
            1,
            1.0,
            0.0,
            (500.0, 0.0),
            OutputFormat::Mp3High,
        );
        let output = encode(&job, Variant::High);
        assert!(frames_in(&output)
            .iter()
            .all(|frame| frame.mono && frame.kbps == 48));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_and_very_short_recordings_give_files_a_player_opens() {
        if !have_encoder() {
            return;
        }
        let dir = test_dir("mp3-short");
        for (name, seconds) in [("empty", 0.0), ("click", 0.001), ("frame", 0.024)] {
            let job = tone_wav(
                &dir.join(format!("{name}.wav")),
                48_000,
                2,
                seconds,
                0.0,
                (1000.0, 1000.0),
                OutputFormat::Mp3Standard,
            );
            let output = encode(&job, Variant::Standard);
            let frames = frames_in(&output).len();
            assert!(frames >= 2, "{name}: {frames} frames");
            assert!(!decode(&output, 48_000, 2).is_empty(), "{name}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancelling_keeps_the_wav_and_removes_the_temporary_file() {
        if !have_encoder() {
            return;
        }
        let dir = test_dir("mp3-cancel");
        let job = tone_wav(
            &dir.join("mic_2026-11-26_14-03-09.wav"),
            44_100,
            2,
            5.0,
            0.0,
            (440.0, 440.0),
            OutputFormat::Mp3High,
        );
        let wav_before = std::fs::read(&job.wav).unwrap();
        let codec = Mp3Codec::new(Variant::High);
        // The app starts closing once the encoder has reported some progress.
        let cancel = AtomicBool::new(false);
        let report = |done: f32| {
            if done > 0.0 {
                cancel.store(true, Ordering::Relaxed);
            }
        };

        let result = convert_file(&job, &codec, &Control::new(&cancel, &report));

        assert_eq!(result, Err(ConvertError::Cancelled));
        assert!(!temp_path(&job.wav.with_extension("mp3")).exists());
        assert!(!job.wav.with_extension("mp3").exists());
        assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoding_fails_when_the_wav_is_not_the_recording() {
        if !have_encoder() {
            return;
        }
        let dir = test_dir("mp3-mismatch");
        let job = tone_wav(
            &dir.join("a.wav"),
            48_000,
            2,
            0.5,
            0.0,
            (500.0, 500.0),
            OutputFormat::Mp3Standard,
        );
        let codec = Mp3Codec::new(Variant::Standard);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        let output = dir.join("a.mp3");
        // The queue says 44.1 kHz, the file is 48 kHz.
        let other_rate = Job {
            sample_rate: 44_100,
            ..job.clone()
        };
        assert!(codec.encode(&other_rate, &output, &control).is_err());
        // More frames counted than the file holds.
        let longer = Job {
            frames: job.frames + 10,
            ..job.clone()
        };
        assert!(codec.encode(&longer, &output, &control).is_err());
        let missing = Job {
            wav: dir.join("missing.wav"),
            ..job.clone()
        };
        assert!(codec.encode(&missing, &output, &control).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

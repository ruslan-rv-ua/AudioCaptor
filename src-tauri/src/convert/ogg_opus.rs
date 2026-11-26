//! Ogg Opus files (`.opus`): libopus through the `opus` crate, the Ogg container
//! through the `ogg` crate, the layout of RFC 7845. The encoder settings and the
//! two variants are those of decision-8.

use super::wav::WavPcm;
use super::{Codec, Control, ConvertError, Job};
use ogg::{PacketReader, PacketWriteEndInfo, PacketWriter};
use opus::{Application, Bitrate, Channels, Decoder, Encoder};
use rubato::{FftFixedIn, Resampler};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// The rate of an Opus decoder's output and of the positions in the Ogg stream,
/// whatever rate the file was recorded at.
const OPUS_RATE: u32 = 48_000;
/// One packet holds 20 ms.
const PACKETS_PER_SECOND: u32 = 50;
/// libopus asks for room for 4000 bytes: no packet is larger.
const MAX_PACKET_BYTES: usize = 4000;
/// The longest Opus packet holds 120 ms: 5760 samples per channel at 48 kHz.
const MAX_PACKET_SAMPLES: usize = 5760;

/// The two entries of the format list that write Opus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variant {
    /// "Opus - speech": mono, the smallest files.
    Speech,
    /// "Opus - music": stereo.
    Music,
}

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub channels: u16,
    /// Bits per second.
    pub bitrate: i32,
}

/// The settings of a variant for a recording at `sample_rate`. At 16 and 8 kHz
/// the sample rate itself caps the quality, so the bitrate is lower: more bits
/// would add nothing.
pub fn settings(variant: Variant, sample_rate: u32) -> Settings {
    let (channels, full, at_16_khz, at_8_khz) = match variant {
        Variant::Speech => (1, 32_000, 24_000, 16_000),
        Variant::Music => (2, 96_000, 48_000, 32_000),
    };
    let bitrate = match sample_rate {
        0..=8_000 => at_8_khz,
        8_001..=16_000 => at_16_khz,
        _ => full,
    };
    Settings { channels, bitrate }
}

/// The rate the encoder runs at. libopus takes 8, 12, 16, 24 and 48 kHz; any
/// other rate (44.1 kHz) is resampled to 48 kHz first.
fn encoder_rate(sample_rate: u32) -> u32 {
    match sample_rate {
        8_000 | 12_000 | 16_000 | 24_000 | 48_000 => sample_rate,
        _ => OPUS_RATE,
    }
}

/// How many frames at rate `to` last as long as `frames` at rate `from`,
/// rounded up to a whole frame.
fn frames_at(frames: u64, from: u32, to: u32) -> u64 {
    (frames * u64::from(to)).div_ceil(u64::from(from.max(1)))
}

fn opus_channels(channels: usize) -> Channels {
    if channels == 1 {
        Channels::Mono
    } else {
        Channels::Stereo
    }
}

impl From<opus::Error> for ConvertError {
    fn from(e: opus::Error) -> Self {
        ConvertError::failed(e)
    }
}

impl From<ogg::OggReadError> for ConvertError {
    fn from(e: ogg::OggReadError) -> Self {
        ConvertError::failed(format!("the Opus file is damaged: {e}"))
    }
}

pub struct OpusCodec {
    variant: Variant,
}

impl OpusCodec {
    pub fn new(variant: Variant) -> Self {
        Self { variant }
    }
}

impl Codec for OpusCodec {
    fn extension(&self) -> &'static str {
        "opus"
    }

    fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        let settings = settings(self.variant, job.sample_rate);
        let mut input = PcmInput::open(job, usize::from(settings.channels))?;
        let (rate, channels) = (input.rate, input.channels);

        let mut encoder = Encoder::new(rate, opus_channels(channels), Application::Audio)?;
        encoder.set_bitrate(Bitrate::Bits(settings.bitrate))?;
        encoder.set_vbr(true)?;
        encoder.set_vbr_constraint(false)?;
        encoder.set_complexity(10)?;

        // Positions in the stream count samples at 48 kHz: `ticks` per input frame.
        let ticks = u64::from(OPUS_RATE / rate);
        let lookahead = encoder.get_lookahead()?.max(0) as u64;
        // A player drops this many samples at the start: the encoder's delay.
        let pre_skip = u16::try_from(lookahead * ticks).map_err(ConvertError::failed)?;
        let frame = (rate / PACKETS_PER_SECOND) as usize;
        // The position of the last page tells a player where the recording ends,
        // to the sample, so the file lasts exactly as long as the recording.
        let end = u64::from(pre_skip) + input.total * ticks;
        // Silence after the recording pushes its last samples through the delay.
        let packets = (input.total + lookahead).div_ceil(frame as u64).max(1);

        let serial = stream_serial();
        let mut stream = PacketWriter::new(BufWriter::new(File::create(output)?));
        // Each of the two header packets has a page to itself.
        stream.write_packet(
            opus_head(channels, pre_skip, job.sample_rate),
            serial,
            PacketWriteEndInfo::EndPage,
            0,
        )?;
        stream.write_packet(opus_tags(), serial, PacketWriteEndInfo::EndPage, 0)?;

        let mut pcm = vec![0.0f32; frame * channels];
        let mut packet = vec![0u8; MAX_PACKET_BYTES];
        let mut position = 0u64;
        for index in 0..packets {
            control.check()?;
            input.fill(&mut pcm)?;
            let len = encoder.encode_float(&pcm, &mut packet)?;
            let last = index + 1 == packets;
            position = if last {
                end
            } else {
                position + frame as u64 * ticks
            };
            let page = if last {
                PacketWriteEndInfo::EndStream
            } else if (index + 1) % u64::from(PACKETS_PER_SECOND) == 0 {
                // A page a second keeps seeking in a player quick.
                PacketWriteEndInfo::EndPage
            } else {
                PacketWriteEndInfo::NormalPacket
            };
            stream.write_packet(packet[..len].to_vec(), serial, page, position)?;
            if index % u64::from(PACKETS_PER_SECOND) == 0 {
                control.progress(index as f32 / packets as f32);
            }
        }

        // On the disk before the WAV may be deleted.
        let mut file = stream.into_inner();
        file.flush()?;
        file.get_ref().sync_all()?;
        Ok(())
    }

    fn verify(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        let expected = frames_at(job.frames, job.sample_rate, OPUS_RATE);
        let decoded = decode_file(output, control, expected, |_| {})?;
        if decoded.playable != expected {
            return Err(ConvertError::failed(format!(
                "the Opus file lasts {} samples at 48 kHz, the recording {expected}",
                decoded.playable
            )));
        }
        Ok(())
    }
}

/// The identification header: the first packet of the stream.
fn opus_head(channels: usize, pre_skip: u16, input_rate: u32) -> Vec<u8> {
    let mut head = b"OpusHead".to_vec();
    head.push(1); // version
    head.push(channels as u8);
    head.extend_from_slice(&pre_skip.to_le_bytes());
    // The rate of the recording; a player may use it, the stream does not.
    head.extend_from_slice(&input_rate.to_le_bytes());
    head.extend_from_slice(&0i16.to_le_bytes()); // output gain
    head.push(0); // channel mapping family 0: mono or stereo
    head
}

/// The comment header: the encoder library and the program that wrote the file.
fn opus_tags() -> Vec<u8> {
    let vendor = opus::version();
    let encoder = concat!("ENCODER=AudioCaptor ", env!("CARGO_PKG_VERSION"));
    let mut tags = b"OpusTags".to_vec();
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor.as_bytes());
    tags.extend_from_slice(&1u32.to_le_bytes());
    tags.extend_from_slice(&(encoder.len() as u32).to_le_bytes());
    tags.extend_from_slice(encoder.as_bytes());
    tags
}

/// The number that marks the pages of one stream. A file holds one stream, so
/// any number will do; a varying one keeps streams apart if files are joined.
fn stream_serial() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.subsec_nanos() ^ since.as_secs() as u32)
        .unwrap_or(0x4155_4443)
}

/// The job's WAV as the encoder takes it: f32 samples at a rate libopus accepts,
/// mixed down to mono for the speech variant.
struct PcmInput {
    wav: WavPcm,
    wav_channels: usize,
    channels: usize,
    rate: u32,
    /// Frames at `rate` in the whole recording.
    total: u64,
    /// Frames handed out so far.
    delivered: u64,
    resampling: Option<Resampling>,
    raw: Vec<i16>,
    native: Vec<f32>,
}

impl PcmInput {
    fn open(job: &Job, channels_wanted: usize) -> Result<Self, ConvertError> {
        let wav = WavPcm::open(&job.wav, job.frames)?;
        if wav.sample_rate() != job.sample_rate || wav.channels() != job.channels {
            return Err(ConvertError::failed(format!(
                "the WAV file is {} Hz with {} channels, the recording was {} Hz with {}",
                wav.sample_rate(),
                wav.channels(),
                job.sample_rate,
                job.channels
            )));
        }
        let wav_channels = usize::from(wav.channels());
        let channels = channels_wanted.clamp(1, wav_channels);
        let rate = encoder_rate(job.sample_rate);
        let resampling = if rate == job.sample_rate {
            None
        } else {
            Some(Resampling::new(job.sample_rate, rate, channels)?)
        };
        Ok(Self {
            wav,
            wav_channels,
            channels,
            rate,
            total: frames_at(job.frames, job.sample_rate, rate),
            delivered: 0,
            resampling,
            raw: Vec::new(),
            native: Vec::new(),
        })
    }

    /// Fills `out` (interleaved, a whole number of frames) with the next frames
    /// of the recording, and with silence once the recording has ended.
    fn fill(&mut self, out: &mut [f32]) -> Result<(), ConvertError> {
        let want = out.len() / self.channels;
        let mut filled = 0;
        while filled < want && self.delivered < self.total {
            let room = (want - filled).min((self.total - self.delivered) as usize);
            let target = &mut out[filled * self.channels..(filled + room) * self.channels];
            let frames = match self.resampling.as_mut() {
                Some(resampling) => {
                    if resampling.is_empty() {
                        let chunk = resampling.chunk;
                        let frames = read_native(
                            &mut self.wav,
                            self.wav_channels,
                            self.channels,
                            chunk,
                            &mut self.raw,
                            &mut self.native,
                        )?;
                        resampling.push(&self.native, frames)?;
                    }
                    resampling.take(target)
                }
                None => {
                    let frames = read_native(
                        &mut self.wav,
                        self.wav_channels,
                        self.channels,
                        room,
                        &mut self.raw,
                        &mut self.native,
                    )?;
                    if frames == 0 {
                        // Cannot happen: `total` frames are in the WAV.
                        break;
                    }
                    target[..frames * self.channels].copy_from_slice(&self.native);
                    frames
                }
            };
            filled += frames;
            self.delivered += frames as u64;
        }
        out[filled * self.channels..].fill(0.0);
        Ok(())
    }
}

/// Reads up to `max` frames of the WAV into `out` as f32 samples with `channels`
/// channels, averaging the WAV's channels when it has more. Returns the frames.
fn read_native(
    wav: &mut WavPcm,
    wav_channels: usize,
    channels: usize,
    max: usize,
    raw: &mut Vec<i16>,
    out: &mut Vec<f32>,
) -> Result<usize, ConvertError> {
    let frames = wav.read(max, raw)?;
    out.clear();
    if channels == wav_channels {
        out.extend(raw.iter().map(|&sample| f32::from(sample) / 32768.0));
    } else {
        out.extend(raw.chunks_exact(wav_channels).map(|frame| {
            let sum: i32 = frame.iter().map(|&sample| i32::from(sample)).sum();
            sum as f32 / (32768.0 * wav_channels as f32)
        }));
    }
    Ok(frames)
}

/// Changes the sample rate of the recording to one libopus accepts (44.1 kHz to
/// 48 kHz), a chunk at a time.
struct Resampling {
    fft: FftFixedIn<f32>,
    channels: usize,
    /// Input frames per chunk.
    chunk: usize,
    input: Vec<Vec<f32>>,
    /// Resampled samples not yet taken, interleaved, from `taken` on.
    ready: Vec<f32>,
    taken: usize,
    /// Frames of the resampler's own delay still to drop from its output, so
    /// the sound stays where it was in time.
    skip: usize,
}

impl Resampling {
    fn new(from: u32, to: u32, channels: usize) -> Result<Self, ConvertError> {
        // The resampler works on blocks of `unit` input frames (147 for 44.1 to
        // 48 kHz); a chunk of whole blocks gives the same output every time.
        let unit = (from / gcd(from, to)) as usize;
        let chunk = unit * 2048usize.div_ceil(unit);
        let fft = FftFixedIn::<f32>::new(from as usize, to as usize, chunk, 1, channels)
            .map_err(ConvertError::failed)?;
        let skip = fft.output_delay();
        Ok(Self {
            fft,
            channels,
            chunk,
            input: vec![vec![0.0; chunk]; channels],
            ready: Vec::new(),
            taken: 0,
            skip,
        })
    }

    fn is_empty(&self) -> bool {
        self.taken == self.ready.len()
    }

    /// Resamples the next chunk: `frames` frames from `samples` (interleaved),
    /// padded with silence when the recording has ended.
    fn push(&mut self, samples: &[f32], frames: usize) -> Result<(), ConvertError> {
        for (channel, input) in self.input.iter_mut().enumerate() {
            for (frame, slot) in input.iter_mut().enumerate() {
                *slot = if frame < frames {
                    samples[frame * self.channels + channel]
                } else {
                    0.0
                };
            }
        }
        let output = self
            .fft
            .process(&self.input, None)
            .map_err(ConvertError::failed)?;
        let frames_out = output.first().map_or(0, Vec::len);
        let dropped = self.skip.min(frames_out);
        self.skip -= dropped;
        self.ready.clear();
        self.taken = 0;
        for frame in dropped..frames_out {
            for channel in &output {
                self.ready.push(channel[frame]);
            }
        }
        Ok(())
    }

    /// Moves resampled frames into `out` and returns how many.
    fn take(&mut self, out: &mut [f32]) -> usize {
        let samples = out.len().min(self.ready.len() - self.taken);
        out[..samples].copy_from_slice(&self.ready[self.taken..self.taken + samples]);
        self.taken += samples;
        samples / self.channels
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a.max(1)
    } else {
        gcd(b, a % b)
    }
}

/// What an Ogg Opus file turned out to hold.
#[derive(Debug)]
struct DecodedFile {
    #[cfg_attr(not(test), allow(dead_code))]
    channels: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    input_rate: u32,
    #[cfg_attr(not(test), allow(dead_code))]
    pre_skip: u64,
    /// Samples per channel at 48 kHz that a player plays: the position of the
    /// last page less the pre-skip.
    playable: u64,
}

/// Decodes a whole Ogg Opus file and fails if anything in it is wrong: a page
/// with a bad checksum, a packet libopus does not decode, a missing end. The
/// decoded samples (48 kHz, interleaved, the pre-skip included) go to `sink`.
/// `expected` is only the length to measure progress against.
fn decode_file(
    path: &Path,
    control: &Control,
    expected: u64,
    mut sink: impl FnMut(&[f32]),
) -> Result<DecodedFile, ConvertError> {
    let mut stream = PacketReader::new(BufReader::new(File::open(path)?));

    let head = stream.read_packet_expected()?.data;
    if head.len() < 19 || &head[..8] != b"OpusHead" || head[8] >> 4 != 0 || head[18] != 0 {
        return Err(ConvertError::failed("the Opus file has no valid header"));
    }
    let channels = usize::from(head[9]);
    if !(1..=2).contains(&channels) {
        return Err(ConvertError::failed("the Opus file has no valid header"));
    }
    let pre_skip = u64::from(u16::from_le_bytes([head[10], head[11]]));
    let input_rate = u32::from_le_bytes([head[12], head[13], head[14], head[15]]);
    let tags = stream.read_packet_expected()?.data;
    if tags.len() < 8 || &tags[..8] != b"OpusTags" {
        return Err(ConvertError::failed("the Opus file has no comment header"));
    }

    let mut decoder = Decoder::new(OPUS_RATE, opus_channels(channels))?;
    let mut pcm = vec![0.0f32; MAX_PACKET_SAMPLES * channels];
    let mut decoded = 0u64;
    let mut position = 0u64;
    let mut ended = false;
    let length = (expected + pre_skip).max(1) as f32;
    while let Some(packet) = stream.read_packet()? {
        control.check()?;
        if packet.data.is_empty() {
            // libopus would take an empty packet for a lost one and make up sound.
            return Err(ConvertError::failed("the Opus file has an empty packet"));
        }
        let samples = decoder.decode_float(&packet.data, &mut pcm, false)?;
        decoded += samples as u64;
        sink(&pcm[..samples * channels]);
        if packet.last_in_page() {
            position = packet.absgp_page();
            control.progress(position as f32 / length);
        }
        ended = packet.last_in_stream();
    }
    if !ended {
        return Err(ConvertError::failed("the Opus file is cut short"));
    }
    if position < pre_skip || decoded < position {
        return Err(ConvertError::failed(
            "the Opus file holds less sound than it says",
        ));
    }
    Ok(DecodedFile {
        channels,
        input_rate,
        pre_skip,
        playable: position - pre_skip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::OutputFormat;
    use crate::convert::test_util::{free_run, test_dir, tone_wav};
    use std::path::PathBuf;

    /// What a player plays of an Ogg Opus file: 48 kHz, interleaved.
    struct Played {
        info: DecodedFile,
        samples: Vec<f32>,
    }

    impl Played {
        fn channel(&self, channel: usize) -> Vec<f32> {
            self.samples
                .iter()
                .skip(channel)
                .step_by(self.info.channels)
                .copied()
                .collect()
        }
    }

    fn play(path: &Path) -> Played {
        let (cancel, report) = free_run();
        let mut all = Vec::new();
        let info = decode_file(path, &Control::new(cancel, report), 0, |samples| {
            all.extend_from_slice(samples)
        })
        .unwrap();
        let start = info.pre_skip as usize * info.channels;
        let end = start + info.playable as usize * info.channels;
        Played {
            samples: all[start..end].to_vec(),
            info,
        }
    }

    /// Encodes `job` and checks it as the conversion does; returns the file.
    fn encode(job: &Job, variant: Variant) -> PathBuf {
        let codec = OpusCodec::new(variant);
        let output = job.wav.with_extension("opus");
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        codec.encode(job, &output, &control).unwrap();
        codec.verify(job, &output, &control).unwrap();
        output
    }

    /// The first sample whose level reaches `level`.
    fn onset(samples: &[f32], level: f32) -> usize {
        samples
            .iter()
            .position(|sample| sample.abs() >= level)
            .expect("the tone is in the file")
    }

    /// Upward zero crossings per second in `samples`: the pitch of a pure tone.
    fn pitch(samples: &[f32]) -> f32 {
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
            .count();
        crossings as f32 * OPUS_RATE as f32 / samples.len() as f32
    }

    #[test]
    fn settings_are_those_of_decision_8() {
        let speech = |rate| settings(Variant::Speech, rate);
        let music = |rate| settings(Variant::Music, rate);
        for rate in [44_100, 48_000] {
            assert_eq!(
                speech(rate),
                Settings {
                    channels: 1,
                    bitrate: 32_000
                }
            );
            assert_eq!(
                music(rate),
                Settings {
                    channels: 2,
                    bitrate: 96_000
                }
            );
        }
        assert_eq!(speech(16_000).bitrate, 24_000);
        assert_eq!(speech(8_000).bitrate, 16_000);
        assert_eq!(music(16_000).bitrate, 48_000);
        assert_eq!(music(8_000).bitrate, 32_000);
        assert_eq!((speech(8_000).channels, music(8_000).channels), (1, 2));
    }

    #[test]
    fn only_44_1_khz_is_resampled() {
        assert_eq!(encoder_rate(8_000), 8_000);
        assert_eq!(encoder_rate(16_000), 16_000);
        assert_eq!(encoder_rate(48_000), 48_000);
        assert_eq!(encoder_rate(44_100), 48_000);
        // An hour is the same number of 48 kHz samples at every rate.
        for rate in [8_000u32, 16_000, 44_100, 48_000] {
            assert_eq!(
                frames_at(u64::from(rate) * 3600, rate, OPUS_RATE),
                172_800_000
            );
        }
        assert_eq!(frames_at(1, 44_100, OPUS_RATE), 2);
        assert_eq!(frames_at(0, 44_100, OPUS_RATE), 0);
    }

    #[test]
    fn every_profile_rate_gives_a_file_as_long_as_the_recording() {
        let dir = test_dir("rates");
        for rate in [8_000u32, 16_000, 44_100, 48_000] {
            for (variant, format, channels) in [
                (Variant::Speech, OutputFormat::OpusSpeech, 1),
                (Variant::Music, OutputFormat::OpusMusic, 2),
            ] {
                let what = format!("{rate} Hz {variant:?}");
                // Half a second of silence, then a 1 kHz tone: 2.3 s in all, not
                // a whole number of packets.
                let wav = dir.join(format!("{rate}-{variant:?}.wav"));
                let job = tone_wav(&wav, rate, 2, 2.3, 0.5, (1000.0, 1000.0), format);
                let played = play(&encode(&job, variant));

                assert_eq!(played.info.channels, channels, "{what}");
                assert_eq!(played.info.input_rate, rate, "{what}");
                // The same duration as the recording, to the sample.
                assert_eq!(
                    played.info.playable,
                    frames_at(job.frames, rate, OPUS_RATE),
                    "{what}"
                );
                assert_eq!(played.info.playable, 110_400, "{what}");

                let sound = played.channel(0);
                // The tone starts where it started in the recording (0.5 s is
                // sample 24000), so neither the encoder's delay nor the
                // resampler's shifted the sound.
                let start = onset(&sound, 0.2) as i64;
                assert!((start - 24_000).abs() < 480, "{what}: tone at {start}");
                // And it is still 1 kHz, so the sample rate was not mixed up.
                let hz = pitch(&sound[48_000..96_000]);
                assert!((hz - 1000.0).abs() < 10.0, "{what}: {hz} Hz");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn music_keeps_the_two_channels_apart_and_speech_mixes_them() {
        let dir = test_dir("channels");
        let tones = (1000.0, 400.0);
        let job = tone_wav(
            &dir.join("music.wav"),
            48_000,
            2,
            2.0,
            0.0,
            tones,
            OutputFormat::OpusMusic,
        );
        let played = play(&encode(&job, Variant::Music));
        let left = pitch(&played.channel(0)[24_000..72_000]);
        let right = pitch(&played.channel(1)[24_000..72_000]);
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
            OutputFormat::OpusSpeech,
        );
        let played = play(&encode(&job, Variant::Speech));
        assert_eq!(played.info.channels, 1);
        let mono = &played.samples[24_000..72_000];
        assert!((pitch(mono) - 1000.0).abs() < 10.0);
        let peak = mono
            .iter()
            .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
        assert!((0.2..0.32).contains(&peak), "peak {peak}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_and_very_short_recordings_give_valid_files() {
        let dir = test_dir("short");
        for (name, seconds) in [("empty", 0.0), ("click", 0.001), ("packet", 0.02)] {
            let job = tone_wav(
                &dir.join(format!("{name}.wav")),
                48_000,
                2,
                seconds,
                0.0,
                (1000.0, 1000.0),
                OutputFormat::OpusMusic,
            );
            let played = play(&encode(&job, Variant::Music));
            assert_eq!(played.info.playable, job.frames, "{name}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_the_file_is_not_as_long_as_the_recording() {
        let dir = test_dir("duration");
        let job = tone_wav(
            &dir.join("a.wav"),
            16_000,
            2,
            1.0,
            0.0,
            (500.0, 500.0),
            OutputFormat::OpusSpeech,
        );
        let output = encode(&job, Variant::Speech);
        let codec = OpusCodec::new(Variant::Speech);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        // One frame more or less in the recording is already a mismatch.
        for frames in [job.frames + 1, job.frames - 1] {
            let other = Job {
                frames,
                ..job.clone()
            };
            let result = codec.verify(&other, &output, &control);
            assert!(matches!(result, Err(ConvertError::Failed(_))), "{result:?}");
        }
        assert_eq!(codec.verify(&job, &output, &control), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_for_a_cut_or_damaged_file() {
        let dir = test_dir("damaged");
        let job = tone_wav(
            &dir.join("a.wav"),
            48_000,
            2,
            3.0,
            0.0,
            (500.0, 500.0),
            OutputFormat::OpusMusic,
        );
        let output = encode(&job, Variant::Music);
        let good = std::fs::read(&output).unwrap();
        let codec = OpusCodec::new(Variant::Music);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);

        // Cut in the middle of a page, and cut exactly between two pages.
        let second_page = good
            .windows(4)
            .enumerate()
            .filter(|(_, bytes)| bytes == b"OggS")
            .map(|(at, _)| at)
            .nth(3)
            .unwrap();
        for cut in [good.len() / 2, second_page] {
            std::fs::write(&output, &good[..cut]).unwrap();
            let result = codec.verify(&job, &output, &control);
            assert!(matches!(result, Err(ConvertError::Failed(_))), "{result:?}");
        }

        // One flipped bit in the sound.
        let mut flipped = good.clone();
        flipped[good.len() / 2] ^= 0x10;
        std::fs::write(&output, &flipped).unwrap();
        let result = codec.verify(&job, &output, &control);
        assert!(matches!(result, Err(ConvertError::Failed(_))), "{result:?}");

        std::fs::write(&output, b"").unwrap();
        assert!(codec.verify(&job, &output, &control).is_err());
        std::fs::write(&output, &good).unwrap();
        assert_eq!(codec.verify(&job, &output, &control), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoding_fails_when_the_wav_is_not_the_recording() {
        let dir = test_dir("mismatch");
        let job = tone_wav(
            &dir.join("a.wav"),
            48_000,
            2,
            0.5,
            0.0,
            (500.0, 500.0),
            OutputFormat::OpusMusic,
        );
        let codec = OpusCodec::new(Variant::Music);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        let output = dir.join("a.opus");
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

    #[test]
    fn headers_follow_rfc_7845() {
        let head = opus_head(2, 312, 44_100);
        assert_eq!(head.len(), 19);
        assert_eq!(&head[..8], b"OpusHead");
        assert_eq!(head[8], 1);
        assert_eq!(head[9], 2);
        assert_eq!(u16::from_le_bytes([head[10], head[11]]), 312);
        assert_eq!(
            u32::from_le_bytes([head[12], head[13], head[14], head[15]]),
            44_100
        );
        assert_eq!(&head[16..], [0, 0, 0]);

        let tags = opus_tags();
        assert_eq!(&tags[..8], b"OpusTags");
        let vendor_len = u32::from_le_bytes([tags[8], tags[9], tags[10], tags[11]]) as usize;
        let vendor = std::str::from_utf8(&tags[12..12 + vendor_len]).unwrap();
        assert!(vendor.starts_with("libopus"), "{vendor}");
        let rest = &tags[12 + vendor_len..];
        assert_eq!(u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]), 1);
        let comment_len = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]) as usize;
        assert_eq!(rest.len(), 8 + comment_len);
        let comment = std::str::from_utf8(&rest[8..]).unwrap();
        assert!(comment.starts_with("ENCODER=AudioCaptor "), "{comment}");
    }
}

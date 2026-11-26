//! Turns a finished WAV recording into the compressed format of its profile.
//!
//! Every format goes the same way (decision-7, decision-8): the WAV is encoded
//! into a temporary file, that file is read back and must decode and last as
//! long as the recording, then it gets its final name and the WAV is deleted.
//! If a step fails, the temporary file is removed and the WAV stays. A format
//! adds only its `Codec`; `queue` runs the conversions one file at a time.

pub mod media_foundation;
pub mod mp3;
pub mod ogg_opus;
pub mod queue;
pub mod wav;

use crate::audio::types::OutputFormat;
use crate::audio::writer::RecordedFile;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// One WAV file to convert.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub wav: PathBuf,
    /// Frames the recorder wrote to the WAV (`RecordedFile::frames`). The
    /// conversion reads this many and checks the new file against it.
    pub frames: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: OutputFormat,
}

impl Job {
    pub fn new(file: RecordedFile, format: OutputFormat) -> Self {
        Self {
            wav: file.path,
            frames: file.frames,
            sample_rate: file.sample_rate,
            channels: file.channels,
            format,
        }
    }

    /// Seconds of audio in the recording.
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / f64::from(self.sample_rate.max(1))
    }
}

/// Why a conversion produced no file. Either way the WAV is still there.
#[derive(Debug, Clone, PartialEq)]
pub enum ConvertError {
    /// The app is closing; the job is done again the next time it starts.
    Cancelled,
    /// Windows has no encoder for the format: an N edition without the Media
    /// Feature Pack has no MP3 encoder. The window says how to get it.
    NoEncoder,
    Failed(String),
}

impl ConvertError {
    pub fn failed(reason: impl std::fmt::Display) -> Self {
        ConvertError::Failed(reason.to_string())
    }
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::Cancelled => f.write_str("the app is closing"),
            ConvertError::NoEncoder => f.write_str("Windows has no encoder for this format"),
            ConvertError::Failed(reason) => f.write_str(reason),
        }
    }
}

impl From<std::io::Error> for ConvertError {
    fn from(e: std::io::Error) -> Self {
        ConvertError::failed(e)
    }
}

/// A codec's line to whoever runs it: when to stop, and how far it has got.
pub struct Control<'a> {
    cancel: &'a AtomicBool,
    report: &'a (dyn Fn(f32) + Sync),
    from: f32,
    span: f32,
}

impl<'a> Control<'a> {
    /// `report` gets the share of the work that is done, from 0 to 1.
    pub fn new(cancel: &'a AtomicBool, report: &'a (dyn Fn(f32) + Sync)) -> Self {
        Self {
            cancel,
            report,
            from: 0.0,
            span: 1.0,
        }
    }

    /// The same control for one stage of the work: the stage's 0 to 1 is
    /// reported as `from` to `from + span`.
    fn stage(&self, from: f32, span: f32) -> Control<'_> {
        Control {
            cancel: self.cancel,
            report: self.report,
            from,
            span,
        }
    }

    /// Fails with `Cancelled` once the app is closing. A codec calls this often.
    pub fn check(&self) -> Result<(), ConvertError> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(ConvertError::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Reports that `done` (0 to 1) of this stage is finished.
    pub fn progress(&self, done: f32) {
        (self.report)(self.from + self.span * done.clamp(0.0, 1.0));
    }
}

/// One compressed format: how to write its file and how to check it.
pub trait Codec: Send {
    /// Extension of the compressed file, without the dot.
    fn extension(&self) -> &'static str;

    /// The share of a conversion that encoding takes; the check takes the rest.
    fn encode_share(&self) -> f32 {
        ENCODE_SHARE
    }

    /// Encodes the job's WAV into `output`.
    fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError>;

    /// Reads `output` back: it must decode without an error and last as long as
    /// the recording.
    fn verify(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError>;
}

/// The codec of a format, or `None` for WAV: a WAV recording needs no conversion.
pub fn codec_for(format: OutputFormat) -> Option<Box<dyn Codec>> {
    use mp3::Mp3Codec;
    use ogg_opus::OpusCodec;
    match format {
        OutputFormat::Wav => None,
        OutputFormat::OpusSpeech => Some(Box::new(OpusCodec::new(ogg_opus::Variant::Speech))),
        OutputFormat::OpusMusic => Some(Box::new(OpusCodec::new(ogg_opus::Variant::Music))),
        OutputFormat::Mp3Speech => Some(Box::new(Mp3Codec::new(mp3::Variant::Speech))),
        OutputFormat::Mp3Standard => Some(Box::new(Mp3Codec::new(mp3::Variant::Standard))),
        OutputFormat::Mp3High => Some(Box::new(Mp3Codec::new(mp3::Variant::High))),
    }
}

/// The share of a conversion that encoding takes, unless a codec says
/// otherwise; the check takes the rest. An hour of audio took 18-23 s to
/// encode and 3-7 s to decode in Opus (TASK-27).
const ENCODE_SHARE: f32 = 0.8;

/// Converts the job's WAV with `codec` and returns the path of the new file,
/// which then stands in for the WAV. On an error the WAV is untouched and no
/// other file is left behind.
pub fn convert_file(
    job: &Job,
    codec: &dyn Codec,
    control: &Control,
) -> Result<PathBuf, ConvertError> {
    let output = job.wav.with_extension(codec.extension());
    let temp = temp_path(&output);
    if let Err(e) = write_checked(job, codec, control, &output, &temp) {
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    // From here on the recording is safe in the new file.
    if let Err(e) = std::fs::remove_file(&job.wav) {
        log::warn!(
            "Converted to {} but could not delete {}: {e}",
            output.display(),
            job.wav.display()
        );
    }
    control.progress(1.0);
    Ok(output)
}

fn write_checked(
    job: &Job,
    codec: &dyn Codec,
    control: &Control,
    output: &Path,
    temp: &Path,
) -> Result<(), ConvertError> {
    control.check()?;
    let share = codec.encode_share();
    codec.encode(job, temp, &control.stage(0.0, share))?;
    codec.verify(job, temp, &control.stage(share, 1.0 - share))?;
    control.check()?;
    std::fs::rename(temp, output)?;
    Ok(())
}

/// The name a compressed file has while it is written and checked, for example
/// `mix_2026-11-26_14-03-09.opus.tmp`.
pub fn temp_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;
    use std::f32::consts::TAU;

    /// A new empty folder for one test. The name is partly Cyrillic: recordings
    /// often lie in such folders. The number keeps tests that run side by side
    /// apart, whatever they call their folders.
    pub fn test_dir(name: &str) -> PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "audiocaptor-convert-тест-{}-{}-{name}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Writes a 16-bit WAV: silence for `silence` seconds, then tones of
    /// `left_hz` and `right_hz` (the second one only in a stereo file) until
    /// `seconds`. Returns the job for it.
    pub fn tone_wav(
        path: &Path,
        sample_rate: u32,
        channels: u16,
        seconds: f32,
        silence: f32,
        (left_hz, right_hz): (f32, f32),
        format: OutputFormat,
    ) -> Job {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let frames = (seconds * sample_rate as f32).round() as u64;
        let quiet = (silence * sample_rate as f32).round() as u64;
        for i in 0..frames {
            let t = i as f32 / sample_rate as f32;
            for channel in 0..channels {
                let hz = if channel == 0 { left_hz } else { right_hz };
                let value = if i < quiet {
                    0.0
                } else {
                    0.5 * (TAU * hz * t).sin()
                };
                writer.write_sample((value * 32767.0) as i16).unwrap();
            }
        }
        writer.finalize().unwrap();
        Job {
            wav: path.to_path_buf(),
            frames,
            sample_rate,
            channels,
            format,
        }
    }

    /// A control that never cancels and ignores progress.
    pub fn free_run() -> (&'static AtomicBool, &'static (dyn Fn(f32) + Sync)) {
        static NEVER: AtomicBool = AtomicBool::new(false);
        (&NEVER, &|_| {})
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::*;
    use super::*;

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn speech_job(dir: &Path) -> Job {
        tone_wav(
            &dir.join("mix_2026-11-26_14-03-09.wav"),
            48_000,
            2,
            1.0,
            0.0,
            (440.0, 440.0),
            OutputFormat::OpusSpeech,
        )
    }

    /// The Opus codec with a check that always fails, as if the new file were
    /// shorter than the recording.
    struct FailingCheck(ogg_opus::OpusCodec);

    impl Codec for FailingCheck {
        fn extension(&self) -> &'static str {
            self.0.extension()
        }
        fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
            self.0.encode(job, output, control)
        }
        fn verify(&self, _: &Job, _: &Path, _: &Control) -> Result<(), ConvertError> {
            Err(ConvertError::failed(
                "the file is shorter than the recording",
            ))
        }
    }

    #[test]
    fn conversion_leaves_only_the_compressed_file() {
        let dir = test_dir("ok");
        let job = speech_job(&dir);
        let codec = codec_for(job.format).unwrap();
        let (cancel, report) = free_run();

        let output = convert_file(&job, codec.as_ref(), &Control::new(cancel, report)).unwrap();

        assert_eq!(output, dir.join("mix_2026-11-26_14-03-09.opus"));
        assert_eq!(names(&dir), ["mix_2026-11-26_14-03-09.opus"]);
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wav_format_has_no_codec() {
        assert!(codec_for(OutputFormat::Wav).is_none());
        for (format, extension) in [
            (OutputFormat::OpusSpeech, "opus"),
            (OutputFormat::OpusMusic, "opus"),
            (OutputFormat::Mp3Speech, "mp3"),
            (OutputFormat::Mp3Standard, "mp3"),
            (OutputFormat::Mp3High, "mp3"),
        ] {
            assert_eq!(codec_for(format).unwrap().extension(), extension);
        }
    }

    #[test]
    fn failed_encoding_keeps_the_wav_and_nothing_else() {
        let dir = test_dir("short");
        let mut job = speech_job(&dir);
        // The recorder counted more frames than the file holds.
        job.frames += 48_000;
        let wav_before = std::fs::read(&job.wav).unwrap();
        let codec = codec_for(job.format).unwrap();
        let (cancel, report) = free_run();

        let result = convert_file(&job, codec.as_ref(), &Control::new(cancel, report));

        assert!(matches!(result, Err(ConvertError::Failed(_))), "{result:?}");
        assert_eq!(names(&dir), ["mix_2026-11-26_14-03-09.wav"]);
        assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_check_keeps_the_wav_and_removes_the_temporary_file() {
        let dir = test_dir("check");
        let job = speech_job(&dir);
        let wav_before = std::fs::read(&job.wav).unwrap();
        let codec = FailingCheck(ogg_opus::OpusCodec::new(ogg_opus::Variant::Speech));
        let (cancel, report) = free_run();

        let result = convert_file(&job, &codec, &Control::new(cancel, report));

        assert_eq!(
            result,
            Err(ConvertError::Failed(
                "the file is shorter than the recording".into()
            ))
        );
        assert_eq!(names(&dir), ["mix_2026-11-26_14-03-09.wav"]);
        assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancelling_keeps_the_wav_and_removes_the_temporary_file() {
        let dir = test_dir("cancel");
        let job = tone_wav(
            &dir.join("mic_2026-11-26_14-03-09.wav"),
            48_000,
            2,
            5.0,
            0.0,
            (440.0, 440.0),
            OutputFormat::OpusMusic,
        );
        let wav_before = std::fs::read(&job.wav).unwrap();
        let codec = codec_for(job.format).unwrap();
        // The app starts closing once the encoder has reported some progress.
        let cancel = AtomicBool::new(false);
        let report = |done: f32| {
            if done > 0.0 {
                cancel.store(true, Ordering::Relaxed);
            }
        };

        let result = convert_file(&job, codec.as_ref(), &Control::new(&cancel, &report));

        assert_eq!(result, Err(ConvertError::Cancelled));
        assert_eq!(names(&dir), ["mic_2026-11-26_14-03-09.wav"]);
        assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn progress_runs_from_zero_to_one_across_both_stages() {
        let dir = test_dir("progress");
        let job = tone_wav(
            &dir.join("mix.wav"),
            48_000,
            2,
            4.0,
            0.0,
            (440.0, 440.0),
            OutputFormat::OpusSpeech,
        );
        let codec = codec_for(job.format).unwrap();
        let cancel = AtomicBool::new(false);
        let seen = std::sync::Mutex::new(Vec::new());
        let report = |done: f32| seen.lock().unwrap().push(done);

        convert_file(&job, codec.as_ref(), &Control::new(&cancel, &report)).unwrap();

        let seen = seen.into_inner().unwrap();
        assert!(seen.len() >= 4, "{seen:?}");
        assert!(seen.windows(2).all(|pair| pair[0] <= pair[1]), "{seen:?}");
        assert!(seen.iter().all(|done| (0.0..=1.0).contains(done)));
        // Both stages report: encoding below `ENCODE_SHARE`, the check above it.
        assert!(seen.iter().any(|done| *done > 0.0 && *done < ENCODE_SHARE));
        assert!(seen.iter().any(|done| (ENCODE_SHARE..1.0).contains(done)));
        assert_eq!(seen.last(), Some(&1.0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn temporary_name_adds_tmp_to_the_final_name() {
        assert_eq!(
            temp_path(Path::new("Recordings/mix_2026-11-26_14-03-09.opus")),
            Path::new("Recordings/mix_2026-11-26_14-03-09.opus.tmp")
        );
    }
}

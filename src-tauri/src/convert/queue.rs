//! The queue of recordings waiting to be converted (decision-8): one worker
//! thread converts one file at a time, in the order the recordings were stopped.
//!
//! The queue is also kept in a file in the data folder. What is still in it when
//! the app closes (the file being converted included) is converted the next
//! time the app starts.

use super::{codec_for, convert_file, Codec, Control, ConvertError, Job};
use crate::audio::types::OutputFormat;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// The file in the data folder that holds the queue. It exists only while
/// recordings wait to be converted.
pub const QUEUE_FILE: &str = "conversion-queue.json";

/// How often the progress of a conversion is passed on.
const STATUS_INTERVAL: Duration = Duration::from_millis(500);

/// What the queue tells the rest of the app.
pub trait QueueEvents: Send + Sync + 'static {
    /// The queue changed, or the file being converted got further.
    fn status(&self, status: &Status);
    /// A recording was converted: `output` now stands in for its WAV.
    fn converted(&self, job: &Job, output: &Path);
    /// A recording could not be converted. Its WAV is kept.
    fn failed(&self, job: &Job, reason: &str);
}

/// Where the queue stands.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Files not yet converted, the one being converted included.
    pub files: usize,
    /// Files waiting behind the one being converted.
    pub waiting: usize,
    /// The format the next file is converted to, e.g. `Opus`.
    pub format: Option<&'static str>,
    /// How much of the file being converted is done, 0 to 1.
    pub progress: f32,
    /// Seconds until the queue is empty, once there is something to go by.
    pub eta_seconds: Option<u32>,
}

/// What `enqueue` did. The window gets it when a recording stops, to say what
/// happens to the recording next.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Queued {
    /// Files added to the queue.
    pub files: usize,
    /// Files that were in the queue before them.
    pub ahead: usize,
    /// The format the added files are converted to.
    pub format: Option<&'static str>,
}

type CodecFor = dyn Fn(OutputFormat) -> Option<Box<dyn Codec>> + Send + Sync;

/// A handle to the queue; clones share it.
#[derive(Clone)]
pub struct ConversionQueue {
    inner: Arc<Inner>,
}

struct Inner {
    state: Mutex<State>,
    wake: Condvar,
    /// Set when the app closes: the worker stops at once.
    cancel: AtomicBool,
    data_dir: PathBuf,
    events: Box<dyn QueueEvents>,
    codecs: Box<CodecFor>,
}

struct State {
    /// Files to convert, oldest first. The first one stays here while it is
    /// converted, so the file on disk still lists it if the app is killed.
    jobs: VecDeque<Job>,
    /// Set while the first job is being converted.
    current: Option<Current>,
    /// Files the last run left behind, until `take_left_over` reports them.
    left_over: usize,
    worker: Option<std::thread::JoinHandle<()>>,
    /// Seconds of work a second of audio took in the last file converted.
    pace: Option<f64>,
}

struct Current {
    started: Instant,
    done: f32,
    reported: Instant,
}

impl State {
    fn status(&self) -> Status {
        let files = self.jobs.len();
        let waiting = files - usize::from(self.current.is_some() && files > 0);
        let eta_seconds = match (&self.current, self.jobs.front()) {
            (Some(current), Some(job)) => estimate(
                self.pace,
                current.started.elapsed().as_secs_f64(),
                f64::from(current.done),
                job.seconds(),
                self.jobs.iter().skip(1).map(Job::seconds).sum(),
            ),
            _ => None,
        };
        Status {
            files,
            waiting,
            format: self.jobs.front().map(|job| job.format.family()),
            progress: self.current.as_ref().map_or(0.0, |current| current.done),
            eta_seconds,
        }
    }
}

/// Seconds until the queue is empty: `done` (0 to 1) of a file with
/// `current_seconds` of audio took `elapsed` seconds, and `waiting_seconds` of
/// audio wait behind it. Until this file has run long enough to go by, the pace
/// of the file before it (`earlier_pace`, seconds of work per second of audio)
/// is used; with neither there is no estimate.
fn estimate(
    earlier_pace: Option<f64>,
    elapsed: f64,
    done: f64,
    current_seconds: f64,
    waiting_seconds: f64,
) -> Option<u32> {
    let pace = if elapsed >= 0.5 && done >= 0.02 && current_seconds > 0.0 {
        elapsed / (done * current_seconds)
    } else {
        earlier_pace?
    };
    let audio_left = (1.0 - done).max(0.0) * current_seconds + waiting_seconds;
    Some((pace * audio_left).round() as u32)
}

impl ConversionQueue {
    /// Opens the queue kept in `data_dir`, with what the last run left in it.
    /// Nothing is converted before `start`.
    pub fn open(data_dir: PathBuf, events: impl QueueEvents) -> Self {
        Self::with_codecs(data_dir, Box::new(events), Box::new(codec_for))
    }

    fn with_codecs(data_dir: PathBuf, events: Box<dyn QueueEvents>, codecs: Box<CodecFor>) -> Self {
        let jobs = load(&data_dir, &codecs);
        if !jobs.is_empty() {
            log::info!("{} recordings wait to be converted", jobs.len());
        }
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State {
                    left_over: jobs.len(),
                    jobs,
                    current: None,
                    worker: None,
                    pace: None,
                }),
                wake: Condvar::new(),
                cancel: AtomicBool::new(false),
                data_dir,
                events,
                codecs,
            }),
        }
    }

    /// Adds recordings to the end of the queue. Those that need no conversion
    /// (WAV) are left out.
    pub fn enqueue(&self, jobs: impl IntoIterator<Item = Job>) -> Queued {
        let (queued, status) = {
            let mut state = self.inner.lock();
            let ahead = state.jobs.len();
            state.jobs.extend(
                jobs.into_iter()
                    .filter(|job| (self.inner.codecs)(job.format).is_some()),
            );
            let files = state.jobs.len() - ahead;
            if files == 0 {
                return Queued::default();
            }
            self.inner.save(&state.jobs);
            let format = state.jobs.back().map(|job| job.format.family());
            (
                Queued {
                    files,
                    ahead,
                    format,
                },
                state.status(),
            )
        };
        self.inner.wake.notify_all();
        self.inner.events.status(&status);
        queued
    }

    /// How many files the last run left to convert. Answered once; later calls
    /// return 0.
    pub fn take_left_over(&self) -> usize {
        std::mem::take(&mut self.inner.lock().left_over)
    }

    /// Lets the conversions run. The window calls this once it listens to the
    /// queue's events, so it does not miss the end of a short conversion.
    pub fn start(&self) {
        let mut state = self.inner.lock();
        if state.worker.is_none() && !self.inner.cancel.load(Ordering::SeqCst) {
            let inner = Arc::clone(&self.inner);
            match std::thread::Builder::new()
                .name("converter".into())
                .spawn(move || work(&inner))
            {
                Ok(worker) => state.worker = Some(worker),
                Err(e) => log::error!("Could not start the conversion thread: {e}"),
            }
        }
    }

    /// True while a file is being converted or waits for it.
    pub fn is_busy(&self) -> bool {
        !self.inner.lock().jobs.is_empty()
    }

    pub fn status(&self) -> Status {
        self.inner.lock().status()
    }

    /// Stops the worker because the app is closing, and waits for it. The file
    /// being converted stays a WAV, its temporary file is removed, and the queue
    /// file keeps it for the next start.
    pub fn shutdown(&self) {
        self.inner.cancel.store(true, Ordering::SeqCst);
        let worker = self.inner.lock().worker.take();
        self.inner.wake.notify_all();
        if let Some(worker) = worker {
            let _ = worker.join();
        }
    }
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        // A panic elsewhere must not stop conversions: the state stays usable.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Takes note of how far the file being converted is, and passes it on
    /// every half second.
    fn progress(&self, done: f32) {
        let status = {
            let mut state = self.lock();
            let Some(current) = state.current.as_mut() else {
                return;
            };
            current.done = done;
            if current.reported.elapsed() < STATUS_INTERVAL {
                return;
            }
            current.reported = Instant::now();
            state.status()
        };
        self.events.status(&status);
    }

    /// Writes the queue to its file, or removes the file once the queue is empty.
    fn save(&self, jobs: &VecDeque<Job>) {
        let path = self.data_dir.join(QUEUE_FILE);
        let result = if jobs.is_empty() {
            match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            }
        } else {
            let stored = StoredQueue {
                version: 1,
                jobs: jobs
                    .iter()
                    .map(|job| StoredJob::new(job, &self.data_dir))
                    .collect(),
            };
            serde_json::to_string_pretty(&stored)
                .map_err(std::io::Error::other)
                .and_then(|json| crate::settings::write_atomic(&path, &json))
        };
        if let Err(e) = result {
            log::error!("Could not save the conversion queue: {e}");
        }
    }
}

/// How one conversion ended.
enum Outcome {
    Converted(PathBuf),
    Failed(String),
    /// The WAV is no longer there: the user moved or deleted it.
    Gone,
}

/// The worker thread: converts the first file of the queue until the app closes.
fn work(inner: &Inner) {
    loop {
        let (job, status) = {
            let mut state = inner.lock();
            loop {
                if inner.cancel.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(job) = state.jobs.front().cloned() {
                    let now = Instant::now();
                    state.current = Some(Current {
                        started: now,
                        done: 0.0,
                        reported: now,
                    });
                    break (job, state.status());
                }
                state = inner.wake.wait(state).unwrap_or_else(|e| e.into_inner());
            }
        };
        inner.events.status(&status);
        log::info!("Converting {} to {:?}", job.wav.display(), job.format);
        let started = Instant::now();

        let outcome = if !job.wav.exists() {
            Outcome::Gone
        } else {
            let report = |done: f32| inner.progress(done);
            let control = Control::new(&inner.cancel, &report);
            match (inner.codecs)(job.format) {
                None => Outcome::Failed("this format cannot be written".into()),
                Some(codec) => match convert_file(&job, codec.as_ref(), &control) {
                    Ok(output) => Outcome::Converted(output),
                    Err(ConvertError::Failed(reason)) => Outcome::Failed(reason),
                    Err(ConvertError::Cancelled) => {
                        // The app is closing: the job stays first in the queue.
                        inner.lock().current = None;
                        log::info!("Conversion of {} put off", job.wav.display());
                        return;
                    }
                },
            }
        };

        // The file leaves the queue whether or not it worked: a failed one is
        // not tried again, its WAV is what the user keeps.
        let now_empty = {
            let mut state = inner.lock();
            if let (Outcome::Converted(_), Some(current)) = (&outcome, &state.current) {
                if job.seconds() >= 1.0 {
                    state.pace = Some(current.started.elapsed().as_secs_f64() / job.seconds());
                }
            }
            state.current = None;
            state.jobs.pop_front();
            inner.save(&state.jobs);
            state.jobs.is_empty().then(|| state.status())
        };
        match outcome {
            Outcome::Converted(output) => {
                log::info!(
                    "Converted: {} ({:.0} s of audio in {:.1} s)",
                    output.display(),
                    job.seconds(),
                    started.elapsed().as_secs_f64()
                );
                inner.events.converted(&job, &output);
            }
            Outcome::Failed(reason) => {
                log::error!("Could not convert {}: {reason}", job.wav.display());
                inner.events.failed(&job, &reason);
            }
            Outcome::Gone => log::warn!("Not converted, the file is gone: {}", job.wav.display()),
        }
        if let Some(status) = now_empty {
            inner.events.status(&status);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredQueue {
    version: u32,
    jobs: Vec<StoredJob>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredJob {
    /// Relative to the data folder when the WAV lies inside it: the folder may
    /// have moved by the next start (a USB stick, an update through Scoop).
    wav: PathBuf,
    frames: u64,
    sample_rate: u32,
    channels: u16,
    /// The id of the `OutputFormat`.
    format: String,
}

impl StoredJob {
    fn new(job: &Job, data_dir: &Path) -> Self {
        Self {
            wav: job
                .wav
                .strip_prefix(data_dir)
                .unwrap_or(&job.wav)
                .to_path_buf(),
            frames: job.frames,
            sample_rate: job.sample_rate,
            channels: job.channels,
            format: job.format.id().to_string(),
        }
    }
}

/// Reads the queue file. Entries that cannot be converted here are left out: a
/// format this version does not write (the file may come from a newer one) and
/// a WAV that is no longer there.
fn load(data_dir: &Path, codecs: &CodecFor) -> VecDeque<Job> {
    let path = data_dir.join(QUEUE_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return VecDeque::new(),
        Err(e) => {
            log::error!("Could not read {}: {e}", path.display());
            return VecDeque::new();
        }
    };
    let stored: StoredQueue = match serde_json::from_str(&text) {
        Ok(stored) => stored,
        Err(e) => {
            log::error!("Could not read {}: {e}", path.display());
            return VecDeque::new();
        }
    };
    stored
        .jobs
        .into_iter()
        .filter_map(|stored| {
            // `join` keeps an absolute path as it is.
            let wav = data_dir.join(&stored.wav);
            let format = OutputFormat::from_id(&stored.format).filter(|&f| codecs(f).is_some());
            match format {
                Some(format) if wav.exists() => Some(Job {
                    wav,
                    frames: stored.frames,
                    sample_rate: stored.sample_rate,
                    channels: stored.channels,
                    format,
                }),
                _ => {
                    log::warn!("Left out of the conversion queue: {}", wav.display());
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::temp_path;
    use crate::convert::test_util::{test_dir, tone_wav};
    use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};

    const WAIT: Duration = Duration::from_secs(10);

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        Status(Status),
        Converted(String),
        Failed(String, String),
    }

    /// Passes every event of the queue to the test.
    struct Events(Sender<Event>);

    fn name(path: &Path) -> String {
        path.file_name().unwrap().to_string_lossy().into_owned()
    }

    impl QueueEvents for Events {
        fn status(&self, status: &Status) {
            let _ = self.0.send(Event::Status(status.clone()));
        }
        fn converted(&self, _: &Job, output: &Path) {
            let _ = self.0.send(Event::Converted(name(output)));
        }
        fn failed(&self, job: &Job, reason: &str) {
            let _ = self
                .0
                .send(Event::Failed(name(&job.wav), reason.to_string()));
        }
    }

    /// A codec that waits in `encode` until the test says how it should end.
    struct Gated {
        started: Sender<String>,
        proceed: Receiver<Result<(), String>>,
    }

    impl Codec for Gated {
        fn extension(&self) -> &'static str {
            "opus"
        }
        fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
            std::fs::write(output, b"half a file")?;
            self.started.send(name(&job.wav)).unwrap();
            loop {
                control.check()?;
                match self.proceed.recv_timeout(Duration::from_millis(2)) {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(reason)) => return Err(ConvertError::Failed(reason)),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        return Err(ConvertError::failed("the test is over"))
                    }
                }
            }
        }
        fn verify(&self, _: &Job, _: &Path, _: &Control) -> Result<(), ConvertError> {
            Ok(())
        }
    }

    /// A data folder with recordings in it and a queue with the gated codec.
    struct Bench {
        data_dir: PathBuf,
        queue: ConversionQueue,
        events: Receiver<Event>,
        started: Receiver<String>,
        proceed: Sender<Result<(), String>>,
    }

    impl Bench {
        fn new(test: &str) -> Self {
            let data_dir = test_dir(test);
            std::fs::create_dir_all(data_dir.join("Recordings")).unwrap();
            Self::open(data_dir)
        }

        /// Opens the queue of `data_dir` as a start of the app does.
        fn open(data_dir: PathBuf) -> Self {
            let (event_tx, events) = unbounded();
            let (started_tx, started) = unbounded();
            let (proceed, proceed_rx) = unbounded();
            let queue = ConversionQueue::with_codecs(
                data_dir.clone(),
                Box::new(Events(event_tx)),
                Box::new(move |format| {
                    (format != OutputFormat::Wav).then(|| {
                        Box::new(Gated {
                            started: started_tx.clone(),
                            proceed: proceed_rx.clone(),
                        }) as Box<dyn Codec>
                    })
                }),
            );
            Self {
                data_dir,
                queue,
                events,
                started,
                proceed,
            }
        }

        /// Makes a short recording in the data folder and returns its job.
        fn recording(&self, name: &str) -> Job {
            tone_wav(
                &self.data_dir.join("Recordings").join(name),
                8_000,
                2,
                0.5,
                0.0,
                (440.0, 440.0),
                OutputFormat::OpusSpeech,
            )
        }

        fn exists(&self, name: &str) -> bool {
            self.data_dir.join("Recordings").join(name).exists()
        }

        /// Waits until the worker starts to convert a file; returns its name.
        fn next_started(&self) -> String {
            self.started
                .recv_timeout(WAIT)
                .expect("a conversion starts")
        }

        fn nothing_started(&self) -> bool {
            std::thread::sleep(Duration::from_millis(50));
            self.started.is_empty()
        }

        fn finish(&self) {
            self.proceed.send(Ok(())).unwrap();
        }

        /// Waits for the status that says the queue is empty; returns the events
        /// up to it, the statuses left out.
        fn until_empty(&self) -> Vec<Event> {
            let mut seen = Vec::new();
            loop {
                match self.events.recv_timeout(WAIT).expect("the queue empties") {
                    Event::Status(status) if status.files == 0 => return seen,
                    Event::Status(_) => {}
                    event => seen.push(event),
                }
            }
        }

        fn stored(&self) -> Option<serde_json::Value> {
            let text = std::fs::read_to_string(self.data_dir.join(QUEUE_FILE)).ok()?;
            Some(serde_json::from_str(&text).unwrap())
        }

        fn stored_wavs(&self) -> Vec<String> {
            self.stored().map_or_else(Vec::new, |stored| {
                stored["jobs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|job| job["wav"].as_str().unwrap().replace('\\', "/"))
                    .collect()
            })
        }
    }

    impl Drop for Bench {
        fn drop(&mut self) {
            self.queue.shutdown();
            let _ = std::fs::remove_dir_all(&self.data_dir);
        }
    }

    #[test]
    fn files_are_converted_one_at_a_time_in_the_order_they_were_stopped() {
        let bench = Bench::new("order");
        let (a, b, c) = (
            bench.recording("a.wav"),
            bench.recording("b.wav"),
            bench.recording("c.wav"),
        );
        assert_eq!(bench.queue.take_left_over(), 0);
        bench.queue.start();
        assert!(!bench.queue.is_busy());

        // One recording with two files, as "Mix + Microphone" gives.
        let queued = bench.queue.enqueue([a, b]);
        assert_eq!(
            queued,
            Queued {
                files: 2,
                ahead: 0,
                format: Some("Opus")
            }
        );
        assert_eq!(bench.next_started(), "a.wav");
        let status = bench.queue.status();
        assert_eq!((status.files, status.waiting), (2, 1));
        assert_eq!(status.format, Some("Opus"));

        // The next recording is stopped while the first file is converted.
        let queued = bench.queue.enqueue([c]);
        assert_eq!((queued.files, queued.ahead), (1, 2));
        let status = bench.queue.status();
        assert_eq!((status.files, status.waiting), (3, 2));
        // Only one file at a time: b and c wait for a.
        assert!(bench.nothing_started());
        assert!(bench.exists("b.wav") && bench.exists("c.wav"));

        bench.finish();
        assert_eq!(bench.next_started(), "b.wav");
        assert_eq!(bench.queue.status().waiting, 1);
        assert!(bench.nothing_started());
        bench.finish();
        assert_eq!(bench.next_started(), "c.wav");
        assert_eq!(bench.queue.status().waiting, 0);
        bench.finish();

        assert_eq!(
            bench.until_empty(),
            [
                Event::Converted("a.opus".into()),
                Event::Converted("b.opus".into()),
                Event::Converted("c.opus".into()),
            ]
        );
        assert!(!bench.queue.is_busy());
        for name in ["a", "b", "c"] {
            assert!(bench.exists(&format!("{name}.opus")));
            assert!(!bench.exists(&format!("{name}.wav")));
            assert!(!bench.exists(&format!("{name}.opus.tmp")));
        }
        assert!(bench.stored().is_none(), "the queue file is gone");
    }

    #[test]
    fn nothing_is_converted_before_the_window_is_ready() {
        let bench = Bench::new("not-started");
        let queued = bench.queue.enqueue([bench.recording("a.wav")]);
        assert_eq!(queued.files, 1);
        assert!(bench.queue.is_busy());
        assert!(bench.nothing_started());
        let status = bench.queue.status();
        assert_eq!((status.files, status.waiting), (1, 1));
        assert_eq!(status.eta_seconds, None);
        assert_eq!(bench.stored_wavs(), ["Recordings/a.wav"]);

        // It was queued in this run, so it is not left over from the last one.
        assert_eq!(bench.queue.take_left_over(), 0);
        bench.queue.start();
        assert_eq!(bench.next_started(), "a.wav");
        bench.finish();
        assert_eq!(bench.until_empty(), [Event::Converted("a.opus".into())]);
    }

    #[test]
    fn what_is_left_at_exit_is_converted_at_the_next_start() {
        let bench = Bench::new("resume");
        let jobs = [
            bench.recording("a.wav"),
            bench.recording("b.wav"),
            bench.recording("c.wav"),
        ];
        let wav_before = std::fs::read(&jobs[1].wav).unwrap();
        bench.queue.start();
        bench.queue.enqueue(jobs);
        assert_eq!(bench.next_started(), "a.wav");
        bench.finish();
        // The app closes while b is converted and c waits.
        assert_eq!(bench.next_started(), "b.wav");
        assert!(bench.exists("b.opus.tmp"));
        bench.queue.shutdown();

        // Nothing is damaged: b is still a whole WAV, with no half an Opus file.
        assert!(bench.exists("a.opus") && !bench.exists("a.wav"));
        assert!(bench.exists("b.wav") && bench.exists("c.wav"));
        assert!(!bench.exists("b.opus") && !bench.exists("b.opus.tmp"));
        assert_eq!(
            std::fs::read(bench.data_dir.join("Recordings/b.wav")).unwrap(),
            wav_before
        );
        // Both files are still in the queue file, by a path inside the data folder.
        assert_eq!(
            bench.stored_wavs(),
            ["Recordings/b.wav", "Recordings/c.wav"]
        );
        let stored = bench.stored().unwrap();
        assert_eq!(stored["jobs"][0]["frames"], 4000);
        assert_eq!(stored["jobs"][0]["sampleRate"], 8000);
        assert_eq!(stored["jobs"][0]["format"], "OpusSpeech");

        // The next start: the queue is read again and says what is left.
        let next = Bench::open(bench.data_dir.clone());
        assert!(next.queue.is_busy());
        assert!(next.nothing_started());
        assert_eq!(next.queue.take_left_over(), 2);
        assert_eq!(next.queue.take_left_over(), 0, "reported once");
        next.queue.start();
        assert_eq!(next.next_started(), "b.wav");
        next.finish();
        assert_eq!(next.next_started(), "c.wav");
        next.finish();
        assert_eq!(
            next.until_empty(),
            [
                Event::Converted("b.opus".into()),
                Event::Converted("c.opus".into()),
            ]
        );
        assert!(next.exists("b.opus") && next.exists("c.opus"));
        assert!(!next.exists("b.wav") && !next.exists("c.wav"));
        assert!(next.stored().is_none());
        // `next` removes the folder; `bench` then finds nothing to clean up.
    }

    #[test]
    fn a_file_that_fails_keeps_its_wav_and_the_next_one_goes_on() {
        let bench = Bench::new("failure");
        bench.queue.start();
        bench
            .queue
            .enqueue([bench.recording("a.wav"), bench.recording("b.wav")]);
        assert_eq!(bench.next_started(), "a.wav");
        bench.proceed.send(Err("the disk is full".into())).unwrap();
        assert_eq!(bench.next_started(), "b.wav");
        // The failed file has left the queue: it is not tried again.
        assert_eq!(bench.stored_wavs(), ["Recordings/b.wav"]);
        bench.finish();

        assert_eq!(
            bench.until_empty(),
            [
                Event::Failed("a.wav".into(), "the disk is full".into()),
                Event::Converted("b.opus".into()),
            ]
        );
        assert!(bench.exists("a.wav"));
        assert!(!bench.exists("a.opus") && !bench.exists("a.opus.tmp"));
        assert!(bench.exists("b.opus") && !bench.exists("b.wav"));
    }

    #[test]
    fn wav_recordings_are_not_queued() {
        let bench = Bench::new("wav");
        let mut job = bench.recording("a.wav");
        job.format = OutputFormat::Wav;
        assert_eq!(bench.queue.enqueue([job]), Queued::default());
        assert!(!bench.queue.is_busy());
        assert!(bench.stored().is_none());
        assert!(bench.events.is_empty());
    }

    #[test]
    fn a_wav_that_is_gone_is_skipped_without_an_error() {
        let bench = Bench::new("gone");
        let (a, b) = (bench.recording("a.wav"), bench.recording("b.wav"));
        std::fs::remove_file(&a.wav).unwrap();
        bench.queue.enqueue([a, b]);
        bench.queue.start();
        assert_eq!(bench.next_started(), "b.wav");
        bench.finish();
        assert_eq!(bench.until_empty(), [Event::Converted("b.opus".into())]);
    }

    #[test]
    fn queue_file_entries_that_cannot_be_converted_are_left_out() {
        let bench = Bench::new("load");
        let kept = bench.recording("kept.wav");
        bench.recording("newer.wav");
        let outside = test_dir("load-outside").join("far.wav");
        tone_wav(
            &outside,
            8_000,
            2,
            0.1,
            0.0,
            (440.0, 440.0),
            OutputFormat::OpusMusic,
        );
        // A queue file with a format of a newer version, a WAV that is gone, a
        // WAV outside the data folder (an absolute path) and one inside it.
        let stored = serde_json::json!({
            "version": 1,
            "jobs": [
                {"wav": "Recordings/newer.wav", "frames": 800, "sampleRate": 8000, "channels": 2, "format": "Mp3Standard"},
                {"wav": "Recordings/gone.wav", "frames": 800, "sampleRate": 8000, "channels": 2, "format": "OpusSpeech"},
                {"wav": outside, "frames": 800, "sampleRate": 8000, "channels": 2, "format": "OpusMusic"},
                {"wav": "Recordings/kept.wav", "frames": 4000, "sampleRate": 8000, "channels": 2, "format": "OpusSpeech"},
            ],
        });
        std::fs::write(bench.data_dir.join(QUEUE_FILE), stored.to_string()).unwrap();

        let next = Bench::open(bench.data_dir.clone());
        assert_eq!(next.queue.status().files, 2);
        assert_eq!(next.queue.take_left_over(), 2);
        next.queue.start();
        assert_eq!(next.next_started(), "far.wav");
        next.finish();
        assert_eq!(next.next_started(), "kept.wav");
        next.finish();
        next.until_empty();
        assert!(outside.with_extension("opus").exists());
        assert!(next.exists("kept.opus") && !next.exists(&name(&kept.wav)));
        assert!(next.exists("newer.wav"), "left as it is");
        let _ = std::fs::remove_dir_all(outside.parent().unwrap());
    }

    #[test]
    fn an_unreadable_queue_file_is_an_empty_queue() {
        let bench = Bench::new("broken");
        std::fs::write(bench.data_dir.join(QUEUE_FILE), "{ \"jobs\": [").unwrap();
        let next = Bench::open(bench.data_dir.clone());
        assert!(!next.queue.is_busy());
        assert_eq!(next.queue.take_left_over(), 0);
    }

    #[test]
    fn real_conversion_through_the_queue_replaces_the_wav() {
        // The queue with the real codecs, as the app runs it.
        let data_dir = test_dir("real");
        let (event_tx, events) = unbounded();
        let queue = ConversionQueue::open(data_dir.clone(), Events(event_tx));
        let job = tone_wav(
            &data_dir.join("mix_2026-11-26_14-03-09.wav"),
            44_100,
            2,
            1.0,
            0.0,
            (440.0, 440.0),
            OutputFormat::OpusMusic,
        );
        queue.start();
        queue.enqueue([job.clone()]);
        let mut converted = None;
        loop {
            match events.recv_timeout(WAIT).expect("the queue empties") {
                Event::Status(status) if status.files == 0 => break,
                Event::Status(status) => assert_eq!(status.format, Some("Opus")),
                Event::Converted(name) => converted = Some(name),
                Event::Failed(_, reason) => panic!("{reason}"),
            }
        }
        assert_eq!(converted.as_deref(), Some("mix_2026-11-26_14-03-09.opus"));
        assert!(!job.wav.exists());
        assert!(!temp_path(&job.wav.with_extension("opus")).exists());
        assert!(!data_dir.join(QUEUE_FILE).exists());
        queue.shutdown();
        let _ = std::fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn time_left_counts_the_whole_queue() {
        // Nothing to go by yet: the file has only just started.
        assert_eq!(estimate(None, 0.1, 0.01, 3600.0, 0.0), None);
        // Half of an hour-long file took 10 s: 10 s more.
        assert_eq!(estimate(None, 10.0, 0.5, 3600.0, 0.0), Some(10));
        // With another hour waiting behind it: 10 s and 20 s.
        assert_eq!(estimate(None, 10.0, 0.5, 3600.0, 3600.0), Some(30));
        // A file that has just started goes by the pace of the one before it.
        assert_eq!(estimate(Some(0.01), 0.0, 0.0, 600.0, 1200.0), Some(18));
        // Its own pace wins once it has run for a while.
        assert_eq!(estimate(Some(1.0), 5.0, 0.25, 2000.0, 0.0), Some(15));
        assert_eq!(estimate(None, 30.0, 1.0, 3600.0, 0.0), Some(0));
    }
}

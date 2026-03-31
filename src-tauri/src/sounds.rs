use crossbeam_channel::{bounded, Sender};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

const START_SOUND: &[u8] = include_bytes!("../sounds/start.ogg");
const PAUSE_SOUND: &[u8] = include_bytes!("../sounds/pause.ogg");
const STOP_SOUND: &[u8] = include_bytes!("../sounds/stop.ogg");

#[derive(Debug, Clone, Copy)]
pub enum SoundKind {
    Start,
    Pause,
    Stop,
}

type SoundMsg = (SoundKind, Option<Sender<()>>);

/// Thread-safe handle to the sound playback thread.
/// Stores only a Sender, so it is Send + Sync.
pub struct SoundEngine {
    tx: Sender<SoundMsg>,
}

impl SoundEngine {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = bounded::<SoundMsg>(8);

        std::thread::Builder::new()
            .name("sound-engine".into())
            .spawn(move || {
                let (_stream, handle) = match OutputStream::try_default() {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Sound engine: failed to open output: {e}");
                        return;
                    }
                };

                while let Ok((kind, done_tx)) = rx.recv() {
                    let data: &[u8] = match kind {
                        SoundKind::Start => START_SOUND,
                        SoundKind::Pause => PAUSE_SOUND,
                        SoundKind::Stop => STOP_SOUND,
                    };

                    let cursor = Cursor::new(data);
                    let decoder = match Decoder::new(cursor) {
                        Ok(d) => d,
                        Err(e) => {
                            log::warn!("Failed to decode notification sound: {e}");
                            if let Some(tx) = done_tx {
                                let _ = tx.send(());
                            }
                            continue;
                        }
                    };

                    match Sink::try_new(&handle) {
                        Ok(sink) => {
                            sink.append(decoder);
                            if let Some(tx) = done_tx {
                                sink.sleep_until_end();
                                let _ = tx.send(());
                            } else {
                                sink.detach();
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to create audio sink: {e}");
                            if let Some(tx) = done_tx {
                                let _ = tx.send(());
                            }
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn sound thread: {e}"))?;

        Ok(Self { tx })
    }

    /// Non-blocking: queues the sound and returns immediately.
    pub fn play(&self, kind: SoundKind) {
        let _ = self.tx.try_send((kind, None));
    }

    /// Blocking: plays the sound and waits for it to finish before returning.
    pub fn play_blocking(&self, kind: SoundKind) {
        let (done_tx, done_rx) = bounded::<()>(1);
        if self.tx.send((kind, Some(done_tx))).is_ok() {
            let _ = done_rx.recv();
        }
    }
}

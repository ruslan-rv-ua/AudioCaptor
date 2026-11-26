//! Media Foundation, the media platform of Windows, loaded only when a
//! conversion needs it: the MP3 encoder is part of it (decision-5).
//!
//! Windows N editions have no Media Foundation until the Media Feature Pack is
//! installed. A program that imports `mfplat.dll` the usual way, as the `MF*`
//! functions of the `windows` crate do, does not even start there. So the app
//! loads the library itself the first time it is needed and calls the few
//! functions it uses through pointers. The interfaces (`IMFTransform` and the
//! rest) and the GUIDs come from the `windows` crate: they need no import.
//! Never call the crate's `MF*` functions directly.

use super::ConvertError;
use std::ffi::c_void;
use std::io;
use std::mem::ManuallyDrop;
use std::sync::OnceLock;
use windows::core::{s, w, Interface, GUID, HRESULT, PCSTR};
use windows::Win32::Media::MediaFoundation::{
    IMFMediaBuffer, IMFMediaType, IMFSample, IMFTransform, MFSTARTUP_LITE,
    MFT_MESSAGE_COMMAND_DRAIN, MFT_MESSAGE_NOTIFY_BEGIN_STREAMING,
    MFT_MESSAGE_NOTIFY_END_OF_STREAM, MFT_MESSAGE_NOTIFY_START_OF_STREAM, MFT_OUTPUT_DATA_BUFFER,
    MFT_OUTPUT_STREAM_PROVIDES_SAMPLES, MF_E_NOTACCEPTING, MF_E_TRANSFORM_NEED_MORE_INPUT,
    MF_VERSION,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
};

impl From<windows::core::Error> for ConvertError {
    fn from(e: windows::core::Error) -> Self {
        ConvertError::failed(e)
    }
}

type Startup = unsafe extern "system" fn(u32, u32) -> HRESULT;
type Shutdown = unsafe extern "system" fn() -> HRESULT;
type CreateObject = unsafe extern "system" fn(*mut *mut c_void) -> HRESULT;
type CreateBuffer = unsafe extern "system" fn(u32, *mut *mut c_void) -> HRESULT;
type AnyFunction = unsafe extern "system" fn() -> isize;

/// The functions of `mfplat.dll` the app calls.
struct Platform {
    startup: Startup,
    shutdown: Shutdown,
    create_media_type: CreateObject,
    create_sample: CreateObject,
    create_memory_buffer: CreateBuffer,
}

/// `mfplat.dll`, loaded the first time it is asked for; `None` when this
/// Windows has none.
fn platform() -> Option<&'static Platform> {
    static PLATFORM: OnceLock<Option<Platform>> = OnceLock::new();
    PLATFORM.get_or_init(load).as_ref()
}

fn load() -> Option<Platform> {
    // Only from System32: a DLL of that name next to the exe is never loaded.
    // SAFETY: loading a system library runs no code of ours.
    let module =
        match unsafe { LoadLibraryExW(w!("mfplat.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32) } {
            Ok(module) => module,
            Err(e) => {
                log::warn!("Windows has no Media Foundation: {e}");
                return None;
            }
        };
    let function = |name: PCSTR| -> Option<AnyFunction> {
        // SAFETY: `name` is a constant string and `module` a loaded library.
        let found = unsafe { GetProcAddress(module, name) };
        if found.is_none() {
            log::warn!("Media Foundation lacks {}", unsafe { name.display() });
        }
        found
    };
    // SAFETY: each pointer is cast to the signature that the Media Foundation
    // documentation gives for the function of that name. The library is never
    // unloaded, so the pointers stay valid.
    unsafe {
        Some(Platform {
            startup: std::mem::transmute::<AnyFunction, Startup>(function(s!("MFStartup"))?),
            shutdown: std::mem::transmute::<AnyFunction, Shutdown>(function(s!("MFShutdown"))?),
            create_media_type: std::mem::transmute::<AnyFunction, CreateObject>(function(s!(
                "MFCreateMediaType"
            ))?),
            create_sample: std::mem::transmute::<AnyFunction, CreateObject>(function(s!(
                "MFCreateSample"
            ))?),
            create_memory_buffer: std::mem::transmute::<AnyFunction, CreateBuffer>(function(s!(
                "MFCreateMemoryBuffer"
            ))?),
        })
    }
}

/// COM and Media Foundation, started on the current thread; both stop when
/// the session is dropped. What is made through a session borrows it, so it is
/// released before that.
pub struct Session {
    platform: &'static Platform,
    /// COM was started here and must be stopped here.
    com: bool,
}

impl Session {
    /// Fails with `NoEncoder` when this Windows has no Media Foundation.
    pub fn start() -> Result<Self, ConvertError> {
        let platform = platform().ok_or(ConvertError::NoEncoder)?;
        // S_FALSE (COM already runs on this thread) needs a CoUninitialize as
        // well; RPC_E_CHANGED_MODE (a single-threaded apartment) does not, and
        // COM works there too.
        // SAFETY: no pointers are passed.
        let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        // MFSTARTUP_LITE: no network sources; the app only reads its own files.
        // SAFETY: the function takes two numbers.
        if let Err(e) = unsafe { (platform.startup)(MF_VERSION, MFSTARTUP_LITE) }.ok() {
            if com {
                // SAFETY: matches the successful CoInitializeEx above.
                unsafe { CoUninitialize() };
            }
            log::warn!("Media Foundation did not start: {e}");
            return Err(ConvertError::NoEncoder);
        }
        Ok(Self { platform, com })
    }

    /// Creates an object of class `class`, such as an encoder.
    pub fn create<T: Interface>(&self, class: &GUID) -> windows::core::Result<T> {
        // SAFETY: COM runs on this thread for as long as the session lives.
        unsafe { CoCreateInstance(class, None, CLSCTX_INPROC_SERVER) }
    }

    /// A new, empty media type.
    pub fn media_type(&self) -> windows::core::Result<IMFMediaType> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: on success the function hands over the one reference to a
        // new media type, which `from_raw` takes.
        unsafe {
            (self.platform.create_media_type)(&mut raw).ok()?;
            Ok(IMFMediaType::from_raw(raw))
        }
    }

    /// A sample with one empty buffer that holds `capacity` bytes.
    fn sample(&self, capacity: u32) -> windows::core::Result<(IMFSample, IMFMediaBuffer)> {
        // SAFETY: as in `media_type`.
        unsafe {
            let mut raw = std::ptr::null_mut();
            (self.platform.create_memory_buffer)(capacity, &mut raw).ok()?;
            let buffer = IMFMediaBuffer::from_raw(raw);
            let mut raw = std::ptr::null_mut();
            (self.platform.create_sample)(&mut raw).ok()?;
            let sample = IMFSample::from_raw(raw);
            sample.AddBuffer(&buffer)?;
            Ok((sample, buffer))
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: matches the start in `Session::start`; everything made
        // through the session is gone, since it borrowed the session.
        unsafe {
            let _ = (self.platform.shutdown)();
            if self.com {
                CoUninitialize();
            }
        }
    }
}

/// Where the output of a transform goes, e.g. the MP3 file.
pub type Sink<'s> = dyn FnMut(&[u8]) -> io::Result<()> + 's;

/// Room for what one call to a transform puts out: a few MP3 frames from the
/// encoder (the largest is 1441 bytes), a few frames of PCM from a decoder.
const OUTPUT_BYTES: u32 = 1 << 16;

/// A Media Foundation transform, such as an encoder, with its input and output
/// types set: bytes go in, and the bytes it makes go to a sink.
pub struct Transform<'a> {
    session: &'a Session,
    transform: IMFTransform,
    /// The sample the transform writes its output into, unless it brings its own.
    output: Option<(IMFSample, IMFMediaBuffer)>,
}

impl<'a> Transform<'a> {
    pub fn start(session: &'a Session, transform: IMFTransform) -> Result<Self, ConvertError> {
        // SAFETY: plain calls on a live object.
        let info = unsafe { transform.GetOutputStreamInfo(0)? };
        let output = if info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32 != 0 {
            None
        } else {
            Some(session.sample(info.cbSize.max(OUTPUT_BYTES))?)
        };
        // SAFETY: as above.
        unsafe {
            transform.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)?;
            transform.ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0)?;
        }
        Ok(Self {
            session,
            transform,
            output,
        })
    }

    /// Feeds `data` to the transform; what it puts out meanwhile goes to `sink`.
    /// `timing` places the data in time: its start and length in 100 ns units.
    pub fn push(
        &mut self,
        data: &[u8],
        timing: Option<(i64, i64)>,
        sink: &mut Sink,
    ) -> Result<(), ConvertError> {
        let len = u32::try_from(data.len()).map_err(ConvertError::failed)?;
        let (sample, buffer) = self.session.sample(len.max(1))?;
        // SAFETY: the buffer holds at least `len` bytes while it is locked.
        unsafe {
            let mut at = std::ptr::null_mut();
            buffer.Lock(&mut at, None, None)?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), at, data.len());
            buffer.Unlock()?;
            buffer.SetCurrentLength(len)?;
            if let Some((time, duration)) = timing {
                sample.SetSampleTime(time)?;
                sample.SetSampleDuration(duration)?;
            }
        }
        loop {
            // SAFETY: a plain call on a live object.
            match unsafe { self.transform.ProcessInput(0, &sample, 0) } {
                Ok(()) => break,
                // It is full: take what it has made, then try again.
                Err(e) if e.code() == MF_E_NOTACCEPTING => self.pull(sink)?,
                Err(e) => return Err(e.into()),
            }
        }
        self.pull(sink)
    }

    /// Tells the transform that the input has ended and takes the rest of its
    /// output, such as the last frames of an encoder.
    pub fn finish(&mut self, sink: &mut Sink) -> Result<(), ConvertError> {
        // SAFETY: plain calls on a live object.
        unsafe {
            self.transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0)?;
            self.transform
                .ProcessMessage(MFT_MESSAGE_COMMAND_DRAIN, 0)?;
        }
        self.pull(sink)
    }

    /// Passes everything the transform has made so far to `sink`.
    fn pull(&mut self, sink: &mut Sink) -> Result<(), ConvertError> {
        loop {
            if let Some((_, buffer)) = &self.output {
                // SAFETY: a plain call on a live object.
                unsafe { buffer.SetCurrentLength(0)? };
            }
            let mut data = [MFT_OUTPUT_DATA_BUFFER {
                dwStreamID: 0,
                pSample: ManuallyDrop::new(self.output.as_ref().map(|(sample, _)| sample.clone())),
                dwStatus: 0,
                pEvents: ManuallyDrop::new(None),
            }];
            let mut status = 0;
            // SAFETY: `data` holds a sample of ours with room for the output.
            let result = unsafe { self.transform.ProcessOutput(0, &mut data, &mut status) };
            // The references in `data` are ours to release.
            // SAFETY: each field is taken once and not used again.
            let (sample, events) = unsafe {
                (
                    ManuallyDrop::take(&mut data[0].pSample),
                    ManuallyDrop::take(&mut data[0].pEvents),
                )
            };
            drop(events);
            match result {
                Ok(()) => {
                    if let Some(sample) = sample {
                        write_sample(&sample, sink)?;
                    }
                }
                Err(e) if e.code() == MF_E_TRANSFORM_NEED_MORE_INPUT => return Ok(()),
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// Passes the bytes of `sample` to `sink`.
fn write_sample(sample: &IMFSample, sink: &mut Sink) -> Result<(), ConvertError> {
    // SAFETY: the buffer's bytes are read only while it is locked.
    unsafe {
        let buffer = sample.ConvertToContiguousBuffer()?;
        let mut at = std::ptr::null_mut();
        let mut len = 0;
        buffer.Lock(&mut at, None, Some(&mut len))?;
        let written = if len == 0 {
            Ok(())
        } else {
            sink(std::slice::from_raw_parts(at, len as usize))
        };
        buffer.Unlock()?;
        written?;
    }
    Ok(())
}

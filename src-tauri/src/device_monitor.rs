use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use windows::core::*;
use windows::Win32::Media::Audio::*;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::System::Com::*;

#[windows::core::implement(IMMNotificationClient)]
struct DeviceNotificationClient {
    app: AppHandle,
}

impl IMMNotificationClient_Impl for DeviceNotificationClient_Impl {
    fn OnDeviceStateChanged(&self, _device_id: &PCWSTR, _new_state: DEVICE_STATE) -> windows_core::Result<()> {
        let _ = self.app.emit("audio-devices-changed", ());
        Ok(())
    }

    fn OnDeviceAdded(&self, _device_id: &PCWSTR) -> windows_core::Result<()> {
        let _ = self.app.emit("audio-devices-changed", ());
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &PCWSTR) -> windows_core::Result<()> {
        let _ = self.app.emit("audio-devices-changed", ());

        // Check if removed device is currently being recorded
        let id_str = unsafe {
            device_id.to_string().unwrap_or_else(|e| {
                log::warn!("Failed to convert removed device ID to string: {}", e);
                String::new()
            })
        };
        if let Some(state) = self.app.try_state::<crate::state::SharedState>() {
            if let Ok(s) = state.inner().lock() {
                let is_active = s.recording_mic_id.as_deref() == Some(&id_str)
                    || s.recording_loopback_id.as_deref() == Some(&id_str);
                if is_active && s.recording_state != crate::audio::types::RecordingState::Idle {
                    let _ = self.app.emit("recording-error", "DEVICE_LOST: Active audio device was disconnected");
                    // Stop recording from a separate thread to avoid deadlock
                    let app = self.app.clone();
                    std::thread::spawn(move || {
                        let _ = crate::do_stop_recording(&app);
                    });
                }
            }
        }
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self, _flow: EDataFlow, _role: ERole, _default_device_id: &PCWSTR,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device_id: &PCWSTR, _key: &PROPERTYKEY) -> windows_core::Result<()> {
        Ok(())
    }
}

pub struct DeviceMonitorHandle {
    running: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl DeviceMonitorHandle {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.thread.take() {
            let _ = h.join();
        }
    }
}

impl Drop for DeviceMonitorHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_device_monitor(app: AppHandle) -> anyhow::Result<DeviceMonitorHandle> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let thread = std::thread::Builder::new()
        .name("device-monitor".into())
        .spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

                let enumerator: IMMDeviceEnumerator =
                    match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                        Ok(e) => e,
                        Err(e) => {
                            log::error!("Failed to create device enumerator: {e}");
                            return;
                        }
                    };

                let client: IMMNotificationClient =
                    DeviceNotificationClient { app }.into();

                if let Err(e) = enumerator.RegisterEndpointNotificationCallback(&client) {
                    log::error!("Failed to register notification callback: {e}");
                    return;
                }

                log::info!("Device monitor started");

                while running_clone.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }

                let _ = enumerator.UnregisterEndpointNotificationCallback(&client);
                CoUninitialize();
                log::info!("Device monitor stopped");
            }
        })?;

    Ok(DeviceMonitorHandle {
        running,
        thread: Some(thread),
    })
}

//! Turns off what WebView2 adds as a browser: its keys (F5 and Ctrl+R reload,
//! Ctrl+P prints, Ctrl+F, F3 and Ctrl+G search) and its context menu (the
//! Applications key, Shift+F10, a right click). Tauri 2.12 sets neither, and
//! wry leaves both on, so a reload would reset the window, lose an unsaved
//! profile and, during a recording, make NVDA say the recording started.
//!
//! Editing keys (Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+A, Ctrl+Z) are not browser keys
//! and keep working in text fields.

use tauri::{Manager, Runtime};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2Controller, ICoreWebView2Settings3, ICoreWebView2_11,
    COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
};
use webview2_com::{AcceleratorKeyPressedEventHandler, ContextMenuRequestedEventHandler};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_MENU};
use windows_core_webview2::{Interface, Result};

/// Applies both guards to the main window's webview.
pub fn disable_browser_features<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        log::error!("No main window to turn off the browser keys in");
        return;
    };
    let result = window.with_webview(|webview| {
        // SAFETY: COM calls on the webview's own thread, which `with_webview`
        // runs this closure on; the handlers live as long as the webview.
        if let Err(e) = unsafe { apply(&webview.controller()) } {
            log::error!("Failed to turn off the browser keys and menu: {e}");
        }
    });
    if let Err(e) = result {
        log::error!("Failed to reach the webview: {e}");
    }
}

/// The settings are WebView2's own switch for all browser keys and its menu,
/// but they may hold only from the next navigation on, and the page is
/// already loading when the app's setup runs. Reloading it would lose what
/// the window takes once at start, so two handlers cover the current page.
unsafe fn apply(controller: &ICoreWebView2Controller) -> Result<()> {
    unsafe {
        let core: ICoreWebView2 = controller.CoreWebView2()?;
        let settings = core.Settings()?;
        settings.SetAreDefaultContextMenusEnabled(false)?;
        settings
            .cast::<ICoreWebView2Settings3>()?
            .SetAreBrowserAcceleratorKeysEnabled(false)?;

        let mut token = 0i64;
        controller.add_AcceleratorKeyPressed(
            &AcceleratorKeyPressedEventHandler::create(Box::new(|_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
                args.KeyEventKind(&mut kind)?;
                if kind != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN {
                    return Ok(());
                }
                let mut key = 0u32;
                args.VirtualKey(&mut key)?;
                if is_browser_key(key, is_down(VK_CONTROL.0), is_down(VK_MENU.0)) {
                    args.SetHandled(true)?;
                }
                Ok(())
            })),
            &mut token,
        )?;

        // Without ICoreWebView2_11 (an old runtime) the setting alone remains.
        if let Ok(core11) = core.cast::<ICoreWebView2_11>() {
            core11.add_ContextMenuRequested(
                &ContextMenuRequestedEventHandler::create(Box::new(|_, args| {
                    if let Some(args) = args {
                        args.SetHandled(true)?;
                    }
                    Ok(())
                })),
                &mut token,
            )?;
        }
        Ok(())
    }
}

fn is_down(vk: u16) -> bool {
    // SAFETY: reads the calling thread's keyboard state; no pointers involved.
    unsafe { GetKeyState(i32::from(vk)) < 0 }
}

/// The browser keys that do something in WebView2 and nothing in the app:
/// F5 and F3 with Shift or Ctrl or alone; Ctrl+R, Ctrl+P, Ctrl+F, Ctrl+G with
/// or without Shift. Alt is left out, so Alt+letter mnemonics and AltGr
/// (Ctrl+Alt) typing never reach this.
fn is_browser_key(key: u32, ctrl: bool, alt: bool) -> bool {
    const VK_F3: u32 = 0x72;
    const VK_F5: u32 = 0x74;
    if alt {
        return false;
    }
    match key {
        VK_F3 | VK_F5 => true,
        k if ctrl => matches!(char::from_u32(k), Some('R' | 'P' | 'F' | 'G')),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_browser_key;

    #[test]
    fn browser_keys_are_caught() {
        assert!(is_browser_key(0x74, false, false)); // F5
        assert!(is_browser_key(0x74, true, false)); // Ctrl+F5
        assert!(is_browser_key(0x72, false, false)); // F3
        for letter in ['R', 'P', 'F', 'G'] {
            assert!(is_browser_key(letter as u32, true, false), "Ctrl+{letter}");
        }
    }

    #[test]
    fn app_and_editing_keys_pass() {
        assert!(!is_browser_key(0x70, false, false)); // F1
        assert!(!is_browser_key(0x1B, false, false)); // Escape
        assert!(!is_browser_key('R' as u32, false, false)); // plain R
        assert!(!is_browser_key('R' as u32, true, true)); // AltGr+R
        assert!(!is_browser_key(0x74, false, true)); // Alt+F5
        for letter in ['C', 'V', 'X', 'A', 'Z', 'N', 'S'] {
            assert!(!is_browser_key(letter as u32, true, false), "Ctrl+{letter}");
        }
    }
}

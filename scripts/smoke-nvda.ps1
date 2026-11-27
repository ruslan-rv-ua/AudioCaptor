<#
.SYNOPSIS
    Local NVDA smoke test of the built AudioCaptor.exe.

.DESCRIPTION
    Runs a copy of the built exe, presses keys the way a keyboard user does and reads from
    nvda.log what NVDA said at every step. NVDA must be running with the logging level
    "Input/output" (NVDA menu, Preferences, Settings, General). The report goes to the console
    and to %TEMP%\audiocaptor-smoke-report.txt.

    Scenarios; each one starts the app afresh and quits it at the end:
      launch   NVDA names the window once (AudioCaptor) and then the profile list; Tab goes
               round every control, each one has a name and NVDA reads it.
      record   Alt+S, Alt+P, Alt+R and Alt+T announce the new state once, Alt+I the current one,
               and a second Alt+I in the same state says it again.
               When the control with focus is disabled or goes away (Alt+S on the profile list,
               Alt+P on Pause, Alt+R on Resume, Alt+T on Stop), focus moves to the button of the
               new state and NVDA names it, without "unavailable"; focus on Stop stays there
               through a pause. Recording really starts (system audio of the default playback
               device, so no microphone is needed); the WAV file is not checked.
      dialogs  About (F1), Settings, New Profile and the delete confirmation: focus enters the
               dialog, Escape closes it and focus returns to where the dialog was opened. F1 in
               Settings opens About on top of Settings: UI Automation finds About in the middle
               of the window, and Settings again once About is closed. In New Profile, Tab
               reaches the Format list, NVDA reads its name and value, and Down changes it. In
               Edit Profile, Opus is chosen and saved: the profile in settings.json has it.
               Last, a new profile is saved with Enter in its Name field; in the next New
               Profile NVDA says nothing about autofill and one Escape closes the dialog.
      tray     Escape hides the window; starting the exe again shows it and NVDA announces it.
      settings Start with an unreadable settings.json: the app keeps a copy next to it
               (settings.unreadable.json), NVDA reads the message that the settings were reset
               and where the old file is kept.
      convert  Recordings left to convert to Opus by the last run (two generated WAV files, the
               first one large enough to take about half a minute): NVDA says that the app is
               finishing them, Alt+I says about how long is left and how many files wait
               (and says it again when pressed again a moment later),
               Alt+F4 asks before it exits, Exit leaves the WAV files whole and no half-written
               Opus file, and the next start finishes the job. While it runs a new recording
               starts and stops (it joins the queue), and NVDA says when all is converted. Last,
               a recording that cannot be converted: NVDA reads the message that says where its
               WAV is kept, and the WAV is still there. The generated files need -ConvertMB
               megabytes of free space in the temp folder.
      mp3      A recording in a profile with the format "MP3 - standard": when it stops, NVDA
               says that the conversion to MP3 started and then that it finished, and the
               recording is an .mp3 file. Then the same with AUDIOCAPTOR_NO_MP3_ENCODER=1, which
               makes the app act as if Windows had no MP3 encoder (Windows N without the Media
               Feature Pack): NVDA reads how to get the encoder and the end with errors, and the
               WAV is kept.
      flac     A profile with the format "FLAC (lossless)": in Edit Profile NVDA reads the
               Format list with that value. A recording in the profile: when it stops, NVDA says
               that the conversion to FLAC started and then that it finished, and the recording
               is a .flac file.

    Exit code: 0 everything passed; 1 a step or an NVDA check failed, or the test copy wrote to
    %LOCALAPPDATA%\com.audiocaptor.app; 2 the test could not run or stopped early (no exe, NVDA
    not running, wrong logging level, your AudioCaptor would not quit, an error in the test
    itself). The report says which.

    What it protects:
    - Your data. The test runs a copy of the exe from %TEMP%\audiocaptor-smoke with its own
      AudioCaptor-data (English interface, two test profiles). Like every copy of the app, it
      keeps the data of its built-in browser (WebView2) in AudioCaptor-webview next to its exe;
      the test checks that the copy created it there. AudioCaptor 0.1.1 and older kept that data
      in %LOCALAPPDATA%\com.audiocaptor.app\EBWebView. Nothing in
      %LOCALAPPDATA%\com.audiocaptor.app may change during the run, and the folder must not
      appear if it was not there: the app keeps all its data next to its exe (decision-1), so
      any change there fails the test. There the test reads only the names of the top entries,
      the number of files and folders in each and the time of the newest change. The test folder
      is deleted at the end; the copy's own log is kept as %TEMP%\audiocaptor-smoke-app.log.
    - Your AudioCaptor. The app allows one copy and holds the Pause key, so a running AudioCaptor
      is asked to quit, as its close button does, and is not started again. If it does not quit
      (for example, it is recording and asks to confirm), the test stops and leaves it alone.
    - Your other programs. While the test copy is hidden or closed, focus stays in a small window
      of this script, so NVDA does not read your other programs into the report. Notifications
      that NVDA reads during the run do end up in it: turn on "Do not disturb" to keep them out.
    - Only the part of nvda.log written during the run is read.

    Do not touch the keyboard or mouse while the test runs (two to three minutes for all
    scenarios; the convert scenario alone takes over a minute).

    How it starts and ends. Once the exe, NVDA and its log are found, one beep sounds and the test
    waits -PauseSec seconds before it presses the first key. When the run ends, three beeps sound
    if the scenarios ran (exit code 0 or 1) and two lower ones if the test stopped (exit code 2).
    The beeps are played as sound through the default playback device. An agent asks the person
    in its chat before it starts the test; the script itself asks nothing.

.PARAMETER Scenario
    One or more of launch, record, dialogs, tray, settings, convert, mp3, flac. All of them when omitted.

.PARAMETER ConvertMB
    The size of the large WAV file of the convert scenario, in megabytes. Default: 900, which a
    2024 desktop PC converts in about half a minute. The conversion has to last longer than the
    steps made while it runs (about 10 s); raise the number on a much faster PC if the report
    says that the conversion ended too early.

.PARAMETER PauseSec
    The pause between the first beep and the first key press, in seconds. Default: 5.

.PARAMETER Exe
    The exe to test. Default: src-tauri\target\release\AudioCaptor.exe (just build).

.PARAMETER NvdaLog
    NVDA's log file, if it is not nvda.log in the temp folder.

.EXAMPLE
    just smoke

.EXAMPLE
    just smoke tray

.EXAMPLE
    just smoke dialogs -PauseSec 10

.EXAMPLE
    powershell -NoProfile -File scripts/smoke-nvda.ps1 dialogs -Exe path\to\AudioCaptor.exe
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
    [string[]]$Scenario,
    [string]$Exe,
    [string]$NvdaLog,
    [int]$PauseSec = 5,
    [int]$ConvertMB = 900,
    [int]$TimeoutSec = 8,
    [int]$SpeechWaitSec = 5,
    [int]$SettleMs = 700
)

$ErrorActionPreference = 'Stop'

# --- Arguments ---------------------------------------------------------------------------------
$AllScenarios = @('launch', 'record', 'dialogs', 'tray', 'settings', 'convert', 'mp3', 'flac')
$Selected = @($Scenario | Where-Object { $_ } | ForEach-Object { $_.ToLowerInvariant() })
if ($Selected.Count -eq 0 -or $Selected -contains 'all') { $Selected = $AllScenarios }
$unknown = @($Selected | Where-Object { $AllScenarios -notcontains $_ })
# Reported once the beeps can play (after the native helpers below).
$ArgError = $null
if ($unknown.Count) {
    $ArgError = "Unknown scenario: $($unknown -join ', '). Scenarios: $($AllScenarios -join ', '); none for all of them."
}
$Selected = @($AllScenarios | Where-Object { $Selected -contains $_ })  # their own order, no repeats
if ($PauseSec -lt 0) { $ArgError = '-PauseSec must be 0 or more.' }
if ($ConvertMB -lt 1) { $ArgError = '-ConvertMB must be 1 or more.' }

# Windows PowerShell 5.1 does not set $PSScriptRoot inside param() defaults.
$DefaultExe = Join-Path $PSScriptRoot '..\src-tauri\target\release\AudioCaptor.exe'
if (-not $Exe) { $Exe = $DefaultExe }

# --- Constants ---------------------------------------------------------------------------------
$AppTitle = 'AudioCaptor'      # window title (tauri.conf.json) and process name
$HomeTitle = 'Smoke test home' # the test's own window: its title must not contain "AudioCaptor"
$ReportPath = Join-Path $env:TEMP 'audiocaptor-smoke-report.txt'
$AppLogCopy = Join-Path $env:TEMP 'audiocaptor-smoke-app.log'
# A fixed folder: Windows keeps a record for every exe path that shows a tray icon.
$Root = Join-Path $env:TEMP 'audiocaptor-smoke'
$TestExe = Join-Path $Root 'AudioCaptor.exe'
$DataDir = Join-Path $Root 'AudioCaptor-data'
$SettingsFile = Join-Path $DataDir 'settings.json'
# The app keeps the data of its built-in browser (WebView2) next to its exe
# ("appDirectoriesOverride" in tauri.conf.json); WebView2 puts it in an EBWebView subfolder.
$WebviewDir = Join-Path $Root 'AudioCaptor-webview'
$BrowserEntry = 'EBWebView'
# AudioCaptor 0.1.1 and older keep their browser data in EBWebView here (Tauri's default). The
# test copy must not change anything here: the app keeps all its data next to its exe (decision-1).
$UserAppDir = Join-Path $env:LOCALAPPDATA 'com.audiocaptor.app'

# English interface strings from messages/en.json (keys in the comments). The test copy runs
# with "language": "en".
$UI = @{
    ProfileList    = 'Recording profile'        # profile_section, the name of the profile list
    MinimizeToTray = 'Minimize to tray'         # btn_minimize_to_tray_aria
    About          = 'About AudioCaptor'        # about_btn_aria, about_title
    OpenSettings   = 'Open settings'            # settings_btn_aria
    CreateProfile  = 'Create new profile'       # create_profile_aria
    EditProfile    = 'Edit profile'             # edit_profile_aria
    DeleteProfile  = 'Delete profile'           # delete_profile_aria
    Microphone     = 'Microphone'               # mic_label
    LoopbackDevice = 'Loopback Device'          # loopback_label
    MicVolume      = 'Microphone Volume'        # mic_volume_label
    LoopbackVolume = 'Loopback Volume'          # loopback_volume_label
    Start          = 'Start recording (Alt+S)'  # start_recording_aria
    Pause          = 'Pause recording (Alt+P)'  # pause_recording_aria
    Resume         = 'Resume recording (Alt+R)' # resume_recording_aria
    Stop           = 'Stop recording (Alt+T)'   # stop_recording_aria
    Ready          = 'Ready'                    # status_ready (Alt+I while idle)
    Started        = 'Recording started'        # live_recording_started
    Paused         = 'Recording paused'         # live_recording_paused
    Resumed        = 'Recording resumed'        # live_recording_resumed
    Stopped        = 'Recording stopped'        # live_recording_stopped
    RecordingInfo  = 'Recording, 00:'           # live_status_info with status_recording
    PausedInfo     = 'Paused, 00:'              # live_status_info with status_paused
    Settings       = 'Settings'                 # settings_dialog_title
    CurrentHotkey  = 'Current hotkey: Pause'    # settings_hotkey_current_aria
    NewProfile     = 'New Profile'              # profile_dialog_create_title
    EditTitle      = 'Edit Profile'             # profile_dialog_edit_title
    Save           = 'Save'                     # btn_save
    Format         = 'Format'                   # field_output_format, the name of the format list
    FormatWav      = 'WAV (uncompressed)'       # format_wav
    FormatSpeech   = 'speech (mono)'            # format_opus_speech, the part after the dash
    FormatFlac     = 'FLAC (lossless)'          # format_flac
    DeleteTitle    = 'Delete this profile?'     # delete_profile_confirm
    Cancel         = 'Cancel'                   # btn_cancel
    Exit           = 'Exit'                     # btn_exit
    ExitTitle      = 'Exit AudioCaptor?'        # confirm_exit_conversion_title
    ExitBody       = 'Recordings are still being converted. If you exit now, AudioCaptor will finish converting them the next time it starts.'  # confirm_exit_conversion_body
    Resumed2       = 'Finishing the conversion of recordings from last time. Files: 2.'  # live_conversion_resumed
    Resumed1       = 'Finishing the conversion of recordings from last time. Files: 1.'  # live_conversion_resumed
    NotConverted   = 'Could not finish the conversion to Opus. The recording is kept as WAV:'  # error_conversion_failed
    EndWithErrors  = 'Conversion finished with errors. Recordings kept as WAV: 1.'  # live_conversion_finished_with_errors
    ReadyConverting = 'Ready. Converting to Opus'    # status_ready, then conversion_time_left or conversion_in_progress
    TimeLeft       = 'seconds left|minute left|minutes left'  # time_about_* inside conversion_time_left
    Waiting1       = 'Files waiting: 1.'        # conversion_files_waiting
    Waiting2       = 'Files waiting: 2.'        # conversion_files_waiting
    StoppedWaiting = 'Recording stopped. The recording is waiting to be converted to Opus.'  # live_recording_stopped_conversion_waiting
    Converted      = 'Conversion finished.'     # live_conversion_finished
    ConvertErrors  = 'Conversion finished with errors|Could not finish the conversion'  # live_conversion_finished_with_errors, error_conversion_failed
    Mp3Started     = 'Recording stopped. Conversion to MP3 started.'  # live_recording_stopped_conversion_started
    FlacStarted    = 'Recording stopped. Conversion to FLAC started.'  # live_recording_stopped_conversion_started
    NoEncoder      = 'Windows has no MP3 encoder. To record in MP3, install the Media Feature Pack'  # error_conversion_no_encoder
}

# The environment variable that makes the app act as if Windows had no MP3 encoder (mp3.rs).
$NoMp3Switch = 'AUDIOCAPTOR_NO_MP3_ENCODER'

# The two profiles of the test copy. The first one is active: system audio only, so Alt+S
# records without a microphone. The second one makes "Delete profile" available (the last
# profile cannot be deleted).
$Profiles = @{ System = 'Smoke system audio'; Spare = 'Smoke spare' }

# The controls of the main window in Tab order, as the test copy shows them: idle, two
# profiles, a playback device selected, no microphone.
$TabOrder = @($UI.MinimizeToTray, $UI.About, $UI.OpenSettings, $UI.ProfileList, $UI.CreateProfile,
    $UI.EditProfile, $UI.DeleteProfile, $UI.Microphone, $UI.LoopbackDevice, $UI.MicVolume,
    $UI.LoopbackVolume, $UI.Start)

# NVDA says role and state words in its own language; any variant counts. Ukrainian words are
# written as \u escapes so that this file stays ASCII: Windows PowerShell 5.1 reads a file without
# a BOM in the ANSI code page.
$Role = @{
    Dialog = 'dialog|' + [regex]::Unescape('\u0434\u0456\u0430\u043b\u043e\u0433')
}
# What NVDA says on a text field that has browser autofill suggestions ("has auto complete").
$Autofill = 'auto complete|autocomplete|' + [regex]::Unescape('\u0430\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u043d\u0435 \u0437\u0430\u043f\u043e\u0432\u043d\u0435\u043d\u043d\u044f')
# The name the dialogs scenario types into New Profile and saves with Enter: lower-case letters
# only, since Send-Keys types a letter key without Shift.
$AutofillName = 'zed'
# What NVDA says when the control with focus becomes disabled.
$Unavailable = 'unavailable|' + [regex]::Unescape('\u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u043d\u043e')

# What NVDA says when focus falls back onto the page: the app name, or the page's document and the
# application around the controls. Neither has a name of its own (backlog TASK-18), so NVDA names
# them by their roles.
$PageNoise = $AppTitle + '|document|' + [regex]::Unescape('\u0434\u043e\u043a\u0443\u043c\u0435\u043d\u0442') +
    '|application|' + [regex]::Unescape('\u0434\u043e\u0434\u0430\u0442\u043e\u043a')

# What NVDA says when it reads the web content through UI Automation instead of the usual way (the
# web content pane, the address of the page): a sign of the broken first reading of a WebView2
# window.
$WebviewNoise = 'tauri.localhost|web content|' + [regex]::Unescape('\u0432\u0435\u0431-\u0432\u043c\u0456\u0441\u0442')

# --- Native helpers ----------------------------------------------------------------------------
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, WindowsBase
if (-not ('AcSmokeNative' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class AcSmokeNative {
    [DllImport("user32.dll")] static extern uint MapVirtualKey(uint code, uint mapType);
    [DllImport("user32.dll")] static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool attach);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr hWnd);
    [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr hWnd, uint cmd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int max);
    delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc callback, IntPtr lParam);

    // The top-level window of this process with this title and at least one child window (the
    // web content), shown or hidden. UI Automation does not list a hidden window.
    public static IntPtr FindProcessWindow(uint processId, string title) {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr h, IntPtr l) {
            uint pid;
            GetWindowThreadProcessId(h, out pid);
            if (pid != processId) return true;
            StringBuilder text = new StringBuilder(256);
            GetWindowText(h, text, 256);
            if (text.ToString() != title || GetWindow(h, 5) == IntPtr.Zero) return true;  // GW_CHILD
            found = h;
            return false;
        }, IntPtr.Zero);
        return found;
    }

    // Windows lets only the foreground owner switch the foreground window. Sharing the input
    // state of the current foreground thread for a moment lets this process do it.
    public static bool Activate(IntPtr hwnd) {
        IntPtr fg = GetForegroundWindow();
        if (fg == hwnd) return true;
        uint me = GetCurrentThreadId();
        uint pid;
        uint fgThread = fg == IntPtr.Zero ? 0 : GetWindowThreadProcessId(fg, out pid);
        bool attached = fgThread != 0 && fgThread != me && AttachThreadInput(me, fgThread, true);
        try {
            BringWindowToTop(hwnd);
            SetForegroundWindow(hwnd);
        } finally {
            if (attached) AttachThreadInput(me, fgThread, false);
        }
        return GetForegroundWindow() == hwnd;
    }

    // Presses the keys in order and releases them in reverse order. Real scan codes: the web
    // content reads KeyboardEvent.code, which stays empty without one.
    public static void Chord(byte[] vks, bool[] extended) {
        for (int i = 0; i < vks.Length; i++) {
            keybd_event(vks[i], (byte)MapVirtualKey(vks[i], 0), extended[i] ? 1u : 0u, UIntPtr.Zero);
            Thread.Sleep(30);
        }
        for (int i = vks.Length - 1; i >= 0; i--) {
            keybd_event(vks[i], (byte)MapVirtualKey(vks[i], 0), (extended[i] ? 1u : 0u) | 2u, UIntPtr.Zero);
            Thread.Sleep(30);
        }
    }

    // Plays count beeps of hz for ms each, gapMs apart, as one sound through the default playback
    // device and waits until they end. The leading silence lets an idle audio device wake up
    // without cutting off the first beep.
    public static void Beeps(int hz, int ms, int gapMs, int count) {
        const int rate = 44100;
        int lead = rate / 10;
        int tone = rate * ms / 1000;
        int gap = rate * gapMs / 1000;
        int total = lead + count * tone + (count - 1) * gap;
        int fade = rate / 200;  // 5 ms in and out, against clicks
        System.IO.MemoryStream s = new System.IO.MemoryStream();
        System.IO.BinaryWriter w = new System.IO.BinaryWriter(s);
        w.Write(Encoding.ASCII.GetBytes("RIFF")); w.Write(36 + total * 2);
        w.Write(Encoding.ASCII.GetBytes("WAVEfmt ")); w.Write(16);
        w.Write((short)1); w.Write((short)1); w.Write(rate); w.Write(rate * 2); w.Write((short)2); w.Write((short)16);
        w.Write(Encoding.ASCII.GetBytes("data")); w.Write(total * 2);
        for (int i = 0; i < lead; i++) w.Write((short)0);
        for (int b = 0; b < count; b++) {
            if (b > 0) for (int i = 0; i < gap; i++) w.Write((short)0);
            for (int i = 0; i < tone; i++) {
                double env = Math.Min(1.0, Math.Min(i, tone - 1 - i) / (double)fade);
                w.Write((short)(Math.Sin(2 * Math.PI * hz * i / rate) * env * 0.4 * short.MaxValue));
            }
        }
        w.Flush();
        s.Position = 0;
        using (System.Media.SoundPlayer p = new System.Media.SoundPlayer(s)) p.PlaySync();
    }

    // Writes a 48 kHz stereo 16-bit WAV of the given number of frames: the same block of noise
    // over and over. Nothing plays it; it is only there to be converted. Returns its size.
    public static long WriteNoiseWav(string path, long frames) {
        byte[] block = new byte[1 << 20];
        new Random(28).NextBytes(block);
        long bytes = frames * 4;
        using (System.IO.FileStream f = System.IO.File.Create(path)) {
            System.IO.BinaryWriter w = new System.IO.BinaryWriter(f);
            w.Write(Encoding.ASCII.GetBytes("RIFF")); w.Write((uint)(36 + bytes));
            w.Write(Encoding.ASCII.GetBytes("WAVEfmt ")); w.Write(16);
            w.Write((short)1); w.Write((short)2); w.Write(48000); w.Write(48000 * 4); w.Write((short)4); w.Write((short)16);
            w.Write(Encoding.ASCII.GetBytes("data")); w.Write((uint)bytes);
            w.Flush();
            for (long left = bytes; left > 0; ) {
                int n = (int)Math.Min(left, block.Length);
                f.Write(block, 0, n);
                left -= n;
            }
            return f.Length;
        }
    }

    [ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
    class MMDeviceEnumerator { }

    [ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IMMDeviceEnumerator {
        [PreserveSig] int EnumAudioEndpoints(int dataFlow, int stateMask, out IntPtr devices);
        [PreserveSig] int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice endpoint);
    }

    [ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IMMDevice {
        [PreserveSig] int Activate(ref Guid iid, int clsCtx, IntPtr activationParams, out IntPtr iface);
        [PreserveSig] int OpenPropertyStore(int access, out IntPtr properties);
        [PreserveSig] int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
        [PreserveSig] int GetState(out int state);
    }

    // The endpoint id of the default playback device (eRender, eConsole), the same id the app
    // lists for it, or null when Windows has none.
    public static string DefaultPlaybackDeviceId() {
        IMMDeviceEnumerator enumerator = (IMMDeviceEnumerator)new MMDeviceEnumerator();
        IMMDevice device;
        if (enumerator.GetDefaultAudioEndpoint(0, 0, out device) != 0 || device == null) return null;
        string id;
        if (device.GetId(out id) != 0) return null;
        return id;
    }
}
'@
}
$AE = [System.Windows.Automation.AutomationElement]
$Scope = [System.Windows.Automation.TreeScope]

# --- Beeps -------------------------------------------------------------------------------------
# One beep before the wait; three when the scenarios ran, two lower ones when the test stopped.
# A sound that cannot play must not change the result.
function Invoke-Beep([ValidateSet('start', 'done', 'stopped')][string]$Kind) {
    try {
        switch ($Kind) {
            'start'   { [AcSmokeNative]::Beeps(880, 250, 0, 1) }
            'done'    { [AcSmokeNative]::Beeps(1000, 150, 120, 3) }
            'stopped' { [AcSmokeNative]::Beeps(440, 300, 150, 2) }
        }
    } catch { Write-Host "(The beep could not play: $($_.Exception.Message))" }
}

if ($ArgError) {
    Write-Host $ArgError
    Invoke-Beep 'stopped'
    exit 2
}

# --- Keyboard ----------------------------------------------------------------------------------
$VK = @{ Ctrl = 0x11; Alt = 0x12; Shift = 0x10; Enter = 0x0D; Tab = 0x09; Esc = 0x1B; Space = 0x20
         End = 0x23; Home = 0x24; Left = 0x25; Up = 0x26; Right = 0x27; Down = 0x28; F1 = 0x70; F4 = 0x73 }
$ExtendedVK = @(0x23, 0x24, 0x25, 0x26, 0x27, 0x28)

# Each chord is keys joined with '+', such as 'Alt+S' or 'Shift+Tab'.
function Send-Keys([string[]]$Chords) {
    foreach ($chord in $Chords) {
        $vks = New-Object System.Collections.Generic.List[byte]
        $ext = New-Object System.Collections.Generic.List[bool]
        foreach ($name in $chord.Split('+')) {
            if ($VK.ContainsKey($name)) { $code = [int]$VK[$name] }
            elseif ($name.Length -eq 1) { $code = [int][char]$name.ToUpperInvariant() }
            else { throw "Unknown key: $name" }
            $vks.Add([byte]$code)
            $ext.Add($ExtendedVK -contains $code)
        }
        [AcSmokeNative]::Chord($vks.ToArray(), $ext.ToArray())
        Start-Sleep -Milliseconds 250
    }
}

# --- Waiting -----------------------------------------------------------------------------------
# Returns the first truthy result of $Condition; throws when there is none within $Seconds.
function Wait-Until([string]$What, [scriptblock]$Condition, [int]$Seconds = $TimeoutSec) {
    $deadline = (Get-Date).AddSeconds($Seconds)
    do {
        try { $r = & $Condition; if ($r) { return $r } } catch { }
        Start-Sleep -Milliseconds 100
    } while ((Get-Date) -lt $deadline)
    throw "timed out waiting for $What"
}

# --- The app window ----------------------------------------------------------------------------
$script:AppProc = $null

# Win32 handle of the test copy's window, shown or hidden; zero when there is none.
function Get-AppHandle {
    if (-not $script:AppProc) { return [IntPtr]::Zero }
    return [AcSmokeNative]::FindProcessWindow([uint32]$script:AppProc.Id, $AppTitle)
}

function Test-AppShown {
    $h = Get-AppHandle
    return $h -ne [IntPtr]::Zero -and [AcSmokeNative]::IsWindowVisible($h)
}

function Test-AppForeground {
    $h = Get-AppHandle
    return $h -ne [IntPtr]::Zero -and [AcSmokeNative]::IsWindowVisible($h) -and [AcSmokeNative]::GetForegroundWindow() -eq $h
}

# The UI Automation element of the visible app window, or $null.
function Get-AppWindow {
    if (-not $script:AppProc) { return $null }
    $c = [System.Windows.Automation.AndCondition]::new([System.Windows.Automation.Condition[]]@(
            [System.Windows.Automation.PropertyCondition]::new($AE::ProcessIdProperty, $script:AppProc.Id),
            [System.Windows.Automation.PropertyCondition]::new($AE::NameProperty, $AppTitle)))
    return $AE::RootElement.FindFirst($Scope::Children, $c)
}

function Find-InApp($Property, [string]$Value) {
    $w = Get-AppWindow
    if ($null -eq $w) { return $null }
    return $w.FindFirst($Scope::Descendants, [System.Windows.Automation.PropertyCondition]::new($Property, $Value))
}

# An element of the web content by its DOM id or by its accessible name.
function Find-ById([string]$Id) { return Find-InApp $AE::AutomationIdProperty $Id }
function Find-ByName([string]$Name) { return Find-InApp $AE::NameProperty $Name }

# The element that has the keyboard: name, control type and DOM id, or $null.
function Get-Focus {
    try {
        $e = $AE::FocusedElement
        if ($null -eq $e) { return $null }
        $name = [string]$e.Current.Name
        $type = $e.Current.ControlType.ProgrammaticName -replace '^ControlType\.', ''
        $id = [string]$e.Current.AutomationId
        return [pscustomobject]@{ Name = $name; Type = $type; Id = $id; Key = "$type|$id|$name" }
    } catch { return $null }
}

function Format-Focus($Focus) {
    if (-not $Focus) { return 'nothing' }
    if ($Focus.Name) { return "'$($Focus.Name)' ($($Focus.Type))" }
    return "an element without a name ($($Focus.Type))"
}

# Starts the test copy (or a second copy of it).
function Start-TestCopy {
    return Start-Process -FilePath $TestExe -WorkingDirectory $Root -PassThru
}

# Waits, without UI Automation, until the window is on screen and in the foreground, then gives
# NVDA time to read it: queries into the web content must not change how NVDA first reads it.
# Returns $false if the window did not come to the foreground.
function Wait-ShownQuietly([int]$Seconds = 20, [int]$ReadMs = 1500) {
    $deadline = (Get-Date).AddSeconds($Seconds)
    do {
        if ($script:AppProc.HasExited) { return $false }
        if (Test-AppForeground) {
            $script:ShownAfterMs = [int]((Get-Date) - $script:AppProc.StartTime).TotalMilliseconds
            Start-Sleep -Milliseconds $ReadMs
            return $true
        }
        Start-Sleep -Milliseconds 20
    } while ((Get-Date) -lt $deadline)
    return $false
}

# Settings of the test copy, written before every start: English, two profiles, the default
# playback device selected for system audio. No BOM: serde_json rejects it, and the app then
# silently falls back to its defaults.
function Write-TestSettings {
    if ($script:BrokenSettings) {
        # Cut off in the middle, as after a power loss while saving: not valid JSON.
        [IO.File]::WriteAllText($SettingsFile, '{ "version": 5, "profiles": [', (New-Object Text.UTF8Encoding $false))
        return
    }
    # Profiles as settings version 5 wrote them, without a format: every start of the test copy
    # also goes through the migration to version 6. The convert, mp3 and flac scenarios give the active
    # profile a format ($script:ProfileFormat), so that its recording is converted.
    $newProfile = {
        param([string]$Id, [string]$Name, [string]$Mode, [string]$Format)
        $p = [ordered]@{ id = $Id; name = $Name; description = ''; outputFolder = 'Recordings'; outputMode = $Mode
                         sampleRate = 48000; micVolume = 1.0; loopbackVolume = 0.5
                         micFilename = 'mic'; loopbackFilename = 'loopback'; mixFilename = 'mix' }
        if ($Format) { $p.outputFormat = $Format }
        $p
    }
    $systemFormat = $script:ProfileFormat
    $settings = [ordered]@{
        version                    = 5
        selectedMic                = $null
        selectedLoopback           = $script:PlaybackDeviceId
        hotkey                     = 'Pause'
        soundEnabled               = $true
        profiles                   = @((& $newProfile 'smoke-system' $Profiles.System 'Loopback' $systemFormat),
                                       (& $newProfile 'smoke-spare' $Profiles.Spare 'Mix' ''))
        activeProfileId            = 'smoke-system'
        language                   = 'en'
        confirmExitDuringRecording = $true
        theme                      = 'dark'
        minimizeToTrayOnFocusLoss  = $false
    }
    [IO.File]::WriteAllText($SettingsFile, ($settings | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
}

# Starts the test copy and waits until its window is on screen, in the foreground, and focus is
# on the profile list, where the app puts it once it has loaded.
function Start-App {
    Write-TestSettings
    $script:ShownAfterMs = $null
    $script:AppProc = Start-TestCopy
    if (Wait-ShownQuietly) {
        $script:StepInfo.Add("window in the foreground $($script:ShownAfterMs) ms after the process started")
    } else {
        if ($script:AppProc.HasExited) { throw "the app quit right after the start (exit code $($script:AppProc.ExitCode))" }
        if (-not (Test-AppShown)) { throw 'the window did not appear within 20 s' }
        $null = [AcSmokeNative]::Activate((Get-AppHandle))
        $script:StepInfo.Add('the window appeared but did not come to the foreground by itself; the test brought it there')
    }
    if (-not (Test-Path -LiteralPath (Join-Path $WebviewDir $BrowserEntry))) {
        throw "the test copy did not put its browser data (WebView2) into $WebviewDir, next to its exe"
    }
    # After a reset the interface speaks the Windows language, not the test copy's English, so the
    # list has another name there.
    if (-not $script:BrokenSettings) {
        $null = Wait-Until "focus on the '$($UI.ProfileList)' list" { (Get-Focus).Name -eq $UI.ProfileList } 15
    }
}

# Quits the test copy the way its close button does (WM_CLOSE; the app quits when it is not
# recording), so that its tray icon goes away. Ends the process if it is still there after 5 s.
function Stop-App {
    $p = $script:AppProc
    $script:AppProc = $null
    if (-not $p -or $p.HasExited) { return }
    $h = [AcSmokeNative]::FindProcessWindow([uint32]$p.Id, $AppTitle)
    if ($h -ne [IntPtr]::Zero) { $null = [AcSmokeNative]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) }  # WM_CLOSE
    if (-not $p.WaitForExit(5000)) {
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        $null = $p.WaitForExit(5000)
        $Notes.Add("[$script:CurrentScenario] the test copy did not quit within 5 s of WM_CLOSE and was ended.")
    }
}

# Tab inside a dialog until $Reached is true for the control with focus.
function Move-InDialog([scriptblock]$Reached, [string]$What) {
    for ($k = 0; $k -lt 15 -and -not (& $Reached); $k++) {
        $prev = Get-Focus
        Send-Keys 'Tab'
        try { $null = Wait-Until 'focus to move' { $g = Get-Focus; $g -and (-not $prev -or $g.Key -ne $prev.Key) } 3 } catch { }
    }
    if (-not (& $Reached)) { throw "Tab did not reach $What in 15 presses" }
}

# Tab or Shift+Tab, whichever is shorter by $TabOrder, until the control with this name has focus.
function Move-Focus([string]$Name) {
    $f = Get-Focus
    if ($f -and $f.Name -eq $Name) { return }
    $back = $false
    $i = if ($f) { [array]::IndexOf($TabOrder, [string]$f.Name) } else { -1 }
    $j = [array]::IndexOf($TabOrder, $Name)
    if ($i -ge 0 -and $j -ge 0) {
        $n = $TabOrder.Count
        $back = (($i - $j + $n) % $n) -lt (($j - $i + $n) % $n)
    }
    $key = if ($back) { 'Shift+Tab' } else { 'Tab' }
    for ($k = 0; $k -lt 20; $k++) {
        $prev = Get-Focus
        Send-Keys $key
        try { $null = Wait-Until 'focus to move' { $g = Get-Focus; $g -and (-not $prev -or $g.Key -ne $prev.Key) } 3 } catch { }
        if ((Get-Focus).Name -eq $Name) { return }
    }
    throw "$key did not reach '$Name' in 20 presses"
}

# Focus must end on the control with this name; the app may move it there a moment after the
# change. Wrong focus fails the step but not the scenario.
function Confirm-Focus([string]$Name) {
    try { $null = Wait-Until "focus on '$Name'" { (Get-Focus).Name -eq $Name } 3 } catch { }
    $f = Get-Focus
    if (-not $f -or $f.Name -ne $Name) {
        $script:SoftFail = "focus went to $(Format-Focus $f) instead of '$Name'"
    }
}

# Escape closes the dialog that contains the element with this DOM id; focus must then return to
# the control named $Opener (the app moves it once the dialog is gone, on the next frame).
function Close-Dialog([string]$InsideId, [string]$Opener) {
    Send-Keys 'Esc'
    $null = Wait-Until 'the dialog to close' { -not (Find-ById $InsideId) } 3
    Confirm-Focus $Opener
}

# The names of the element in the middle of the app window and of its parents, innermost first.
# UI Automation finds that element by what is drawn there, so the dialog drawn on top is among them.
function Get-NamesAtCenter {
    $r = (Get-AppWindow).Current.BoundingRectangle
    $e = $AE::FromPoint((New-Object System.Windows.Point ($r.X + $r.Width / 2), ($r.Y + $r.Height / 2)))
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    $names = New-Object System.Collections.Generic.List[string]
    for ($i = 0; $e -and $i -lt 30; $i++) {
        $names.Add([string]$e.Current.Name)
        $e = $walker.GetParent($e)
    }
    return $names.ToArray()
}

# The dialog named $Name, not the one named $Not, must be drawn in the middle of the window.
function Test-OnTop([string]$Name, [string]$Not) {
    try { $null = Wait-Until "the '$Name' dialog in the middle of the window" { $n = @(Get-NamesAtCenter); ($n -contains $Name) -and ($n -notcontains $Not) } 2 } catch { }
    $names = @(Get-NamesAtCenter)
    $shown = (@($names | Where-Object { $_ } | ForEach-Object { "'$_'" })) -join ' < '
    $script:StepInfo.Add("in the middle of the window: $shown")
    Add-Check "the '$Name' dialog is drawn on top" (($names -contains $Name) -and ($names -notcontains $Not)) "the middle of the window shows $shown"
}

# --- The test's own window ---------------------------------------------------------------------
# A small window with one text field, open for the whole run, on its own STA thread. The app
# window opens over it, so focus returns to it whenever the app hides or quits, and NVDA reads it
# instead of your own programs.
$script:HomeSync = $null
$script:HomePs = $null

function Start-HomeWindow {
    $script:HomeSync = [hashtable]::Synchronized(@{ Handle = [IntPtr]::Zero; Title = $HomeTitle })
    $rs = [runspacefactory]::CreateRunspace()
    $rs.ApartmentState = 'STA'
    $rs.ThreadOptions = 'ReuseThread'
    $rs.Open()
    $rs.SessionStateProxy.SetVariable('sync', $script:HomeSync)
    $ps = [powershell]::Create()
    $ps.Runspace = $rs
    $null = $ps.AddScript({
            Add-Type -AssemblyName System.Windows.Forms
            $f = New-Object System.Windows.Forms.Form
            $f.Text = $sync.Title
            $f.StartPosition = 'CenterScreen'
            $f.Width = 500
            $f.Height = 120
            $t = New-Object System.Windows.Forms.TextBox
            $t.Dock = 'Top'
            $t.AccessibleName = 'Smoke test field'
            $f.Controls.Add($t)
            $f.Add_Shown({ $sync.Handle = $f.Handle; $f.Activate(); $t.Focus() })
            [void]$f.ShowDialog()
            $f.Dispose()
        })
    $script:HomePs = $ps
    $script:HomeAsync = $ps.BeginInvoke()
    $null = Wait-Until 'the test window' { $script:HomeSync.Handle -ne [IntPtr]::Zero } 15
    Enter-HomeWindow
}

function Test-HomeFocused {
    return $script:HomeSync -and $script:HomeSync.Handle -ne [IntPtr]::Zero -and
        [AcSmokeNative]::GetForegroundWindow() -eq $script:HomeSync.Handle
}

# Puts the test window in the foreground (Windows may refuse the first attempt).
function Enter-HomeWindow {
    if (-not $script:HomeSync) { return }
    try { $null = Wait-Until 'focus in the test window' { Test-HomeFocused } 1 }
    catch {
        $null = Wait-Until 'focus in the test window' {
            $null = [AcSmokeNative]::Activate($script:HomeSync.Handle)
            Start-Sleep -Milliseconds 150
            Test-HomeFocused
        } 5
    }
}

function Stop-HomeWindow {
    if ($script:HomeSync -and $script:HomeSync.Handle -ne [IntPtr]::Zero) {
        $null = [AcSmokeNative]::PostMessage($script:HomeSync.Handle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)  # WM_CLOSE
    }
    if ($script:HomePs) {
        try { $null = $script:HomeAsync.AsyncWaitHandle.WaitOne(3000) } catch { }
        try { $script:HomePs.Runspace.Dispose(); $script:HomePs.Dispose() } catch { }
    }
    $script:HomePs = $null
    $script:HomeSync = $null
}

# --- NVDA log ----------------------------------------------------------------------------------
# Positions are byte offsets in nvda.log: the run reads only what NVDA wrote after it started.
$script:LogPath = $null
$script:LogStart = 0L

function Find-NvdaLog {
    if ($NvdaLog) {
        if (Test-Path -LiteralPath $NvdaLog) { return (Resolve-Path -LiteralPath $NvdaLog).Path }
        return $null
    }
    $dirs = @($env:TEMP, $env:TMP, [Environment]::GetEnvironmentVariable('TEMP', 'User'), (Join-Path $env:LOCALAPPDATA 'Temp')) |
        Where-Object { $_ } | ForEach-Object { [Environment]::ExpandEnvironmentVariables($_) } | Select-Object -Unique
    $logs = @($dirs | ForEach-Object { Join-Path $_ 'nvda.log' } | Where-Object { Test-Path -LiteralPath $_ } |
            ForEach-Object { Get-Item -LiteralPath $_ })
    if (-not $logs.Count) { return $null }
    return ($logs | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}

# The logging level saved in NVDA's configuration, for the explanation when nothing is logged.
function Get-NvdaSavedLevel {
    $ini = Join-Path $env:APPDATA 'nvda\nvda.ini'
    if (-not (Test-Path -LiteralPath $ini)) { return $null }
    $m = Select-String -LiteralPath $ini -Pattern '^\s*loggingLevel\s*=\s*(\S+)' | Select-Object -First 1
    if ($m) { return $m.Matches[0].Groups[1].Value }
    return 'INFO (not set, the default)'
}

function Get-LogLength {
    $fs = [IO.File]::Open($script:LogPath, 'Open', 'Read', 'ReadWrite')
    try { return $fs.Length } finally { $fs.Dispose() }
}

function Read-LogRange([long]$From, [long]$To) {
    $fs = [IO.File]::Open($script:LogPath, 'Open', 'Read', 'ReadWrite')
    try {
        if ($From -gt $fs.Length) { $From = 0 }  # NVDA restarted and began a new log
        if ($To -gt $fs.Length -or $To -lt $From) { $To = $fs.Length }
        $count = [int]($To - $From)
        $buf = New-Object byte[] $count
        $null = $fs.Seek($From, 'Begin')
        $read = 0
        while ($read -lt $count) {
            $n = $fs.Read($buf, $read, $count - $read)
            if ($n -le 0) { break }
            $read += $n
        }
        return [Text.Encoding]::UTF8.GetString($buf, 0, $read)
    } finally { $fs.Dispose() }
}

$HeaderRx = [regex]'^(?<lvl>[A-Z]+) - (?<src>\S+) \((?<t>\d\d:\d\d:\d\d\.\d{3})\) - .*:\s*$'
# Speech commands such as LangChangeCommand ('uk_UA') carry quoted arguments that are not speech.
$CommandRx = [regex]"\b\w+Command\s*\((?:[^()'""]|'(?:[^'\\]|\\.)*'|""(?:[^""\\]|\\.)*"")*\)"
$StringRx = [regex]"'((?:[^'\\]|\\.)*)'|""((?:[^""\\]|\\.)*)"""

function ConvertFrom-PyLiteral([string]$Text) {
    return [regex]::Replace($Text, '\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|.)', {
            param($m)
            $e = $m.Groups[1].Value
            if ($e.Length -gt 1) { return [string][char][Convert]::ToInt32($e.Substring(1), 16) }
            if ($e -ceq 'n' -or $e -ceq 't' -or $e -ceq 'r') { return ' ' }
            return $e
        })
}

# One "Speaking [...]" line as text: its strings joined with ', '.
function Get-SpeechText([string]$Line) {
    $body = $CommandRx.Replace($Line.Substring('Speaking'.Length), '')
    $parts = foreach ($m in $StringRx.Matches($body)) {
        $raw = if ($m.Groups[1].Success) { $m.Groups[1].Value } else { $m.Groups[2].Value }
        $t = ((ConvertFrom-PyLiteral $raw) -replace '\s+', ' ').Trim()
        if ($t) { $t }
    }
    return ($parts -join ', ')
}

# What NVDA said (Kind 'say') and its errors (Kind 'error') in a piece of its log.
function Get-NvdaEvents([string]$Text) {
    $cur = $null
    foreach ($line in ($Text -split '\r?\n')) {
        $h = $HeaderRx.Match($line)
        if ($h.Success) { $cur = $h; continue }
        if ($null -eq $cur) { continue }
        $src = $cur.Groups['src'].Value
        $lvl = $cur.Groups['lvl'].Value
        if ($src -eq 'speech.speech.speak' -and $line.StartsWith('Speaking')) {
            $s = Get-SpeechText $line
            if ($s) { [pscustomobject]@{ Kind = 'say'; Text = $s } }
        } elseif ($lvl -eq 'ERROR' -or $lvl -eq 'CRITICAL') {
            [pscustomobject]@{ Kind = 'error'; Text = ('{0}: {1}' -f $src, $line.Trim()) }
        }
        $cur = $null
    }
}

# Everything NVDA said since this log position, utterances joined with ' | '.
function Get-SaidSince([long]$From) {
    return (@(Get-NvdaEvents (Read-LogRange $From (Get-LogLength)) | Where-Object { $_.Kind -eq 'say' } |
                ForEach-Object { $_.Text }) -join ' | ')
}

function Get-Count([string]$Haystack, [string]$Needle) {
    if (-not $Needle -or -not $Haystack) { return 0 }
    $i = 0; $n = 0
    while (($i = $Haystack.IndexOf($Needle, $i, [StringComparison]::OrdinalIgnoreCase)) -ge 0) {
        $n++
        $i += $Needle.Length
    }
    return $n
}

# True when any of the '|'-separated variants was said.
function Test-SaidAny([string]$Haystack, [string]$Variants) {
    foreach ($v in $Variants.Split('|')) { if ((Get-Count $Haystack $v) -ge 1) { return $true } }
    return $false
}

function Format-Variants([string]$Variants) {
    return (($Variants.Split('|') | ForEach-Object { """$_""" }) -join ' or ')
}

# NVDA writes speech and key presses to its log only at the "Input/output" level (or "Debug").
# The test window has just taken focus, so NVDA has read it; End in its text field adds a key.
# Returns 'ok', 'nospeech' (key presses logged, no speech) or 'nolog'.
function Test-NvdaLogging {
    Send-Keys 'End'
    $deadline = (Get-Date).AddSeconds(5)
    $io = $false
    do {
        Start-Sleep -Milliseconds 250
        $text = Read-LogRange $script:LogStart (Get-LogLength)
        $io = [regex]::IsMatch($text, '(?m)^IO - ')
        $spoke = @(Get-NvdaEvents $text | Where-Object { $_.Kind -eq 'say' }).Count -gt 0
        if ($io -and $spoke) { return 'ok' }
    } while ((Get-Date) -lt $deadline)
    if ($io) { return 'nospeech' }
    return 'nolog'
}

# --- Steps -------------------------------------------------------------------------------------
$Results = New-Object System.Collections.Generic.List[object]
$Notes = New-Object System.Collections.Generic.List[string]
$NvdaErrors = New-Object System.Collections.Generic.List[string]
$script:CurrentScenario = ''
$script:StepNumber = 0
$script:Aborted = $false
$script:SoftFail = $null
$script:StepInfo = New-Object System.Collections.Generic.List[string]
$script:StepChecks = New-Object System.Collections.Generic.List[object]
$script:StopReason = $null

function New-Check([string]$Text, [bool]$Ok, [string]$Detail = '') {
    return [pscustomobject]@{ Text = $Text; Ok = $Ok; Detail = $Detail }
}

# A check computed by the step itself, reported with the NVDA checks.
function Add-Check([string]$Text, [bool]$Ok, [string]$Detail = '') {
    $script:StepChecks.Add((New-Check $Text $Ok $Detail))
}

# Runs one step of the current scenario and records what NVDA said meanwhile.
# Expect    - each must be said at least once
# Once      - each must be said exactly once (a name NVDA must not repeat)
# ExpectAny - for each entry, one of its '|'-separated variants must be said (role words, which
#             depend on NVDA's language)
# Forbid    - none of its '|'-separated variants may be said
# A step fails when its action throws (the rest of the scenario is then skipped), when it sets
# $script:SoftFail (the scenario goes on), or when a check fails.
function Invoke-Step {
    param(
        [string]$Title,
        [scriptblock]$Action,
        [string[]]$Expect = @(),
        [string[]]$Once = @(),
        [string[]]$ExpectAny = @(),
        [string[]]$Forbid = @()
    )
    $script:StepNumber++
    $r = [pscustomobject]@{ Scenario = $script:CurrentScenario; N = $script:StepNumber; Title = $Title
                            Status = 'OK'; Detail = ''; Info = @(); Speech = @(); Checks = @() }
    $Results.Add($r)
    if ($script:Aborted) {
        $r.Status = 'SKIPPED'
        $r.Detail = 'an earlier step of this scenario failed'
        Write-Host ("  {0}. {1}: SKIPPED" -f $r.N, $Title)
        return
    }

    $from = Get-LogLength
    $script:SoftFail = $null
    $script:StepInfo = New-Object System.Collections.Generic.List[string]
    $script:StepChecks = New-Object System.Collections.Generic.List[object]
    try { $null = & $Action }
    catch { $r.Status = 'FAILED'; $r.Detail = $_.Exception.Message; $script:Aborted = $true }
    if ($r.Status -eq 'OK' -and $script:SoftFail) { $r.Status = 'FAILED'; $r.Detail = $script:SoftFail }

    if ($r.Status -eq 'OK' -and ($Expect.Count -or $Once.Count -or $ExpectAny.Count)) {
        # NVDA can lag behind the app: wait until it has said what is expected.
        $deadline = (Get-Date).AddSeconds($SpeechWaitSec)
        do {
            $said = Get-SaidSince $from
            $done = $true
            foreach ($p in @($Expect + $Once)) { if ((Get-Count $said $p) -lt 1) { $done = $false } }
            foreach ($a in $ExpectAny) { if (-not (Test-SaidAny $said $a)) { $done = $false } }
            if ($done) { break }
            Start-Sleep -Milliseconds 200
        } while ((Get-Date) -lt $deadline)
    }
    Start-Sleep -Milliseconds $SettleMs  # trailing speech, such as a repeated announcement

    $events = @(Get-NvdaEvents (Read-LogRange $from (Get-LogLength)))
    $r.Speech = @($events | Where-Object { $_.Kind -eq 'say' } | ForEach-Object { $_.Text })
    foreach ($e in $events) { if ($e.Kind -eq 'error') { $NvdaErrors.Add($e.Text) } }
    $all = $r.Speech -join ' | '
    $checks = New-Object System.Collections.Generic.List[object]
    foreach ($p in $Expect) { $checks.Add((New-Check "NVDA says ""$p""" ((Get-Count $all $p) -ge 1))) }
    foreach ($p in $Once) {
        $n = Get-Count $all $p
        $checks.Add((New-Check "NVDA says ""$p"" once" ($n -eq 1) "said $n times"))
    }
    foreach ($a in $ExpectAny) { $checks.Add((New-Check "NVDA says $(Format-Variants $a)" (Test-SaidAny $all $a))) }
    foreach ($p in $Forbid) { $checks.Add((New-Check "NVDA does not say $(Format-Variants $p)" (-not (Test-SaidAny $all $p)))) }
    foreach ($c in $script:StepChecks) { $checks.Add($c) }
    $r.Checks = $checks.ToArray()
    $r.Info = $script:StepInfo.ToArray()

    $failedChecks = @($r.Checks | Where-Object { -not $_.Ok }).Count
    $mark = if ($r.Status -ne 'OK') { $r.Status } elseif ($failedChecks) { "CHECKS FAILED: $failedChecks" } else { 'OK' }
    Write-Host ("  {0}. {1}: {2}" -f $r.N, $Title, $mark)
}

# --- Scenarios ---------------------------------------------------------------------------------
# Tab from the profile list until focus comes back to it. Every stop must have a name that NVDA
# reads, and every control of $TabOrder must be among the stops.
function Test-TabRound {
    $start = Get-Focus
    if (-not $start -or $start.Name -ne $UI.ProfileList) { throw "focus is on $(Format-Focus $start), not on the '$($UI.ProfileList)' list" }
    $stops = New-Object System.Collections.Generic.List[object]
    $prev = $start
    $round = $false
    for ($i = 1; $i -le 30 -and -not $round; $i++) {
        $from = Get-LogLength
        Send-Keys 'Tab'
        $now = $null
        try { $now = Wait-Until 'focus to move' { $g = Get-Focus; if ($g -and $g.Key -ne $prev.Key) { $g } } 3 } catch { $now = Get-Focus }
        if (-not $now) { $now = [pscustomobject]@{ Name = ''; Type = 'nothing'; Id = ''; Key = "nothing|$i" } }
        # Let NVDA read the new focus before the next Tab.
        if ($now.Name) {
            $deadline = (Get-Date).AddSeconds($SpeechWaitSec)
            while ((Get-Date) -lt $deadline -and (Get-Count (Get-SaidSince $from) $now.Name) -lt 1) { Start-Sleep -Milliseconds 150 }
        }
        Start-Sleep -Milliseconds 300
        $said = Get-SaidSince $from
        $stops.Add([pscustomobject]@{ N = $i; Name = $now.Name; Type = $now.Type; Said = $said })
        if ($now.Key -eq $start.Key) { $round = $true }
        $prev = $now
    }
    foreach ($s in $stops) {
        $what = if ($s.Said) { $s.Said } else { 'nothing' }
        $script:StepInfo.Add(('Tab {0}: {1}; NVDA said: {2}' -f $s.N, (Format-Focus $s), $what))
    }
    $unnamed = @($stops | Where-Object { -not $_.Name } | ForEach-Object { "Tab $($_.N) ($($_.Type))" })
    Add-Check 'every Tab stop has a name' ($unnamed.Count -eq 0) ($unnamed -join ', ')
    $unread = @($stops | Where-Object { $_.Name -and (Get-Count $_.Said $_.Name) -lt 1 } | ForEach-Object { "Tab $($_.N) '$($_.Name)'" })
    Add-Check 'NVDA reads the name at every Tab stop' ($unread.Count -eq 0) ($unread -join ', ')
    $missed = @($TabOrder | Where-Object { $n = $_; -not @($stops | Where-Object { $_.Name -eq $n }).Count } | ForEach-Object { "'$_'" })
    Add-Check 'Tab reaches every control of the window' ($missed.Count -eq 0) "not reached: $($missed -join ', ')"
    Add-Check "Tab comes back to '$($UI.ProfileList)' (a full round)" $round "not within $($stops.Count) presses"
    $extra = @($stops | Where-Object { $_.Name -ne $UI.ProfileList -and $TabOrder -notcontains $_.Name } | ForEach-Object { Format-Focus $_ })
    if ($extra.Count) { $script:StepInfo.Add("Tab also stops on: $($extra -join ', ')") }
}

function Invoke-LaunchScenario {
    # On the first show NVDA names the window, then the page's document and the application around
    # the controls, by role only (backlog TASK-18), and then the profile list: the app name once.
    Invoke-Step 'Start: the window opens with focus on the profile list; NVDA says AudioCaptor once, then the list' {
        Start-App
    } -Once $AppTitle -Expect $UI.ProfileList, $Profiles.System -Forbid $WebviewNoise

    Invoke-Step 'Tab goes round the window: it reaches every control, each has a name and NVDA reads it' {
        Test-TabRound
    }
}

function Invoke-RecordScenario {
    Invoke-Step 'Start: the window opens with focus on the profile list' {
        if (-not $script:PlaybackDeviceId) { throw 'Windows has no default playback device; this scenario records system audio and needs one' }
        Start-App
    }

    Invoke-Step 'Alt+I while idle: NVDA says Ready' { Send-Keys 'Alt+I' } -Expect $UI.Ready

    # The announcement region speaks only a changed text; the app makes a repeated one differ.
    Invoke-Step 'Alt+I again in the same state: NVDA says Ready again' { Send-Keys 'Alt+I' } -Once $UI.Ready

    # Each state has its own record buttons, and the profile list is disabled while recording.
    # When the control with focus goes away or is disabled, focus moves to the first button of the
    # new state, and NVDA names it: not "unavailable", not the page (the app name, its document or
    # application).
    Invoke-Step "Alt+S on the profile list starts recording: NVDA says Recording started; focus moves to '$($UI.Pause)'" {
        Send-Keys 'Alt+S'
        $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
        Confirm-Focus $UI.Pause
    } -Once $UI.Started -Expect $UI.Pause -Forbid $Unavailable, $PageNoise

    Invoke-Step 'Alt+I while recording: NVDA says Recording and the time' {
        Start-Sleep -Seconds 1  # past 00:00:00
        Send-Keys 'Alt+I'
    } -Expect $UI.RecordingInfo

    Invoke-Step "Alt+P on '$($UI.Pause)' pauses: NVDA says Recording paused; focus moves to '$($UI.Resume)'" {
        Send-Keys 'Alt+P'
        $null = Wait-Until "the '$($UI.Resume)' button" { Find-ByName $UI.Resume }
        Confirm-Focus $UI.Resume
    } -Once $UI.Paused -Expect $UI.Resume -Forbid $Unavailable, $PageNoise

    Invoke-Step 'Alt+I while paused: NVDA says Paused and the time' { Send-Keys 'Alt+I' } -Expect $UI.PausedInfo

    # A resumed recording has its own announcement, not the one for a new recording.
    Invoke-Step "Alt+R on '$($UI.Resume)' resumes: NVDA says Recording resumed; focus moves to '$($UI.Pause)'" {
        Send-Keys 'Alt+R'
        $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
        Confirm-Focus $UI.Pause
    } -Once $UI.Resumed -Expect $UI.Pause -Forbid $UI.Started, $Unavailable, $PageNoise

    Invoke-Step "Tab to the '$($UI.Stop)' button" { Move-Focus $UI.Stop } -Expect $UI.Stop

    # Stop is the same button while recording and while paused, so focus stays on it.
    Invoke-Step "Alt+P on '$($UI.Stop)' pauses: NVDA says Recording paused; focus stays on '$($UI.Stop)'" {
        Send-Keys 'Alt+P'
        $null = Wait-Until "the '$($UI.Resume)' button" { Find-ByName $UI.Resume }
        Confirm-Focus $UI.Stop
    } -Once $UI.Paused -Forbid $Unavailable, $PageNoise

    Invoke-Step "Alt+T on '$($UI.Stop)' stops: NVDA says Recording stopped; focus moves to '$($UI.Start)'" {
        Send-Keys 'Alt+T'
        $null = Wait-Until "the '$($UI.Start)' button" { Find-ByName $UI.Start }
        Confirm-Focus $UI.Start
    } -Once $UI.Stopped -Expect $UI.Start -Forbid $Unavailable, $PageNoise

    Invoke-Step 'Alt+I after stopping: NVDA says Ready' { Send-Keys 'Alt+I' } -Expect $UI.Ready
}

# Buttons are pressed with Space: it acts on key up, so no key event reaches the dialog that opens.
function Invoke-DialogsScenario {
    Invoke-Step 'Start: the window opens with focus on the profile list' { Start-App }

    Invoke-Step 'F1 opens About AudioCaptor: focus on Close, NVDA reads the title once' {
        Send-Keys 'F1'
        $null = Wait-Until "focus on About's Close button" { (Get-Focus).Id -eq 'about-dialog-close' }
    } -Once $UI.About -ExpectAny $Role.Dialog

    Invoke-Step "Escape closes About: focus back on the '$($UI.ProfileList)' list where F1 was pressed" {
        Close-Dialog 'about-dialog-close' $UI.ProfileList
    } -Expect $UI.ProfileList

    Invoke-Step "Tab to the '$($UI.OpenSettings)' button" { Move-Focus $UI.OpenSettings } -Expect $UI.OpenSettings

    Invoke-Step 'Space opens Settings: focus on the hotkey field, NVDA reads the dialog' {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on the hotkey field' { (Get-Focus).Id -eq 'settings-hotkey-display' }
    } -Once $UI.Settings -Expect $UI.CurrentHotkey -ExpectAny $Role.Dialog

    # F1 works anywhere: About opens over Settings and must be drawn on top of it, since it has focus.
    Invoke-Step 'F1 in Settings opens About on top of Settings: focus on Close, NVDA reads the title once' {
        Send-Keys 'F1'
        $null = Wait-Until "focus on About's Close button" { (Get-Focus).Id -eq 'about-dialog-close' }
        Test-OnTop $UI.About $UI.Settings
    } -Once $UI.About -ExpectAny $Role.Dialog

    Invoke-Step 'Escape closes About: Settings on top again, focus back on the hotkey field' {
        Close-Dialog 'about-dialog-close' $UI.CurrentHotkey
        Test-OnTop $UI.Settings $UI.About
    } -Expect $UI.CurrentHotkey

    Invoke-Step "Escape closes Settings: focus back on the '$($UI.OpenSettings)' button" {
        Close-Dialog 'settings-hotkey-display' $UI.OpenSettings
    } -Expect $UI.OpenSettings

    Invoke-Step "Tab to the '$($UI.CreateProfile)' button" { Move-Focus $UI.CreateProfile } -Expect $UI.CreateProfile

    Invoke-Step 'Space opens New Profile: focus on the Name field, NVDA reads the dialog' {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on the Name field' { (Get-Focus).Id -eq 'profile-name' }
    } -Once $UI.NewProfile -ExpectAny $Role.Dialog

    # The list of recording formats: it has a name, the keyboard reaches it and changes it.
    Invoke-Step "Tab to the '$($UI.Format)' list: NVDA reads its name and the default, WAV" {
        Move-InDialog { (Get-Focus).Id -eq 'profile-format' } "the '$($UI.Format)' list"
    } -Expect $UI.Format, $UI.FormatWav

    Invoke-Step 'Down in the list: NVDA reads the next format, Opus for speech' {
        Send-Keys 'Down'
    } -Expect $UI.FormatSpeech

    Invoke-Step "Escape closes New Profile: focus back on the '$($UI.CreateProfile)' button" {
        Close-Dialog 'profile-name' $UI.CreateProfile
    } -Expect $UI.CreateProfile

    Invoke-Step "Tab to the '$($UI.DeleteProfile)' button" { Move-Focus $UI.DeleteProfile } -Expect $UI.DeleteProfile

    Invoke-Step 'Space opens the delete confirmation: focus on Cancel, NVDA reads the dialog' {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on Cancel' { (Get-Focus).Id -eq 'confirm-delete-cancel' }
    } -Once $UI.DeleteTitle -Expect $UI.Cancel -ExpectAny $Role.Dialog

    Invoke-Step "Escape closes the confirmation: focus back on the '$($UI.DeleteProfile)' button, both profiles kept" {
        Close-Dialog 'confirm-delete-cancel' $UI.DeleteProfile
        Start-Sleep -Milliseconds 1200  # the app saves settings 0.5 s after a change
        $names = @(([IO.File]::ReadAllText($SettingsFile) | ConvertFrom-Json).profiles | ForEach-Object { $_.name })
        if ($names.Count -ne 2) { throw "a profile was deleted: settings.json now lists $($names -join ', ')" }
    } -Expect $UI.DeleteProfile

    # The format chosen in the dialog has to reach the profile in settings.json.
    Invoke-Step "Tab to the '$($UI.EditProfile)' button" { Move-Focus $UI.EditProfile } -Expect $UI.EditProfile

    Invoke-Step 'Space opens Edit Profile: focus on the Name field, NVDA reads the dialog' {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on the Name field' { (Get-Focus).Id -eq 'profile-name' }
    } -Once $UI.EditTitle -ExpectAny $Role.Dialog

    Invoke-Step "Opus for speech is chosen in '$($UI.Format)' and saved: focus on the '$($UI.ProfileList)' list, the profile in settings.json has the format" {
        Move-InDialog { (Get-Focus).Id -eq 'profile-format' } "the '$($UI.Format)' list"
        Send-Keys 'Down'
        Move-InDialog { (Get-Focus).Name -eq $UI.Save } "the '$($UI.Save)' button"
        Send-Keys 'Space'
        $null = Wait-Until 'the dialog to close' { -not (Find-ById 'profile-name') } 3
        Confirm-Focus $UI.ProfileList
        Start-Sleep -Milliseconds 300
        $settings = [IO.File]::ReadAllText($SettingsFile) | ConvertFrom-Json
        $saved = $settings.profiles | Where-Object { $_.id -eq 'smoke-system' }
        Add-Check 'the profile in settings.json has the format chosen in the dialog (OpusSpeech)' ($saved.outputFormat -eq 'OpusSpeech') "outputFormat is '$($saved.outputFormat)'"
        Add-Check 'settings.json, written as version 5 by the test, is version 6 now' ($settings.version -eq 6) "version is $($settings.version)"
    } -Expect $UI.FormatSpeech, $UI.Save, $UI.ProfileList

    # Browser autofill (backlog TASK-23): WebView2 remembers a name saved with Enter and offers it
    # in the empty Name field next time; the first Escape then only hides the suggestions. The
    # browser data lasts for the whole run, so the second New Profile sees what the first one left.
    Invoke-Step "Tab to the '$($UI.CreateProfile)' button" { Move-Focus $UI.CreateProfile } -Expect $UI.CreateProfile

    Invoke-Step "Space opens New Profile; the name '$AutofillName' typed in Name and Enter create the profile" {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on the Name field' { (Get-Focus).Id -eq 'profile-name' }
        Send-Keys ([char[]]$AutofillName | ForEach-Object { [string]$_ })
        Send-Keys 'Enter'
        $null = Wait-Until 'the dialog to close' { -not (Find-ById 'profile-name') } 3
        Start-Sleep -Milliseconds 1200  # the app saves settings 0.5 s after a change
        $names = @(([IO.File]::ReadAllText($SettingsFile) | ConvertFrom-Json).profiles | ForEach-Object { $_.name })
        Add-Check "settings.json has the new profile '$AutofillName'" ($names -contains $AutofillName) "profiles: $($names -join ', ')"
    } -Once $UI.NewProfile

    Invoke-Step "Tab to the '$($UI.CreateProfile)' button again" { Move-Focus $UI.CreateProfile } -Expect $UI.CreateProfile

    Invoke-Step 'Space opens New Profile again: focus on the empty Name field, NVDA says nothing about autofill' {
        Send-Keys 'Space'
        $null = Wait-Until 'focus on the Name field' { (Get-Focus).Id -eq 'profile-name' }
    } -Once $UI.NewProfile -Forbid $Autofill

    Invoke-Step "One Escape in the empty Name field closes New Profile: focus back on the '$($UI.CreateProfile)' button" {
        Close-Dialog 'profile-name' $UI.CreateProfile
    } -Expect $UI.CreateProfile -Forbid $Autofill
}

function Invoke-TrayScenario {
    Invoke-Step 'Start: the window opens with focus on the profile list' { Start-App }

    Invoke-Step 'Escape hides the window to the tray; the app keeps running' {
        Send-Keys 'Esc'
        $null = Wait-Until 'the window to hide' { -not (Test-AppShown) } 3
        Start-Sleep -Milliseconds 500
        if ($script:AppProc.HasExited) { throw "the app quit instead of hiding (exit code $($script:AppProc.ExitCode))" }
        Enter-HomeWindow
    }

    # The second copy finds the first one, hands over to it and quits (one copy at a time).
    Invoke-Step 'Starting AudioCaptor.exe again shows the window in the foreground; NVDA announces AudioCaptor' {
        $clock = [Diagnostics.Stopwatch]::StartNew()
        $second = Start-TestCopy
        if (-not $second.WaitForExit(10000)) {
            Stop-Process -Id $second.Id -Force -ErrorAction SilentlyContinue
            throw 'the second copy kept running instead of handing over to the first one'
        }
        $shown = $false
        try { $null = Wait-Until 'the window on screen and in the foreground' { Test-AppForeground } 6; $shown = $true } catch { }
        if (-not $shown) {
            if (Test-AppShown) { throw 'the window came back on screen but not to the foreground' }
            throw 'the window stayed hidden after the second start'
        }
        $script:StepInfo.Add("window in the foreground $($clock.ElapsedMilliseconds) ms after the second start")
        $copies = @(Get-Process $AppTitle -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $TestExe })
        if ($copies.Count -ne 1) { $script:SoftFail = "$($copies.Count) test copies are running after the second start" }
    } -Expect $AppTitle
}

function Invoke-SettingsScenario {
    $copy = Join-Path $DataDir 'settings.unreadable.json'
    Remove-Item -LiteralPath $copy -Force -ErrorAction SilentlyContinue
    $script:BrokenSettings = $true
    try {
        # The message is a role=alert block: NVDA reads it when the window has loaded. Its language
        # follows the Windows UI language (the reset settings carry it), so both are accepted.
        Invoke-Step 'Start with an unreadable settings.json: NVDA reads that the settings were reset and where the old file is kept' {
            Start-App
            Add-Check 'the old file is kept as settings.unreadable.json' (Test-Path -LiteralPath $copy)
            Add-Check 'the kept copy is the broken file, unchanged' ((Test-Path -LiteralPath $copy) -and ([IO.File]::ReadAllText($copy) -eq '{ "version": 5, "profiles": [')) 
            Add-Check 'settings.json is readable again' ($null -ne ([IO.File]::ReadAllText($SettingsFile) | ConvertFrom-Json -ErrorAction SilentlyContinue))
        } -ExpectAny ('Could not read your settings|' + [regex]::Unescape('\u041d\u0435 \u0432\u0434\u0430\u043b\u043e\u0441\u044f \u043f\u0440\u043e\u0447\u0438\u0442\u0430\u0442\u0438 \u043d\u0430\u043b\u0430\u0448\u0442\u0443\u0432\u0430\u043d\u043d\u044f')), 'settings.unreadable.json'
    } finally {
        $script:BrokenSettings = $false
    }
}

# The files of the test copy's Recordings folder: name -> size.
function Get-RecordingFiles {
    $files = @{}
    $dir = Join-Path $DataDir 'Recordings'
    if (Test-Path -LiteralPath $dir) { foreach ($f in @(Get-ChildItem -LiteralPath $dir -File)) { $files[$f.Name] = $f.Length } }
    return $files
}

function Format-RecordingFiles($Files) {
    if (-not $Files.Count) { return 'no files' }
    return (@($Files.Keys | Sort-Object | ForEach-Object { '{0} ({1:0.0} MB)' -f $_, ($Files[$_] / 1MB) }) -join ', ')
}

# The WAV files listed in the test copy's conversion queue, in order; none when there is no file.
function Get-QueuedWavs {
    $queue = Join-Path $DataDir 'conversion-queue.json'
    if (-not (Test-Path -LiteralPath $queue)) { return @() }
    return @(([IO.File]::ReadAllText($queue) | ConvertFrom-Json).jobs | ForEach-Object { ([string]$_.wav) -replace '\\', '/' })
}

# The conversion must still be running when a step is made during it.
function Assert-Converting([string]$Step) {
    if (-not (Test-Path -LiteralPath (Join-Path $DataDir 'conversion-queue.json'))) {
        throw "the conversion ended before $Step; give the scenario a larger file: -ConvertMB $($ConvertMB * 2)"
    }
}

function Invoke-ConvertScenario {
    $recordings = Join-Path $DataDir 'Recordings'
    $queueFile = Join-Path $DataDir 'conversion-queue.json'
    $long = 'smoke-long.wav'
    $short = 'smoke-short.wav'
    $broken = 'smoke-broken.wav'
    $job = { param([string]$Name, [long]$Frames) [ordered]@{ wav = "Recordings/$Name"; frames = $Frames; sampleRate = 48000; channels = 2; format = 'OpusMusic' } }
    $script:ProfileFormat = 'OpusSpeech'
    try {
        # What the last run would have left behind: two recordings stopped but not yet converted.
        Invoke-Step "Start with two recordings left to convert: NVDA says that the app is finishing them" {
            if (-not $script:PlaybackDeviceId) { throw 'Windows has no default playback device; this scenario records system audio and needs one' }
            # The folder may hold the recording of the record scenario: this one counts the files.
            Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
            $null = New-Item -ItemType Directory -Force -Path $recordings
            $clock = [Diagnostics.Stopwatch]::StartNew()
            $longFrames = [long]$ConvertMB * 1MB / 4
            $shortFrames = 48000L * 3
            $null = [AcSmokeNative]::WriteNoiseWav((Join-Path $recordings $long), $longFrames)
            $null = [AcSmokeNative]::WriteNoiseWav((Join-Path $recordings $short), $shortFrames)
            $queue = [ordered]@{ version = 1; jobs = @((& $job $long $longFrames), (& $job $short $shortFrames)) }
            [IO.File]::WriteAllText($queueFile, ($queue | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
            $script:StepInfo.Add(('generated {0} in {1:0.0} s' -f (Format-RecordingFiles (Get-RecordingFiles)), $clock.Elapsed.TotalSeconds))
            Start-App
        } -Expect $UI.Resumed2, $UI.ProfileList

        Invoke-Step 'Alt+I during the conversion: NVDA says that it runs, about how long is left and that one file waits' {
            Assert-Converting 'Alt+I'
            Send-Keys 'Alt+I'
        } -Expect $UI.ReadyConverting, $UI.Waiting1 -ExpectAny $UI.TimeLeft

        Invoke-Step 'Alt+I again a moment later: NVDA says it all again' {
            Assert-Converting 'the repeated Alt+I'
            Send-Keys 'Alt+I'
        } -Once $UI.ReadyConverting -Expect $UI.Waiting1 -ExpectAny $UI.TimeLeft

        Invoke-Step 'Alt+F4 during the conversion asks first: focus on Cancel, NVDA reads the question' {
            Assert-Converting 'Alt+F4'
            Send-Keys 'Alt+F4'
            $null = Wait-Until 'focus on Cancel' { (Get-Focus).Id -eq 'confirm-exit-cancel' }
        } -Once $UI.ExitTitle -Expect $UI.ExitBody, $UI.Cancel -ExpectAny $Role.Dialog

        Invoke-Step "Escape closes the question: the app keeps running, focus back on the '$($UI.ProfileList)' list" {
            Close-Dialog 'confirm-exit-cancel' $UI.ProfileList
            if ($script:AppProc.HasExited) { throw 'the app quit instead of going on' }
        } -Expect $UI.ProfileList

        Invoke-Step 'Alt+F4, Tab to Exit, Space: the app quits; both WAV files are whole, no Opus file is left half-written' {
            Assert-Converting 'the exit'
            $before = Get-RecordingFiles
            Send-Keys 'Alt+F4'
            $null = Wait-Until 'focus on Cancel' { (Get-Focus).Id -eq 'confirm-exit-cancel' }
            Send-Keys 'Tab'
            $null = Wait-Until "focus on '$($UI.Exit)'" { (Get-Focus).Name -eq $UI.Exit } 3
            $sawTemp = Test-Path -LiteralPath (Join-Path $recordings 'smoke-long.opus.tmp')
            Send-Keys 'Space'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Exit' }
            Start-Sleep -Milliseconds 300
            $after = Get-RecordingFiles
            $script:StepInfo.Add("files after the exit: $(Format-RecordingFiles $after); the queue file lists: $((Get-QueuedWavs) -join ', ')")
            Add-Check 'the conversion was under way when the app quit (its temporary file was there)' $sawTemp
            Add-Check 'both WAV files are there with the size they had' (($after[$long] -eq $before[$long]) -and ($after[$short] -eq $before[$short]) -and $before[$long] -gt 0)
            $extra = @($after.Keys | Where-Object { $_ -ne $long -and $_ -ne $short })
            Add-Check 'no Opus file and no temporary file is left' ($extra.Count -eq 0) ($extra -join ', ')
            Add-Check 'the queue file still lists both recordings' (((Get-QueuedWavs) -join ', ') -eq "Recordings/$long, Recordings/$short")
        } -Expect $UI.Exit

        Invoke-Step 'Start again: NVDA says that the app is finishing the two recordings' {
            Start-App
        } -Expect $UI.Resumed2, $UI.ProfileList

        # The interface answers during the conversion: a recording starts and stops as usual.
        Invoke-Step "Alt+S during the conversion starts a recording: NVDA says Recording started; focus moves to '$($UI.Pause)'" {
            Assert-Converting 'Alt+S'
            Send-Keys 'Alt+S'
            $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
            Confirm-Focus $UI.Pause
        } -Once $UI.Started -Expect $UI.Pause -Forbid $Unavailable, $PageNoise

        Invoke-Step "Alt+T stops it: NVDA says that the recording waits to be converted; focus moves to '$($UI.Start)'" {
            Start-Sleep -Seconds 2
            Assert-Converting 'Alt+T'
            Send-Keys 'Alt+T'
            $null = Wait-Until "the '$($UI.Start)' button" { Find-ByName $UI.Start }
            Confirm-Focus $UI.Start
        } -Once $UI.StoppedWaiting -Expect $UI.Start -Forbid $Unavailable, $PageNoise

        Invoke-Step 'Alt+I: NVDA says that two files wait now' {
            Assert-Converting 'the second Alt+I'
            Send-Keys 'Alt+I'
        } -Expect $UI.ReadyConverting, $UI.Waiting2 -ExpectAny $UI.TimeLeft

        Invoke-Step 'The queue empties: NVDA says Conversion finished; three Opus files, no WAV, no queue file' {
            $clock = [Diagnostics.Stopwatch]::StartNew()
            $null = Wait-Until 'the conversion queue to empty' { -not (Test-Path -LiteralPath $queueFile) } 300
            Start-Sleep -Milliseconds 500
            $files = Get-RecordingFiles
            $script:StepInfo.Add(('waited {0:0} s more; files: {1}' -f $clock.Elapsed.TotalSeconds, (Format-RecordingFiles $files)))
            $opus = @($files.Keys | Where-Object { $_ -like '*.opus' -and $files[$_] -gt 0 })
            $other = @($files.Keys | Where-Object { $_ -notlike '*.opus' })
            Add-Check 'both left-over recordings are Opus files now' (($opus -contains 'smoke-long.opus') -and ($opus -contains 'smoke-short.opus'))
            Add-Check 'the new recording is an Opus file too' (@($opus | Where-Object { $_ -like 'loopback_*.opus' }).Count -eq 1)
            Add-Check 'no WAV file and no temporary file is left' ($other.Count -eq 0) ($other -join ', ')
        } -Once $UI.Converted -Forbid $UI.ConvertErrors

        Invoke-Step 'Alt+I after the conversion: NVDA says Ready and nothing about a conversion' {
            Send-Keys 'Alt+I'
        } -Expect $UI.Ready -Forbid 'Converting'

        Invoke-Step 'Alt+F4 with nothing left to convert: the app quits without asking' {
            Send-Keys 'Alt+F4'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Alt+F4' }
        } -Forbid $UI.ExitTitle

        # A recording that cannot be converted: its WAV holds half the frames the queue says, so the
        # conversion fails at the end of the file, some seconds after the start. (An alert that
        # comes among the focus changes of the start is one NVDA does not read.)
        Invoke-Step 'Start with a recording that cannot be converted: NVDA says that the app is finishing it' {
            Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
            $null = New-Item -ItemType Directory -Force -Path $recordings
            $frames = [long]$ConvertMB * 1MB / 16
            $null = [AcSmokeNative]::WriteNoiseWav((Join-Path $recordings $broken), $frames)
            $queue = [ordered]@{ version = 1; jobs = @((& $job $broken ($frames * 2))) }
            [IO.File]::WriteAllText($queueFile, ($queue | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
            Start-App
        } -Expect $UI.Resumed1, $UI.ProfileList

        Invoke-Step 'The conversion fails: NVDA reads the message with the WAV, then the end with errors; the WAV is kept and nothing else is left' {
            $before = (Get-RecordingFiles)[$broken]
            $null = Wait-Until 'the conversion queue to empty' { -not (Test-Path -LiteralPath $queueFile) } 120
            Start-Sleep -Milliseconds 500
            $files = Get-RecordingFiles
            $script:StepInfo.Add("files: $(Format-RecordingFiles $files)")
            Add-Check 'the WAV is kept with the size it had' (($files[$broken] -eq $before) -and $before -gt 0)
            $extra = @($files.Keys | Where-Object { $_ -ne $broken })
            Add-Check 'no Opus file and no temporary file is left' ($extra.Count -eq 0) ($extra -join ', ')
        } -Expect $UI.NotConverted, $broken, $UI.EndWithErrors

        Invoke-Step 'Alt+F4 after the failure: the app quits without asking' {
            Send-Keys 'Alt+F4'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Alt+F4' }
        } -Forbid $UI.ExitTitle
    } finally {
        $script:ProfileFormat = ''
        Stop-App
        # The generated files are large: free the space at once, whatever happened.
        Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $queueFile -Force -ErrorAction SilentlyContinue
    }
}

# The files a recording in the Loopback profile left, after its conversion has ended.
function Get-LoopbackFiles([string]$QueueFile) {
    $null = Wait-Until 'the conversion queue to empty' { -not (Test-Path -LiteralPath $QueueFile) } 30
    Start-Sleep -Milliseconds 500
    $files = Get-RecordingFiles
    $script:StepInfo.Add("files: $(Format-RecordingFiles $files)")
    return $files
}

# A recording in a profile with the format "MP3 - standard" becomes an .mp3 file. Then the same as
# on Windows N without the Media Feature Pack, which the switch $NoMp3Switch fakes: the conversion
# fails at once, NVDA reads how to get the encoder, and the WAV is kept.
function Invoke-Mp3Scenario {
    $recordings = Join-Path $DataDir 'Recordings'
    $queueFile = Join-Path $DataDir 'conversion-queue.json'
    $script:ProfileFormat = 'Mp3Standard'
    try {
        Invoke-Step 'Start with an MP3 profile: the window opens with focus on the profile list' {
            if (-not $script:PlaybackDeviceId) { throw 'Windows has no default playback device; this scenario records system audio and needs one' }
            Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
            Start-App
        }

        Invoke-Step "Alt+S starts a recording: NVDA says Recording started; focus moves to '$($UI.Pause)'" {
            Send-Keys 'Alt+S'
            $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
            Confirm-Focus $UI.Pause
        } -Once $UI.Started -Expect $UI.Pause -Forbid $Unavailable, $PageNoise

        Invoke-Step 'Alt+T stops it: NVDA says that the conversion to MP3 started and that it finished; an MP3 file and no WAV are left' {
            Start-Sleep -Seconds 3
            Send-Keys 'Alt+T'
            $null = Wait-Until "the '$($UI.Start)' button" { Find-ByName $UI.Start }
            $files = Get-LoopbackFiles $queueFile
            $mp3 = @($files.Keys | Where-Object { $_ -like 'loopback_*.mp3' -and $files[$_] -gt 0 })
            Add-Check 'the recording is an MP3 file now' ($mp3.Count -eq 1)
            if ($mp3.Count -eq 1) {
                $bytes = [IO.File]::ReadAllBytes((Join-Path $recordings $mp3[0]))
                Add-Check 'the file starts with an MP3 frame' ($bytes.Length -gt 4 -and $bytes[0] -eq 0xFF -and ($bytes[1] -band 0xE0) -eq 0xE0)
            }
            $other = @($files.Keys | Where-Object { $_ -notlike '*.mp3' })
            Add-Check 'no WAV file and no temporary file is left' ($other.Count -eq 0) ($other -join ', ')
        } -Once $UI.Mp3Started, $UI.Converted -Forbid $UI.ConvertErrors, $Unavailable, $PageNoise

        Invoke-Step 'Alt+F4: the app quits without asking' {
            Send-Keys 'Alt+F4'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Alt+F4' }
        } -Forbid $UI.ExitTitle

        Invoke-Step "Start again as if Windows had no MP3 encoder ($NoMp3Switch=1)" {
            Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
            # The test copy gets the variable at its start; the test itself does not keep it.
            Set-Item "Env:$NoMp3Switch" '1'
            try { Start-App } finally { Remove-Item "Env:$NoMp3Switch" -ErrorAction SilentlyContinue }
        }

        Invoke-Step "Alt+S starts a recording: NVDA says Recording started" {
            Send-Keys 'Alt+S'
            $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
        } -Once $UI.Started

        Invoke-Step 'Alt+T stops it: NVDA reads that Windows has no MP3 encoder and how to get it, then the end with errors; the WAV is kept' {
            Start-Sleep -Seconds 2
            Send-Keys 'Alt+T'
            $null = Wait-Until "the '$($UI.Start)' button" { Find-ByName $UI.Start }
            $files = Get-LoopbackFiles $queueFile
            $wav = @($files.Keys | Where-Object { $_ -like 'loopback_*.wav' -and $files[$_] -gt 44 })
            Add-Check 'the recording is kept as WAV' ($wav.Count -eq 1)
            $other = @($files.Keys | Where-Object { $_ -notlike '*.wav' })
            Add-Check 'no MP3 file and no temporary file is left' ($other.Count -eq 0) ($other -join ', ')
        } -Expect $UI.NoEncoder, $UI.EndWithErrors -Forbid $UI.Converted

        Invoke-Step 'Alt+F4 after the failure: the app quits without asking' {
            Send-Keys 'Alt+F4'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Alt+F4' }
        } -Forbid $UI.ExitTitle
    } finally {
        $script:ProfileFormat = ''
        Remove-Item "Env:$NoMp3Switch" -ErrorAction SilentlyContinue
        Stop-App
        Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $queueFile -Force -ErrorAction SilentlyContinue
    }
}

# A profile with the format "FLAC (lossless)": the Format list of Edit Profile shows it, and a
# recording in the profile becomes a .flac file.
function Invoke-FlacScenario {
    $recordings = Join-Path $DataDir 'Recordings'
    $queueFile = Join-Path $DataDir 'conversion-queue.json'
    $script:ProfileFormat = 'Flac'
    try {
        Invoke-Step 'Start with a FLAC profile: the window opens with focus on the profile list' {
            if (-not $script:PlaybackDeviceId) { throw 'Windows has no default playback device; this scenario records system audio and needs one' }
            Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
            Start-App
        }

        Invoke-Step "Tab to the '$($UI.EditProfile)' button" { Move-Focus $UI.EditProfile } -Expect $UI.EditProfile

        Invoke-Step 'Space opens Edit Profile: focus on the Name field, NVDA reads the dialog' {
            Send-Keys 'Space'
            $null = Wait-Until 'focus on the Name field' { (Get-Focus).Id -eq 'profile-name' }
        } -Once $UI.EditTitle -ExpectAny $Role.Dialog

        Invoke-Step "Tab to the '$($UI.Format)' list: NVDA reads its name and the format of the profile, FLAC" {
            Move-InDialog { (Get-Focus).Id -eq 'profile-format' } "the '$($UI.Format)' list"
        } -Expect $UI.Format, $UI.FormatFlac

        Invoke-Step "Escape closes Edit Profile: focus back on the '$($UI.EditProfile)' button" {
            Close-Dialog 'profile-name' $UI.EditProfile
        } -Expect $UI.EditProfile

        Invoke-Step "Alt+S starts a recording: NVDA says Recording started; focus moves to '$($UI.Pause)'" {
            Send-Keys 'Alt+S'
            $null = Wait-Until "the '$($UI.Pause)' button" { Find-ByName $UI.Pause }
            Confirm-Focus $UI.Pause
        } -Once $UI.Started -Expect $UI.Pause -Forbid $Unavailable, $PageNoise

        Invoke-Step 'Alt+T stops it: NVDA says that the conversion to FLAC started and that it finished; a FLAC file and no WAV are left' {
            Start-Sleep -Seconds 3
            Send-Keys 'Alt+T'
            $null = Wait-Until "the '$($UI.Start)' button" { Find-ByName $UI.Start }
            $files = Get-LoopbackFiles $queueFile
            $flac = @($files.Keys | Where-Object { $_ -like 'loopback_*.flac' -and $files[$_] -gt 0 })
            Add-Check 'the recording is a FLAC file now' ($flac.Count -eq 1)
            if ($flac.Count -eq 1) {
                $bytes = [IO.File]::ReadAllBytes((Join-Path $recordings $flac[0]))
                Add-Check 'the file starts with the FLAC marker' ($bytes.Length -gt 4 -and [Text.Encoding]::ASCII.GetString($bytes, 0, 4) -ceq 'fLaC')
            }
            $other = @($files.Keys | Where-Object { $_ -notlike '*.flac' })
            Add-Check 'no WAV file and no temporary file is left' ($other.Count -eq 0) ($other -join ', ')
        } -Once $UI.FlacStarted, $UI.Converted -Forbid $UI.ConvertErrors, $Unavailable, $PageNoise

        Invoke-Step 'Alt+F4: the app quits without asking' {
            Send-Keys 'Alt+F4'
            if (-not $script:AppProc.WaitForExit(8000)) { throw 'the app did not quit within 8 s of Alt+F4' }
        } -Forbid $UI.ExitTitle
    } finally {
        $script:ProfileFormat = ''
        Stop-App
        Remove-Item -LiteralPath $recordings -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $queueFile -Force -ErrorAction SilentlyContinue
    }
}

function Invoke-Scenario([string]$Name) {
    $script:CurrentScenario = $Name
    $script:StepNumber = 0
    $script:Aborted = $false
    Write-Host "Scenario $Name"
    try {
        switch ($Name) {
            'launch'  { Invoke-LaunchScenario }
            'record'  { Invoke-RecordScenario }
            'dialogs' { Invoke-DialogsScenario }
            'tray'    { Invoke-TrayScenario }
            'settings' { Invoke-SettingsScenario }
            'convert' { Invoke-ConvertScenario }
            'mp3'     { Invoke-Mp3Scenario }
            'flac'    { Invoke-FlacScenario }
        }
    } finally {
        Stop-App
        try { Enter-HomeWindow } catch { }
    }
}

# --- Preparation -------------------------------------------------------------------------------
# Stops the run before or between scenarios: the report gives the reason, the exit code is 2.
function Stop-Run([string]$Reason) {
    $script:StopReason = $Reason
    throw $Reason
}

# The newest source file changed after the exe was built, or $null.
function Get-NewerSource([datetime]$Than) {
    $repo = Split-Path $PSScriptRoot -Parent
    $files = New-Object System.Collections.Generic.List[object]
    foreach ($d in 'src', 'src-tauri\src', 'messages') {
        $p = Join-Path $repo $d
        if (Test-Path -LiteralPath $p) {
            foreach ($f in @(Get-ChildItem -LiteralPath $p -Recurse -File | Where-Object { $_.FullName -notmatch '\\paraglide\\' })) { $files.Add($f) }
        }
    }
    foreach ($f in 'index.html', 'package.json', 'src-tauri\Cargo.toml', 'src-tauri\tauri.conf.json') {
        $p = Join-Path $repo $f
        if (Test-Path -LiteralPath $p) { $files.Add((Get-Item -LiteralPath $p)) }
    }
    $newest = $files | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($newest -and $newest.LastWriteTime -gt $Than) { return $newest }
    return $null
}

# Only whether the folder exists, how many files and folders it holds and when the newest of them
# changed: no names, no contents.
function Get-FolderState([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return [pscustomobject]@{ Key = 'none'; Text = 'does not exist' } }
    $items = @(Get-ChildItem -LiteralPath $Path -Recurse -Force -ErrorAction SilentlyContinue)
    $newest = (Get-Item -LiteralPath $Path -Force).LastWriteTimeUtc.Ticks
    foreach ($i in $items) { if ($i.LastWriteTimeUtc.Ticks -gt $newest) { $newest = $i.LastWriteTimeUtc.Ticks } }
    $when = (New-Object DateTime $newest, ([DateTimeKind]::Utc)).ToLocalTime()
    return [pscustomobject]@{ Key = "$($items.Count)|$newest"; Text = ('{0} files and folders, newest change {1:yyyy-MM-dd HH:mm:ss}' -f $items.Count, $when) }
}

# The state of every entry directly in the folder: name -> state (see Get-FolderState).
function Get-EntryStates([string]$Path) {
    $states = @{}
    if (Test-Path -LiteralPath $Path) {
        foreach ($e in @(Get-ChildItem -LiteralPath $Path -Force)) { $states[$e.Name] = Get-FolderState $e.FullName }
    }
    return $states
}

function Get-StatesKey($States) {
    return (($States.Keys | Sort-Object | ForEach-Object { "$_=$($States[$_].Key)" }) -join ';')
}

# A copy that has just quit may still be writing for a moment: read until two reads half a second
# apart agree.
function Get-SettledEntryStates([string]$Path) {
    $states = Get-EntryStates $Path
    for ($i = 0; $i -lt 10; $i++) {
        Start-Sleep -Milliseconds 500
        $again = Get-EntryStates $Path
        if ((Get-StatesKey $again) -eq (Get-StatesKey $states)) { break }
        $states = $again
    }
    return $states
}

# Asks every running AudioCaptor to quit, as its close button does. Test copies left by an earlier
# run are ended.
function Close-UserAudioCaptor {
    foreach ($p in @(Get-Process $AppTitle -ErrorAction SilentlyContinue)) {
        $path = $null
        try { $path = $p.Path } catch { }
        if ($path -and $path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase)) {
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
            $null = $p.WaitForExit(5000)
            continue
        }
        $where = if ($path) { $path } else { "process $($p.Id)" }
        # Its WebView2 browser process may write to the browser data for a moment after the app
        # quits; the test waits for it before it reads the state of that folder.
        $browsers = @()
        try {
            $browsers = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe' AND ParentProcessId = $($p.Id)" |
                    ForEach-Object { [int]$_.ProcessId })
        } catch { }
        $h = [AcSmokeNative]::FindProcessWindow([uint32]$p.Id, $AppTitle)
        if ($h -ne [IntPtr]::Zero) { $null = [AcSmokeNative]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) }  # WM_CLOSE
        if (-not $p.WaitForExit(8000)) {
            Stop-Run "Your AudioCaptor ($where) did not quit within 8 s. It may be recording and asking whether to stop: answer it or close the app yourself, then run the test again."
        }
        foreach ($b in $browsers) {
            try { $null = (Get-Process -Id $b -ErrorAction Stop).WaitForExit(10000) } catch { }
        }
        $Notes.Add("Your AudioCaptor ($where) was closed for the test and was not started again.")
    }
}

# A fresh test folder with a copy of the exe.
function Initialize-TestFolder {
    for ($i = 0; $i -lt 20 -and (Test-Path -LiteralPath $Root); $i++) {
        try { Remove-Item -LiteralPath $Root -Recurse -Force } catch { Start-Sleep -Milliseconds 500 }
    }
    if (Test-Path -LiteralPath $Root) { Stop-Run "Cannot clear $Root, left by an earlier run: a program still uses it. Close it and run the test again." }
    Remove-Item -LiteralPath $AppLogCopy -Force -ErrorAction SilentlyContinue  # the log of an earlier run
    $script:FolderReady = $true
    # No AudioCaptor-webview: the test copy has to create it itself (Start-App checks it).
    $null = New-Item -ItemType Directory -Force -Path $DataDir
    Copy-Item -LiteralPath $Exe -Destination $TestExe
}

# One beep, then the pause. Nothing is pressed before this returns.
function Wait-ForStart {
    Invoke-Beep 'start'
    Write-Host "Starting in $PauseSec s."
    Start-Sleep -Seconds $PauseSec
}

# --- Run ---------------------------------------------------------------------------------------
$exeTime = $null
$userDirBefore = $false
$userBefore = $null
$userAfter = $null
$testBrowser = $null
$script:RunClock = $null
$script:FolderReady = $false
$script:AppLogCopied = $false
$script:PlaybackDeviceId = $null
$script:ProfileFormat = ''
try {
    if (-not (Test-Path -LiteralPath $Exe)) { Stop-Run "AudioCaptor.exe not found: $Exe. Build it first: just build" }
    $Exe = (Resolve-Path -LiteralPath $Exe).Path
    $exeTime = (Get-Item -LiteralPath $Exe).LastWriteTime
    if ($Exe -eq (Resolve-Path -LiteralPath $DefaultExe -ErrorAction SilentlyContinue).Path) {
        $newer = Get-NewerSource $exeTime
        if ($newer) { $Notes.Add(('The exe was built before {0} changed ({1:yyyy-MM-dd HH:mm}). To test the latest code, rebuild it: just build' -f $newer.Name, $newer.LastWriteTime)) }
    }
    if (-not (Get-Process nvda -ErrorAction SilentlyContinue)) { Stop-Run 'NVDA is not running. Start NVDA and run the test again.' }
    $script:LogPath = Find-NvdaLog
    if (-not $script:LogPath) { Stop-Run "nvda.log was not found in the temp folder ($env:TEMP). Pass its path: -NvdaLog <path>" }
    try { $script:PlaybackDeviceId = [AcSmokeNative]::DefaultPlaybackDeviceId() } catch { }

    Write-Host "AudioCaptor NVDA smoke test: $($Selected -join ', ')."
    Write-Host 'It takes up to three minutes. Do not touch the keyboard or mouse until the report appears.'
    if (Get-Process $AppTitle -ErrorAction SilentlyContinue) { Write-Host 'Your running AudioCaptor will be closed and not started again.' }
    Wait-ForStart

    $script:RunClock = [Diagnostics.Stopwatch]::StartNew()
    $script:LogStart = Get-LogLength
    try { Start-HomeWindow } catch { Stop-Run "The test could not open its own window: $($_.Exception.Message)" }
    switch (Test-NvdaLogging) {
        'nolog' {
            $saved = Get-NvdaSavedLevel
            $hint = if ($saved) { " NVDA's saved configuration says: loggingLevel = $saved." } else { '' }
            Stop-Run ("NVDA writes no speech to its log ($script:LogPath). Set its logging level to Input/output " +
                "(NVDA menu, Preferences, Settings, General, Logging level) and run the test again.$hint")
        }
        'nospeech' {
            Stop-Run 'NVDA logs key presses but said nothing for the test window. Check that speech is on (NVDA+S: speech mode "talk") and run the test again.'
        }
    }
    Close-UserAudioCaptor
    Initialize-TestFolder
    $userDirBefore = Test-Path -LiteralPath $UserAppDir
    $userBefore = Get-SettledEntryStates $UserAppDir
    foreach ($name in $Selected) { Invoke-Scenario $name }
} catch {
    if (-not $script:StopReason) {
        $script:StopReason = "Error in the test: $($_.Exception.Message) (smoke-nvda.ps1, line $($_.InvocationInfo.ScriptLineNumber))"
    }
} finally {
    Stop-App
    if ($null -ne $userBefore) { $userAfter = Get-SettledEntryStates $UserAppDir }
    Stop-HomeWindow
    if ($script:FolderReady) {
        $testBrowser = Get-FolderState (Join-Path $WebviewDir $BrowserEntry)
        $appLog = Join-Path $DataDir 'logs\audiocaptor.log'
        if (Test-Path -LiteralPath $appLog) {
            Copy-Item -LiteralPath $appLog -Destination $AppLogCopy -Force
            $script:AppLogCopied = $true
        }
        for ($i = 0; $i -lt 20 -and (Test-Path -LiteralPath $Root); $i++) {
            try { Remove-Item -LiteralPath $Root -Recurse -Force } catch { Start-Sleep -Milliseconds 500 }
        }
        if (Test-Path -LiteralPath $Root) { $Notes.Add("Could not delete $Root (still in use); the next run deletes it.") }
    }
}

# --- Report ------------------------------------------------------------------------------------
$lines = New-Object System.Collections.Generic.List[string]
$lines.Add(('AudioCaptor NVDA smoke test, {0:yyyy-MM-dd HH:mm}' -f (Get-Date)))
$lines.Add("Scenarios: $($Selected -join ', ')")
if ($exeTime) { $lines.Add(('Exe: {0}, built {1:yyyy-MM-dd HH:mm}' -f $Exe, $exeTime)) } else { $lines.Add("Exe: $Exe") }
if ($script:LogPath) { $lines.Add(('NVDA log: {0}, read from byte {1} on (only what NVDA wrote during this run)' -f $script:LogPath, $script:LogStart)) }
if ($testBrowser) { $lines.Add("Browser data of the test copy, next to its exe ($(Join-Path $WebviewDir $BrowserEntry)): $($testBrowser.Text).") }
$browserChanged = $false
$wroteOutside = $false
if ($null -ne $userBefore -and $null -ne $userAfter) {
    if (-not $userDirBefore -and (Test-Path -LiteralPath $UserAppDir)) {
        $wroteOutside = $true
        $lines.Add("FAILED: AudioCaptor created $UserAppDir, outside the folder of its exe.")
    }
    $names = @(@($userBefore.Keys) + @($userAfter.Keys) | Sort-Object -Unique)
    if ($names -notcontains $BrowserEntry) {
        $lines.Add("Browser data of AudioCaptor 0.1.1 and older ($(Join-Path $UserAppDir $BrowserEntry)): none, before or after the run.")
    }
    foreach ($n in $names) {
        $b = $userBefore[$n]
        $a = $userAfter[$n]
        $bText = if ($b) { $b.Text } else { 'did not exist' }
        $aText = if ($a) { $a.Text } else { 'does not exist' }
        $same = ($b -and $a -and $b.Key -eq $a.Key)
        $path = Join-Path $UserAppDir $n
        if ($n -eq $BrowserEntry) {
            if ($same) { $lines.Add("Browser data of AudioCaptor 0.1.1 and older ($path): unchanged ($aText).") }
            else {
                $browserChanged = $true
                $lines.Add("FAILED: the test copy changed the browser data of AudioCaptor 0.1.1 and older ($path): before $bText; after $aText.")
            }
        } elseif (-not $same) {
            $wroteOutside = $true
            $lines.Add("FAILED: AudioCaptor wrote to $path, outside the folder of its exe. Before: $bText; after: $aText.")
        }
    }
}
foreach ($n in $Notes) { $lines.Add("Note: $n") }
if ($script:StopReason) { $lines.Add("STOPPED: $script:StopReason") }

$stepsOk = 0; $stepsFailed = 0; $stepsSkipped = 0; $checksOk = 0; $checksAll = 0
$failures = New-Object System.Collections.Generic.List[string]
$scenarioName = ''
foreach ($r in $Results) {
    if ($r.Scenario -ne $scenarioName) {
        $scenarioName = $r.Scenario
        $lines.Add('')
        $lines.Add("Scenario $scenarioName")
    }
    $failedChecks = @($r.Checks | Where-Object { -not $_.Ok })
    $status = $r.Status
    if ($status -eq 'OK' -and $failedChecks.Count) { $status = 'FAILED' }
    $line = '  {0}. {1}: {2}' -f $r.N, $r.Title, $status
    if ($r.Detail) { $line += " ($($r.Detail))" }
    $lines.Add($line)
    foreach ($i in $r.Info) { $lines.Add("      $i") }
    $said = if ($r.Speech.Count) { $r.Speech -join ' | ' } else { 'nothing' }
    if ($r.Status -ne 'SKIPPED') { $lines.Add("      NVDA said: $said") }
    foreach ($c in $r.Checks) {
        $checksAll++
        if ($c.Ok) { $checksOk++ }
        $text = '      check: {0}: {1}' -f $c.Text, $(if ($c.Ok) { 'OK' } else { 'FAILED' })
        if (-not $c.Ok -and $c.Detail) { $text += " ($($c.Detail))" }
        $lines.Add($text)
    }
    switch ($status) { 'OK' { $stepsOk++ } 'FAILED' { $stepsFailed++ } default { $stepsSkipped++ } }
    if ($status -eq 'FAILED') {
        $why = @()
        if ($r.Detail) { $why += $r.Detail }
        foreach ($c in $failedChecks) { $why += $(if ($c.Detail) { "$($c.Text): $($c.Detail)" } else { $c.Text }) }
        $failures.Add(('[{0}] step {1}, {2}: {3}' -f $r.Scenario, $r.N, $r.Title, ($why -join '; ')))
        $failures.Add("      NVDA said: $said")
    }
}

$lines.Add('')
$passed = -not $script:StopReason -and $stepsFailed -eq 0 -and -not $browserChanged -and -not $wroteOutside
$verdict = if ($script:StopReason) { 'STOPPED' } elseif ($passed) { 'PASSED' } else { 'FAILED' }
$lines.Add(('Result: {0}. Steps: {1} OK, {2} failed, {3} skipped. NVDA checks: {4} of {5} OK.' -f $verdict, $stepsOk, $stepsFailed, $stepsSkipped, $checksOk, $checksAll))
if ($script:RunClock) { $lines.Add(('The keyboard was busy for {0:0} s.' -f $script:RunClock.Elapsed.TotalSeconds)) }
if ($failures.Count) {
    $lines.Add('Failed steps:')
    foreach ($f in $failures) { $lines.Add("  $f") }
}
if ($NvdaErrors.Count) { $lines.Add("Errors in the NVDA log during the run: $($NvdaErrors.Count). First: $($NvdaErrors[0])") }
if ($script:AppLogCopied) { $lines.Add("Log of the test copy: $AppLogCopy") }

[IO.File]::WriteAllLines($ReportPath, $lines, (New-Object Text.UTF8Encoding $true))
Write-Host ''
$lines | ForEach-Object { Write-Host $_ }
Write-Host "Report saved: $ReportPath"

if ($script:StopReason) { Invoke-Beep 'stopped'; exit 2 }
Invoke-Beep 'done'
if ($passed) { exit 0 }
exit 1

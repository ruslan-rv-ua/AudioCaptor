# AudioCaptor — Development Guide

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://v2.tauri.app/) (Rust backend + WebView frontend) |
| Backend | Rust 2021 edition |
| Frontend | Svelte 5 + TypeScript + Vite |
| Audio capture | [wasapi](https://github.com/HEnquist/wasapi-rs) (WASAPI bindings) |
| Audio mixing | Real-time mixer with [rubato](https://github.com/HEnquist/rubato) resampling |
| WAV writing | [hound](https://github.com/ruuda/hound) |
| Sound notifications | [rodio](https://github.com/RustAudio/rodio) (OGG playback via `include_bytes!`) |
| Global hotkeys | `tauri-plugin-global-shortcut` |
| Single instance | `tauri-plugin-single-instance` |
| Logging | `tauri-plugin-log` (file + console) |
| Package manager | [pnpm](https://pnpm.io/) |
| Task runner | [just](https://github.com/casey/just) |

## Prerequisites

- **Windows 10+** (the app is Windows-only by design)
- [Rust](https://rustup.rs/) via rustup: the version is pinned in `rust-toolchain.toml`, and rustup installs it automatically
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/) (`npm install -g pnpm`)
- [just](https://github.com/casey/just) (optional, for task shortcuts)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11; available on most Windows 10 machines)

## Getting Started

```bash
# Install JavaScript dependencies
pnpm install

# Start development server (Rust + Vite hot-reload)
just dev       # or: pnpm tauri dev
```

The app will open automatically. Rust changes trigger a rebuild; frontend changes hot-reload instantly.

## Project Structure

```
AudioCaptor/
├── src/                        # Frontend (Svelte 5 + TypeScript)
│   ├── App.svelte              # Main application component
│   ├── main.ts                 # Entry point
│   ├── app.css                 # Global styles & CSS tokens
│   └── lib/
│       ├── i18n.ts             # Paraglide i18n initialization
│       ├── components/         # UI components
│       ├── stores/             # Svelte state stores
│       ├── types/              # TypeScript type definitions
│       └── utils/              # IPC invoke helpers
├── src-tauri/                  # Backend (Rust / Tauri)
│   ├── src/
│   │   ├── lib.rs              # Tauri commands & app setup
│   │   ├── main.rs             # Entry point
│   │   ├── audio/              # Audio capture, mixing, writing
│   │   │   ├── capture.rs      # WASAPI mic & loopback capture
│   │   │   ├── devices.rs      # Device enumeration
│   │   │   ├── mixer.rs        # Real-time audio mixer
│   │   │   ├── writer.rs       # WAV file writer
│   │   │   └── types.rs        # Audio types & enums
│   │   ├── device_monitor.rs   # IMMNotificationClient hot-plug detection
│   │   ├── hotkey.rs           # Global hotkey FSM (short/long press)
│   │   ├── profiles.rs         # Recording profiles CRUD
│   │   ├── settings.rs         # Settings persistence & migration
│   │   ├── portable.rs         # Portable mode (paths relative to exe)
│   │   ├── sounds.rs           # Notification sounds (start/pause/stop)
│   │   ├── state.rs            # Shared application state
│   │   └── tray.rs             # System tray icon & menu
│   ├── sounds/                 # OGG notification sound files
│   ├── icons/                  # App icons (all platforms)
│   ├── capabilities/           # Tauri permission capabilities
│   ├── tauri.conf.json         # Tauri configuration
│   └── Cargo.toml              # Rust dependencies
├── messages/                   # i18n message files (en, uk)
├── backlog/                    # Tasks, milestones, decisions & research (Backlog.md)
├── scripts/
│   └── smoke-nvda.ps1          # NVDA smoke test of the built exe (just smoke)
├── public/                     # Static assets (favicon)
├── index.html                  # HTML entry point
├── package.json                # Node.js config & scripts
├── vite.config.ts              # Vite bundler configuration
├── justfile                    # Task runner recipes
└── tsconfig*.json              # TypeScript configuration
```

## Build Commands

| Command | Description |
|---------|-------------|
| `just dev` | Start dev server with hot-reload |
| `just build` | Production build (optimized, slow compile) |
| `just build-fast` | Fast build (less optimization, quick compile) |
| `just clean` | Clean Rust build artifacts |
| `just install` | Install JS dependencies |
| `just vite-dev` | Frontend-only dev server (no Rust) |
| `just check` | All local checks: frontend build and type check, clippy, tests, `cargo fmt --check`; stops at the first failure |
| `just smoke [scenario]` | NVDA smoke test of the release exe, local only (see «Перевірка з NVDA» below) |

<details>
<summary>Equivalent pnpm commands</summary>

```bash
pnpm tauri dev                                       # dev
pnpm tauri build --no-bundle                         # build (release profile)
pnpm tauri build --no-bundle -- --profile release-fast  # build-fast
pnpm vite dev --port 1420                            # vite-dev
```

</details>

## Build Profiles

| Profile | Optimizations | Use Case |
|---------|--------------|----------|
| `release` | `lto=true`, `codegen-units=1`, `opt-level="s"`, `strip=true`, `panic="abort"` | Final distribution |
| `release-fast` | `lto=false`, `codegen-units=16`, `opt-level=1`, `panic="unwind"` | Quick iteration during development |

The output binary is located at:
- **release**: `src-tauri/target/release/AudioCaptor.exe`
- **release-fast**: `src-tauri/target/release-fast/AudioCaptor.exe`

## Portable Mode

AudioCaptor runs as a **single portable executable** — no installation required. All data is stored in `AudioCaptor-data/` next to the exe:

```
AudioCaptor.exe
AudioCaptor-data/
├── settings.json     # User preferences
├── logs/             # Application logs
└── Recordings/       # Default recordings folder
```

These directories are created automatically on first launch (see `portable.rs`).

## App Icons

Icons are generated from a single 1024×1024 source PNG using the Tauri CLI:

```bash
pnpm tauri icon src-tauri/icons/128x128.png
```

This generates all required variants in `src-tauri/icons/` (ICO for Windows, ICNS for macOS, PNGs for Linux, plus mobile icons). The Windows Store Square logos and the webview favicon are generated separately with ImageMagick.

Icon integration points:
- `src-tauri/icons/` — all platform icon variants
- `src-tauri/tauri.conf.json` → `bundle.icon` — references for the packaging step
- `public/favicon.png` — webview favicon (referenced in `index.html`)

## Architecture Overview

### Audio Pipeline

```
┌──────────┐     ┌───────────┐     ┌─────────┐     ┌───────────┐
│ WASAPI   │────▶│ Ring      │────▶│ Mixer   │────▶│ WAV       │
│ Capture  │     │ Buffers   │     │ Thread  │     │ Writer    │
│ Threads  │     └───────────┘     │         │     │ (hound)   │
│          │                       │ resample│     └───────────┘
│ • mic    │                       │ volume  │
│ • loopback│                      │ clip    │
└──────────┘                       └─────────┘
```

- **Capture threads**: One per audio source (mic, loopback) using WASAPI shared mode
- **Ring buffers**: Lock-free transfer from capture threads to mixer
- **Mixer thread**: Resamples if needed, applies volume, clips, and writes to WAV
- **Pause**: Mixer keeps reading buffers (prevents desync) but skips writing

### IPC Commands

Frontend communicates with backend via Tauri IPC commands:

| Command | Direction |
|---------|-----------|
| `get_audio_devices` | Frontend → Backend |
| `start_recording` | Frontend → Backend |
| `pause_recording` | Frontend → Backend |
| `resume_recording` | Frontend → Backend |
| `stop_recording` | Frontend → Backend |
| `load_settings` / `save_settings` | Frontend ↔ Backend |
| `recording-state-changed` | Backend → Frontend (event) |

### Global Hotkey FSM

The global hotkey (default: `Pause/Break`) uses a finite state machine with short/long press detection:

| Current State | Short press (<500ms) | Long press (≥500ms) |
|--------------|---------------------|---------------------|
| Idle | → Start recording | → Start recording |
| Recording | → Pause | → Stop |
| Paused | → Resume | → Stop |

## Accessibility

Accessibility (a11y) is a **highest-priority requirement**:

- All UI elements have ARIA labels and roles
- Full keyboard navigation (Tab, Enter, Escape)
- Mnemonics: Alt+S (Start), Alt+P (Pause), Alt+R (Resume), Alt+T (Stop), Alt+I (Status info)
- `aria-live` region announces state changes to screen readers
- `decorations: true` in Tauri config (required for screen reader compatibility)
- Svelte compile-time a11y checks are enabled
- Sound notifications duplicate visual state changes

## Перевірка з NVDA (`just smoke`)

`just smoke` перевіряє зібраний exe разом з NVDA: скрипт `scripts/smoke-nvda.ps1` натискає клавіші
замість користувача й читає з журналу NVDA, що той сказав на кожному кроці. Перевірка лише локальна:
CI і Release її не запускають.

Що потрібно:

- Зібраний exe: `just build`. Тест бере `src-tauri/target/release/AudioCaptor.exe`; інший exe —
  параметр `-Exe`. Якщо код змінено після збірки, звіт про це попереджає.
- Запущений NVDA з рівнем журналу «Введення/виведення» (меню NVDA → Параметри → Налаштування →
  Загальні → Рівень журналювання). Тест шукає `nvda.log` у теці `%TEMP%`; інший шлях — параметр
  `-NvdaLog`. Якщо NVDA не запущено або він не пише мовлення в журнал, тест зупиняється з
  поясненням і кодом виходу 2.
- Пристрій відтворення за замовчуванням у Windows: сценарій `record` записує системний звук.
- Руки геть від клавіатури й миші, доки не з'явиться звіт: тест забирає клавіатуру й голос NVDA
  приблизно на дві хвилини.

Запуск:

- `just smoke` — усі сценарії, `just smoke <сценарій>` — один (кілька — через пробіл). Кожен
  сценарій запускає програму заново й наприкінці закриває її:
  - `launch` — NVDA оголошує вікно; Tab обходить усі елементи керування, у кожного є назва, і NVDA
    її читає;
  - `record` — Alt+S, Alt+P, Alt+R і Alt+T оголошують новий стан, Alt+I — поточний. Запис справді
    починається (системний звук, мікрофон не потрібен), але файл WAV тест не перевіряє;
  - `dialogs` — «About AudioCaptor» (F1), налаштування, новий профіль і підтвердження видалення:
    фокус заходить у діалог, Escape закриває його, і фокус повертається туди, звідки діалог
    відкрили;
  - `tray` — Escape ховає вікно, повторний запуск exe показує його, і NVDA його оголошує. Поки
    TASK-9 не виправлено, цей сценарій падає саме на повторному запуску.
- Звіт (англійською) друкується в консоль і зберігається в `%TEMP%\audiocaptor-smoke-report.txt`:
  для кожного кроку — результат, що сказав NVDA і які перевірки не пройшли, а наприкінці — список
  невдалих кроків. Код виходу: 0 — усе пройшло, 1 — є невдалі кроки, 2 — тест не запустився або
  зупинився (причину названо у звіті).
- Тестова копія працює з англійським інтерфейсом. Слова ролей, які NVDA каже своєю мовою
  (наприклад, «діалог»), тест приймає англійською й українською.

Що тест не чіпає:

- Тест запускає копію exe з `%TEMP%\audiocaptor-smoke` з власною `AudioCaptor-data` (англійський
  інтерфейс, два тестові профілі, вибрано пристрій відтворення за замовчуванням). Дані WebView2 ця
  копія теж тримає там: змінну `WEBVIEW2_USER_DATA_FOLDER` скрипт ставить лише для її процесу.
  Решта копій програми тримає дані WebView2 в `%LOCALAPPDATA%\com.audiocaptor.app\EBWebView`; звіт
  показує, що ця тека не змінилася, а якщо змінилася — тест не пройшов. Інші зміни в
  `%LOCALAPPDATA%\com.audiocaptor.app` (наприклад, у `logs`) робить сама програма, і тест не може
  їх відвернути: звіт показує їх як примітку. У цій теці тест читає лише назви верхніх елементів,
  кількість файлів і тек у кожному та час найновішої зміни. Наприкінці тека тесту видаляється, а
  журнал тестової копії лишається в `%TEMP%\audiocaptor-smoke-app.log`.
- Якщо працює ваш AudioCaptor, тест просить його закритися, як це робить кнопка закриття вікна,
  і потім не запускає його знову; звіт про це каже. Програма дозволяє лише одну копію й тримає
  клавішу Pause/Break. Якщо ваш AudioCaptor не закрився (наприклад, іде запис і він питає, чи
  зупинити), тест зупиняється й програму не чіпає. Тож не запускайте тест під час запису.
- Поки тестову копію сховано чи закрито, фокус стоїть у маленькому вікні скрипта («Smoke test
  home»), тож NVDA не читає у звіт ваших інших програм. Сповіщення, які NVDA прочитає під час
  прогону, у звіт таки потраплять: щоб їх не було, увімкніть режим «Не турбувати».
- З журналу NVDA тест читає лише те, що записано під час прогону.

Запускайте після змін в інтерфейсі, фокусі, діалогах, оголошеннях, показі чи хованні вікна, а
також після оновлення Tauri чи WebView2. Новий елемент головного вікна додавайте до `$TabOrder` у
скрипті, новий текст — до `$UI` (англійські рядки з `messages/en.json`).

Скрипт написано для Windows PowerShell 5.1 і лише символами ASCII: файл без BOM ця версія читає в
кодуванні ANSI. Українські слова записано в ньому як послідовності `\u`.

## CI/CD

### GitHub Actions Workflows

| Workflow | Trigger | Description |
|----------|---------|-------------|
| **CI** (`ci.yml`) | pull request → `develop`, or manual (`workflow_dispatch`); a push starts nothing | The same checks as `just check`, with the Rust version from `rust-toolchain.toml` |
| **Release** (`release.yml`) | manual (`workflow_dispatch`) | Checks the version and CHANGELOG, runs the same checks as `just check`, builds `AudioCaptor.exe`; publishes a GitHub Release only when `dry_run` is off |

### Release Process

Release робить усе по черзі й зупиняється на першій помилці, ще до тегу й релізу:

1. Звіряє введену версію з `package.json`, `src-tauri/Cargo.toml` і `src-tauri/tauri.conf.json`
   і перевіряє, що тегу `v<версія>` ще немає (див. «Захист від повторної версії» нижче).
2. Бере опис релізу — розділ `## [<версія>]` з `CHANGELOG.md`. Немає розділу — немає релізу.
3. Ставить Rust із `rust-toolchain.toml` і виконує ті самі перевірки, що `just check`.
4. Збирає `AudioCaptor.exe` і кладе його разом з описом релізу в артефакти запуску (зберігаються 7 днів).
5. Лише коли `dry_run` вимкнено, окреме завдання `publish` створює тег `v<версія>` на зібраному коміті
   й реліз з файлом `AudioCaptor.exe`. Тільки воно має право запису в репозиторій.

Порядок справжнього релізу:

1. Версію в трьох файлах і розділ у `CHANGELOG.md` вже оновлено, гілку релізу завершено, `main` запушено.
2. Пробний запуск на `main` (див. нижче) — зелений.
3. Запустити Release на `main` з вимкненим `dry_run`:
   `gh workflow run release.yml --ref main -f version=0.2.0 -f dry_run=false`
4. Дочекатись створення GitHub Release. Scoop оновиться сам — див. нижче.

### Пробний запуск

Пробний запуск (`dry_run`, увімкнено за замовчуванням) проходить усі кроки, крім публікації:
звірку версії, CHANGELOG, перевірки й збірку. Завдання `publish` пропускається, а збірка працює
з токеном лише для читання, тож тегу й релізу не буде навіть через помилку в workflow.
Зібраний exe можна завантажити з артефактів запуску й перевірити вручну.

Запуск на будь-якій запушеній гілці (версія має збігатися з версією в коді цієї гілки):

```powershell
gh workflow run release.yml --ref <гілка> -f version=<версія> -f dry_run=true
```

Те саме — кнопка «Run workflow» у вкладці Actions: вибрати гілку й лишити позначку `dry_run`.

**Чому не можна створювати справжній реліз для перевірки.** Кожен опублікований реліз упродовж доби
забирає Excavator у `scoop-bucket` і роздає користувачам Scoop як нову версію. Видалити реліз
потім уже пізно: користувачі встигнуть його поставити, а маніфест посилатиметься на зниклий файл.
Тому зміни в `release.yml` перевіряють лише пробним запуском, а тег `v*` створюють тільки для
справжніх версій.

**Захист від повторної версії.** Одразу після звірки версії Release перевіряє, чи є на GitHub
тег `v<версія>`. Справжній запуск з такою версією зупиняється до збірки. Без цього
`softprops/action-gh-release` оновив би наявний реліз і замінив би в ньому `AudioCaptor.exe` новою
збіркою: хеш файлу змінився б, і маніфест Scoop із записаним старим хешем перестав би встановлюватись.
Пробний запуск з такою версією лише попереджає і йде далі — він нічого не публікує, тож його можна
робити з поточною версією коду, навіть якщо її реліз уже є.

### Оновлення Scoop-маніфесту

Цей репозиторій **нічого не надсилає** до `scoop-bucket`. Оновлення робить сам bucket:
його workflow **Excavator** читає поля `checkver` і `autoupdate` у `bucket/audiocaptor.json`,
знаходить новий реліз, збирає URL і бере хеш — і комітить оновлений маніфест.

Запускається він **вручну** з вкладки Actions репозиторію
[scoop-bucket](https://github.com/ruslan-rv-ua/scoop-bucket/actions), плюс раз на добу
о 04:20 UTC як підстраховка. Натискати безпечно будь-коли: якщо оновлювати нема чого,
він нічого не змінює.

Раніше тут був workflow `update-scoop.yml`, який слав `repository_dispatch` до bucket
і для цього тримав секрет `SCOOP_BUCKET_TOKEN`. Обидва прибрано: PAT із правом запису
в чужий репозиторій треба ротувати, і він відмовляє мовчки — реліз проходить, а bucket
тихо лишається позаду.

### Required Secrets

Жодного. Workflow цього репозиторію користуються тільки тимчасовим `GITHUB_TOKEN`,
який GitHub видає кожному запуску й забирає після нього.

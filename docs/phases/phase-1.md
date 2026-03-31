# Етап 1 — MVP: Базовий запис аудіо

## Мета

Мінімально працюючий продукт: запис аудіо з мікрофона та/або loopback у WAV-файл з керуванням через UI та базовою підтримкою скрін-рідерів.

---

## Функціональність

### 1. Ініціалізація проекту

- Створити проект Tauri v2 + Svelte 5 + TypeScript + Vite (`npm create tauri-app@latest -- --template svelte-ts`)
- Налаштувати структуру каталогів: `src-tauri/`, `src/`, `messages/`
- Налаштувати `Cargo.toml` з усіма залежностями: `wasapi`, `hound`, `dasp`, `rubato`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tauri-plugin-log`, `tauri-plugin-single-instance`
- Налаштувати `tauri.conf.json`: `decorations: true` (обов'язково для a11y), capabilities, window config
- Статична лінковка CRT: `.cargo/config.toml` з `target-feature=+crt-static`
- Release profile в `Cargo.toml`: `panic = "abort"`, `codegen-units = 1`, `lto = true`, `opt-level = "s"`, `strip = true`
- Портативний режим: визначення базової директорії через `std::env::current_exe().parent()`
- Створення `settings.json`, `logs/`, `recordings/` при першому запуску
- Логування через `tauri-plugin-log` з `Folder` target відносно exe, ротація за розміром
- Єдиний екземпляр через `tauri-plugin-single-instance`

### 2. Захоплення аудіо (FR1.1, FR1.2, FR1.4, FR1.5, FR1.7)

- Перелік доступних аудіопристроїв (мікрофони та loopback) через `wasapi::DeviceCollection`
- Вибір конкретного мікрофона та loopback-пристрою
- Захоплення з мікрофона в окремому потоці через WASAPI `AudioClient`
- Захоплення loopback в окремому потоці через WASAPI з `AUDCLNT_STREAMFLAGS_LOOPBACK`
- Ring buffer для передачі даних між потоками захоплення та мікшером

### 3. Мікшування в реальному часі (FR2.1–FR2.5)

- Окремий Mixer Thread, що зчитує ring buffers мікрофона та loopback
- Ресемплінг через `rubato` при різних sample rates джерел
- Контроль гучності мікрофона (0.0–4.0, за замовчуванням 1.0) через `dasp::Signal::scale_amp()`
- Контроль гучності loopback (0.0–4.0, за замовчуванням 0.5) через `dasp::Signal::scale_amp()`
- Мікшування потоків через `dasp::Signal::add_amp()`
- Кліпінг вихідного сигналу до [-1.0, 1.0] через `dasp::Signal::clip_amp()`
- Конвертація моно→стерео через `dasp::Frame`

### 4. Керування записом (FR3.1–FR3.6)

- FSM станів: Idle → Recording → Paused → Recording → Idle
- Глобальний стан додатку в `Mutex<AppState>` з `RecordingState`
- Start: ініціалізація потоків захоплення + mixer + WAV writer
- Pause: mixer продовжує читати буфери, але не пише у WAV (запобігає десинхронізації)
- Resume: mixer відновлює запис у WAV
- Stop: зупинка потоків, фіналізація WAV-файлу через `hound::WavWriter::finalize()`
- IPC-команди: `start_recording`, `pause_recording`, `resume_recording`, `stop_recording`
- IPC-подія `recording-state-changed` з поточним станом та тривалістю

### 5. Файловий вивід (FR5.1–FR5.3, FR5.6, FR5.8, FR5.10, FR5.11)

- Три базові режими виводу:
  - **Мікрофон** — один WAV-файл з мікрофона
  - **Loopback** — один WAV-файл із системного звуку
  - **Мікс** — один WAV-файл зі змішаним аудіо
- Запис через `hound::WavWriter` з `WavSpec` (sample rate, bit depth, channels)
- Підтримка sample rates: 8000, 16000, 44100, 48000 Гц
- Автоматичне іменування: `recording_{YYYY-MM-DD_HH-MM-SS}.wav`
- Папка за замовчуванням: `{exe_dir}/recordings/`

### 6. Базова доступність (FR8.1, FR8.2, FR8.4, FR8.5)

- `decorations: true` у `tauri.conf.json` — обов'язково (Tauri Issue #12901)
- Усі елементи UI мають `role`, `aria-label`
- Повна клавіатурна навігація: Tab між елементами, Enter для активації, Escape для закриття діалогів
- Стан запису доступний для скрін-рідерів (aria-атрибути на елементах стану)
- Svelte compile-time a11y перевірки увімкнені

### 7. UI (Svelte 5)

- Головне вікно з елементами:
  - Вибір мікрофона (select/combobox)
  - Вибір loopback-пристрою (select/combobox)
  - Вибір режиму виводу (мікрофон / loopback / мікс)
  - Повзунки гучності мікрофона та loopback
  - Кнопки керування записом (старт/пауза/стоп)
  - Індикатор стану запису з таймером тривалості
- IPC-виклики через `@tauri-apps/api/core` `invoke()`
- Підписка на події через `@tauri-apps/api/event` `listen()`
- Мова UI: англійська (хардкод, без i18n-інфраструктури)

---

## Технічні вимоги та обмеження

| Вимога | Деталі |
|--------|--------|
| OC-1 | Windows only |
| OC-2 | Портативний .exe, збірка `pnpm tauri build --bundles none` |
| OC-3 | Усі шляхи відносно `exe_path.parent()` |
| NR3 | CPU < 1% під час запису |
| NR5 | Затримка < 20мс (WASAPI shared mode ~10мс + mixer) |
| NR6 | Логування у файл з ротацією |
| NR7 | Одночасний захоплення з мікрофона та loopback |
| NR8 | Безшовна пауза/відновлення (потоки не зупиняються) |

---

## Definition of Done

- [ ] Проект збирається в єдиний портативний .exe через `pnpm tauri build --bundles none`
- [ ] Exe не має DLL-залежностей крім системних (перевірка через `dumpbin /dependents`)
- [ ] При першому запуску створюються `settings.json`, `logs/`, `recordings/`
- [ ] Користувач може обрати мікрофон зі списку доступних
- [ ] Користувач може обрати loopback-пристрій зі списку доступних
- [ ] Запис з мікрофона працює — отриманий WAV-файл відтворюється коректно
- [ ] Запис loopback працює — отриманий WAV-файл відтворюється коректно
- [ ] Запис міксу працює — обидва джерела чутні у файлі
- [ ] Повзунки гучності змінюють рівень відповідного джерела
- [ ] Кліпінг працює — при гучності > 1.0 немає спотворень
- [ ] Пауза зупиняє запис даних у файл, відновлення продовжує без розривів
- [ ] Зупинка зберігає коректний WAV-файл з правильним іменем та timestamp
- [ ] Файли зберігаються в `{exe_dir}/recordings/`
- [ ] Усі UI-елементи доступні через Tab-навігацію
- [ ] NVDA озвучує всі елементи інтерфейсу (label, role, стан)
- [ ] Діалогові вікна закриваються через Escape
- [ ] Логи пишуться у `{exe_dir}/logs/`
- [ ] Другий екземпляр не запускається (фокус на існуючий)

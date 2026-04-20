# Рев'ю якості коду AudioCaptor

> **Тип задачі:** 🔧 Рефакторинг / якість
> **Стек:** Rust, Tauri v2, Svelte 5, TypeScript, Windows
> **Статус:** ✅ Всі знахідки виправлені

## TL;DR

> Проєкт загалом якісний — хороша структура, тести, a11y, i18n. Виявлено 2 баги (24-bit аудіо конверсія, пропущений SeparateFiles в hotkey перевірці пристроїв), кілька дублювань коду, неточності в документації та один a11y-дефект (`<html lang>` захардкоджений).

---

## 🐛 Баги

### 1. Некоректна конверсія 24-bit PCM аудіо

**Файл:** `src-tauri/src/audio/capture.rs`, функція `bytes_to_f32`

**Проблема:** Для 24-bit PCM семплів не працює sign extension від'ємних значень:

```rust
24 => data
    .chunks_exact(3)
    .map(|c| {
        let sample = i32::from_le_bytes([c[0], c[1], c[2], 0]) >> 8;
        sample as f32 / (1 << 23) as f32
    })
```

`from_le_bytes([c[0], c[1], c[2], 0])` ставить `0` в старший байт, тому після `>> 8` від'ємні значення стають додатними. Наприклад, 24-bit `-1` (0xFFFFFF) → результат ≈ +0.0078 замість ≈ -0.000000119.

**Виправлення:**
```rust
let sample = i32::from_le_bytes([0, c[0], c[1], c[2]]) >> 8;
```

**Рівень впевненості:** ✅ підтверджено (арифметика бітових операцій)
**Ризик:** Спотворення аудіо при записі з 24-bit пристроїв. Більшість пристроїв дають 16-bit або 32-bit float, тому баг може не проявлятись часто.

---

### 2. `SeparateFiles` пропущено в hotkey перевірці пристроїв

**Файл:** `src-tauri/src/hotkey.rs`, рядки ~130–145

**Проблема:** Перед стартом запису через hotkey виконується проактивна перевірка наявності пристроїв. Але `OutputMode::SeparateFiles` пропущений в обох `matches!`:

```rust
let needs_mic = matches!(
    profile.output_mode,
    OutputMode::Microphone | OutputMode::Mix
        | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
    // ← OutputMode::SeparateFiles відсутній!
);
let needs_loopback = matches!(
    profile.output_mode,
    OutputMode::Loopback | OutputMode::Mix
        | OutputMode::MixPlusMicrophone | OutputMode::MixPlusLoopback
    // ← OutputMode::SeparateFiles відсутній!
);
```

В `lib.rs` (start_recording_inner) та `recording.svelte.ts` — SeparateFiles присутній коректно.

**Наслідок:** При SeparateFiles без обраного пристрою — hotkey спробує почати запис замість warning sound. Запис впаде з помилкою "DEVICE_NOT_FOUND", яка прийде через event.

**Рівень впевненості:** ✅ підтверджено

---

## ⚠️ Проблеми a11y / коректності

### 3. `<html lang="en">` захардкоджений

**Файл:** `index.html`

```html
<html lang="en">
```

Атрибут `lang` не змінюється при виборі української мови. Screen readers (NVDA, JAWS, Narrator) використовують `lang` для вибору голосу / вимови. При `lang="en"` і українському інтерфейсі — NVDA читатиме український текст англійським голосом.

**Виправлення:** Встановлювати `document.documentElement.lang` при завантаженні мови в `App.svelte` onMount або в `initLanguage()`.

**Рівень впевненості:** ✅ підтверджено

---

### 4. `settingsVersion` default = 4 замість 5

**Файл:** `src/lib/stores/settings.svelte.ts`, рядок 8

```ts
let settingsVersion = $state(4);
```

Поточна версія settings — **5**. Цей default діє лише до першого `loadSettingsFields()`, але якщо `loadSettings()` впаде з помилкою, фронтенд буде працювати з version=4.

**Рівень впевненості:** ✅ підтверджено

---

## 🔁 Дублювання коду

### 5. `get_audio_devices` та `refresh_devices` — ідентичні команди

**Файл:** `src-tauri/src/lib.rs`

Обидві команди роблять абсолютно те саме — `audio::devices::list_all_devices()`. Одну з них варто видалити, або `refresh_devices` має викликати `get_audio_devices`.

---

### 6. `formatDuration` дублюється

Ідентична функція в:
- `src/App.svelte` (рядок ~99)
- `src/lib/components/StatusIndicator.svelte` (рядок ~15)

Варто винести в `src/lib/utils/format.ts`.

---

### 7. Repeated mode → device requirement logic

Логіка "який OutputMode потребує mic / loopback" повторюється в **4 місцях**:
- `lib.rs` → `start_recording_inner()` (2 match блоки)
- `hotkey.rs` → `handle_shortcut_event()` (2 matches!)
- `recording.svelte.ts` → `canRecord`, `needsMic`, `needsLoopback`, `readinessHint`

Для Rust: метод на `OutputMode`:
```rust
impl OutputMode {
    pub fn needs_mic(&self) -> bool { ... }
    pub fn needs_loopback(&self) -> bool { ... }
}
```

---

## 🧹 Dead code / зайве

### 8. Dead variable `should_stop` в `state_event_loop`

**Файл:** `src-tauri/src/lib.rs`, функція `state_event_loop`

```rust
let should_stop = false;
(payload, should_stop)
// ...
if should_stop { break; }
```

`should_stop` завжди `false`. Код ніколи не виконається.

---

## ⚡ Продуктивність

### 9. `read_settings()` читає файл з диска на кожну зміну стану запису

**Файл:** `src-tauri/src/tray.rs`, `update_tray_recording_state()`

```rust
let uk = crate::settings::read_settings().language == "uk";
```

Кожен виклик `update_tray_recording_state` (start / pause / resume / stop + 4Hz state emitter) читає і парсить `settings.json` з диска — тільки щоб дізнатися мову. Мова не змінюється під час запису.

**Варіант:** Зберігати language в `AppState` або передавати параметром.

---

## 📝 Документація

### 10. DEVELOPMENT.md посилається на неіснуючі файли

Секція "Documentation" в кінці DEVELOPMENT.md:
```
- [PRD](docs/requirements/PRD.md)
- [Development Phases](docs/phases/index.md)
- [Tech Stack Research](docs/stack-tauri-v2.md)
```

Жоден з цих файлів **не існує** у репозиторії.

---

### 11. Шлях до іконки в DEVELOPMENT.md

Команда генерації іконок:
```bash
pnpm tauri icon docs/requirements/resources/logo_audiocaptor.png
```

Файл `docs/requirements/resources/logo_audiocaptor.png` не існує (ми це вже виправили в README, але тут залишилось).

---

## 🔧 Конфігурація

### 12. Версія дублюється в 4 місцях

`0.1.0` вказано в:
- `package.json`
- `Cargo.toml`
- `tauri.conf.json`
- `bucket/audiocaptor.json`

При оновленні версії легко забути одне з місць. Tauri 2 не синхронізує їх автоматично.

---

### 13. COM cleanup пропущено на шляху помилки

**Файл:** `src-tauri/src/device_monitor.rs`

Якщо `RegisterEndpointNotificationCallback` повертає помилку, функція виходить з `return` **без** виклику `CoUninitialize()`, хоча `CoInitializeEx` вже було викликано.

---

## Порівняльна таблиця пріоритетів

| # | Проблема | Тип | Статус |
|---|----------|-----|--------|
| 1 | 24-bit аудіо конверсія | 🐛 Баг | ✅ Виправлено |
| 2 | SeparateFiles в hotkey | 🐛 Баг | ✅ Виправлено |
| 3 | `<html lang>` | ♿ A11y | ✅ Виправлено |
| 4 | settingsVersion = 4 | 🔧 Коректність | ✅ Виправлено |
| 5 | Ідентичні команди devices | 🔁 Дублювання | ✅ Виправлено |
| 6 | formatDuration дублювання | 🔁 Дублювання | ✅ Виправлено |
| 7 | Mode→device logic ×4 | 🔁 Дублювання | ✅ Виправлено |
| 8 | Dead code should_stop | 🧹 Cleanup | ✅ Виправлено |
| 9 | read_settings() в tray | ⚡ Perf | ✅ Виправлено |
| 10 | Мертві посилання в docs | 📝 Docs | ✅ Виправлено |
| 11 | Шлях до іконки в docs | 📝 Docs | ✅ Виправлено |
| 12 | Версія в 4 місцях | 🔧 Config | ℹ️ Задокументовано |
| 13 | COM cleanup | 🧹 Resource | ✅ Виправлено |

---

## Що НЕ потребує змін

- **Архітектура** — чітке розділення audio pipeline, state management, UI. Адекватна для desktop app.
- **Тести** — є unit-тести для settings migration, profiles CRUD, audio types, capture conversions, mixer utils, hotkey atomics. Покриття базових сценаріїв достатнє.
- **a11y** — ARIA labels, keyboard navigation, live regions, focus management в діалогах — все на хорошому рівні.
- **i18n** — Paraglide compile-time messages, два повних locale файли, tray menu дублює вручну (прийнятно).
- **Security** — CSP в tauri.conf.json, мінімальні capabilities, Tauri v2 permission model.
- **CSS** — consistent design tokens, dark/light theme, proper focus styles.

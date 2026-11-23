---
id: doc-1
title: 'Дослідження: перекодування записів у стиснені формати (FLAC, MP3)'
type: other
created_date: '2026-11-23T11:21:37Z'
updated_date: '2026-11-23T11:21:37Z'
---
# Постобробка записів: перекодування у інші формати

> **Тип задачі:** ✨ Нова фіча
> **Стек:** Rust, Tauri v2, hound (WAV), Windows desktop
> **Дата:** 2026-04-20

## TL;DR
> **Доцільно** як фіча для майбутнього релізу. Рекомендовано: підтримка **FLAC** (lossless, pure Rust — `flacenc`) + **MP3** (lossy, FFI до LAME — `mp3lame-encoder`). Конвертація після зупинки запису в окремому потоці, з опцією видалення WAV при успіху. Для v0.1.0 — не потрібно, але варто закласти в roadmap.

## Аналіз проблеми

### Поточний стан

- Записи зберігаються виключно у **WAV** (PCM 16-bit signed, stereo, 48 kHz за замовчуванням)
- 1 хвилина запису ≈ **11.5 МБ** (48000 × 2 × 2 байт × 60 с)
- 10 хвилин ≈ **115 МБ**, 1 година ≈ **691 МБ**
- Жодної постобробки не виконується
- Файли: `{назва}_{timestamp}.wav` у `AudioCaptor-data/Recordings/`

### Чому це потрібно

1. **Розмір файлів** — WAV займає багато місця; користувачі з обмеженим диском (SSD) помітять це при регулярному використанні
2. **Сумісність** — MP3 легше ділитися; деякі сервіси не приймають WAV
3. **Архівування** — FLAC зменшує розмір без втрати якості
4. **Очікування користувачів** — конкурентні рекордери (OBS, Audacity, ShareX) підтримують кілька форматів

### Розмір файлів за форматами (1 хвилина, стерео, 48 kHz)

| Формат | Тип | Розмір | Коефіцієнт стиснення | Якість |
|---|---|---|---|---|
| WAV 16-bit | Lossless | ~11.5 МБ | 1.0× | Джерело |
| FLAC | Lossless | ~5–7 МБ | 0.5–0.6× | Ідентичний джерелу |
| MP3 320 kbps | Lossy | ~2.4 МБ | 0.21× | Прозорий для більшості |
| MP3 192 kbps | Lossy | ~1.4 МБ | 0.12× | Майже прозорий |
| MP3 128 kbps | Lossy | ~0.96 МБ | 0.08× | Добрий для мовлення |
| OGG Vorbis q5 | Lossy | ~1.1 МБ | 0.10× | Майже прозорий |
| Opus 128 kbps | Lossy | ~0.96 МБ | 0.08× | Відмінний |

**Факт:** для мовленнєвого контенту (записи нарад, подкасти) стиснення FLAC ще ефективніше (0.3–0.4×), бо тиша та пеніода добре стискаються.

## Огляд Rust-бібліотек для кодування

### FLAC

| Крейт | Тип | Завантажень | Оновлено | Примітки |
|---|---|---|---|---|
| **`flacenc`** | Pure Rust | 456K | 4 міс. | ✅ Найкращий варіант. Активний, задокументований |
| `flac-bound` | FFI (libflac) | 16K | 1 рік | Потребує C-компіляції |
| `flac-codec` | Pure Rust | 3.8K | 5 днів | Новий, ще не перевірений |

**Рекомендація:** `flacenc` — pure Rust, без C-залежностей, 456K завантажень, ліцензія Apache-2.0.

### MP3

| Крейт | Тип | Завантажень | Оновлено | Примітки |
|---|---|---|---|---|
| **`mp3lame-encoder`** | FFI (LAME) | 620K | 9 днів | ✅ Найкращий. LAME — стандарт індустрії |
| `shine-rs` | Pure Rust | 341 | 3 міс. | Незрілий |
| `oxideav-mp3` | Pure Rust | 42 | 1 день | Щойно створений, мінімальний CBR encoder |

**Рекомендація:** `mp3lame-encoder` — FFI до LAME (battle-tested MP3 encoder), 620K завантажень, ліцензія LGPL-2.1 (LAME).

**Ліцензійна примітка:** LAME ліцензований під LGPL-2.1. При статичному лінкуванні (що робить Cargo за замовчуванням) це вимагає або: (a) надання об'єктних файлів для перелінкування, або (b) динамічного лінкування. MP3 патенти Fraunhofer **закінчились у 2017 році** — сам формат вільний.

### Opus / OGG

| Крейт | Тип | Завантажень | Оновлено | Примітки |
|---|---|---|---|---|
| `libopusenc` | FFI | 1K | 1 рік | Робочий, але мало використовується |
| `opus-oxide` | Pure Rust | 1.5K | 2 роки | Незрілий |
| `vorbis_rs` | FFI (libvorbis) | 130K | 1 рік | Для Vorbis (не Opus) |
| `ogg` | Pure Rust | 6.8M | 1 рік | Тільки OGG контейнер |

**Рекомендація:** Opus/Vorbis мають менш зрілу Rust-екосистему для кодування. Можна додати пізніше.

### FFmpeg (мета-підхід)

| Підхід | Плюси | Мінуси |
|---|---|---|
| `tauri-plugin-ffmpeg` | Всі формати | 153 завантажень, сирий, +30 МБ до бінарного |
| System FFmpeg | Всі формати | Потребує встановленого FFmpeg, не портативний |
| Bundled FFmpeg | Всі формати | +30-80 МБ до розміру, ліцензія GPL/LGPL |

**Рекомендація:** ❌ Занадто важкий для цього застосунку. AudioCaptor.exe зараз ~7 МБ — FFmpeg збільшив би у 5-10 разів.

## Розглянуті рішення

### Варіант A: Тільки FLAC (pure Rust)

**Суть:** Після зупинки запису конвертувати WAV → FLAC. Опціонально видалити WAV.

**Реалізація:**
```rust
// Cargo.toml
flacenc = "0.5"

// postprocess.rs
use flacenc::component::BitRepr;
use flacenc::config::Encoder as FlacConfig;
use flacenc::source::MemSource;

fn wav_to_flac(wav_path: &Path) -> anyhow::Result<PathBuf> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    let samples: Vec<i32> = reader.samples::<i16>().map(|s| s.unwrap() as i32).collect();
    
    let config = FlacConfig::default();
    let source = MemSource::from_samples(&samples, spec.channels as usize, spec.bits_per_sample as usize, spec.sample_rate as usize);
    let flac_stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size)?;
    
    let flac_path = wav_path.with_extension("flac");
    let mut file = std::fs::File::create(&flac_path)?;
    flac_stream.write(&mut file)?;
    Ok(flac_path)
}
```

**Переваги:**
- Zero C dependencies — чистий Rust, компілюється скрізь
- Lossless — жодної втрати якості
- 50–60% стиснення — суттєва економія
- Проста ліцензія (Apache-2.0)
- FLAC підтримується всюди (Windows, macOS, Linux, Android, iOS, web)

**Недоліки:**
- Тільки lossless — для максимального стиснення потрібен MP3/Opus
- Повне завантаження WAV у пам'ять для кодування (10 хв = ~115 МБ RAM)

**Рівень впевненості:** ✅

---

### Варіант B: FLAC + MP3

**Суть:** Підтримати обидва формати. FLAC для lossless, MP3 для максимального стиснення.

**Додатково до A:**
```rust
// Cargo.toml
mp3lame-encoder = "0.2"

fn wav_to_mp3(wav_path: &Path, bitrate: u32) -> anyhow::Result<PathBuf> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    let mut encoder = mp3lame_encoder::Builder::new().unwrap();
    encoder.set_num_channels(spec.channels as u8).unwrap();
    encoder.set_sample_rate(spec.sample_rate).unwrap();
    encoder.set_brate(mp3lame_encoder::Bitrate::Kbps192).unwrap();
    encoder.set_quality(mp3lame_encoder::Quality::Best).unwrap();
    let mut encoder = encoder.build().unwrap();
    
    // ... encoding loop (read chunks, encode, write)
    let mp3_path = wav_path.with_extension("mp3");
    Ok(mp3_path)
}
```

**Переваги:**
- Покриває обидва сценарії: lossless (FLAC) і maximum compression (MP3)
- MP3 — найбільш універсальний lossy формат
- Користувач обирає що йому потрібно

**Недоліки:**
- `mp3lame-encoder` — FFI, потребує компіляції LAME (C бібліотека)
- LAME під LGPL-2.1 — ліцензійні нюанси при статичному лінкуванні
- Два кодери — більший бінарний розмір (+~200 КБ для LAME)
- MP3 не підтримує >2 каналів та >48 kHz (для AudioCaptor це ОК — завжди stereo 48 kHz)

**Рівень впевненості:** ✅

---

### Варіант C: Тільки MP3

**Суть:** Підтримати лише MP3 конвертацію.

**Переваги:**
- Одна залежність
- MP3 = максимально universal формат
- Різко зменшує розмір файлів (8–20× стиснення)

**Недоліки:**
- Lossy — втрата якості (незворотна)
- LGPL нюанси
- Не підходить для користувачів, яким потрібна lossless якість

**Рівень впевненості:** ⚠️ занадто обмежено

---

### Варіант D: Зовнішня конвертація (не вбудована)

**Суть:** Не вбудовувати конвертацію. Документувати як використовувати зовнішні інструменти (ffmpeg CLI, Audacity).

**Переваги:**
- Нуль складності в застосунку
- Нуль додаткових залежностей

**Недоліки:**
- Погана accessibility: blind user повинен знайти, встановити та використати CLI інструмент
- Ручний процес для кожного файлу
- Суперечить філософії "простий рекордер для незрячих"

**Рівень впевненості:** ⚠️

## Порівняльна таблиця

| Критерій | A: FLAC only | B: FLAC + MP3 | C: MP3 only | D: Зовнішній |
|---|---|---|---|---|
| Стиснення (мовлення) | 2–3× | 2–12× | 8–12× | Будь-яке |
| Якість | Lossless | Вибір | Lossy | Будь-яка |
| Складність реалізації | ~100 рядків | ~200 рядків | ~100 рядків | 0 |
| C-залежності | 0 | 1 (LAME) | 1 (LAME) | 0 |
| Ліцензійний ризик | Нуль | LGPL (LAME) | LGPL (LAME) | Нуль |
| Розмір бінарного | +~50 КБ | +~250 КБ | +~200 КБ | 0 |
| Accessibility UX | ✅ | ✅ | ✅ | ❌ |
| Покриття сценаріїв | Archival | Archival + sharing | Sharing | Всі |

## Рекомендація

**Обраний варіант:** **B (FLAC + MP3)** — але **не для v0.1.0**.

**Чому:**
- FLAC покриває lossless сценарій (archival, якість), pure Rust, без ліцензійних проблем
- MP3 покриває lossy сценарій (sharing, економія місця), universal формат
- Разом — повністю покривають потреби цільової аудиторії
- Складність помірна (~200 рядків коду + UI елементи)

**Якщо потрібно MVP → Варіант A (тільки FLAC):**
- Pure Rust, нуль C-залежностей, нуль ліцензійних проблем
- Вже дає 2–3× стиснення
- MP3 можна додати пізніше

**Коли обирати D:** якщо фіча не затребувана і ресурси краще вкласти в інше.

## Реалізація (архітектура)

### Налаштування у RecordingProfile

```rust
// profiles.rs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PostProcessFormat {
    #[default]
    None,       // Залишити WAV
    Flac,       // Конвертувати у FLAC
    Mp3(u32),   // Конвертувати у MP3 (bitrate kbps)
}

pub struct RecordingProfile {
    // ... існуючі поля ...
    pub post_process: PostProcessFormat,
    pub delete_wav_after_convert: bool,
}
```

### Потік виконання

```
do_stop_recording()
  └→ mixer thread завершує запис WAV
     └→ emit("recording-stopped")
        └→ spawn postprocess thread
           ├→ read WAV
           ├→ encode FLAC/MP3
           ├→ write new file
           ├→ if delete_wav && success → delete WAV
           └→ emit("postprocess-complete" | "postprocess-error")
```

### Важливі деталі

1. **Пам'ять:** Для великих файлів — потоковий підхід (читати chunk → кодувати → писати), не завантажувати весь WAV у RAM
2. **Прогрес:** Для accessibility — `aria-live` анонс ("Конвертація...", "Готово")
3. **Помилка:** Якщо конвертація провалилась — WAV **НЕ** видаляється (safe default)
4. **Атомарність:** Записувати у тимчасовий файл → rename (щоб не залишити неповний файл)
5. **Concurrency:** Конвертація в окремому потоці, не блокує UI/запис

### Кроки впровадження

1. Додати `flacenc` у Cargo.toml
2. Створити `src/postprocess.rs` з функціями `wav_to_flac()`, (пізніше `wav_to_mp3()`)
3. Додати `PostProcessFormat` + `delete_wav_after_convert` до `RecordingProfile`
4. Settings migration (vN → vN+1)
5. Викликати конвертацію після `do_stop_recording`
6. Додати UI-елементи в налаштування профілю
7. Додати accessibility-анонси (live region)
8. (Опціонально) Додати `mp3lame-encoder` для MP3

## Ризики та міграція
- **Ризик A (FLAC, пам'ять):** `flacenc` потребує всі семпли в пам'яті (поточний API). Для 1-годинного запису = ~691 МБ RAM. **Мітигація:** потоковий підхід через `FrameSink`, або прийняти обмеження (записи рідко довші 30 хв)
- **Ризик B (MP3, ліцензія):** LAME під LGPL-2.1. **Мітигація:** AudioCaptor під MIT — потрібно або динамічне лінкування, або розподіл з об'єктними файлами. Альтернатива: чекати зрілості pure-Rust MP3 encoder
- **Ризик (видалення WAV):** Випадкове видалення при помилці конвертації. **Мітигація:** delete тільки після verify нового файлу (перевірити розмір > 0, валідний заголовок)
- **Відкат:** Прибрати залежність + поле з profiles. serde `#[serde(default)]` — старі файли працюватимуть

## Відкриті питання
- [ ] Чи підтримує `flacenc` потокове кодування (chunk by chunk) без завантаження всього файлу?
- [ ] Чи допустимий LGPL для цього проєкту? Якщо ні — відкласти MP3 до появи зрілого pure-Rust encoder
- [ ] Чи потрібна підтримка Opus/OGG? (для v0.x.0 — ні, екосистема незріла)
- [ ] Де показувати прогрес конвертації? (status bar? toast? aria-live region?)

## Джерела
- [flacenc](https://crates.io/crates/flacenc) — 456K завантажень, Apache-2.0, pure Rust
- [mp3lame-encoder](https://crates.io/crates/mp3lame-encoder) — 620K завантажень, FFI до LAME
- [LAME](https://lame.sourceforge.io/) — LGPL-2.1, стандарт MP3 кодування
- MP3 патенти: термін дії закінчився 2017-04-16 (останній патент Fraunhofer)
- [FLAC format](https://xiph.org/flac/) — вільний, відкритий lossless кодек
- Кодобаза AudioCaptor: writer.rs (WavSpec 16-bit), profiles.rs (RecordingProfile), lib.rs (recording flow)

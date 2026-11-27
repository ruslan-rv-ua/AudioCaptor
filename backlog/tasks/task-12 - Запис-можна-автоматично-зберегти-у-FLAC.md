---
id: TASK-12
title: Запис можна автоматично зберегти у FLAC
status: To Do
assignee: []
created_date: '2026-11-23T11:32:40Z'
updated_date: '2026-11-27T16:49:45Z'
labels:
  - backend
  - frontend
  - audio
  - a11y
  - model-fable
  - effort-medium
milestone: m-6
dependencies:
  - TASK-11
  - TASK-28
references:
  - src-tauri/src/audio/writer.rs
  - src-tauri/src/profiles.rs
  - src-tauri/src/settings.rs
  - src-tauri/src/lib.rs
  - src/lib/components/ProfileDialog.svelte
  - src-tauri/src/convert/mod.rs
  - src-tauri/src/convert/mp3.rs
  - src-tauri/Cargo.toml
priority: medium
type: feature
ordinal: 13400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
FLAC зберігає звук без втрат. За замірами TASK-11 година мовлення у FLAC займає близько 18 % від WAV (у 5,5 раза менше), а музики — близько 56 % (у 1,8 раза менше). Це третій формат стиснення після Opus і MP3 (рішення decision-5): для тих, кому потрібна точна копія запису, наприклад для архіву.

Спільну частину — вибір формату в профілі, конвертацію у фоні, тимчасовий файл, перевірку перед видаленням WAV, оголошення для скрінрідера — збудовано для Opus у TASK-28 і доповнено для MP3 у TASK-29 (`src-tauri/src/convert/`). Тут додається ще один `Codec`. У списку формату в профілі FLAC має один варіант (рішення decision-6): рівень стиснення впливає лише на швидкість і розмір, а не на якість, тож його задано в програмі.

Кодер вибрала TASK-11 (рішення decision-10): libFLAC 1.5.0 через крейт `libflac-sys` 0.3.4 без Ogg, рівень 5, блок 4096 кадрів, таблиця переходів кожні 10 с, MD5 у STREAMINFO. WAV читається частинами за кількістю кадрів, яку рахувала програма, тож пам'ять не росте з тривалістю запису. Найближчий зразок коду (кодер і перевірка) — `C:\Temp\audiocaptor-task11\size\libflac\src\codec.rs`.

Збирання: `libflac-sys` сам не збирається з налаштуваннями AudioCaptor (бере динамічний CRT, а крейт `cmake` відкидає прапорці оптимізації). Обидві проблеми знімає файл CMake на три рядки, підключений змінною `CMAKE_TOOLCHAIN_FILE_x86_64_pc_windows_msvc` у `[env]` файлу `src-tauri/.cargo/config.toml`; зразок — `C:\Temp\audiocaptor-task11\bench\cmake\static-crt.cmake`. Він діє й на libopus: без Ninja вона теж збиратиметься з оптимізацією. Задача змінює залежності й збирання, тож перед закриттям потрібен ручний запуск CI.

На відміну від Opus і MP3, FLAC — без втрат: розкодований файл має збігатися з WAV семпл у семпл. Після вдалого запису в теці лишається тільки `.flac` (рішення decision-7), тож WAV видаляється лише після такої звірки.

Залежно від режиму запис дає один або два WAV-файли (наприклад, «Мікс + Мікрофон» чи «Окремі файли»); у FLAC має опинитися кожен.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 У списку формату в профілі є варіант FLAC; вибір зберігається в профілі
- [ ] #2 Запис із форматом FLAC дає в теці профілю файл `.flac` для кожної доріжки; розкодований, він збігається семпл у семпл із тим, що дав би WAV
- [ ] #3 FLAC працює для кожної частоти, яку пропонує діалог профілю: 8, 16, 44,1 і 48 кГц
- [ ] #4 Після вдалого запису в теці лишається тільки `.flac`; WAV видаляється лише після того, як FLAC звірено з ним семпл у семпл
- [ ] #5 Збій кодування й вихід із програми під час кодування поводяться так само, як для Opus у TASK-28: WAV цілий, неповного `.flac` немає, а користувач чує звук попередження і бачить повідомлення
- [ ] #6 Запис тривалістю 60 хвилин дає FLAC у межах часу й пам'яті, записаних у рішенні decision-10
- [ ] #7 `README.md`, `README_UK.md` і `CHANGELOG.md` описують нову можливість
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [ ] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-27, головна сесія (вибір власника): опис оновлено за рішенням decision-10 (кодер, рівень, заміряні розміри, файл CMake для збирання, зразок коду); AC #6 тепер вимагає меж і часу, і пам'яті з decision-10. Прототип і звук TASK-11 лишаються в `C:\Temp\audiocaptor-task11` для цієї задачі.
<!-- SECTION:NOTES:END -->

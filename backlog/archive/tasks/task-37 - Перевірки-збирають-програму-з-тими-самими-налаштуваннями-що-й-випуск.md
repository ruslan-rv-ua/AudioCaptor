---
id: TASK-37
title: Перевірки збирають програму з тими самими налаштуваннями, що й випуск
status: Done
assignee:
  - '@claude'
created_date: '2026-11-27T17:25:09Z'
updated_date: '2026-11-27T20:15:14Z'
labels:
  - ci
  - model-sonnet
  - effort-low
  - testing
milestone: m-8
dependencies: []
priority: low
ordinal: 14100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Перевірки (`just check`, кроки CI і Release до збирання) збирають програму не так, як випуск. Налаштування випуску лежать у `src-tauri/.cargo/config.toml`: статичний C runtime (`+crt-static`) і, з TASK-12, файл `src-tauri/cmake/msvc-toolchain.cmake` для бібліотек на C (libFLAC, libopus). Cargo читає цей файл лише тоді, коли працює в теці `src-tauri`: так його запускає Tauri (`just dev`, `just build`, `pnpm tauri build`). А clippy і тести викликаються з кореня з `--manifest-path`, тож ідуть без обох налаштувань: Rust — з динамічним C runtime, libFLAC — без оптимізації і з налагоджувальним C runtime (у конфігурації Debug CMake сам вибирає `/MDd`).

Наслідки, помічені 27.11.2026 у TASK-12:
- Помилку компонування, яку знайшла TASK-11 (libFLAC з динамічним runtime, LNK1120), `just check` і ручний запуск CI не побачили б: її показує лише `just build` або пробний запуск Release (`dry_run=true`).
- Тестовий exe залежить від налагоджувальних DLL C runtime. Вони є там, де встановлено Visual Studio чи Build Tools (ПК розробки, `windows-latest`), тож тести проходять, але це випадковість середовища.
- У тестах libFLAC зібрано без `NDEBUG`: спрацьовують її внутрішні `assert`, а на пошкоджених файлах вона пише в stderr.

Рішення власника (27.11.2026): перенести `.cargo/config.toml` у корінь репозиторію, щоб усі виклики cargo мали ті самі налаштування, що й випуск (варіант 1). Опис нинішнього стану — у розділі «Бібліотеки на C: файл CMake» в `DEVELOPMENT.md`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Налаштування збирання (статичний C runtime і файл CMake для бібліотек на C) діють на всі виклики cargo: `just check`, `just dev`, `just build`, кроки CI і Release
- [x] #2 `just check`, `just build` і пробний запуск Release проходять, а тести FLAC ідуть зі статичним runtime і libFLAC з оптимізацією
- [x] #3 `DEVELOPMENT.md` описує, де лежать налаштування збирання і на що вони діють
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Усі локальні перевірки проходять: `just check`
- [x] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Перенести `src-tauri/.cargo/config.toml` у `.cargo/config.toml` в корені; шлях до файлу CMake тепер `src-tauri/cmake/msvc-toolchain.cmake` (відносно теки з `.cargo`).
2. Оптимізувати libFLAC і libopus у профілі dev: `opt-level = 2` для `libflac-sys` і `opusic-sys` у `src-tauri/Cargo.toml`; крейт `cmake` тоді бере конфігурацію RelWithDebInfo, для неї файл CMake додає `/O2 /Ob1 /DNDEBUG`.
3. Перевірити: статичний runtime у тестових exe, прапорці libFLAC у журналі CMake, `just check`, `just build` (розмір exe і `dumpbin /dependents` як у 0.4.0), час `just check` до і після.
4. Оновити `DEVELOPMENT.md`.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Робоча сесія, 27.11.2026, гілка `feature/task-37-build-settings`, коміт 9b4f70a.

- `.cargo/config.toml` перенесено в корінь; шлях до файлу CMake: `src-tauri/cmake/msvc-toolchain.cmake` (`relative = true` рахується від кореня).
- У профілі dev `opt-level = 2` для `libflac-sys` і `opusic-sys` (`src-tauri/Cargo.toml`). Крейт `cmake` тоді вибирає RelWithDebInfo; файл CMake додає для неї `/O2 /Ob1 /DNDEBUG`. Окремого вибору власника не знадобилося.
- Доказ: `cargo test --manifest-path ... -v` з кореня і `cargo build -v` з `src-tauri` передають rustc `+crt-static`. libFLAC для тестів: RelWithDebInfo, у FLAC.vcxproj `Optimization=MaxSpeed`, `RuntimeLibrary=MultiThreaded`, `NDEBUG`; 9 тестів FLAC пройшли.
- `just build`: exe 5 431 808 байт, як локальна збірка 0.4.0; список `dumpbin /dependents` той самий. libFLAC для випуску не перезбиралася: її налаштування не змінилися.
- Час `just check`: перший після перенесення 81 с; далі без змін у коді 6 с до / 9 с після, після зміни `lib.rs` 13 с до / 11 с після.
- Не перевірено: ручний запуск CI і пробний запуск Release на GitHub (чекають згоди власника); `just dev` з вікном програми (перевірено `cargo build` з `src-tauri`, який Tauri і запускає).
- Тестовий exe і до перенесення не імпортував `VCRUNTIME140.dll`, тож статичний runtime видно в командах rustc, а не в списку DLL тестового exe.

2026-11-27, головна сесія: гілку `feature/task-37-build-settings` злито в `develop`; головна сесія повторила `just check` (134 тести). Перед закриттям — ручний запуск CI і пробний запуск Release (вибір власника). decision-10 тепер називає нове місце `.cargo/config.toml`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Налаштування збирання випуску тепер діють на всі виклики cargo. `.cargo/config.toml` перенесено з `src-tauri/` у корінь репозиторію, бо cargo шукає його в теці запуску й вище, а `just check`, CI і кроки перевірок у Release викликають cargo з кореня. Статичний C runtime (`+crt-static`) і файл CMake для libFLAC і libopus тепер бачать і перевірки, і `just dev`, і `just build`. У профілі dev для `libflac-sys` і `opusic-sys` піднято `opt-level` до 2, а файл CMake дає конфігурації RelWithDebInfo прапорці `/O2 /Ob1 /DNDEBUG`, тож у тестах бібліотеки на C оптимізовані й без внутрішніх `assert`. `DEVELOPMENT.md` описує, де лежать налаштування; decision-10 називає нове місце файлу.

Перевірки: `just check` (134 тести; головна сесія повторила), `just build` — exe байт у байт як у локальній збірці 0.4.0, список DLL той самий. Звичайний `just check` триває як раніше; лише перший після перенесення перезбирав усе (81 с). Ручний CI: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46060695052; пробний запуск Release: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46060698779 — обидва зелені.

Не перевірено: `just dev` з вікном програми (зібрано лише профіль dev з `src-tauri`).
<!-- SECTION:FINAL_SUMMARY:END -->

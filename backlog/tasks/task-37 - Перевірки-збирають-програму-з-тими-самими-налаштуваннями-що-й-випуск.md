---
id: TASK-37
title: Перевірки збирають програму з тими самими налаштуваннями, що й випуск
status: To Do
assignee: []
created_date: '2026-11-27T17:25:09Z'
updated_date: '2026-11-27T19:21:46Z'
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
- [ ] #1 Налаштування збирання (статичний C runtime і файл CMake для бібліотек на C) діють на всі виклики cargo: `just check`, `just dev`, `just build`, кроки CI і Release
- [ ] #2 `just check`, `just build` і пробний запуск Release проходять, а тести FLAC ідуть зі статичним runtime і libFLAC з оптимізацією
- [ ] #3 `DEVELOPMENT.md` описує, де лежать налаштування збирання і на що вони діють
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

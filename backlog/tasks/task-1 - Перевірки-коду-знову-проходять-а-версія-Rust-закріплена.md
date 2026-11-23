---
id: TASK-1
title: Перевірки коду знову проходять, а версія Rust закріплена
status: To Do
assignee: []
created_date: '2026-11-23T11:28:13Z'
labels:
  - backend
  - ci
  - model-sonnet
  - effort-medium
milestone: m-0
dependencies: []
references:
  - src-tauri/src/audio/capture.rs
  - src-tauri/src/settings.rs
  - .github/workflows/ci.yml
priority: high
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Автоматична перевірка на GitHub (CI) червона з 2026-08-26: запуски 32940169820, 32943970973 і 32945514129 впали на кроці clippy. Код тоді не змінювався. Причина в тому, що CI щоразу встановлює найновіший Rust (`dtolnay/rust-toolchain@stable`), а Rust 1.98 додав до clippy правило `chunks_exact_to_as_chunks`. CI запускає clippy з `-D warnings`, тож кожне зауваження стає помилкою.

Перевірено локально 2026-11-23 з Rust 1.99.1:

- три зауваження `chunks_exact_to_as_chunks` у функції `bytes_to_f32` (`src-tauri/src/audio/capture.rs`, рядки 261, 265, 269);
- з кодом тестів (`--all-targets`, як вимагає Definition of Done) ще чотири зауваження `field_reassign_with_default` у тестах `src-tauri/src/settings.rs` (рядки 152, 190, 199, 276);
- інших зауважень немає, усі 64 тести проходять.

`bytes_to_f32` перетворює байти з аудіопристрою на семпли. Помилку з розширенням знака для 24 біт у ній виправлено в коміті 3624beb, і це покрито тестами: після змін функція має повертати ті самі значення.

Рішення власника від 2026-11-23: версію Rust закріпити у `rust-toolchain.toml` (1.99.1, як в axygen-totp і revisor), щоб новий випуск Rust не міг сам зробити перевірки червоними. Оновлення версії Rust відтоді — окрема свідома зміна.

Доки ця задача не виконана, пункт Definition of Done про перевірки Rust не може пройти в жодній іншій задачі.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Команда `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` завершується без зауважень
- [ ] #2 Команда `cargo test --manifest-path src-tauri/Cargo.toml` проходить; тести `bytes_to_f32` для 16, 24 і 32 біт лишилися без змін
- [ ] #3 У корені репозиторію є `rust-toolchain.toml` з точною версією 1.99.1 і компонентами `clippy` та `rustfmt`; коментар у файлі пояснює, навіщо версію закріплено
- [ ] #4 Команда `cargo --version` у теці проєкту показує 1.99.1, тобто версію з `rust-toolchain.toml`
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

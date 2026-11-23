---
id: TASK-4
title: >-
  CI запускається лише для pull request і вручну та перевіряє те саме, що just
  check
status: To Do
assignee: []
created_date: '2026-11-23T11:29:00Z'
updated_date: '2026-11-23T11:29:13Z'
labels:
  - ci
  - model-sonnet
  - effort-medium
milestone: m-0
dependencies:
  - TASK-1
  - TASK-3
references:
  - .github/workflows/ci.yml
  - DEVELOPMENT.md
  - CLAUDE.md
priority: medium
type: chore
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Рішення власника від 2026-11-23: CI запускається лише для pull request і вручну, як у quick-snippets. Власник розробляє сам, гілки завершує локально через git flow і pull request не відкриває. CI потрібен для чужих pull request (репозиторій публічний) і для перевірки на чистій машині за бажанням.

Стан `.github/workflows/ci.yml` на 2026-11-23:

- запускається на push у `main` і на pull request у `develop`; вручну запустити не можна;
- виконує `pnpm vite:build`, `pnpm check`, `cargo check` і `cargo clippy -- -D warnings` (без коду тестів); 64 тести не запускає взагалі;
- крок `cargo check` повторює те, що вже перевіряє clippy;
- встановлює найновіший Rust (`dtolnay/rust-toolchain@stable`), а не закріплену версію з `rust-toolchain.toml` (TASK-1);
- дії `actions/checkout@v4`, `actions/setup-node@v4` і `pnpm/action-setup@v4` розраховані на Node 20, який GitHub виводить з ужитку: кожен запуск із серпня має попередження «Node.js 20 is deprecated».

Розділ CI/CD у `DEVELOPMENT.md` каже «push / PR → develop», що не відповідає ні теперішньому стану, ні новому.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CI запускається для pull request і вручну (`workflow_dispatch`); push у `develop` чи `main` запуску не починає
- [ ] #2 CI виконує ті самі перевірки, що `just check`, зокрема `cargo test`, clippy з `--all-targets` і `cargo fmt --check`; кроку `cargo check` немає
- [ ] #3 CI збирає версією Rust із `rust-toolchain.toml` і не встановлює іншої
- [ ] #4 У журналі запуску немає попередження про застарілий Node 20
- [ ] #5 Запуск вручну на гілці задачі зелений, і його номер записано в нотатках (push і запуск — з дозволу власника)
- [ ] #6 Розділ CI/CD у `DEVELOPMENT.md` і розділ «Checks» у `CLAUDE.md` кажуть, коли запускається CI і як запустити його вручну
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

---
id: TASK-4
title: >-
  CI запускається лише для pull request і вручну та перевіряє те саме, що just
  check
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:29:00Z'
updated_date: '2026-11-23T12:44:51Z'
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

Обмеження GitHub: ручний запуск (`workflow_dispatch`) можливий лише для workflow, у якого цей тригер уже є у версії на типовій гілці (`develop`). Поки новий `ci.yml` не злито, `gh workflow run ci.yml --ref <гілка>` для гілки задачі, найімовірніше, не спрацює. Побачити зелений запуск до злиття можна через pull request у `develop`: тригер `pull_request` бере файл із гілки. Інакше CI запускають вручну на `develop` одразу після злиття.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 CI запускається для pull request і вручну (`workflow_dispatch`); push у `develop` чи `main` запуску не починає
- [x] #2 CI виконує ті самі перевірки, що `just check`, зокрема `cargo test`, clippy з `--all-targets` і `cargo fmt --check`; кроку `cargo check` немає
- [x] #3 CI збирає версією Rust із `rust-toolchain.toml` і не встановлює іншої
- [x] #4 У журналі запуску немає попередження про застарілий Node 20
- [x] #5 Запуск CI на GitHub із новим `ci.yml` зелений — через pull request з гілки задачі або вручну на `develop` одразу після злиття, — і його номер записано в нотатках (push, pull request і запуск — з дозволу власника)
- [x] #6 Розділ CI/CD у `DEVELOPMENT.md` і розділ «Checks» у `CLAUDE.md` кажуть, коли запускається CI і як запустити його вручну
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [x] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Виконала робоча сесія «TASK-4 · Sonnet · medium» (коміт 8d5abc0 на гілці `feature/task-4-ci-pr-manual`); перевірка головної сесії, 2026-11-23:

- `ci.yml` розібрано як YAML без помилок: тригери `pull_request` (лише в `develop`) і `workflow_dispatch`; кроки — ті самі п'ять перевірок, що в `just check`, у тому ж порядку; `cargo check` немає.
- Дії: `actions/checkout@v7` (v7.0.1), `actions/setup-node@v7` (v7.1.0), `pnpm/action-setup@v6` (v6.1.1), `Swatinem/rust-cache@v2` (v2.9.3). Через GitHub API перевірено, що всі чотири посилання існують і в їхніх `action.yml` стоїть `using: node24`. Дію `dtolnay/rust-toolchain` прибрано; Rust ставить крок `rustup toolchain install`, який без аргументів бере версію з `rust-toolchain.toml` (rustup 1.29.3 локально).
- `just check` на гілці проходить. Гілку завершено (`git flow feature finish --no-fetch --no-push`), `develop` відправлено на GitHub (c8ac8f0..8d5abc0); push нового запуску не почав.
- Ручний запуск на `develop`: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45175822725 (`gh workflow run ci.yml --ref develop`), 2026-11-23 12:35:55–12:41:28 UTC, success, усі кроки зелені. У журналі: rustup встановив `1.99.1-x86_64-pc-windows-msvc` («the active toolchain … has been installed»), інших версій не ставив; 66 тестів пройшли; жодного попередження про Node 20 і жодної позначки `##[warning]`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
CI тепер запускається лише для pull request у `develop` і вручну, перевіряє те саме, що `just check`, і збирає закріпленою версією Rust.

- `.github/workflows/ci.yml` (8d5abc0): тригери `pull_request` і `workflow_dispatch`, push нічого не запускає; кроки — збірка фронтенду, `pnpm check`, clippy з `--all-targets -- -D warnings`, `cargo test`, `cargo fmt --check`; Rust ставить `rustup toolchain install` з `rust-toolchain.toml`; дії на Node 24 (checkout v7, setup-node v7, pnpm/action-setup v6, rust-cache v2).
- `DEVELOPMENT.md` (таблиця CI/CD) і `CLAUDE.md` (розділ «Checks») кажуть, коли запускається CI і як запустити його вручну, а також що ручний запуск гілки можливий лише тоді, коли `ci.yml` з `workflow_dispatch` уже є на `develop`.

Перевірено на GitHub: ручний запуск 45175822725 на `develop` зелений за 5,5 хвилини; у журналі Rust 1.99.1, 66 тестів, жодного попередження про Node 20. Це перший зелений CI з 2026-08-26.

Пункти Definition of Done про тексти інтерфейсу, перевірку з NVDA і CHANGELOG не мали до чого застосовуватися: задача змінює лише CI.

Що лишилося: Release workflow досі збирає без перевірок, з `dtolnay/rust-toolchain@stable` і діями на Node 20 — це TASK-5.
<!-- SECTION:FINAL_SUMMARY:END -->

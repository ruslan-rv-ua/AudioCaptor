---
id: TASK-3
title: Rust-код відформатовано, а всі локальні перевірки запускає just check
status: To Do
assignee: []
created_date: '2026-11-23T11:28:58Z'
updated_date: '2026-11-23T11:29:13Z'
labels:
  - backend
  - testing
  - model-sonnet
  - effort-low
milestone: m-0
dependencies:
  - TASK-1
references:
  - justfile
  - backlog/config.yml
  - CLAUDE.md
priority: medium
type: chore
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rust-код проєкту ніколи не проходив через `cargo fmt`. Станом на 2026-11-23 `cargo fmt --check` показує 57 відмінностей у 10 файлах: `lib.rs` (18), `audio/mixer.rs` (15), `tray.rs` (9), `device_monitor.rs` (6), `audio/writer.rs` (3), `portable.rs` (2) і по одній в `audio/capture.rs`, `audio/mod.rs`, `settings.rs`, `state.rs`. Файлу `rustfmt.toml` немає. Без єдиного форматування кожна зміна в цих файлах тягне за собою випадкові правки відступів, і різницю важко читати.

Локальні перевірки зараз — це чотири окремі команди, перелічені в Definition of Done і в розділі «Checks» файлу `CLAUDE.md`. За рішенням власника від 2026-11-23 CI запускається лише для pull request і вручну, тож саме локальні перевірки вирішують, чи задача готова. Одна команда зменшує шанс пропустити якусь із них.

Так само зроблено у quick-snippets (TASK-48): форматування окремим комітом, його хеш у `.git-blame-ignore-revs`, усі перевірки в одному рецепті `just`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Команда `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` не знаходить відмінностей; стиль rustfmt за замовчуванням, без `rustfmt.toml`
- [ ] #2 Форматування зроблено окремим комітом без інших змін, і його повний хеш записано в `.git-blame-ignore-revs`
- [ ] #3 Рецепт `just check` запускає `pnpm vite:build`, `pnpm check`, clippy з `--all-targets -- -D warnings`, `cargo test` і `cargo fmt --check` та зупиняється на першій невдачі
- [ ] #4 Типові пункти Definition of Done у `backlog/config.yml` і розділ «Checks» у `CLAUDE.md` називають `just check` замість окремих команд
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

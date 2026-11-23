---
id: TASK-3
title: Rust-код відформатовано, а всі локальні перевірки запускає just check
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:28:58Z'
updated_date: '2026-11-23T12:28:36Z'
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
- [x] #1 Команда `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` не знаходить відмінностей; стиль rustfmt за замовчуванням, без `rustfmt.toml`
- [x] #2 Форматування зроблено окремим комітом без інших змін, і його повний хеш записано в `.git-blame-ignore-revs`
- [x] #3 Рецепт `just check` запускає `pnpm vite:build`, `pnpm check`, clippy з `--all-targets -- -D warnings`, `cargo test` і `cargo fmt --check` та зупиняється на першій невдачі
- [x] #4 Типові пункти Definition of Done у `backlog/config.yml` і розділ «Checks» у `CLAUDE.md` називають `just check` замість окремих команд
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
Перевірка головної сесії, 2026-11-23:

- Коміт c9616db містить лише форматування: `cargo fmt` (rustfmt 1.10.0 з Rust 1.99.1), застосований до `develop` на 4406b66 у тимчасовій копії репозиторію, дав дерево `src-tauri`, що повністю збігається з c9616db. Інших файлів, крім `.rs`, у коміті немає.
- `.git-blame-ignore-revs` (ef3c8a1) містить повний хеш c9616dbea35b40fa479d73a61b3a9048556a1f08.
- `just check` (cb90630) проходить усі п'ять кроків і з Git Bash, і з PowerShell (`sh` береться з `C:\scoop\shims`): 0 помилок у `pnpm check`, clippy без зауважень, 66 тестів, `cargo fmt --check` без відмінностей.
- Гілку `feature/task-3-fmt-just-check` завершено командою `git flow feature finish --no-fetch --no-push` перемотуванням, тож хеш c9616db не змінився.
- Локально ввімкнено `git config blame.ignoreRevsFile .git-blame-ignore-revs`: у `src-tauri/src/audio/mixer.rs` на коміт форматування тепер припадає 3 рядки замість 107. Це рядки, які форматування створило заново і які git не може приписати жодному раннішому коміту.
- Головна сесія додала `just check` до таблиці команд у `DEVELOPMENT.md` (коміт 4367293, гілка `feature/task-3-dev-docs`). Решту `DEVELOPMENT.md` оновлює TASK-8.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Rust-код відформатовано стилем rustfmt за замовчуванням, а всі локальні перевірки тепер запускає одна команда `just check`.

- c9616db: форматування 10 файлів у `src-tauri/src`, без інших змін. ef3c8a1: його хеш у `.git-blame-ignore-revs`, тож GitHub і локальний `git blame` (налаштування ввімкнено на цьому комп'ютері) пропускають цей коміт.
- cb90630: рецепт `check` у `justfile` запускає `pnpm vite:build`, `pnpm check`, clippy з `--all-targets -- -D warnings`, `cargo test` і `cargo fmt --check` та зупиняється на першій невдачі. Типовий Definition of Done у `backlog/config.yml` і розділ «Checks» у `CLAUDE.md` називають `just check`.
- 4367293: `just check` додано до таблиці команд у `DEVELOPMENT.md`.

Перевірено: дерево після форматування збігається з результатом `cargo fmt` на попередньому стані `develop`; `just check` проходить і з Git Bash, і з PowerShell, 66 тестів.

Пункти Definition of Done про тексти інтерфейсу, перевірку з NVDA і CHANGELOG не мали до чого застосовуватися: задача не змінює поведінки програми. Наявні задачі й чернетки зберігають власні копії старих пунктів Definition of Done з окремими командами; `just check` їх покриває.

Що лишилося відкритим: CI на GitHub ще не перевіряє форматування й не запускає тести — це TASK-4.
<!-- SECTION:FINAL_SUMMARY:END -->

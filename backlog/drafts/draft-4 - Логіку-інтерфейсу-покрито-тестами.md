---
id: DRAFT-4
title: Логіку інтерфейсу покрито тестами
status: Draft
assignee: []
created_date: '2026-11-23T11:30:23Z'
updated_date: '2026-11-27T19:21:49Z'
labels:
  - frontend
  - testing
  - model-sonnet
  - effort-medium
milestone: m-11
dependencies: []
priority: low
type: enhancement
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rust-частина має 64 тести, а вебчастина — жодного: `pnpm check` перевіряє лише типи. При цьому в сховищах Svelte є логіка, від якої залежить поведінка. Готовність до запису й підказки (`canRecord`, `needsMic`, `needsLoopback`, `readinessHint` у `src/lib/stores/recording.svelte.ts`) повторюють правила з Rust (`OutputMode::needs_mic` і `needs_loopback`). Перевірка якості коду у квітні 2026 знайшла помилку саме в такому повторенні: режим «Окремі файли» було пропущено в перевірці гарячої клавіші.

Що має вирішити власник: чи додавати Vitest і тести сховищ, і чи запускати їх разом з іншими локальними перевірками.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [ ] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

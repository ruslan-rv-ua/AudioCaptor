---
id: DRAFT-2
title: Залежності переведено на нові мажорні версії
status: Draft
assignee: []
created_date: '2026-11-23T11:30:22Z'
updated_date: '2026-11-28T18:50:04Z'
labels:
  - backend
  - frontend
  - model-opus
  - effort-high
milestone: m-11
dependencies: []
priority: low
type: chore
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Частина залежностей вебчастини відстала на мажорну версію, але оновити їх ще не можна або не варто окремо:

- `typescript` 5.8.3 → 7: `svelte-check` (4.7.8) підтримує TypeScript лише 5 і 6. Рішення власника (25.11.2026): переходити, коли `svelte-check` підтримає 7.
- `@types/node` 22 → новіша: типи мають відповідати версії Node, на якій збирають CI і Release (зараз 22), тож вони змінюються разом із нею.

Paraglide 3 — у TASK-63 (колишня DRAFT-7). Звукові й системні бібліотеки Rust (`windows`, `wasapi`, `ringbuf`, `rubato`, `rodio`) винесено в окремі задачі етапу 0.5.0.
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25, стан версій (поточна → остання): `@inlang/paraglide-js` 2.29.3 → 3.2.2, `typescript` 5.8.3 → 7.0.2 (є й 6.0.3), `@types/node` 20 → 26, `wasapi` 0.17.0 → 0.27.0, `rubato` 0.16.2 → 5.1.0, `ringbuf` 0.5.3 → 0.6.0 (0.4 → 0.5 зробила TASK-6), `rodio` 0.20.1 → 0.23.0, `windows` 0.59 → 0.62.2. `chrono` тягне лише сама програма.

2026-11-27, план етапів: чернетка в етапі «Супровід». Коли дійде черга, розбити її на задачі по одній звуковій бібліотеці, кожна з повною перевіркою запису (рішення власника 25.11.2026).

2026-11-28, головна сесія: за рішенням власника чернетку розбито на задачі етапу 0.5.0: TASK-56 (windows), TASK-57 (wasapi), TASK-58 (ringbuf), TASK-59 (rubato), TASK-60 (rodio). Тут лишилися TypeScript 7 і типи Node.
<!-- SECTION:NOTES:END -->

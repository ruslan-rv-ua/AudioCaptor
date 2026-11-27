---
id: DRAFT-2
title: Залежності переведено на нові мажорні версії
status: Draft
assignee: []
created_date: '2026-11-23T11:30:22Z'
updated_date: '2026-11-27T19:21:50Z'
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
Частина залежностей відстала на одну чи кілька мажорних версій (стан на 2026-11-23). Сумісні оновлення робить TASK-6; тут — переходи з несумісними змінами, кожен із яких треба оцінити окремо.

JS: `typescript` 5.8.3 → 7.0.2, `@inlang/paraglide-js` 2.15.1 → 3.2.2, `@types/node` 20 → 26.

Rust: `rodio` 0.20.1 → 0.23.0 (звукові сигнали), `rubato` 0.16.2 → 5.1.0 (передискретизація в мікшері), `wasapi` 0.17.0 → 0.27.0 (захоплення звуку), `windows` і `windows-core` 0.59 → 0.62, `ringbuf` 0.4.8 → 0.6.0 (якщо цей перехід не зроблено в TASK-6 заради закриття вразливості). Крейт `chrono` позначено як непідтримуваний (RUSTSEC-2026-0543); ним формується лише мітка часу в назві файлу запису.

Крейти `wasapi`, `rubato` і `ringbuf` — серце запису: після кожного переходу треба наново перевірити запис у всіх шести режимах і на всіх частотах.

Рішення власника (2026-11-25):
- звукові бібліотеки (`wasapi`, `rubato`, `ringbuf`, `rodio`, `windows` і `windows-core`) — після випуску 0.2.0, по одній бібліотеці на задачу, кожна з повною перевіркою запису;
- `chrono` замінити й `@types/node` оновити ще до 0.2.0 — це TASK-26;
- TypeScript 7 — коли його підтримає `svelte-check` (4.7.8 підтримує TypeScript 5 і 6);
- Paraglide 3 — разом із DRAFT-7.

Коли 0.2.0 вийде, решту цієї чернетки розбити на задачі.
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
<!-- SECTION:NOTES:END -->

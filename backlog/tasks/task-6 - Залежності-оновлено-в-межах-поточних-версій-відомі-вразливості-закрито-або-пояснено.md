---
id: TASK-6
title: >-
  Залежності оновлено в межах поточних версій, відомі вразливості закрито або
  пояснено
status: To Do
assignee: []
created_date: '2026-11-23T11:30:58Z'
labels:
  - backend
  - frontend
  - security
  - model-opus
  - effort-medium
milestone: m-0
dependencies:
  - TASK-1
references:
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - package.json
  - pnpm-lock.yaml
priority: medium
type: chore
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Залежності востаннє оновлювалися у квітні 2026 і ніколи не перевірялися на відомі вразливості. Перевірка 2026-11-23 показала таке.

**Rust.** `cargo audit` 0.23.0 проти `src-tauri/Cargo.lock` (599 крейтів): 6 вразливостей у 5 крейтах і 20 попереджень. Ці крейти компілюються в `AudioCaptor.exe`.

| Крейт | Версія | Запис | Виправлено у |
|---|---|---|---|
| cookie | 0.18.1 | RUSTSEC-2026-0594 | 0.18.3 |
| quick-xml | 0.38.4 | RUSTSEC-2026-0194, RUSTSEC-2026-0195 | 0.41.0 |
| ringbuf | 0.4.8 | RUSTSEC-2026-0293 | 0.5.2 |
| rkyv | 0.7.46 | RUSTSEC-2026-0235 | 0.8.17 |
| url | 2.5.8 | RUSTSEC-2026-0592 | 2.5.9 |

`ringbuf` — пряма залежність: через її буфери звук іде від потоків захоплення до мікшера (`audio/capture.rs`, `audio/mixer.rs`). Виправлення є лише у версії 0.5.2 і новіших, тобто з несумісними змінами API. Вразливість стосується випадку, коли падає `Drop` елемента; у буферах AudioCaptor лежать `f32`. Решта чотири крейти приходять через Tauri та його залежності.

Серед попереджень — `chrono` (RUSTSEC-2026-0543, «unmaintained»), теж пряма залежність: нею формується мітка часу в назві файлу запису (`lib.rs`).

`cargo update --dry-run` пропонує сумісні оновлення, зокрема `tauri` 2.10.3 → 2.12.7, `tauri-build` 2.5.6 → 2.7.6 і плагіни: `dialog` 2.7.0 → 2.8.4, `global-shortcut` 2.3.1 → 2.4.1, `log` 2.8.0 → 2.10.2, `single-instance` 2.4.0 → 2.5.5.

**JS.** `pnpm audit`: 33 вразливості (2 low, 17 moderate, 14 high), наприклад `vite` 8.0.3 (GHSA-9jrq-w75r-8gcw, виправлено у 8.1.6) і `devalue` через `svelte` (GHSA-wf3x-273g-mvxv, виправлено у 5.9.3). Майже все це інструменти збірки, які в exe не потрапляють; сам Svelte потрапляє. `pnpm outdated` показує сумісні оновлення: `vite` → 8.3.8, `svelte` → 5.59.1, `svelte-check` → 4.7.8, `@sveltejs/vite-plugin-svelte` → 7.3.1, `@tauri-apps/api` → 2.12.5, `@tauri-apps/cli` → 2.12.6, `@tauri-apps/plugin-dialog` → 2.8.4.

Пакети Tauri з боку Rust (`tauri`, плагіни) і з боку JS (`@tauri-apps/api`, плагіни) мають лишатися на однакових мінорних версіях, тому обидві частини — в одній задачі.

Поза межами задачі — переходи на нові мажорні версії (DRAFT-2) і питання, чи робити таку перевірку регулярною (DRAFT-5). Виняток — `ringbuf`, якщо без переходу вразливість не закрити.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Вивід `cargo audit` і `pnpm audit` до і після змін збережено в нотатках задачі
- [ ] #2 Кожну з шести вразливостей у Rust-крейтах або закрито оновленням, або в нотатках пояснено, чому вона не стосується AudioCaptor
- [ ] #3 `pnpm audit` не показує вразливостей рівня high; кожну, що лишилася, перелічено в нотатках із причиною
- [ ] #4 Кожне попередження `cargo audit`, що лишилося, перелічено в нотатках із причиною
- [ ] #5 Крейт `tauri` і пакет `@tauri-apps/api` мають однакову мінорну версію, а `pnpm tauri build --no-bundle` не скаржиться на розбіжність версій
- [ ] #6 Після оновлень `pnpm tauri build --no-bundle` дає `AudioCaptor.exe`, який запускається, а пробний запис у режимі «Мікс» дає WAV, що відтворюється
- [ ] #7 У `CHANGELOG.md` запис є лише тоді, коли закрита вразливість справді стосувалася програми або оновлення змінило її поведінку
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

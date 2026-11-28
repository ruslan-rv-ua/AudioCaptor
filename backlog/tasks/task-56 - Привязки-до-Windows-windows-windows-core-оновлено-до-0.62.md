---
id: TASK-56
title: Прив'язки до Windows (windows, windows-core) оновлено до 0.62
status: To Do
assignee: []
created_date: '2026-11-28T18:44:00Z'
labels:
  - backend
  - model-sonnet
  - effort-medium
milestone: m-10
dependencies: []
priority: low
type: chore
ordinal: 20200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Частина DRAFT-2 (рішення власника 25.11.2026 і 28.11.2026: звукові й системні бібліотеки оновлюються по одній на задачу, кожна з повною перевіркою запису; етап 0.5.0). Користувачі нічого не мають помітити.

Програма сама викликає Windows через `windows` і `windows-core` 0.59: кодування MP3 через Media Foundation (`convert/media_foundation.rs`, `convert/mp3.rs`), стеження за аудіопристроями (`device_monitor.rs`), клавіші вікна (`webview.rs`), місцевий час у журналі (`lib.rs`), мову системи (`portable.rs`), чергу конвертації (`convert/queue.rs`). Tauri уже бере `windows` 0.62.2, тож у збірці дві версії, а новий `wasapi` 0.27 теж потребує 0.62. Після цієї задачі програма й Tauri користуються однією версією, і можна оновлювати `wasapi`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `windows` і `windows-core` у `src-tauri/Cargo.toml` — 0.62, та сама версія, що в Tauri; `just check` проходить
- [ ] #2 Конвертація в MP3 працює, як раніше: тести Rust і сценарій `mp3` у `just smoke`
- [ ] #3 Програма помічає підключення й відключення аудіопристроїв, як раніше
- [ ] #4 Час у журналі — за годинником Windows, мова програми за мовою системи визначається, як раніше, а клавіші й меню браузера у вікні вимкнено, як після TASK-44
- [ ] #5 Повний `just smoke` проходить (відомий нестійкий крок DRAFT-10 — після повтору свого сценарію)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

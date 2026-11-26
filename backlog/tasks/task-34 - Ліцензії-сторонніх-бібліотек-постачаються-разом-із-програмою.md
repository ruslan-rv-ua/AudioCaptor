---
id: TASK-34
title: Ліцензії сторонніх бібліотек постачаються разом із програмою
status: To Do
assignee: []
created_date: '2026-11-26T08:52:14Z'
labels:
  - frontend
  - ci
  - docs
  - model-sonnet
  - effort-medium
milestone: m-4
dependencies: []
references:
  - src/lib/components/AboutDialog.svelte
  - src-tauri/Cargo.toml
  - package.json
  - justfile
priority: medium
type: task
ordinal: 12900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Більшість бібліотек, з яких зібрано AudioCaptor (Tauri, serde та інші під MIT чи Apache-2.0), вимагають постачати тексти своїх ліцензій разом із програмою. Зараз їх немає ні в діалозі About, ні в релізі: README згадує лише власну ліцензію MIT.

Кодери, які додаються за рішенням decision-5, роблять це обов'язковим: libopus, `opus-pure` і `rusty-opus` поширюються за BSD-3-Clause, яка прямо вимагає відтворити повідомлення про авторські права в документації чи інших матеріалах, що постачаються з програмою, а `flacenc` — за Apache-2.0. Тому задачу треба виконати до випуску 0.2.0.

AudioCaptor — один exe, тож тексти мають бути доступні в самій програмі.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Тексти ліцензій усіх сторонніх бібліотек Rust і JavaScript, що потрапляють у `AudioCaptor.exe`, збираються автоматично під час збирання
- [ ] #2 У програмі є місце, де ці тексти можна прочитати з клавіатури і скрінрідером, без інтернету
- [ ] #3 Нова залежність потрапляє в ці тексти без ручної правки, а `just check` падає, якщо ліцензію якоїсь бібліотеки не вдалося визначити
- [ ] #4 `README.md` і `README_UK.md` кажуть, де знайти ліцензії сторонніх бібліотек
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

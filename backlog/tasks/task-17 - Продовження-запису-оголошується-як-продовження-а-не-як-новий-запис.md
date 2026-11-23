---
id: TASK-17
title: Продовження запису оголошується як продовження, а не як новий запис
status: To Do
assignee: []
created_date: '2026-11-23T18:19:57Z'
labels:
  - frontend
  - a11y
  - model-sonnet
  - effort-low
milestone: m-1
dependencies: []
priority: medium
type: bug
ordinal: 9300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Після паузи клавіша Alt+R відновлює запис, але NVDA оголошує «Recording started» — той самий текст, що й на початку запису. Користувач не може відрізнити продовження від нового запису. Оголошення формує src/App.svelte (live_recording_started для стану Recording) без огляду на попередній стан.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Після паузи Alt+R, кнопка продовження, гаряча клавіша й меню трея дають оголошення «Recording resumed» (українською — «Запис продовжено»), а старт запису й надалі звучить як «Recording started»
- [ ] #2 Новий текст є в `messages/en.json` і `messages/uk.json`
- [ ] #3 Сценарій `record` у `just smoke` перевіряє окреме оголошення продовження й проходить
- [ ] #4 У `CHANGELOG.md` є запис про виправлення
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

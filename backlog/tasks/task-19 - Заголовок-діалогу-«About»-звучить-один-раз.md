---
id: TASK-19
title: Заголовок діалогу «About» звучить один раз
status: To Do
assignee: []
created_date: '2026-11-23T18:19:58Z'
labels:
  - frontend
  - a11y
  - model-sonnet
  - effort-low
milestone: m-1
dependencies: []
priority: low
type: bug
ordinal: 9500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Коли відкривається діалог «About», NVDA читає його заголовок тричі. Ймовірні причини — заголовок діалогу пов'язаний і як назва, і як опис, або фокус спершу стає на заголовок.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Під час відкриття діалогу «About» NVDA читає заголовок один раз
- [ ] #2 Інші діалоги (налаштування, профіль, підтвердження) теж не повторюють заголовок; сценарій `dialogs` у `just smoke` це перевіряє
- [ ] #3 У `CHANGELOG.md` є запис про виправлення
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

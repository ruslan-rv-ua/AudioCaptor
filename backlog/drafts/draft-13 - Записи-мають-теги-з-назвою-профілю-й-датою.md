---
id: DRAFT-13
title: Записи мають теги з назвою профілю й датою
status: Draft
assignee: []
created_date: '2026-11-26T08:52:23Z'
updated_date: '2026-11-27T19:21:49Z'
labels:
  - backend
  - audio
  - model-sonnet
  - effort-low
milestone: m-10
dependencies:
  - TASK-28
priority: low
type: feature
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Opus, MP3 і FLAC зберігають теги: назву, альбом, дату. Програвачі, телефони й скрінрідери показують їх замість назви файлу, тож записи легше знаходити. AudioCaptor міг би записувати, наприклад, назву профілю як альбом, а дату й час запису як назву.

Потрібне рішення власника: чи потрібні теги і які саме. Має сенс, коли з'являться стиснені формати (TASK-28).
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

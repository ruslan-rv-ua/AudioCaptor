---
id: DRAFT-6
title: Дані вбудованого браузера лежать поруч з exe
status: Draft
assignee: []
created_date: '2026-11-23T18:20:09Z'
updated_date: '2026-11-24T22:42:54Z'
labels:
  - backend
  - model-sonnet
  - effort-medium
milestone: m-2
dependencies: []
priority: medium
type: task
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14) 2026-11-23. Дані вбудованого браузера WebView2 (кеш і його власний стан) програма тримає поза `AudioCaptor-data`: Tauri за замовчуванням кладе профіль браузера в `%LOCALAPPDATA%\com.audiocaptor.app`, спільний для всіх копій AudioCaptor на комп'ютері. Це суперечить духу рішення decision-1 (усі дані портативної програми — поруч з exe) і означає, що тестова копія й ваша робоча ділять один профіль браузера.

У quick-snippets це вирішено так: профіль браузера лежить у `quick-snippets-webview/` поруч з exe, окремо від теки даних, щоб Scoop не зберігав кеш між оновленнями, а резервні копії теки даних його не захоплювали.

Що має вирішити власник:
- лишити як є;
- перенести профіль браузера в окрему теку поруч з exe (як у quick-snippets);
- перенести його всередину `AudioCaptor-data` (тоді Scoop зберігатиме кеш між оновленнями).

Старий профіль у `%LOCALAPPDATA%` програма в жодному разі не видаляє: ним можуть користуватися інші копії.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25: власник переніс чернетку у віху 0.2.0 (m-2), щоб вона не затримувала випуск 0.1.1.
<!-- SECTION:NOTES:END -->

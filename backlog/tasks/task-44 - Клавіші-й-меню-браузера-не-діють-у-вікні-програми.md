---
id: TASK-44
title: Клавіші й меню браузера не діють у вікні програми
status: To Do
assignee: []
created_date: '2026-11-27T20:00:47Z'
labels:
  - a11y
  - frontend
  - backend
  - model-sonnet
  - effort-medium
milestone: m-8
dependencies: []
references:
  - src-tauri/src/lib.rs
  - src-tauri/tauri.conf.json
priority: medium
type: bug
ordinal: 14450
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Огляд TASK-42 показав, що вікно WebView2 досі пропускає клавіші браузера. F5, Shift+F5, Ctrl+R і Ctrl+Shift+R перезавантажують сторінку: вікно скидається, незбережений профіль губиться, а під час запису NVDA хибно каже «запис почато». Ctrl+P відкриває друк, Ctrl+F, F3 і Ctrl+G — пошук, а клавіша контекстного меню, Shift+F10 і правий клік — меню браузера з пунктами «Оновити», «Зберегти як», «Друк». Tauri 2.12 не вимикає ні клавіш браузера, ні меню браузера: у wry обидва ввімкнені за замовчуванням. Масштаб (Ctrl+плюс) уже вимкнено, інструментів розробника в збірці для користувачів немає.

Власник вирішив 27.11.2026 винести це з TASK-43 в окрему задачу для патча 0.4.1: це вада, а не нова можливість.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 У зібраній програмі F5, Shift+F5, Ctrl+R, Ctrl+Shift+R, Ctrl+P, Ctrl+F, F3 і Ctrl+G нічого не роблять
- [ ] #2 Клавіша контекстного меню, Shift+F10 і правий клік не відкривають меню браузера; Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+A і Ctrl+Z у текстових полях діалогу профілю працюють як раніше
- [ ] #3 З NVDA перевірено, що клавіші програми (Alt+S, Alt+P, Alt+R, Alt+T, Alt+I, F1, Escape) і клавіші NVDA працюють як раніше
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

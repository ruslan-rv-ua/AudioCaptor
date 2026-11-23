---
id: TASK-20
title: Журнал програми пишеться лише в AudioCaptor-data
status: To Do
assignee: []
created_date: '2026-11-23T18:19:58Z'
labels:
  - backend
  - model-sonnet
  - effort-medium
milestone: m-1
dependencies: []
priority: medium
type: bug
ordinal: 9600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Крім AudioCaptor-data/logs/, програма пише другий журнал у %LOCALAPPDATA%\com.audiocaptor.app\logs: модуль 	auri-plugin-log лишає свої типові місця запису поруч із тим, що задано в src-tauri/src/lib.rs. Цей журнал спільний для всіх копій програми на комп'ютері. 96% його рядків — службові записи аудіобібліотеки рівня TRACE. Коли файл доходить до 1 МБ, модуль його видаляє. Кожен прогін тесту додає туди близько 44 КБ, і тест цього відвернути не може.

Це суперечить рішенню decision-1: усі дані портативної програми лежать в AudioCaptor-data поруч з exe.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Після запуску, запису й виходу з програми в `%LOCALAPPDATA%\com.audiocaptor.app\logs` нічого не з'являється і не змінюється
- [ ] #2 Журнал у `AudioCaptor-data/logs/` не містить рядків рівня TRACE і DEBUG від сторонніх бібліотек
- [ ] #3 Тест `just smoke` більше не показує примітку про записи програми поза її текою
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

---
id: TASK-53
title: Випуск 0.4.2
status: To Do
assignee: []
created_date: '2026-11-28T04:52:50Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-9
dependencies:
  - TASK-52
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 15500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Патч-випуск: WAV, перерваний збоєм програми, відкривається з правильною тривалістю (TASK-48); запис, довший за 6 год 12 хв, лишається одним файлом RF64 (TASK-50); гарячу клавішу, яку не можна задати, NVDA називає, а стара лишається (TASK-49); Tab ходить головним вікном по колу без зайвих слів NVDA (TASK-51); гаряча клавіша захоплюється однаково за будь-якої розкладки (TASK-52). Формат `settings.json` не змінюється.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.4.1 (TASK-41). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.4.2 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 Повний `just smoke` проходить на збірці випуску (відомий нестійкий крок DRAFT-10 — після повтору свого сценарію)
- [ ] #3 `CHANGELOG.md` має розділ 0.4.2 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [ ] #4 Реліз завершено командою `git flow release finish 0.4.2 --notag`: `main` і `develop` містять реліз, а тег `v0.4.2` створив Release workflow
- [ ] #5 GitHub Release v0.4.2 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.2
- [ ] #6 Маніфест Scoop показує 0.4.2 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

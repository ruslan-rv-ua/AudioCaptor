---
id: TASK-31
title: Випуск 0.4.0
status: To Do
assignee: []
created_date: '2026-11-26T08:03:51Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-6
dependencies:
  - TASK-30
  - TASK-12
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 13500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск із новою можливістю: записи можна автоматично зберігати у FLAC (TASK-12).

Профіль із форматом FLAC, відкритий у старішій версії, писатиме WAV (TASK-28), — `CHANGELOG.md` має про це сказати. Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.3.0 (TASK-30). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.4.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 `CHANGELOG.md` має розділ 0.4.0 лише зі змінами, помітними користувачам, і каже, що буде з профілями у форматі FLAC після повернення до старішої версії; опис релізу на GitHub — той самий розділ
- [ ] #3 Реліз завершено командою `git flow release finish 0.4.0 --notag`: `main` і `develop` містять реліз, а тег `v0.4.0` створив Release workflow
- [ ] #4 GitHub Release v0.4.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.0, а пробний запис у FLAC відтворюється
- [ ] #5 Маніфест Scoop показує 0.4.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

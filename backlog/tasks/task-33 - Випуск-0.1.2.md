---
id: TASK-33
title: Випуск 0.1.2
status: To Do
assignee: []
created_date: '2026-11-26T08:52:02Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-7
dependencies:
  - TASK-10
  - TASK-24
  - TASK-25
  - TASK-26
  - TASK-32
  - TASK-19
  - TASK-23
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 10080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск виправлень, не чекаючи Opus (0.2.0); так вирішив власник 26 листопада 2026:

- тека даних вбудованого браузера лежить поруч з exe (TASK-25; уже в `develop` і в розділі Unreleased `CHANGELOG.md`);
- нечитабельний `settings.json` більше не стирається мовчки (TASK-32);
- дві помилки доступності: TASK-19 і TASK-23.

Формат `settings.json` не змінюється. Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.1.1 (TASK-10). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.1.2 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 `CHANGELOG.md` має розділ 0.1.2 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [ ] #3 Реліз завершено командою `git flow release finish 0.1.2 --notag`: `main` і `develop` містять реліз, а тег `v0.1.2` створив Release workflow
- [ ] #4 GitHub Release v0.1.2 містить `AudioCaptor.exe`; завантажений файл показує версію 0.1.2, а пробний запис у WAV відтворюється
- [ ] #5 Маніфест Scoop показує 0.1.2 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

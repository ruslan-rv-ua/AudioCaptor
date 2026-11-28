---
id: TASK-55
title: Випуск 0.5.0
status: To Do
assignee: []
created_date: '2026-11-28T18:29:45Z'
updated_date: '2026-11-28T18:50:04Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-10
dependencies:
  - TASK-43
  - TASK-54
  - TASK-56
  - TASK-57
  - TASK-58
  - TASK-59
  - TASK-60
  - TASK-63
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 20500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск з новими можливостями: клавіші для папки записів, профілів, налаштувань і меню трея, повний шлях у полі теки й довідка (TASK-43); теги з назвою профілю й датою у файлах Opus, MP3 і FLAC (TASK-54).

Також оновлено бібліотеки без змін для користувачів: TASK-56, TASK-57, TASK-58, TASK-59, TASK-60 і TASK-63 (Paraglide 3).

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.4.2 (TASK-53). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.5.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 Повний `just smoke` проходить на збірці випуску (відомий нестійкий крок DRAFT-10 — після повтору свого сценарію)
- [ ] #3 `CHANGELOG.md` має розділ 0.5.0 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [ ] #4 Реліз завершено командою `git flow release finish 0.5.0 --notag`: `main` і `develop` містять реліз, а тег `v0.5.0` створив Release workflow
- [ ] #5 GitHub Release v0.5.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.5.0
- [ ] #6 Маніфест Scoop показує 0.5.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
<!-- AC:END -->

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
2026-11-28: за рішенням власника в етап додано оновлення бібліотек із DRAFT-2 (TASK-56…TASK-60); випуск чекає й на них.

2026-11-28: за рішенням власника в етап додано TASK-63 (Paraglide 3, колишня DRAFT-7); випуск чекає й на неї.
<!-- SECTION:NOTES:END -->

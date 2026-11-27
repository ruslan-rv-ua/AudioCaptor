---
id: TASK-41
title: Випуск 0.4.1
status: To Do
assignee: []
created_date: '2026-11-27T19:21:47Z'
updated_date: '2026-11-27T21:40:00Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-8
dependencies:
  - TASK-40
  - TASK-44
  - TASK-45
  - TASK-46
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 14500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Патч-випуск: Alt+S пояснює, чому запис не почався, коли не вибрано пристрою (TASK-38), а «Запис зупинено» не губиться, коли конвертація закінчується одразу (TASK-39); клавіші й меню браузера більше не перезавантажують вікно й не відкривають друку (TASK-44). Разом із ними йдуть зміни для безпечних і надійних випусків: перевірки збирають програму так само, як випуск (TASK-37), модулі inlang мають закріплені версії (TASK-40). Ще в патчі: час у журналі за годинником Windows (TASK-45) і нечитабельний `settings.json` не затирається пізніше (TASK-46). Формат `settings.json` не змінюється.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.4.0 (TASK-31). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.4.1 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 `CHANGELOG.md` має розділ 0.4.1 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [ ] #3 Реліз завершено командою `git flow release finish 0.4.1 --notag`: `main` і `develop` містять реліз, а тег `v0.4.1` створив Release workflow
- [ ] #4 GitHub Release v0.4.1 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.1
- [ ] #5 Маніфест Scoop показує 0.4.1 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
2026-11-27: TASK-36 (тест із NVDA не натискає клавіш у чужих вікнах) власник відклав на невизначений термін, тож її прибрано із залежностей, і 0.4.1 виходить без неї. Перевірка з NVDA під час випуску, як і раніше, потребує власника за ПК. Опис етапу 0.4.1 (m-8) ще згадує цей тест: CLI не має команди, щоб змінити опис етапу.

2026-11-27: додано TASK-44 (клавіші й меню браузера), винесену з TASK-43 за рішенням власника.

2026-11-27: за вибором власника в патч додано TASK-45 (колишня DRAFT-12) і TASK-46 (колишня DRAFT-19). Виконані й заархівовані TASK-37, TASK-38 і TASK-39 із переліку залежностей випали, бо CLI не приймає архівних задач у `--depends-on`.
<!-- SECTION:NOTES:END -->

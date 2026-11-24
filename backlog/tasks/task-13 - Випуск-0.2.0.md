---
id: TASK-13
title: Випуск 0.2.0
status: To Do
assignee: []
created_date: '2026-11-23T11:32:40Z'
updated_date: '2026-11-24T23:52:06Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-2
dependencies:
  - TASK-10
  - TASK-12
  - TASK-24
  - TASK-25
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск із новою можливістю: записи можна автоматично зберігати у FLAC (TASK-12). Формат `settings.json` переходить на версію 6, тому в `CHANGELOG.md` має бути сказано, що відбувається з налаштуваннями під час оновлення і чи можна повернутися до 0.1.x з тим самим файлом.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.1.1 (TASK-10). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.2.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 `CHANGELOG.md` має розділ 0.2.0 лише зі змінами, помітними користувачам, і каже, що відбувається з `settings.json` під час оновлення та повернення до 0.1.x; опис релізу на GitHub — той самий розділ
- [ ] #3 Реліз завершено командою `git flow release finish 0.2.0 --notag`: `main` і `develop` містять реліз, а тег `v0.2.0` створив Release workflow
- [ ] #4 GitHub Release v0.2.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.2.0, а пробний запис у FLAC відтворюється
- [ ] #5 Маніфест Scoop показує 0.2.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [ ] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

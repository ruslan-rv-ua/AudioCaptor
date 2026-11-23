---
id: TASK-10
title: Випуск 0.1.1
status: To Do
assignee: []
created_date: '2026-11-23T11:31:35Z'
updated_date: '2026-11-23T15:31:20Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-1
dependencies:
  - TASK-2
  - TASK-5
  - TASK-6
  - TASK-8
  - TASK-9
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
  - .github/workflows/release.yml
priority: medium
type: task
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Перший випуск після паузи. Для користувачів він несе виправлення вікна (TASK-9) і те з оновлення залежностей, що змінило exe (TASK-6). Для проєкту це перший справжній прохід оновленого порядку випуску: перевірки перед збіркою, звірка версії, опис релізу з `CHANGELOG.md` (TASK-5), завершення релізу через git flow без тегу (TASK-2) і документ, що все це описує (TASK-8).

Формат `settings.json` лишається версії 5, тож повернення до 0.1.0 безпечне; перед випуском це треба перевірити ще раз.

Порядок випуску — у `DEVELOPMENT.md`. Push і запуск workflow потребують дозволу власника. Кожен опублікований реліз упродовж доби забирає Excavator у `ruslan-rv-ua/scoop-bucket` (запускається щодня о 04:20 UTC або вручну), тож пробних релізів не буває: для перевірки є пробний запуск.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Версія 0.1.1 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [ ] #2 `CHANGELOG.md` має розділ 0.1.1 лише зі змінами, помітними користувачам, а опис релізу на GitHub — той самий розділ
- [ ] #3 Реліз завершено командою `git flow release finish 0.1.1 --notag`: `main` і `develop` містять реліз, а тег `v0.1.1` створив Release workflow
- [ ] #4 GitHub Release v0.1.1 містить `AudioCaptor.exe`; завантажений файл запускається й показує версію 0.1.1 у діалозі «Про програму»
- [ ] #5 Маніфест Scoop у `ruslan-rv-ua/scoop-bucket` показує 0.1.1 після запуску Excavator, а `scoop update audiocaptor` зберігає теку `AudioCaptor-data`
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Зауваження головної сесії, 2026-11-23: розділ 0.1.0 у `CHANGELOG.md` містить рядок «CI/CD: GitHub Actions workflows for CI, Release, and Scoop manifest update». Це подробиця для розробників, а не для користувачів, і вона застаріла: workflow оновлення Scoop прибрано. Під час підготовки 0.1.1 вирішити з власником, чи прибрати рядок із розділу 0.1.0. Опис релізу тепер береться з `CHANGELOG.md`, тож такі рядки потрапляють і на сторінку релізу.
<!-- SECTION:NOTES:END -->

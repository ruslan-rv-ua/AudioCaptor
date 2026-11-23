---
id: TASK-5
title: Release перевіряє код перед збіркою і має пробний запуск
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:29:02Z'
updated_date: '2026-11-23T15:31:20Z'
labels:
  - ci
  - release
  - model-opus
  - effort-medium
milestone: m-0
dependencies:
  - TASK-3
  - TASK-4
references:
  - .github/workflows/release.yml
  - CHANGELOG.md
  - DEVELOPMENT.md
priority: medium
type: enhancement
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Коли CI запускається лише для pull request і вручну (TASK-4), Release workflow стає єдиною обов'язковою перевіркою на чистій машині, перш ніж exe потрапить до користувачів. Так улаштовано у quick-snippets.

Стан `.github/workflows/release.yml` на 2026-11-23:

- збирає й публікує реліз без жодної перевірки коду;
- не звіряє введену версію з версією в коді. Версія записана в трьох файлах (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`), і помилка під час введення опублікувала б, наприклад, `v0.2.0` з exe, який показує 0.1.0;
- опис релізу генерується автоматично і містить лише посилання «Full Changelog» (так виглядає v0.1.0), хоча текст для користувачів є в `CHANGELOG.md`;
- зміни в самому workflow неможливо перевірити, не опублікувавши реліз. А кожен опублікований реліз упродовж доби забирає Excavator у `ruslan-rv-ua/scoop-bucket` і роздає користувачам Scoop.

Тег `v<версія>` створює сам workflow (`softprops/action-gh-release`), а `git flow release finish` запускається з `--notag`. Назву файлу релізу `AudioCaptor.exe` і формат тегу міняти не можна: на них спирається адреса в `autoupdate` маніфесту Scoop (decision-2).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Перед збіркою Release виконує ті самі перевірки, що `just check`; якщо перевірка падає, ні тегу, ні релізу не створено
- [x] #2 Release зупиняється до збірки, якщо введена версія не збігається з версією в `package.json`, `src-tauri/Cargo.toml` і `src-tauri/tauri.conf.json`
- [x] #3 Пробний запуск (`dry_run`) виконує перевірки і збірку, але не створює ні тегу, ні релізу
- [x] #4 Опис релізу на GitHub — це розділ відповідної версії з `CHANGELOG.md`
- [x] #5 Назва файлу релізу `AudioCaptor.exe` і формат тегу `v<версія>` не змінилися
- [x] #6 Release збирає версією Rust із `rust-toolchain.toml`, а в журналі запуску немає попередження про застарілий Node 20
- [x] #7 Пробний запуск на гілці задачі проходить, і його номер записано в нотатках (push і запуск — з дозволу власника)
- [x] #8 `DEVELOPMENT.md` описує пробний запуск і пояснює, чому справжній реліз не можна створювати для перевірки
- [x] #9 Release зупиняється до збірки, якщо тег `v<версія>` уже існує; у пробному запуску лише попереджає
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [x] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Виконала робоча сесія «TASK-5 · Opus · medium»: коміти 81dbde3 і 6615172 на гілці `feature/task-5-release-checks`. Перевірка головної сесії, 2026-11-23:

- Сесія знайшла, що справжній запуск з уже випущеною версією оновив би наявний реліз і замінив би в ньому `AudioCaptor.exe` (документація `softprops/action-gh-release@v3`: існуючий реліз оновлюється, `overwrite_files` за замовчуванням `true`). Через це хеш у маніфесті Scoop перестав би збігатися. Власник погодив захист, і він став критерієм #9 (коміт 6615172): `git ls-remote --tags origin`, у справжньому запуску помилка до збірки, у пробному — попередження.
- Локально перевірено: звірка версії пропускає 0.1.0 і відхиляє `v0.1.0`, `0.2.0` і `0.1`; розділ CHANGELOG для 0.1.0 витягується (21 рядок, без рядків-посилань), для відсутньої версії — помилка; захист знаходить `v0.1.0` і не знаходить `v9.9.9`. Усі дії існують і працюють на Node 24; `release.yml` — коректний YAML; `just check` проходить.
- Пробний запуск 45178681416 (`gh workflow run release.yml --ref feature/task-5-release-checks -f version=0.1.0 -f dry_run=true`), 12:54–13:02 UTC: success. Попередження «Tag v0.1.0 already exists», Rust 1.99.1, 66 тестів, збірка release 3 хв 25 с, артефакт `release` (до 2026-11-30), завдання `publish` пропущено, попереджень про Node 20 немає.
- На GitHub до і після запуску ті самі реліз і тег `v0.1.0`, файл `AudioCaptor.exe` 4 938 752 байти, оновлений 2026-04-20.
- Гілку завершено, `develop` відправлено (66d57bd..6615172), гілку задачі на GitHub прибрано.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release тепер перевіряє все до збірки й має пробний режим, у якому нічого не публікується.

- Звіряє введену версію з `package.json`, `src-tauri/Cargo.toml` і `src-tauri/tauri.conf.json`, зупиняється, якщо тег `v<версія>` уже існує, і бере опис релізу з розділу `## [<версія>]` у `CHANGELOG.md`.
- Проганяє ті самі перевірки, що `just check`, з Rust із `rust-toolchain.toml` і діями на Node 24.
- `dry_run` увімкнено за замовчуванням. Публікує окреме завдання `publish` — єдине з правом запису; у пробному запуску воно пропускається. Exe і опис зберігаються як артефакт запуску на 7 днів.
- Назва файлу `AudioCaptor.exe` і формат тегу `v<версія>` не змінилися; тег ставиться на зібраний коміт.
- `DEVELOPMENT.md` описує кроки Release, пробний запуск, захист від повторної версії й те, чому справжній реліз не можна створювати для перевірки.

Перевірено пробним запуском 45178681416 на GitHub: зелений, публікацію пропущено, на GitHub нічого не змінилося.

Пункти Definition of Done про тексти інтерфейсу, NVDA і CHANGELOG не мали до чого застосовуватися: задача змінює лише Release.

Не перевірено: справжня публікація. Її вперше пройде TASK-10 (випуск 0.1.1).
<!-- SECTION:FINAL_SUMMARY:END -->

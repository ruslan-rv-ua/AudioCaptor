---
id: TASK-5
title: Release перевіряє код перед збіркою і має пробний запуск
status: To Do
assignee: []
created_date: '2026-11-23T11:29:02Z'
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
- [ ] #1 Перед збіркою Release виконує ті самі перевірки, що `just check`; якщо перевірка падає, ні тегу, ні релізу не створено
- [ ] #2 Release зупиняється до збірки, якщо введена версія не збігається з версією в `package.json`, `src-tauri/Cargo.toml` і `src-tauri/tauri.conf.json`
- [ ] #3 Пробний запуск (`dry_run`) виконує перевірки і збірку, але не створює ні тегу, ні релізу
- [ ] #4 Опис релізу на GitHub — це розділ відповідної версії з `CHANGELOG.md`
- [ ] #5 Назва файлу релізу `AudioCaptor.exe` і формат тегу `v<версія>` не змінилися
- [ ] #6 Release збирає версією Rust із `rust-toolchain.toml`, а в журналі запуску немає попередження про застарілий Node 20
- [ ] #7 Пробний запуск на гілці задачі проходить, і його номер записано в нотатках (push і запуск — з дозволу власника)
- [ ] #8 `DEVELOPMENT.md` описує пробний запуск і пояснює, чому справжній реліз не можна створювати для перевірки
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

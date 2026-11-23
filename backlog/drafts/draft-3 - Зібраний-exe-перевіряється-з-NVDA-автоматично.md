---
id: DRAFT-3
title: Зібраний exe перевіряється з NVDA автоматично
status: Draft
assignee: []
created_date: '2026-11-23T11:30:23Z'
labels:
  - a11y
  - testing
  - model-opus
  - effort-high
milestone: m-3
dependencies: []
priority: low
type: enhancement
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Доступність для користувачів скрінрідерів — найвища вимога проєкту, але перевіряється вона лише вручну. Після кожної зміни у фокусі, діалогах чи оголошеннях хтось має запустити NVDA і пройти сценарії сам. Пропущена перевірка означає, що незрячий користувач отримає зламану версію.

У quick-snippets таку перевірку робить `just smoke` (`scripts/smoke-nvda.ps1`): запускає зібраний exe, керує ним із клавіатури й звіряє, що сказав NVDA. Схожі скрипти є в revisor.

Що має вирішити власник:

- чи варта така перевірка зусиль для AudioCaptor;
- які сценарії вона має покривати (запуск, вибір пристроїв, старт, пауза й зупинка запису, діалоги профілю та налаштувань, повернення вікна з трея);
- як перевіряти запис: зі справжніми аудіопристроями чи без них.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [ ] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

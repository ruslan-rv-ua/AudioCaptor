---
id: TASK-8
title: DEVELOPMENT.md описує, як проєкт ведеться зараз
status: To Do
assignee: []
created_date: '2026-11-23T11:31:02Z'
labels:
  - docs
  - model-sonnet
  - effort-medium
milestone: m-0
dependencies:
  - TASK-3
  - TASK-4
  - TASK-5
references:
  - DEVELOPMENT.md
  - CLAUDE.md
priority: medium
type: docs
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`DEVELOPMENT.md` написано для версії 0.1.0 і частково виправлено в серпні 2026. Станом на 2026-11-23 він розходиться з тим, як проєкт ведеться:

- Документ англійський, але розділи «Release Process», «Оновлення Scoop-маніфесту» і «Required Secrets» написано українською (їх додав PR #3).
- Порядок випуску не згадує git flow. Насправді випуск — це `git flow release start`, версія в трьох файлах (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) і в `src-tauri/Cargo.lock`, розділ у `CHANGELOG.md`, `git flow release finish --notag`, push гілок `main` і `develop`, запуск Release на `main`, а потім Excavator у `ruslan-rv-ua/scoop-bucket`. Перевірка якості коду у квітні 2026 окремо зауважила, що версію легко забути в одному з файлів.
- Немає жодного слова про Backlog.md, правила роботи з `CLAUDE.md`, `just check` і закріплену версію Rust.
- У вимогах стоїть «Node.js 18+», тоді як CI користується Node 22.
- Розділ про значки каже, що вони генеруються «з одного файлу 1024×1024», а команда бере `src-tauri/icons/128x128.png`.

Розділи про CI і про пробний запуск Release оновлюють TASK-4 і TASK-5; ця задача зводить документ докупи й перевіряє його цілком.

До початку роботи власник вирішує, якою мовою буде документ: у quick-snippets `DEVELOPMENT.md` український, а README там і тут — двома мовами.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Документ написано однією мовою; якою саме — вирішив власник, і це записано в нотатках
- [ ] #2 Розділ про випуск описує всі кроки від `git flow release start` до оновлення Scoop, зокрема три файли з версією, `Cargo.lock` і прапорець `--notag`
- [ ] #3 Розділ про CI відповідає файлам `ci.yml` і `release.yml`
- [ ] #4 Документ каже, де шукати задачі, рішення й дослідження (Backlog.md), і називає `CLAUDE.md`, `just check` та `rust-toolchain.toml`
- [ ] #5 Вимоги до інструментів (Rust, Node, pnpm) збігаються з тим, що використовують CI і `rust-toolchain.toml`
- [ ] #6 Кожну команду з документа виконано, і вона працює; виняток — команди, що публікують реліз
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

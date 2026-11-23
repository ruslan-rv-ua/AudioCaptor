---
id: TASK-2
title: Гілки й теги відповідають git flow
status: To Do
assignee: []
created_date: '2026-11-23T11:28:27Z'
labels:
  - release
  - model-sonnet
  - effort-low
milestone: m-0
dependencies: []
references:
  - CLAUDE.md
priority: medium
type: chore
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Проєкт ведеться за git flow (git-flow-next; налаштування показує `git-flow overview`), але станом на 2026-11-23 репозиторій від нього відхилився:

- У `main` є два коміти, яких немає в `develop`: c5bf11c і його злиття c0456bc. PR #2 від 2026-08-26 пішов прямо в `main`. Вміст гілок через це не розійшовся (той самий файл `update-scoop.yml` видалено і в `develop`), але за git flow усе з `main` має бути в `develop`, інакше наступне завершення релізу принесе несподівані злиття.
- Локально є анотований тег `0.1.0` (на коміті 190f0e1). Його створила команда `git flow release finish 0.1.0` 2026-04-20, і він існує лише на цьому комп'ютері. На GitHub є тільки `v0.1.0` (на коміті c361ae9): його створив Release workflow, і саме з нього зібрано опублікований exe. Одна версія має два теги на різних комітах.
- Налаштування git flow створюють тег без префікса `v` під час `release finish` і `hotfix finish`: `gitflow.branch.release.tag` і `gitflow.branch.hotfix.tag` дорівнюють `true`. Правило в `CLAUDE.md` інше: реліз завершується з `--notag`, а тег `v<версія>` створює Release workflow.
- У локальних налаштуваннях git лишилися записи давно видалених гілок: `gitflow.branch.feature/help.base` і `gitflow.branch.feature/fixes.base`.

Три серпневі PR прийшли з гілок `chore/...` і `docs/...`, тобто поза префіксами git flow. Відтепер порядок роботи з гілками записано в `CLAUDE.md`.

Задача не змінює файлів проєкту: лише історію гілок, теги й локальні налаштування git.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Команда `git log origin/develop..origin/main` нічого не показує: усе з `main` є в `develop` (push — з дозволу власника)
- [ ] #2 Тег `0.1.0` видалено локально; `git tag -l` показує лише `v0.1.0`, як і GitHub
- [ ] #3 Налаштування `gitflow.branch.release.tag` і `gitflow.branch.hotfix.tag` мають значення `false`: завершення релізу чи hotfix не створює тегу без явної вказівки
- [ ] #4 У локальних налаштуваннях git немає записів `gitflow.branch.feature/help.base` і `gitflow.branch.feature/fixes.base`
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

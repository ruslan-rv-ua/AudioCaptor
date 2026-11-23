---
id: TASK-2
title: Гілки й теги відповідають git flow
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:28:27Z'
updated_date: '2026-11-23T12:18:54Z'
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
- [x] #1 Команда `git log origin/develop..origin/main` нічого не показує: усе з `main` є в `develop` (push — з дозволу власника)
- [x] #2 Тег `0.1.0` видалено локально; `git tag -l` показує лише `v0.1.0`, як і GitHub
- [x] #3 Налаштування `gitflow.branch.release.tag` і `gitflow.branch.hotfix.tag` мають значення `false`: завершення релізу чи hotfix не створює тегу без явної вказівки
- [x] #4 У локальних налаштуваннях git немає записів `gitflow.branch.feature/help.base` і `gitflow.branch.feature/fixes.base`
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

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Перевірити початковий стан: гілка `develop`, чисте робоче дерево, `main` збігається з `origin/main`.
2. Злити `main` у `develop` командою `git flow update` (за налаштуваннями git flow `develop` оновлюється від `main` злиттям) і переконатися, що файли проєкту не змінилися.
3. Видалити локальний тег `0.1.0`; хеші об'єкта тегу й коміту записати в нотатках, щоб тег можна було відновити.
4. Вимкнути створення тегів під час завершення: `git-flow config edit topic release --tag=false` і те саме для `hotfix`; перевірити через `git-flow overview`.
5. Прибрати з локальних налаштувань git розділи `gitflow.branch.feature/help` і `gitflow.branch.feature/fixes`.
6. Прогнати перевірки, відправити `develop` на GitHub (власник дозволив 2026-11-23) і перевірити `git log origin/develop..origin/main`.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-23: виконує головна сесія без окремої робочої сесії й без гілки — задача не змінює файлів проєкту, а власник дозволив виконати її тут разом із відправкою `develop` на GitHub. Початковий стан: `develop` і `origin/develop` на bcc3ceb, `main` і `origin/main` на c0456bc, робоче дерево чисте. Спільного файлу `.gitflow` у репозиторії немає, тож налаштування git flow живуть лише в локальному `.git/config`.

Виконано 2026-11-23:

- `git flow update` на `develop` створив коміт злиття ad6d603 (батьки bcc3ceb і c0456bc). Різниці у файлах між bcc3ceb і ad6d603 немає.
- `git tag -d 0.1.0`: видалено анотований тег (об'єкт c8411a08441dc79321b928382caebf6c32b84411, коміт 190f0e1fe5dbcebac8ea990767e3befe27e69eb0). `git tag -l` і `git ls-remote --tags origin` показують лише `v0.1.0`.
- `git-flow config edit topic release --tag=false` і те саме для `hotfix`. Команда заодно записала явні значення за замовчуванням, яких раніше не було: `tag false` для `bugfix`, `feature`, `support`, `develop` і `main` та `worktree false` для всіх типів тематичних гілок. Інших відмінностей у розділі `gitflow` порівняно зі станом до задачі немає.
- `git config --local --remove-section` для `gitflow.branch.feature/help` і `gitflow.branch.feature/fixes`.
- Перевірки після злиття: `pnpm vite:build` і `pnpm check` — 0 помилок; clippy з `--all-targets -- -D warnings` без зауважень; `cargo test` — 66 з 66.
- `git push origin develop` (bcc3ceb..ad6d603). Після `git fetch` команда `git log origin/develop..origin/main` нічого не показує.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Репозиторій знову відповідає git flow.

- `main` злито в `develop` командою `git flow update` (коміт злиття ad6d603). Файлів проєкту злиття не змінило: обидві гілки вже не мали `update-scoop.yml`. `develop` відправлено на GitHub, і `git log origin/develop..origin/main` нічого не показує.
- Локальний тег `0.1.0` видалено; лишився тільки `v0.1.0`, як і на GitHub. Доки git не прибрав об'єкт тегу, його можна відновити: `git update-ref refs/tags/0.1.0 c8411a08441dc79321b928382caebf6c32b84411`.
- `git flow release finish` і `git flow hotfix finish` більше не створюють тегу без явної вказівки: `gitflow.branch.release.tag` і `gitflow.branch.hotfix.tag` дорівнюють `false`, а `git-flow overview` більше не пише «Creates tags on finish».
- Розділи `gitflow.branch.feature/help` і `gitflow.branch.feature/fixes` прибрано з локальних налаштувань git.

Перевірено після злиття: `pnpm vite:build` і `pnpm check` — 0 помилок; clippy з `--all-targets -- -D warnings` без зауважень; `cargo test` — 66 з 66.

Обмеження: налаштування git flow локальні для цієї копії репозиторію (спільного файлу `.gitflow` немає). У новому клоні чи на іншому комп'ютері завершення релізу знову створювало б тег, тому правило `--notag` у `CLAUDE.md` лишається чинним.

Пункти Definition of Done про тексти інтерфейсу, перевірку з NVDA, документацію й CHANGELOG не мали до чого застосовуватися: задача не змінює файлів проєкту.
<!-- SECTION:FINAL_SUMMARY:END -->

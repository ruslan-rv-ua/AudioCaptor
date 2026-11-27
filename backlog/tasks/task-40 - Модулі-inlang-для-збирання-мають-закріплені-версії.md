---
id: TASK-40
title: Модулі inlang для збирання мають закріплені версії
status: To Do
assignee: []
created_date: '2026-11-27T19:21:46Z'
labels:
  - security
  - frontend
  - model-sonnet
  - effort-low
milestone: m-8
dependencies: []
references:
  - project.inlang/settings.json
priority: medium
type: chore
ordinal: 14200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`project.inlang/settings.json` підключає два модулі inlang прямо з CDN з `@latest`: `@inlang/plugin-message-format` і `@inlang/message-lint-rule-missing-translation`. Paraglide завантажує й виконує їх у Node під час кожної збірки інтерфейсу (`just check`, `just build`, CI, Release), тож нова версія цих пакетів, зламана чи шкідлива, потрапить у збірку й у exe без жодної зміни в репозиторії.

Повністю це питання закриває перехід на Paraglide 3 з модулями з npm (DRAFT-7). Ця задача лише закріплює версії, які збирання використовує зараз, щоб прибрати ризик до того переходу. Власник погодив 27.11.2026.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Адреси обох модулів у `project.inlang/settings.json` мають точні версії замість `@latest`: ті, що збирання завантажує зараз
- [ ] #2 `just check` і `just build` проходять, а тексти інтерфейсу обома мовами не змінилися
- [ ] #3 `DEVELOPMENT.md` каже, як оновлювати ці версії
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

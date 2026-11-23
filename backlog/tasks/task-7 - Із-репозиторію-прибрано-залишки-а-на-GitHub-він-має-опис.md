---
id: TASK-7
title: Із репозиторію прибрано залишки, а на GitHub він має опис
status: To Do
assignee: []
created_date: '2026-11-23T11:31:00Z'
labels:
  - docs
  - model-sonnet
  - effort-low
milestone: m-0
dependencies:
  - TASK-1
references:
  - .gitignore
  - .github/copilot-instructions.md
priority: low
type: chore
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Після паузи в репозиторії лишилося кілька речей, які вже нічому не служать або заважають (стан на 2026-11-23):

- Три згенеровані файли відстежуються git: `src/paraglide/messages/_index.js`, `btn_minimize_to_tray_aria.js` і `settings_minimize_on_focus_loss.js`. Теку `src/paraglide/` записано в `.gitignore`, і `pnpm vite:build` щоразу створює її наново. Через це зміна відповідних текстів показувалася б як зміна відстежуваного файлу.
- `.github/copilot-instructions.md` містить одне правило для GitHub Copilot у VS Code (завершувати кожен хід викликом `vscode_askQuestions`). Проєкт тепер ведеться з Claude Code за правилами з `CLAUDE.md`.
- Локальна тека `.kilo/` лишилася від агента Kilo Code: у ній тільки `node_modules` і порожня тека `plans`. У репозиторій вона не потрапляє завдяки рядку `.kilo/` в `.gitignore`.
- Репозиторій на GitHub не має ні опису, ні тем: у пошуку й на сторінці профілю він показується без пояснення, що це за програма.

Про другий, третій і четвертий пункти рішення ухвалює власник: видаляти чи лишати, і яким має бути опис.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Команда `git ls-files src/paraglide` нічого не показує, а `pnpm vite:build` і `pnpm check` проходять на свіжому клоні репозиторію
- [ ] #2 Файл `.github/copilot-instructions.md` видалено або залишено — за рішенням власника, записаним у нотатках
- [ ] #3 Теку `.kilo/` і її рядок у `.gitignore` прибрано або залишено — за рішенням власника, записаним у нотатках
- [ ] #4 Репозиторій на GitHub має опис і теми; текст погоджено з власником перед зміною
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

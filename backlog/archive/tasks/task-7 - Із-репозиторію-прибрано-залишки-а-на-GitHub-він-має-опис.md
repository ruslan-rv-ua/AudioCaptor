---
id: TASK-7
title: Із репозиторію прибрано залишки, а на GitHub він має опис
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:31:00Z'
updated_date: '2026-11-23T12:45:03Z'
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
- [x] #1 Команда `git ls-files src/paraglide` нічого не показує, а `pnpm vite:build` і `pnpm check` проходять на свіжому клоні репозиторію
- [x] #2 Файл `.github/copilot-instructions.md` видалено або залишено — за рішенням власника, записаним у нотатках
- [x] #3 Теку `.kilo/` і її рядок у `.gitignore` прибрано або залишено — за рішенням власника, записаним у нотатках
- [x] #4 Репозиторій на GitHub має опис і теми; текст погоджено з власником перед зміною
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
Виконала робоча сесія «TASK-7 · Sonnet · low» (коміти 6ff0020 і 7f82868 на гілці `feature/task-7-leftovers`), 2026-11-23.

Рішення власника (діалог із варіантами в робочій сесії; вибрані варіанти дослівно за її звітом):
- `.github/copilot-instructions.md`: «Видалити (Recommended)»
- `.kilo/` і її рядок у `.gitignore`: «Видалити обидва (Recommended)»
- опис і теми репозиторію на GitHub: «Варіант A (Recommended)». Опис: «Portable audio recorder for Windows: microphone, system audio (loopback) or both. Keyboard and screen-reader friendly.» Теми: audio-recorder, windows, wasapi, loopback, portable, accessibility, screen-reader, tauri, rust, svelte.

Перевірка головної сесії:
- `git ls-files src/paraglide` нічого не показує; самі файли на диску лишилися.
- Свіжий клон гілки в тимчасовій теці: `src/paraglide` у ньому до збірки немає; `pnpm install --frozen-lockfile`, `pnpm vite:build` і `pnpm check` (0 помилок, 0 попереджень) проходять; після збірки змін у відстежуваних файлах немає.
- `.github/copilot-instructions.md` видалено; теки `.kilo/` на диску немає, а з `.gitignore` прибрано її рядок разом із заголовком розділу.
- `gh repo view` показує погоджений опис і всі 10 тем.
- `just check` на гілці проходить (66 тестів). Гілку завершено (`git flow feature finish --no-fetch --no-push`).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Із репозиторію прибрано залишки, а на GitHub він має опис і теми.

- 6ff0020: три згенеровані файли в `src/paraglide/` git більше не відстежує; тека є в `.gitignore` і створюється наново під час кожної збірки.
- 7f82868: за рішенням власника видалено `.github/copilot-instructions.md` (правило для GitHub Copilot у VS Code) і рядок `.kilo/` з `.gitignore`; саму локальну теку `.kilo/` видалено з диска.
- На GitHub (з погодження власника): опис «Portable audio recorder for Windows: microphone, system audio (loopback) or both. Keyboard and screen-reader friendly.» і 10 тем.

Перевірено: свіжий клон збирається, `pnpm check` без помилок; `just check` проходить; `gh repo view` показує опис і теми.

Пункти Definition of Done про тексти інтерфейсу, перевірку з NVDA, документацію й CHANGELOG не мали до чого застосовуватися: жоден документ не згадував прибрані файли, а користувачі зміни не помітять.
<!-- SECTION:FINAL_SUMMARY:END -->

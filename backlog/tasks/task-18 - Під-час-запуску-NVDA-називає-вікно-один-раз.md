---
id: TASK-18
title: Під час запуску NVDA називає вікно один раз
status: To Do
assignee: []
created_date: '2026-11-23T18:19:57Z'
updated_date: '2026-11-23T22:05:04Z'
labels:
  - frontend
  - backend
  - a11y
  - model-opus
  - effort-high
milestone: m-1
dependencies: []
references:
  - doc-3
priority: medium
type: bug
ordinal: 9400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Після запуску програми NVDA 6–7 разів поспіль каже «AudioCaptor», перш ніж назве елемент із фокусом. Це заважає й затримує роботу. Схожу ваду з першим показом вікна на WebView2 досліджували в quick-snippets (TASK-29 там): вікно показувалося, коли процес браузера ще запускався, і NVDA читав події не так. Тут спершу треба з'ясувати причину.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Причину повторів з'ясовано й записано в нотатках задачі
- [ ] #2 Після запуску NVDA каже назву вікна один раз, а потім елемент із фокусом
- [ ] #3 Сценарій `launch` у `just smoke` перевіряє кількість повторів назви й проходить
- [ ] #4 У `CHANGELOG.md` є запис про виправлення
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-24, гіпотеза з doc-3 (розділи 2.3, 5 і 6.2), не перевірена: назва «AudioCaptor» стоїть на чотирьох рівнях — заголовок вікна, `<title>` документа, `<body role="application" aria-label="AudioCaptor">` (`index.html`) і `<main aria-label="AudioCaptor">` (`App.svelte`), а ще `<h1>`. Коли фокус уперше заходить у вікно, NVDA каже назви предків, у які він зайшов (`event_focusEntered`): вікно, документ, «AudioCaptor додаток», орієнтир `main`. Додаткові повтори може давати документ режиму огляду кореня, який NVDA будує до входу в `application` (розділ 5, пункт 8), і повторні події фокуса під час показу вікна. Перше, що варто перевірити: прибрати `aria-label` з `main` (чи весь `main`-орієнтир) і подивитися в журналі NVDA, скільки повторів лишиться.
<!-- SECTION:NOTES:END -->

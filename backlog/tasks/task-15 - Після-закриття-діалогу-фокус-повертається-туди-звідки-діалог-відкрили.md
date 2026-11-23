---
id: TASK-15
title: Після закриття діалогу фокус повертається туди, звідки діалог відкрили
status: To Do
assignee: []
created_date: '2026-11-23T18:19:56Z'
updated_date: '2026-11-23T22:05:04Z'
labels:
  - frontend
  - a11y
  - model-sonnet
  - effort-medium
milestone: m-1
dependencies: []
references:
  - doc-3
priority: high
type: bug
ordinal: 9100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Коли діалог закривається клавішею Escape, фокус стає не на елемент, з якого діалог відкрили. Після «About» (F1) він завжди опиняється на кнопці «About AudioCaptor», після налаштувань, профілю й підтвердження видалення — на списку профілів. Користувач скрінрідера втрачає місце, де працював, і мусить шукати його знову. Сценарій dialogs через це падає на чотирьох кроках.

Компоненти діалогів: src/lib/components/AboutDialog.svelte, SettingsDialog.svelte, ProfileDialog.svelte, ConfirmDeleteDialog.svelte; відкриває їх src/App.svelte.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Після закриття діалогів «About», налаштувань, профілю й підтвердження видалення клавішею Escape фокус стоїть на елементі, з якого діалог відкрили
- [ ] #2 Так само фокус повертається, коли діалог закрито кнопкою (Close, Cancel, Save, Delete); якщо елемента вже немає (видалено профіль), фокус стає на список профілів
- [ ] #3 `just smoke dialogs` проходить
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
2026-11-24, з doc-3 (дослідження режимів NVDA, розділи 2.3 і 5): шаблон APG вимагає повертати фокус туди, звідки діалог відкрили. Якщо елемент зник чи фокус не поставлено, фокус падає на `body`, і NVDA про документ нічого не каже. Фокус ставити один раз і одразу на кінцевий елемент, без проміжних обгорток (гонки подій фокуса, nvaccess/nvda#2039). Зараз `App.svelte` після «About» завжди фокусує `.btn-about`, а після інших діалогів — `#profile-select` (`focusProfileSelect`); F1 і меню трея відкривають «About» не з кнопки, тож треба запам'ятовувати `document.activeElement` перед відкриттям.
<!-- SECTION:NOTES:END -->

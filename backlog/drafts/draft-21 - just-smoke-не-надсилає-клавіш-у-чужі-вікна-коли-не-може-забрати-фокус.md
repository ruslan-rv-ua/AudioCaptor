---
id: DRAFT-21
title: just smoke не надсилає клавіш у чужі вікна, коли не може забрати фокус
status: Draft
assignee: []
created_date: '2026-11-27T14:51:52Z'
labels:
  - testing
  - a11y
  - model-opus
  - effort-medium
milestone: m-3
dependencies: []
references:
  - scripts/smoke-nvda.ps1
priority: medium
type: bug
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Уночі 27.11.2026 робоча сесія TASK-30 (випуск 0.3.0) чотири рази запускала `just smoke`, поки за ПК нікого не було, і щоразу тест зупинявся до першого кроку: «STOPPED: NVDA logs key presses but said nothing for the test window». У журналі NVDA — «Foreground took too long to change. Foreground still … (TTOTAL_CMD)»: вікно тесту просило фокус, але передній план лишався в Total Commander, і клавіша End від тесту пішла туди (курсор у списку файлів перемістився). `ForegroundLockTimeout` дорівнював 2147483647, екран не був заблокований. Коли власник сів за ПК, той самий тест пройшов повністю (61 крок з 61).

Отже, без людини за ПК перевірка з NVDA не працює, і автономний випуск зупиняється на ній. Небезпечніше інше: тест шле клавіші туди, де фокус, а в чужому вікні Delete чи Alt+F4 могли б щось зіпсувати.

Потрібне рішення власника, наприклад:
- перед кожною клавішею перевіряти, що на передньому плані вікно тесту, а інакше зупинятися, нічого не натискаючи (варто зробити за будь-якого вибору);
- навчити `scripts/smoke-nvda.ps1` забирати фокус без людини (наприклад, натиснути й відпустити Alt через SendInput перед SetForegroundWindow);
- або лишити як є: випуск із перевіркою NVDA лише тоді, коли власник за ПК.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

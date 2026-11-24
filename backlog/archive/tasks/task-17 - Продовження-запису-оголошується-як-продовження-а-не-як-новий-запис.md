---
id: TASK-17
title: Продовження запису оголошується як продовження, а не як новий запис
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T18:19:57Z'
updated_date: '2026-11-24T18:03:36Z'
labels:
  - frontend
  - a11y
  - model-sonnet
  - effort-low
milestone: m-1
dependencies: []
priority: medium
type: bug
ordinal: 9300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Після паузи клавіша Alt+R відновлює запис, але NVDA оголошує «Recording started» — той самий текст, що й на початку запису. Користувач не може відрізнити продовження від нового запису. Оголошення формує src/App.svelte (live_recording_started для стану Recording) без огляду на попередній стан.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Після паузи Alt+R, кнопка продовження, гаряча клавіша й меню трея дають оголошення «Recording resumed» (українською — «Запис продовжено»), а старт запису й надалі звучить як «Recording started»
- [x] #2 Новий текст є в `messages/en.json` і `messages/uk.json`
- [x] #3 Сценарій `record` у `just smoke` перевіряє окреме оголошення продовження й проходить
- [x] #4 У `CHANGELOG.md` є запис про виправлення
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Усі локальні перевірки проходять: `just check`
- [x] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. src/App.svelte: ефект області оголошень пам'ятає попередній стан запису; перехід Paused → Recording оголошує новий текст live_recording_resumed, решта переходів — як раніше. Alt+R, кнопка Resume, гаряча клавіша й меню трея міняють той самий recording.state, тож виправлення одне на всі.
2. messages/en.json і messages/uk.json: ключ live_recording_resumed — «Recording resumed» / «Запис продовжено». Підписи трея (tray.rs) не змінюються.
3. scripts/smoke-nvda.ps1: рядок Resumed у $UI; крок Alt+R сценарію record чекає «Recording resumed» і не допускає «Recording started».
4. CHANGELOG.md: рядок у [Unreleased] / Fixed; DEVELOPMENT.md — опис сценарію record.
5. just check, just build, just smoke record, повний just smoke — кожен прогін із дозволу власника.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Зроблено на гілці bugfix/task-17-resume-announcement (локально; не відправлено, не завершено):
- 3dc78e8 fix(a11y): announce a resumed recording as resumed, not started — src/App.svelte, messages/en.json, messages/uk.json, CHANGELOG.md.
- bdd7ff7 test(smoke): expect "Recording resumed" after Alt+R — scripts/smoke-nvda.ps1, DEVELOPMENT.md (опис сценарію record).

Як працює: ефект області оголошень у src/App.svelte пам'ятає стан, який оголосив останнім (звичайна змінна announcedState, не реактивна). Recording одразу після Paused оголошується як live_recording_resumed («Recording resumed» / «Запис продовжено»), решта переходів — як раніше. Alt+R, кнопка Resume, глобальна гаряча клавіша й меню трея змінюють той самий recording.state (останні два — через подію recording-state-changed від бекенду), тож одна зміна покриває всі чотири. Підписи трея не змінювалися.

Перевірки (кожен прогін із клавіатурою — з дозволу власника):
- just check: проходить (svelte-check без помилок і попереджень, clippy, 66 тестів, cargo fmt).
- just build: exe від 2026-11-24 18:39.
- just smoke record (19:01): 11 з 11, перевірок NVDA 25 з 25; крок 7: «Recording resumed | Pause recording (Alt+P), кнопка», без «Recording started».
- Повний just smoke: 19:17, 19:25 і 19:45 (після перезапуску NVDA) — щоразу 27 з 28. Крок 7 (продовження) пройшов у всіх трьох. Не пройшов лише крок 3 сценарію record (Alt+S): NVDA сказав «недоступно» і назву сторінки. На exe з develop без змін задачі (19:36) — те саме, тож це не від задачі: NVDA зависає на 0,6–0,7 с, коли тест закриває копію програми між сценаріями launch і record. Записано як DRAFT-10.
- Додаткова перевірка (одноразовий скрипт task-17-resume-check.ps1 у спільних нотатках сесій, 19:53): 33 з 33. «Recording resumed» після Alt+R (з фокусом на Resume і на Stop, двічі за один запис), після Space на кнопці Resume і після глобальної клавіші Pause (двічі); новий запис після зупинки з паузи — «Recording started»; український інтерфейс — «Запис розпочато», «Запис призупинено», «Запис продовжено», «Запис зупинено».

Не перевірено напряму: меню трея (та сама функція do_resume_recording і та сама подія, що в гарячої клавіші) і миша.

- 38cd08f docs(changelog): say the resume announcement is the same for every way to resume — рядок CHANGELOG більше не обіцяє, що NVDA прочитає оголошення після меню трея (вікно тоді зазвичай неактивне, а меню трея напряму не перевірялося).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Продовження запису після паузи тепер оголошується як «Recording resumed» / «Запис продовжено», а не як новий запис.

Що змінилося:
- src/App.svelte: ефект області оголошень пам'ятає останній оголошений стан; Recording одразу після Paused оголошується як live_recording_resumed, решта переходів — як раніше. Alt+R, кнопка Resume, глобальна гаряча клавіша й меню трея міняють той самий стан запису, тож виправлення одне на всі.
- messages/en.json, messages/uk.json: новий текст live_recording_resumed. Підписи трея не змінювалися.
- scripts/smoke-nvda.ps1: крок Alt+R сценарію record чекає «Recording resumed» і не допускає «Recording started». DEVELOPMENT.md і CHANGELOG.md оновлено.

Перевірки:
- just check проходить (у робочій сесії й повторно в головній після огляду).
- just smoke record: 11 з 11. Додаткова перевірка з NVDA: 33 з 33 (Alt+R, пробіл на Resume, глобальна клавіша, два продовження в одному записі, новий запис після зупинки з паузи, український інтерфейс).
- Меню трея напряму не перевірялося: воно викликає ту саму do_resume_recording, що й перевірена гаряча клавіша.
- Повний just smoke: 27 з 28. Не проходить крок 3 сценарію record (Alt+S) через зависання NVDA між сценаріями launch і record, так само на develop без цієї задачі. Записано як DRAFT-10, рішення за власником.

Гілку bugfix/task-17-resume-announcement завершено в develop.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-20
title: Журнал програми пишеться лише в AudioCaptor-data
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T18:19:58Z'
updated_date: '2026-11-24T22:06:28Z'
labels:
  - backend
  - model-sonnet
  - effort-medium
milestone: m-1
dependencies: []
priority: medium
type: bug
ordinal: 9600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14, `scripts/smoke-nvda.ps1`) 2026-11-23 під час двох повних прогонів з англійським інтерфейсом.

Крім `AudioCaptor-data/logs/`, програма пише другий журнал у `%LOCALAPPDATA%\com.audiocaptor.app\logs`: модуль `tauri-plugin-log` лишає свої типові місця запису поруч із тим, що задано в `src-tauri/src/lib.rs`. Цей журнал спільний для всіх копій програми на комп'ютері. 96% його рядків — службові записи аудіобібліотеки рівня TRACE. Коли файл доходить до 1 МБ, модуль його видаляє. Кожен прогін тесту додає туди близько 44 КБ, і тест цього відвернути не може.

Це суперечить рішенню decision-1: усі дані портативної програми лежать в AudioCaptor-data поруч з exe.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Після запуску, запису й виходу з програми в `%LOCALAPPDATA%\com.audiocaptor.app\logs` нічого не з'являється і не змінюється
- [ ] #2 Журнал у `AudioCaptor-data/logs/` не містить рядків рівня TRACE і DEBUG від сторонніх бібліотек
- [ ] #3 Тест `just smoke` більше не показує примітку про записи програми поза її текою
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
1. У `run()` (`src-tauri/src/lib.rs`) замінити типові місця запису модуля журналу, а не додавати до них: `targets([...])` замість `target(...)`. Лишаються консоль (видно лише в `just dev`) і тека `AudioCaptor-data/logs`; місце `LogDir` (`%LOCALAPPDATA%\com.audiocaptor.app\logs`) зникає.
2. Задати рівень `Info`. Власні рядки програми мають рівні info, warn і error (рядків debug і trace у коді немає), а шум — це TRACE і DEBUG від `wasapi` (909 з 939 рядків журналу тестової копії після TASK-18).
3. Оновити DEVELOPMENT.md (таблиця бібліотек, «Портативний режим», розділ про `just smoke`), README, якщо треба, і додати рядок у CHANGELOG.md.
4. `just check`, `just build`.
5. З дозволу власника: `just smoke record`, потім повний `just smoke`. Звірити, що в `%LOCALAPPDATA%\com.audiocaptor.app\logs` нічого не змінилося і звіт не має примітки про записи поза текою.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Робоча сесія 2026-11-24, гілка `bugfix/task-20-logs-in-data-folder` (локальна, не завершена, не відправлена).

Причина (перевірено в коді `tauri-plugin-log` 2.10.2 з `Cargo.lock`):
- `Builder::new()` має два типові місця запису: `Stdout` і `LogDir` (`%LOCALAPPDATA%\com.audiocaptor.app\logs`). `target(...)` додає до них нове місце (`targets.push`), а не замінює їх. Замінює `targets([...])` (або `clear_targets()`).
- Рівень не було задано, а без нього модуль пише все аж до Trace.
- Журнал тестової копії з повного прогону TASK-18 (`%TEMP%\audiocaptor-smoke-app.log`): 939 рядків. 909 — TRACE від `wasapi` (882 з них «read N frames», близько сотні за секунду запису), 30 — INFO самої програми. Інших бібліотек там немає. У коді `wasapi` 0.17.0: 19 викликів trace, 10 debug, 1 warn, 0 info. Під час запису журнал доходив до 1 МБ за кілька хвилин, і модуль видаляв його разом із рядками програми.

Виправлення (коміт 3a758c4, `run()` у `src-tauri/src/lib.rs`):
- `targets([Stdout, Folder AudioCaptor-data/logs])` замість `target(...)`. Консоль лишено: її видно лише в `just dev`, бо release-збірка не має консолі.
- `.level(log::LevelFilter::Info)`. Програма пише лише info, warn і error (`debug!` і `trace!` у коді немає), тож усі її рядки лишаються; єдиний warn у `wasapi` теж.
- CHANGELOG, [Unreleased] / Fixed: один пункт. Власник вибрав додати речення, що стару теку `logs` у `%LOCALAPPDATA%\com.audiocaptor.app` можна видалити. Програма сама нічого там не видаляє.
- DEVELOPMENT.md: таблиця бібліотек, «Портативний режим» (куди пише журнал, чому `targets`, рівень Info, 1 МБ; поза текою лишаються тільки дані WebView2, DRAFT-6), розділ про `just smoke`. README не змінено: вони вже кажуть, що всі дані, разом з `logs/`, лежать в `AudioCaptor-data/`.

Тест (коміт efdd47a; власник вибрав зробити це в цій задачі): будь-яка зміна в `%LOCALAPPDATA%\com.audiocaptor.app` поза `EBWebView` тепер дає у звіті FAILED (раніше — примітку), і тест не проходить (код виходу 1), так само як зміна `EBWebView`. Скрипт лишився з самих символів ASCII; синтаксис перевірено парсером Windows PowerShell 5.1 і PowerShell 7.

Перевірки:
- `just check` двічі (після виправлення і на останньому коміті): svelte-check 0 помилок і 0 попереджень, clippy, 66 тестів, fmt — усе пройшло.
- `just build`: exe 2026-11-24 23:50 з коду коміту 3a758c4 (`lib.rs` після нього не змінювався).
- `just smoke record` і повний `just smoke` НЕ запускалися: на питання перед `record` власник відповів «зараз пропусти». Тому AC #1–#3 на запущеній програмі ще не перевірено.
- Знімок перед прогоном (лише читання): у `%LOCALAPPDATA%\com.audiocaptor.app\logs` один файл `AudioCaptor.log`, 957 386 байт, змінений 2026-11-24 23:18:09 (кінець повного прогону TASK-18), SHA256 AC6F12CD538BAE2142E7FE8406BAB5929352BBCF32EF60B75BBF9A2F71D00DDB.

Знайдене мимохідь: DRAFT-12 — час у журналі пишеться в UTC, а не за годинником Windows.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Журнал програми тепер пишеться лише в `AudioCaptor-data/logs/` і без службових рядків сторонніх бібліотек.

Причина: у `run()` (`src-tauri/src/lib.rs`) виклик `tauri_plugin_log::Builder::new().target(...)` додавав нашу теку до типових місць запису модуля (`Stdout` і `LogDir`, тобто `%LOCALAPPDATA%\com.audiocaptor.app\logs`), а рівень не був заданий (типово Trace). Близько сотні рядків TRACE за секунду запису від бібліотеки wasapi за кілька хвилин доводили файл до 1 МБ, і модуль видаляв його разом із рядками програми.

Що змінилося:
- `src-tauri/src/lib.rs`: `targets([Stdout, Folder])` замінює типовий список; рівень Info (програма пише лише info, warn і error).
- `scripts/smoke-nvda.ps1`: будь-яка зміна в `%LOCALAPPDATA%\com.audiocaptor.app`, не лише в `EBWebView`, тепер означає, що тест не пройшов; примітки більше немає.
- `CHANGELOG.md` (Fixed; користувачам сказано, що стару теку `logs` можна видалити — так вирішив власник), `DEVELOPMENT.md` (таблиця бібліотек, «Портативний режим», розділ про `just smoke`).

Перевірки: `just check` проходить (у робочій сесії двічі й повторно в головній), `just build` проходить.

Не перевірено: тести з NVDA (`just smoke record` і повний `just smoke`) власник вирішив цього разу пропустити. Тож критерії #1–#3 на запущеній програмі не підтверджено й не відмічено. Це покаже перший наступний повний `just smoke`: тепер він не проходить, якщо програма пише в `%LOCALAPPDATA%\com.audiocaptor.app`, а рядки TRACE і DEBUG видно в `%TEMP%\audiocaptor-smoke-app.log`. DoD #3 задачі не стосується: інтерфейс не змінювався.

Побічне: DRAFT-12 (час у журналі записано за UTC).

Гілку bugfix/task-20-logs-in-data-folder завершено в develop.
<!-- SECTION:FINAL_SUMMARY:END -->

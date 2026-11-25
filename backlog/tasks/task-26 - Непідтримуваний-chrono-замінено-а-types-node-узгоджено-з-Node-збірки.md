---
id: TASK-26
title: Непідтримуваний chrono замінено, а @types/node узгоджено з Node збірки
status: In Progress
assignee:
  - '@claude'
created_date: '2026-11-25T01:05:47Z'
updated_date: '2026-11-25T08:04:50Z'
labels:
  - backend
  - frontend
  - model-sonnet
  - effort-low
milestone: m-2
dependencies: []
priority: medium
type: chore
ordinal: 10900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Рішення власника 2026-11-25 (розбір DRAFT-2): ці дві дрібні зміни залежностей зробити до випуску 0.2.0, а великі мажорні переходи (звукові бібліотеки, TypeScript 7, Paraglide 3) — після нього.

Бібліотеку `chrono` позначено як непідтримувану (RUSTSEC-2026-0543). Програма бере з неї лише місцеві дату й час для назви файлу запису (`src-tauri/src/lib.rs`), і більше ніщо не збирає її в програму, тож після заміни вона в програму не потрапить. Її запис у `Cargo.lock` лишається: його згадує `serde_with`, яку бере tauri-utils (див. примітки).

`@types/node` досі версії 20. Локально програму збирають на Node 26, а CI і Release ставлять Node 22 (`actions/setup-node`, `node-version: 22`). Узгодити `@types/node` з Node 22 чи перевести CI і Release на Node 26 — вирішує власник; другий варіант змінює CI, тож тоді потрібен ручний запуск CI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Місцеві дату й час для назви файлу запису програма бере без `chrono`, а формат назви не змінився (`%Y-%m-%d_%H-%M-%S`); його перевіряє тест
- [ ] #2 `chrono` не збирається в програму: його немає в `src-tauri/Cargo.toml`, а `cargo tree -i chrono --target all -e all` нічого не знаходить
- [ ] #3 Мажорна версія `@types/node` у `package.json` збігається з версією Node, на якій CI і Release збирають програму
- [ ] #4 Запис на зібраному exe створює файл із назвою того самого формату, що й до зміни
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Усі локальні перевірки проходять: `just check`
- [ ] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Гілка feature/task-26-deps-chrono-types-node.
2. chrono → GetLocalTime із крейта windows, який програма вже має (додати фічу Win32_System_SystemInformation): місцеві дату й час дає сама Windows, нового крейта не треба. Назву формує окрема чиста функція з SYSTEMTIME; unit-тест перевіряє формат %Y-%m-%d_%H-%M-%S із нулями попереду. chrono прибрати з src-tauri/Cargo.toml і Cargo.lock.
3. @types/node 20 → 22 (рішення власника 2026-11-25: як Node у CI і Release; CI не змінюється).
4. just check, just build.
5. AC #4: копія зібраного exe у тимчасовій теці з власною AudioCaptor-data, запис Alt+S / Alt+T, назва файлу в її Recordings (лише з дозволу власника).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25, робоча сесія. Гілка feature/task-26-deps-chrono-types-node (локальна, не завершена, не відправлена): e0aa0d0, cf0b97b, b4e71b9.

Node: власник обрав `@types/node` 22, як Node у CI і Release (`node-version: 22`); workflow не змінювалися.

Що змінено:
- `chrono` → `GetLocalTime` із крейта `windows` 0.59, який програма вже мала (додано лише фічу `Win32_System_SystemInformation`). Чому не інший крейт: нового крейта не треба, а місцевий час дає сама Windows. `local_timestamp()` у `src-tauri/src/lib.rs` бере час, чиста `format_timestamp(&SYSTEMTIME)` пише `YYYY-MM-DD_hh-mm-ss` (як `%Y-%m-%d_%H-%M-%S`, без мілісекунд). Тест `tests::timestamp_keeps_the_file_name_format` перевіряє нулі попереду (`2026-01-05_07-08-09`) і найбільші значення (`2026-12-31_23-59-59`).
- `chrono` прибрано з `src-tauri/Cargo.toml` і зі списку залежностей `audio-captor` у Cargo.lock; `cargo tree -i chrono --target all -e all` нічого не знаходить, тож у програму він більше не збирається.
- Але запис `chrono` 0.4.45 у `src-tauri/Cargo.lock` лишився: його згадує `serde_with` 3.25.0 (остання версія), яку бере tauri-utils 2.10.4. Типова можливість `std` у `serde_with` називає `chrono` слабкою необов'язковою залежністю (`chrono_0_4?/clock`), а такі залежності Cargo записує в Cargo.lock, навіть коли нічого їх не вмикає (так само там лежить незібраний `jiff` 1.0.2). Без змін у Tauri чи `serde_with` цей запис не прибрати, тож AC #2 виконано лише щодо Cargo.toml. `cargo audit` далі показує RUSTSEC-2026-0543; подробиці й решта таких попереджень — у примітках DRAFT-5.
- `@types/node` 20.19.43 → 22.20.5 (`^22.20.5`); у pnpm-lock.yaml змінилися лише `@types/node` і його згадки в записах vite.
- DEVELOPMENT.md: мажорна версія `@types/node` така сама, як Node у CI і Release; перехід на інший Node міняє обидва.
- CHANGELOG.md не змінено: користувачі нічого не помітять, назва файлу та сама.

Перевірки:
- `just check`: пройшов (svelte-check 0 помилок і 0 попереджень, clippy -D warnings, 67 тестів із новим, cargo fmt --check).
- `just build`: пройшов, exe зібрано 2026-11-25 03:33:01, після останнього коміту з кодом.
- AC #4, з дозволу власника: скрипт task-26-filename-check.ps1 (спільна тека нотаток сесій) запустив копію exe в %TEMP%\audiocaptor-task26 з власною AudioCaptor-data і записав 2 с системного звуку (Alt+S, Alt+T). Файл `loopback_2026-11-25_09-57-37.wav`: час у назві місцевий (Alt+S о 09:57:37.439; журнал копії, де час UTC, — 07:57:37). Стара збірка в TASK-22 назвала файл так само: `loopback_2026-11-25_00-30-07.wav` при 22:30:07 UTC у журналі. Теку копії видалено; %LOCALAPPDATA%\com.audiocaptor.app не змінилася.
<!-- SECTION:NOTES:END -->

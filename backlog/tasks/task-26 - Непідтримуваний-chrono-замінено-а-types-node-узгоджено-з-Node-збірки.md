---
id: TASK-26
title: Непідтримуваний chrono замінено, а @types/node узгоджено з Node збірки
status: To Do
assignee: []
created_date: '2026-11-25T01:05:47Z'
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

Бібліотеку `chrono` позначено як непідтримувану (RUSTSEC-2026-0543). Програма бере з неї лише місцеві дату й час для назви файлу запису (`src-tauri/src/lib.rs`), і більше ніщо в дереві залежностей її не тягне, тож після заміни вона зникне з `Cargo.lock`.

`@types/node` досі версії 20. Локально програму збирають на Node 26, а CI і Release ставлять Node 22 (`actions/setup-node`, `node-version: 22`). Узгодити `@types/node` з Node 22 чи перевести CI і Release на Node 26 — вирішує власник; другий варіант змінює CI, тож тоді потрібен ручний запуск CI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Місцеві дату й час для назви файлу запису програма бере без `chrono`, а формат назви не змінився (`%Y-%m-%d_%H-%M-%S`); його перевіряє тест
- [ ] #2 `chrono` немає ні в `src-tauri/Cargo.toml`, ні в `src-tauri/Cargo.lock`
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

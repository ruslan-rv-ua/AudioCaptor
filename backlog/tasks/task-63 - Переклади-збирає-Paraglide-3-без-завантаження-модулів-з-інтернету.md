---
id: TASK-63
title: Переклади збирає Paraglide 3 без завантаження модулів з інтернету
status: To Do
assignee: []
created_date: '2026-11-23T18:45:48Z'
updated_date: '2026-11-28T18:50:03Z'
labels:
  - security
  - frontend
  - model-sonnet
  - effort-medium
milestone: m-10
dependencies: []
references:
  - project.inlang/settings.json
  - vite.config.ts
priority: medium
type: chore
ordinal: 20250
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Paraglide (`@inlang/paraglide-js` 2.29.3) збирає тексти інтерфейсу з `messages/en.json` і `messages/uk.json`. Під час кожної збірки фронтенду (`pnpm vite:build`, а отже `just check`, `just build`, CI і Release) він завантажує й виконує два модулі inlang з CDN за адресами з `project.inlang/settings.json`. Після TASK-40 їхні версії закріплено, тож чужа нова версія вже не потрапить у збірку непомітно. Але збірка й далі залежить від CDN: коли jsdelivr недоступний, вона не пройде, а `pnpm audit` і `pnpm-lock.yaml` цих модулів не бачать. Paraglide 3 (зараз 3.2.2) — нова мажорна версія; власник ще 25.11.2026 вирішив переходити на неї разом із цією зміною (DRAFT-2).

Рішення власника (2026-11-28): задача в етапі 0.5.0. Користувачі не мають нічого помітити.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `@inlang/paraglide-js` — 3.x у `package.json` і `pnpm-lock.yaml`; `just check` проходить
- [ ] #2 `project.inlang/settings.json` не має адрес CDN: модулі inlang беруться з npm-пакетів, закріплених у `pnpm-lock.yaml`, і збірка фронтенду нічого не завантажує з інтернету
- [ ] #3 Усі тексти інтерфейсу англійською й українською показуються й озвучуються, як раніше, і мова перемикається в налаштуваннях, як раніше; повний `just smoke` проходить
- [ ] #4 Перевірка, що кожен текст має обидві мови, і далі діє (або її замінено рівноцінною)
- [ ] #5 `DEVELOPMENT.md` пояснює, звідки беруться модулі перекладів і як їх оновлювати
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
2026-11-24 (TASK-8): перший запуск сервера інтерфейсу (`pnpm vite dev --port 1420`) чекав на Paraglide близько 32 с («ready in 31800 ms», компіляція locale-modules); перший `just vite-dev` за 40 с так і не відповів. Наступний запуск пропустив компіляцію («inputs unchanged») і стартував за 0,5 с. `pnpm vite:build` у той самий час зібрався за 1,4 с. Чи затримку дає завантаження модулів із CDN, не перевірено.

2026-11-25: власник вирішив оновлювати Paraglide 2 → 3 разом із цією чернеткою (розбір DRAFT-2).

2026-11-27, план етапів: закріплення версій модулів винесено в окрему задачу TASK-40 (етап 0.4.1); тут лишається перехід на Paraglide 3 з модулями з npm.

2026-11-28, головна сесія: чернетку DRAFT-7 перетворено на задачу за рішенням власника, етап 0.5.0. Мітку effort-low замінено на effort-medium: тепер це перехід на нову мажорну версію Paraglide, а не лише закріплення версій. Задача змінює залежності, тож перед закриттям потрібен CI, запущений вручну.
<!-- SECTION:NOTES:END -->

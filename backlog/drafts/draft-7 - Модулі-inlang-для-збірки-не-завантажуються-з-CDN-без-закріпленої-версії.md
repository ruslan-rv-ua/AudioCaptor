---
id: DRAFT-7
title: Модулі inlang для збірки не завантажуються з CDN без закріпленої версії
status: Draft
assignee: []
created_date: '2026-11-23T18:45:48Z'
updated_date: '2026-11-27T19:21:50Z'
labels:
  - security
  - frontend
  - model-sonnet
  - effort-low
milestone: m-11
dependencies: []
references:
  - project.inlang/settings.json
  - vite.config.ts
priority: medium
type: chore
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`project.inlang/settings.json` підключає два модулі inlang прямо з CDN і без закріпленої версії:

- `https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@latest/dist/index.js`
- `https://cdn.jsdelivr.net/npm/@inlang/message-lint-rule-missing-translation@latest/dist/index.js`

Paraglide завантажує й виконує їх у Node під час кожної збірки фронтенду: `pnpm vite:build`, а отже `just check`, `just build`, CI і Release (у `project.inlang/cache/plugins` файли переписуються при кожній збірці). Що саме виконається, вирішує той, хто опублікує наступну версію цих пакетів: зламана або шкідлива версія потрапить у збірку без жодної зміни в репозиторії, а плагін message-format формує код повідомлень, який іде в exe. Без мережі збірка, ймовірно, не пройде. `pnpm audit` і `pnpm-lock.yaml` цих модулів не бачать.

Знайдено 2026-11-23 під час TASK-6; це було й до оновлення. Після оновлення `@inlang/paraglide-js` до 2.29.3 крок Paraglide у `vite build` триває близько 3,6 с (уся збірка до оновлення — 0,7 с).

Що має вирішити власник — один із варіантів:

- закріпити точні версії в адресах модулів;
- підключати модулі з npm-пакетів, закріплених у `pnpm-lock.yaml` (якщо Paraglide це підтримує);
- лишити як є.
<!-- SECTION:DESCRIPTION:END -->

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
<!-- SECTION:NOTES:END -->

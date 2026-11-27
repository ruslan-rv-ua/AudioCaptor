---
id: TASK-40
title: Модулі inlang для збирання мають закріплені версії
status: Done
assignee:
  - '@claude'
created_date: '2026-11-27T19:21:46Z'
updated_date: '2026-11-27T21:48:28Z'
labels:
  - security
  - frontend
  - model-sonnet
  - effort-low
milestone: m-8
dependencies: []
references:
  - project.inlang/settings.json
priority: medium
type: chore
ordinal: 14200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`project.inlang/settings.json` підключає два модулі inlang прямо з CDN з `@latest`: `@inlang/plugin-message-format` і `@inlang/message-lint-rule-missing-translation`. Paraglide завантажує й виконує їх у Node під час кожної збірки інтерфейсу (`just check`, `just build`, CI, Release), тож нова версія цих пакетів, зламана чи шкідлива, потрапить у збірку й у exe без жодної зміни в репозиторії.

Повністю це питання закриває перехід на Paraglide 3 з модулями з npm (DRAFT-7). Ця задача лише закріплює версії, які збирання використовує зараз, щоб прибрати ризик до того переходу. Власник погодив 27.11.2026.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Адреси обох модулів у `project.inlang/settings.json` мають точні версії замість `@latest`: ті, що збирання завантажує зараз
- [x] #2 `just check` і `just build` проходять, а тексти інтерфейсу обома мовами не змінилися
- [x] #3 `DEVELOPMENT.md` каже, як оновлювати ці версії
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Усі локальні перевірки проходять: `just check`
- [x] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-27: версії знайдено так: заголовок `X-JSD-Version` у відповіді jsDelivr на адреси з `@latest` (4.6.0 і 1.4.8); sha256 файлів у `project.inlang/cache/plugins` збігся з `dist/index.js` цих версій. У `plugin-message-format` 4.6.0, 4.6.1 і 4.6.2 (npm latest) `dist/index.js` однаковий байт у байт. Paraglide 2.29.3 сам пропонує `plugin-message-format@4.5.0` у шаблоні init, тож 4.x працює з Paraglide 2. `src/paraglide/` до й після зміни однаковий (`git diff --no-index` без різниці); після збирання в кеші з'явилися файли для нових адрес. `just check` пройшов.

2026-11-27: `just build` пройшов (AudioCaptor.exe зібрано); бандл інтерфейсу `index-BzxfSR-P.js` має ту саму назву-хеш, що й до зміни. DoD #2, #3, #5 не стосуються: текстів інтерфейсу й поведінки задача не змінює, CHANGELOG без запису. Коміт a53ee35 на гілці feature/task-40-pin-inlang. CI вручну ще не запускали.

2026-11-27, головна сесія: гілку `feature/task-40-pin-inlang` злито в `develop` лише локально (вибір власника: без відправки); головна сесія повторила `just check` (134 тести). Задача лишається відкритою, доки власник не скаже відправити `develop` і запустити CI вручну; після зеленого CI — закрити.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Модулі inlang для збирання мають закріплені версії: у `project.inlang/settings.json` замість `@latest` стоять `@inlang/message-lint-rule-missing-translation@1.4.8` і `@inlang/plugin-message-format@4.6.0`, тобто ті версії, які збирання завантажувало до зміни (перевірено заголовком `X-JSD-Version` у відповіді jsDelivr і sha256 файлів у кеші Paraglide). `DEVELOPMENT.md` пояснює, як оновлювати ці версії. Згенерований `src/paraglide/` до й після зміни однаковий (144 файли), бандл інтерфейсу має ту саму назву-хеш.

Перевірки: `just check` (134 тести; головна сесія повторила), `just build`. Ручний CI на `develop`: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46071803938 — зелений.

Не перевірено: збирання без мережі — модулі й далі йдуть з jsDelivr. Повністю це питання закриває DRAFT-7 (Paraglide 3 з модулями з npm).
<!-- SECTION:FINAL_SUMMARY:END -->

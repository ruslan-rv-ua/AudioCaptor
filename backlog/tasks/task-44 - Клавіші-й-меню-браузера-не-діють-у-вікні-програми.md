---
id: TASK-44
title: Клавіші й меню браузера не діють у вікні програми
status: In Progress
assignee:
  - '@claude'
created_date: '2026-11-27T20:00:47Z'
updated_date: '2026-11-27T21:40:00Z'
labels:
  - a11y
  - frontend
  - backend
  - model-sonnet
  - effort-medium
milestone: m-8
dependencies: []
references:
  - src-tauri/src/lib.rs
  - src-tauri/tauri.conf.json
priority: medium
type: bug
ordinal: 14450
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Огляд TASK-42 показав, що вікно WebView2 досі пропускає клавіші браузера. F5, Shift+F5, Ctrl+R і Ctrl+Shift+R перезавантажують сторінку: вікно скидається, незбережений профіль губиться, а під час запису NVDA хибно каже «запис почато». Ctrl+P відкриває друк, Ctrl+F, F3 і Ctrl+G — пошук, а клавіша контекстного меню, Shift+F10 і правий клік — меню браузера з пунктами «Оновити», «Зберегти як», «Друк». Tauri 2.12 не вимикає ні клавіш браузера, ні меню браузера: у wry обидва ввімкнені за замовчуванням. Масштаб (Ctrl+плюс) уже вимкнено, інструментів розробника в збірці для користувачів немає.

Власник вирішив 27.11.2026 винести це з TASK-43 в окрему задачу для патча 0.4.1: це вада, а не нова можливість.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 У зібраній програмі F5, Shift+F5, Ctrl+R, Ctrl+Shift+R, Ctrl+P, Ctrl+F, F3 і Ctrl+G нічого не роблять
- [x] #2 Клавіша контекстного меню, Shift+F10 і правий клік не відкривають меню браузера; Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+A і Ctrl+Z у текстових полях діалогу профілю працюють як раніше
- [x] #3 З NVDA перевірено, що клавіші програми (Alt+S, Alt+P, Alt+R, Alt+T, Alt+I, F1, Escape) і клавіші NVDA працюють як раніше
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
1. У `setup` через `with_webview` поставити в налаштуваннях WebView2 `AreBrowserAcceleratorKeysEnabled` і `AreDefaultContextMenusEnabled` у `false` (Tauri 2.12 таких опцій не має).
2. Ці налаштування можуть подіяти лише з наступної навігації, а перезавантаження на старті загубило б те, що вікно бере один раз (`take_settings_notice`). Тож додати обробники, що діють одразу: `AcceleratorKeyPressed` поглинає F5, F3 і Ctrl+R/P/F/G (зі Shift чи без, без Alt), `ContextMenuRequested` — меню.
3. Додати крок до сценарію `record` у `just smoke`, записи в CHANGELOG і DEVELOPMENT.md.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- Код: `src-tauri/src/webview.rs` (новий модуль, лише Windows), виклик у `setup` у `lib.rs`. Залежності `webview2-com 0.39` і `windows-core 0.62` (як `windows-core-webview2`) — ті самі версії, що вже були в Cargo.lock через wry, нових пакетів не завантажено; до `windows 0.59` додано feature `Win32_UI_Input_KeyboardAndMouse` (`GetKeyState`).
- Документація WebView2 (зразок коду) каже, що обидва налаштування діють «after the next navigation», і що зміни після `NavigationStarting` чекають наступної навігації. Тому самих налаштувань мало для першої сторінки; обробники діють одразу.
- Ctrl+N і Ctrl+S обробник не ловить (юніт-тест це закріплює), тож вони доходять до сторінки для TASK-43. Що з ними робить сам WebView2, живцем не перевіряли: налаштування клавіш браузера вимикає браузерні дії з наступної навігації, а на першій сторінці діє лише обробник. Критерій #3 TASK-43 (Ctrl+N не відкриває вікна) треба перевірити там.
- Перевірки: `just check` пройшов (138 тестів Rust, серед них 2 нові для `is_browser_key`). `just smoke record dialogs` з дозволу власника: 37 кроків OK, 70 перевірок NVDA OK. Новий крок 6 сценарію `record`: під час запису F5, Shift+F5, Ctrl+R, Ctrl+Shift+R, Ctrl+P, Ctrl+F, F3, Ctrl+G, Shift+F10 і клавіша меню — фокус лишився на Pause, Alt+I сказав «Recording, 00:00:08», «Recording started» і назви сторінки не було.
- Власник перевірив вручну у зібраній програмі: правий клік не відкриває меню; Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+Z у полі Name діалогу профілю працюють; клавіші NVDA працюють.
- Не перевіряли: чи новий крок smoke падає на старій збірці (контрольного запуску на `develop` не було).

2026-11-27, головна сесія: гілку `bugfix/task-44-browser-keys` злито в `develop`; головна сесія повторила `just check` (138 тестів). Задача змінила залежності (`webview2-com`, `windows-core` 0.62 — ті самі версії, що вже були в `Cargo.lock` через wry), тож перед закриттям — ручний запуск CI на `develop` (вибір власника: усе з відправкою).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Клавіші й меню браузера WebView2 вимкнено в `src-tauri/src/webview.rs`: налаштування `AreBrowserAcceleratorKeysEnabled` і `AreDefaultContextMenusEnabled` у `false` плюс обробники `AcceleratorKeyPressed` (F5, F3, Ctrl+R/P/F/G) і `ContextMenuRequested`, бо налаштування можуть подіяти лише з наступної навігації. Сценарій `record` у `just smoke` натискає ці клавіші під час запису. `just check` і `just smoke record dialogs` пройшли; власник вручну перевірив правий клік, Ctrl+A/C/X/V/Z у полі Name і клавіші NVDA. Ctrl+N і Ctrl+S обробник пропускає; що з ними робить WebView2, перевірить TASK-43. Змінено залежності (webview2-com, windows-core 0.62 з Cargo.lock), тож перед закриттям потрібен запуск CI вручну.
<!-- SECTION:FINAL_SUMMARY:END -->

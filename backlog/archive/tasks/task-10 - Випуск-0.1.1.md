---
id: TASK-10
title: Випуск 0.1.1
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:31:35Z'
updated_date: '2026-11-24T23:42:58Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-1
dependencies:
  - TASK-2
  - TASK-5
  - TASK-6
  - TASK-8
  - TASK-9
  - TASK-15
  - TASK-16
  - TASK-17
  - TASK-18
  - TASK-20
  - TASK-22
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
  - .github/workflows/release.yml
priority: medium
type: task
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Перший випуск після паузи. Для користувачів він несе виправлення вікна (TASK-9) і те з оновлення залежностей, що змінило exe (TASK-6). Для проєкту це перший справжній прохід оновленого порядку випуску: перевірки перед збіркою, звірка версії, опис релізу з `CHANGELOG.md` (TASK-5), завершення релізу через git flow без тегу (TASK-2) і документ, що все це описує (TASK-8).

Формат `settings.json` лишається версії 5, тож повернення до 0.1.0 безпечне; перед випуском це треба перевірити ще раз.

Порядок випуску — у `DEVELOPMENT.md`. Push і запуск workflow потребують дозволу власника. Кожен опублікований реліз упродовж доби забирає Excavator у `ruslan-rv-ua/scoop-bucket` (запускається щодня о 04:20 UTC або вручну), тож пробних релізів не буває: для перевірки є пробний запуск.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.1.1 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.1.1 лише зі змінами, помітними користувачам, а опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.1.1 --notag`: `main` і `develop` містять реліз, а тег `v0.1.1` створив Release workflow
- [x] #4 GitHub Release v0.1.1 містить `AudioCaptor.exe`; завантажений файл запускається й показує версію 0.1.1 у діалозі «Про програму»
- [x] #5 Маніфест Scoop у `ruslan-rv-ua/scoop-bucket` показує 0.1.1 після запуску Excavator, а `scoop update audiocaptor` зберігає теку `AudioCaptor-data`
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [x] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Перевірити, що формат `settings.json` досі версії 5, як у 0.1.0; спитати власника про рядок «CI/CD…» у розділі 0.1.0 `CHANGELOG.md`.
2. `git flow release start 0.1.1`; версія 0.1.1 у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`; `just check` (оновлює `src-tauri/Cargo.lock`).
3. `CHANGELOG.md`: `## [Unreleased]` стає `## [0.1.1] — <дата>`, внизу посилання на реліз; перечитати розділ очима користувача.
4. Коміт на гілці випуску; `just build`.
5. З дозволу власника: push `release/0.1.1` і пробний запуск Release; звіт про частину 1 головній сесії.
6. Після дозволу власника: `git flow release finish 0.1.1 --notag`, push `main` і `develop`, видалити `release/0.1.1` на GitHub, справжній запуск Release на `main`, `gh release view v0.1.1`.
7. З дозволу власника: завантажений exe — копією в тимчасовій теці з власною `AudioCaptor-data`, «About» показує 0.1.1; запуск Excavator і `scoop update audiocaptor` — лише з дозволу власника.
8. Результати — в нотатки задачі; звіт головній сесії.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Зауваження головної сесії, 2026-11-23: розділ 0.1.0 у `CHANGELOG.md` містить рядок «CI/CD: GitHub Actions workflows for CI, Release, and Scoop manifest update». Це подробиця для розробників, а не для користувачів, і вона застаріла: workflow оновлення Scoop прибрано. Під час підготовки 0.1.1 вирішити з власником, чи прибрати рядок із розділу 0.1.0. Опис релізу тепер береться з `CHANGELOG.md`, тож такі рядки потрапляють і на сторінку релізу.

Зауваження головної сесії, 2026-11-24 (після TASK-8): порядок випуску в `DEVELOPMENT.md` змінено — пробний запуск Release іде на гілці `release/0.1.1`, відправленій на GitHub, ще до `git flow release finish`; після завершення гілку випуску видаляють на GitHub. Excavator за розкладом (04:20 UTC у файлі) у листопаді 2026 фактично запускався близько 13:00 UTC; швидше — запустити його вручну.

Частина 1 (робоча сесія, 2026-11-25):
- Формат `settings.json`: від v0.1.0 у `src-tauri/src/settings.rs` змінено лише форматування й тести; `Settings::default().version` — 5, міграція закінчується на 5; `profiles.rs` і `audio/types.rs` не змінювалися. Повернення до 0.1.0 безпечне.
- Власник вирішив прибрати з розділу 0.1.0 `CHANGELOG.md` рядок «CI/CD: GitHub Actions workflows…».
- `git flow release start 0.1.1` від `develop` (1347962). Версія 0.1.1 у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock` (його оновив `just check`).
- `CHANGELOG.md`: `## [Unreleased]` став `## [0.1.1] — 2026-11-25`, унизу посилання `[0.1.1]`. Розділ перечитано: сім пунктів «Fixed», усі помітні користувачам; запису про оновлення залежностей немає, як вирішено в TASK-6. Порожнього `## [Unreleased]` після перейменування немає (DEVELOPMENT.md його не вимагає).
- Коміт 42c9cc5 `chore(release): 0.1.1` на `release/0.1.1` (п'ять файлів).
- `just check` пройшов: svelte-check 0 помилок і 0 попереджень, clippy без попереджень, 66 тестів, fmt.
- `just build` пройшов: exe 4 944 896 байт, FileVersion і ProductVersion 0.1.1; лише давнє попередження про ідентифікатор `.app` для macOS.
- З дозволу власника: `git push origin release/0.1.1` (разом із гілкою на GitHub потрапив коміт 1347962 з локального `develop`; `develop` і `main` на GitHub не змінилися) і пробний Release https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45504680020 на 42c9cc5 — зелений: версія 0.1.1 у трьох файлах, тегу v0.1.1 ще немає, опис релізу дослівно збігся з розділом 0.1.1, перевірки й збірка пройшли за 4 хв 23 с, `publish` пропущено. SHA256 exe з пробного запуску: b7fc677159c74e097574d3862e0337a3544c4734ba1e319b0527ad9647841e6b.
- Перевірити в частині 2: рядки в `CHANGELOG.md` перенесено вручну (~115 знаків); сторінка релізу на GitHub може показати ці переноси як розриви рядків посеред речення.

Частина 2 (робоча сесія, 2026-11-25), кожен крок — з дозволу власника:
- `git stash push --include-untracked -m "task-10 backlog" -- backlog`, `git flow release finish 0.1.1 --notag`, `git stash pop`. `main` перемотано на 42c9cc5 — той самий коміт, що пройшов пробний запуск; `develop` отримав злиття e8b28b1 «Merge branch 'main' into develop». Тегу git flow не створив. Гілку `release/0.1.1` git flow сам видалив і локально, і на GitHub, тож `git push origin --delete release/0.1.1` не знадобився.
- `git push origin main develop`: main c0456bc..42c9cc5, develop 0a5521d..e8b28b1.
- Справжній Release https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45507059653 на `main` (42c9cc5): build і publish зелені. `gh run watch` один раз обірвався через HTTP 401 під час читання приміток запуску — збій самої команди стеження, запуск ішов далі; далі стежив через `gh run view`.
- Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.1: тег `v0.1.1` на 42c9cc5 створив github-actions[bot]; реліз не чернетка й не попередній; файл `AudioCaptor.exe` 4 947 968 байт, SHA256 aee80e402569d7f138fdd359bc92e16ccf85eaeb7536e8aa2eb7301ba2af4bc5 — той самий, що в журналі запуску. Опис релізу — розділ 0.1.1 `CHANGELOG.md` (різниця лише в порожньому рядку в кінці). GitHub показує ручні переноси рядків як розриви: 18 розривів посеред речень (DRAFT-13).
- AC #4: `AudioCaptor.exe` завантажено з релізу (`gh release download`), хеш збігся, FileVersion 0.1.1. Копія працювала в тимчасовій теці з власною `AudioCaptor-data` і власною `WEBVIEW2_USER_DATA_FOLDER`. Скрипт UI Automation натиснув кнопку «Про AudioCaptor» (інтерфейс українською) і прочитав у діалозі `v0.1.1`; копія закрилася після WM_CLOSE; `%LOCALAPPDATA%\com.audiocaptor.app` не змінилася; журнал копії без помилок. Перший запуск скрипта кнопки не знайшов: він узяв приховане вікно `com.audiocaptor.app-siw` (single-instance) замість головного, і копію закрито примусово. Після виправлення пошуку вікна (за назвою «AudioCaptor») другий запуск, у новій теці, пройшов.
- AC #5: Excavator вручну https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/45510064298 — зелений; коміт b4deafe «audiocaptor: Update to version 0.1.1»: версія 0.1.1, URL релізу v0.1.1, хеш aee80e40…. `scoop update audiocaptor` (Scoop спершу оновив себе й bucket-и): 0.1.0 → 0.1.1, хеш перевірено, `current` веде на 0.1.1, exe 0.1.1 з тим самим SHA256, `AudioCaptor-data` у 0.1.1 — junction на `C:\scoop\persist\audiocaptor\AudioCaptor-data`. Знімок цієї теки до й після оновлення однаковий (назви, розміри, час зміни й SHA256 `settings.json`, `logs\audiocaptor.log`, тека `Recordings`). Тека версії 0.1.0 лишилася: старі версії Scoop прибирає лише `scoop cleanup`. Оновлену програму не запускав.
- Нова чернетка DRAFT-13 «Опис релізу на GitHub читається цілими реченнями» (0.2.0).
- Не закомічено, для головної сесії: `backlog/` (TASK-10, DRAFT-13).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випуск 0.1.1 опубліковано: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.1.

Що зроблено:
- Гілка `release/0.1.1`: версія 0.1.1 у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; розділ `## [0.1.1] — 2026-11-25` у `CHANGELOG.md` (сім виправлень, помітних користувачам); з розділу 0.1.0 прибрано рядок для розробників про CI/CD (рішення власника). Формат `settings.json` лишився версії 5, тож повернення до 0.1.0 безпечне.
- Пробний Release на гілці випуску зелений (run 45504680020). Потім `git flow release finish 0.1.1 --notag`, `main` і `develop` відправлено на GitHub; гілку випуску git flow видалив і там.
- Справжній Release на `main` зелений (run 45507059653): тег `v0.1.1` на 42c9cc5 створив workflow, реліз має `AudioCaptor.exe` (SHA256 aee80e40…).
- Завантажений exe показує 0.1.1 в «About» (копія з окремою текою даних, перевірка через UI Automation).
- Excavator, запущений вручну, оновив маніфест Scoop до 0.1.1; `scoop update audiocaptor` оновив встановлену програму, а `AudioCaptor-data` лишилася незмінною.
- Опис релізу на GitHub спершу показував 18 розривів рядка посеред речень (ручні переноси з `CHANGELOG.md`). З дозволу власника головна сесія склеїла рядки в описі v0.1.1 (`gh release edit`): тепер кожен пункт — один абзац; exe і тег не змінювалися. Як уникати цього надалі — DRAFT-13 (0.2.0).

Кожну відправку, кожен запуск workflow, завантаження й `scoop update` робили з окремого дозволу власника.

Не перевірено: оновлену програму зі Scoop не запускали, щоб не чіпати даних власника; «About» перевірено через UI Automation, а не на слух з NVDA.

Зауваження: після випуску в `CHANGELOG.md` немає заголовка `## [Unreleased]`; перша наступна зміна із записом у CHANGELOG має його додати.
<!-- SECTION:FINAL_SUMMARY:END -->

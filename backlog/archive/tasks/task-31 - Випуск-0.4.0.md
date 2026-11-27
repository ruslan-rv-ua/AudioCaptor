---
id: TASK-31
title: Випуск 0.4.0
status: Done
assignee:
  - '@claude'
created_date: '2026-11-26T08:03:51Z'
updated_date: '2026-11-27T19:07:22Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-6
dependencies:
  - TASK-30
  - TASK-12
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 13500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск із новою можливістю: записи можна автоматично зберігати у FLAC (TASK-12).

Профіль із форматом FLAC, відкритий у старішій версії, писатиме WAV (TASK-28), — `CHANGELOG.md` має про це сказати. Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.3.0 (TASK-30). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.4.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.4.0 лише зі змінами, помітними користувачам, і каже, що буде з профілями у форматі FLAC після повернення до старішої версії; опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.4.0 --notag`: `main` і `develop` містять реліз, а тег `v0.4.0` створив Release workflow
- [x] #4 GitHub Release v0.4.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.0, а пробний запис у FLAC відтворюється
- [x] #5 Маніфест Scoop показує 0.4.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
2026-11-27 (сесія TASK-31), частина 1: гілка release/0.4.0, версія 0.4.0 у package.json, Cargo.toml, tauri.conf.json, Cargo.lock; CHANGELOG: розділ [0.4.0] — 2026-11-27 з нотаток [Unreleased] (FLAC, повернення до 0.3.0, один Escape у діалозі профілю), новий порожній [Unreleased], посилання [0.4.0]; коміт c8decc8. just check (134 тести) і just build зелені; локальний exe 5431808 байт (0.3.0 — 5294592), FileVersion 0.4.0.
just smoke повний: 73 кроки з 74, 165 перевірок NVDA з 167; впав відомий нестабільний крок 4 сценарію record (DRAFT-10: «недоступно | документ | додаток»). Повтор record з дозволу власника: 12 з 12, 26 перевірок з 26. Сценарій flac пройшов увесь.

Пробний Release на release/0.4.0: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46050259097 (build зелений, publish пропущено).

2026-11-27 (сесія TASK-31), частина 2 з дозволу власника («Так, публікуй»): git stash (backlog/), git flow release finish 0.4.0 --notag --no-fetch --no-push, git stash pop на develop. main = c8decc8 (той самий коміт, що пройшов пробний запуск), develop = 8569512 (злиття main); обидві відправлено. Видалення release/0.4.0 на GitHub заблокував захист сесії.
Справжній Release: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46051834921 (build і publish зелені). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.0: тег v0.4.0 на c8decc8, не попередній, AudioCaptor.exe 5430784 байти (0.3.0 — 5294592, +136192), sha256 00bfb4efe3138b86a4e91bf94705015d3febbbb688dbd968fad302805ee36cbf; опис = розділ 0.4.0, без розривів посеред речень.
AC #4: exe завантажено в C:\Temp\ac-0.4.0-check (хеш збігається, FileVersion 0.4.0), власна AudioCaptor-data (settings версії 5 -> міграція, профіль Loopback 48 кГц, FLAC). About (F1, UI Automation) показує v0.4.0. Запис тону 440 Гц через mpv (Alt+S / Alt+T, ~7 с): loopback_….flac 132615 байт, конвертовано за 0,0 с, WAV видалено. FFmpeg: flac, 48 кГц, стерео, 16 біт, 7,28 с, розкодовано до кінця (-xerror, код 0), max -11,3 дБ; mpv (ao=null) до кінця, код 0. %LOCALAPPDATA%\com.audiocaptor.app не змінилася (491 елемент, той самий найновіший час). Тестову теку видалено.
AC #5: Excavator вручну: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46052985301 (зелений), коміт 962e123 «audiocaptor: Update to version 0.4.0», маніфест 0.4.0, hash 00bfb… збігається. scoop update audiocaptor 0.3.0 -> 0.4.0: hash OK, FileVersion 0.4.0. AudioCaptor-data у C:\scoop\persist\audiocaptor без змін (ті самі 3 файли й SHA256). Нову версію зі Scoop на даних власника не запускали.

Поправка: гілки release/0.4.0 на GitHub уже немає (git ls-remote її не показує), тож прибирати нічого не треба.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випущено AudioCaptor 0.4.0: запис у FLAC без втрат (TASK-12) і діалог профілю, що знову закривається першим Escape (TASK-23). Версія 0.4.0 стоїть у `package.json`, `Cargo.toml`, `tauri.conf.json` і `Cargo.lock`. `CHANGELOG.md` має розділ 0.4.0 із пунктом про FLAC-профілі в 0.3.0; опис релізу — той самий розділ. Випуск завершено `git flow release finish 0.4.0 --notag`, `main` і `develop` відправлено, тег `v0.4.0` створив Release workflow. Це перший випуск, зібраний із файлом CMake для бібліотек на C: exe більший за 0.3.0 на 133 КБ, тобто лише на вбудовану libFLAC.

Перевірки: `just check`, `just build`; повний `just smoke` — 73 кроки з 74 (впав відомий нестабільний крок 4 сценарію `record`, DRAFT-10), повтор `record` — 12 з 12. Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46050259097; справжній: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46051834921; реліз: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.0 (AudioCaptor.exe, 5 430 784 байти). Завантажений exe у тестовій теці: About показує v0.4.0, пробний запис у FLAC конвертовано, WAV видалено, FFmpeg розкодовує файл до кінця, mpv програє. Excavator: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46052985301, маніфест 0.4.0 із правильним хешем; `scoop update audiocaptor` 0.3.0 → 0.4.0, дані власника без змін. Головна сесія повторила `just check` на `develop`.

Не перевірено: версію зі Scoop на даних власника не запускали. Власник може сам перевірити TASK-23: відкрити New Profile і натиснути Escape один раз.
<!-- SECTION:FINAL_SUMMARY:END -->

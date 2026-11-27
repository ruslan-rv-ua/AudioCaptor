---
id: TASK-30
title: Випуск 0.3.0
status: Done
assignee:
  - '@claude'
created_date: '2026-11-26T08:03:31Z'
updated_date: '2026-11-27T14:51:53Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-5
dependencies:
  - TASK-13
  - TASK-29
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 13200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск із новою можливістю: записи можна автоматично зберігати в MP3 (TASK-29).

Профіль із форматом MP3, відкритий у 0.2.0, писатиме WAV (TASK-28), — `CHANGELOG.md` має про це сказати. Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.2.0 (TASK-13). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.3.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.3.0 лише зі змінами, помітними користувачам, і каже, що буде з профілями у форматі MP3 після повернення до 0.2.0; опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.3.0 --notag`: `main` і `develop` містять реліз, а тег `v0.3.0` створив Release workflow
- [x] #4 GitHub Release v0.3.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.3.0, а пробний запис у MP3 відтворюється
- [x] #5 Маніфест Scoop показує 0.3.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
2026-11-27 (сесія TASK-30), частина 1: гілка release/0.3.0 (локальна, не відправлена), версія 0.3.0 у package.json, Cargo.toml, tauri.conf.json, Cargo.lock; CHANGELOG: розділ [0.3.0] — 2026-11-27 з нотаток [Unreleased] (пункт про повернення до 0.2.0 уже був), новий порожній [Unreleased], посилання [0.3.0] унизу; коміт 542432d. just check (125 тестів) і just build зелені, exe FileVersion 0.3.0.
ЗУПИНЕНО на just smoke: чотири спроби (04:07–04:12) зупинилися до першого кроку з кодом 2 («NVDA logs key presses but said nothing for the test window»). NVDA говорить, але фокус лишається в Total Commander: NVDA пише «Foreground took too long to change. Foreground still TTOTAL_CMD», і клавіша End від тесту дійшла до Total Commander. Не допомогло й тимчасово згорнути Total Commander (потім його відновлено). Схоже, Windows без користувача за клавіатурою не віддає фокус новому вікну; ForegroundLockTimeout = 2147483647. Push, пробний і справжній Release не запускались; нічого не опубліковано.

2026-11-27 (сесія TASK-30), продовження з дозволу власника («Так, продовжуй»): повний just smoke — 61 крок із 61, 140 перевірок NVDA (власник за ПК; уночі тест не міг забрати фокус).
Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46016056836 (build зелений, publish пропущено).
git flow release finish 0.3.0 --notag (нотатки backlog/ — через git stash, повернуто на develop): main = 542432d, develop = 96b7e7f (злиття main), обидві відправлено. Гілка release/0.3.0 на GitHub лишилася: видалення заблокував захист сесії, її можна прибрати вручну.
Справжній запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46017448071 (build і publish зелені). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.3.0: тег v0.3.0 на 542432d, не попередній, AudioCaptor.exe 5294592 байти, sha256 bd7d148f9c5b714e4e2d92c2233086809c6e311c33a2ec638a8345108c7112d1; опис = розділ 0.3.0, без розривів посеред речень.
AC #4: exe завантажено в C:\Temp\ac-0.3.0-check (хеш збігається, FileVersion 0.3.0), запущено з власною AudioCaptor-data (settings версії 6, профіль Loopback 44,1 кГц, MP3 — стандарт); «About» (F1, UI Automation) показує v0.3.0. Запис системного звуку (тон 440 Гц через mpv, Alt+S / Alt+T, ~7 с) дав loopback_….mp3 110341 байт за 0,2 с конвертації, WAV видалено. FFmpeg: mp3, 44,1 кГц, стерео, 128 кбіт/с, 6,90 с, розкодовано до кінця без помилок, max -11,3 дБ; mpv (ao=null) програв до кінця, код 0. %LOCALAPPDATA%\com.audiocaptor.app не змінилася (491 елемент, той самий найновіший час). Глобальна Pause, надіслана скриптом, до програми не дійшла; без вибраного пристрою (selectedLoopback = null) Alt+S запис не почав — тому в тестові налаштування вписано пристрій відтворення за замовчуванням. Тестову теку видалено.
AC #5: Excavator вручну: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46019017289 (зелений), коміт 6e9f9d9 «audiocaptor: Update to version 0.3.0», маніфест 0.3.0, hash bd7d1… збігається. scoop update audiocaptor (0.2.0 -> 0.3.0): hash OK, FileVersion 0.3.0. AudioCaptor-data у C:\scoop\persist\audiocaptor без змін (ті самі 3 файли й SHA256). Нову версію зі Scoop на даних власника не запускали.

Поправка: гілки release/0.3.0 на GitHub уже немає (git ls-remote її не показує), тож прибирати нічого не треба.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випущено AudioCaptor 0.3.0: запис у MP3 вбудованим кодером Windows (TASK-29). Версія 0.3.0 стоїть у `package.json`, `Cargo.toml`, `tauri.conf.json` і `Cargo.lock`. `CHANGELOG.md` має розділ 0.3.0 із пунктом про MP3-профілі після повернення до 0.2.0; опис релізу — той самий розділ. Випуск завершено `git flow release finish 0.3.0 --notag`, `main` і `develop` відправлено, тег `v0.3.0` створив Release workflow.

Уночі перша спроба зупинилася на `just smoke`: без людини за ПК Windows не віддавав фокус вікну тесту (DRAFT-21), і нічого не публікувалося. Коли власник сів за ПК і написав «Так, продовжуй», випуск пройшов до кінця.

Перевірки: `just check`, `just build`, повний `just smoke` — 61 крок з 61, 140 перевірок NVDA з 140. Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46016056836; справжній: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46017448071; реліз: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.3.0 (AudioCaptor.exe, 5 294 592 байти). Завантажений exe у тестовій теці: About показує v0.3.0, пробний запис у «MP3 — стандарт» конвертовано, WAV видалено, FFmpeg розкодовує файл до кінця, mpv програє. Excavator: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46019017289, маніфест 0.3.0 із правильним хешем; `scoop update audiocaptor` 0.2.0 → 0.3.0, дані власника без змін. Головна сесія повторила `just check` на `develop`.

Не перевірено: версію зі Scoop на даних власника не запускали. Знайдене дорогою: DRAFT-22 (Alt+S мовчить, коли не вибрано пристрою).
<!-- SECTION:FINAL_SUMMARY:END -->

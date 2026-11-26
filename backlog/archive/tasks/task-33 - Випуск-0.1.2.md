---
id: TASK-33
title: Випуск 0.1.2
status: Done
assignee:
  - '@claude'
created_date: '2026-11-26T08:52:02Z'
updated_date: '2026-11-26T15:44:39Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-7
dependencies: []
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 10080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск виправлень, не чекаючи Opus (0.2.0); так вирішив власник 26 листопада 2026:

- тека даних вбудованого браузера лежить поруч з exe (TASK-25; уже в `develop` і в розділі Unreleased `CHANGELOG.md`);
- нечитабельний `settings.json` більше не стирається мовчки (TASK-32);
- дві помилки доступності: TASK-19 і TASK-23.

Формат `settings.json` не змінюється. Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.1.1 (TASK-10). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.1.2 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.1.2 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.1.2 --notag`: `main` і `develop` містять реліз, а тег `v0.1.2` створив Release workflow
- [x] #4 GitHub Release v0.1.2 містить `AudioCaptor.exe`; завантажений файл показує версію 0.1.2, а пробний запис у WAV відтворюється
- [x] #5 Маніфест Scoop показує 0.1.2 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
1. Перевірити, що формат settings.json досі 5 (відкат до 0.1.1 безпечний).
2. git flow release start 0.1.2; версія 0.1.2 у package.json, Cargo.toml, tauri.conf.json; just check (Cargo.lock).
3. CHANGELOG: розділ [0.1.2] з датою, новий порожній [Unreleased], посилання внизу; коміт; just build.
4. З дозволу власника: повний just smoke (TASK-19: dialogs з -Once; сценарій settings).
5. З дозволу власника: push release/0.1.2 і пробний запуск Release; звіт головній сесії.
6. Після дозволу: release finish --notag, push main і develop, справжній запуск, перевірка релізу, Scoop.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-26: у TASK-19 тест NVDA пропущено за рішенням власника. Перед випуском (або раніше, у TASK-23) повний `just smoke` має підтвердити, що NVDA читає назву кожного діалогу один раз (сценарій `dialogs`, перевірка `-Once`) і що Enter закриває «About». Запуск — лише з дозволу власника.

2026-11-26: TASK-23 відкладено на невизначений термін, тож залежності випуску очищено: решта (TASK-10, TASK-24, TASK-25, TASK-26, TASK-32, TASK-19) уже виконані й заархівовані, а заархівовані задачі CLI не дозволяє лишати в списку залежностей.

2026-11-26 (сесія TASK-33): частина 1: гілка release/0.1.2, версія 0.1.2 у чотирьох файлах, just check і just build зелені; повний just smoke: TASK-19 підтверджено (назву кожного діалогу NVDA читає раз, у About фокус на Close), сценарій settings пройшов; запис, крок 3 упав раз (DRAFT-10), повторний just smoke record пройшов 11/11. Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45842013298 (зелений).
Частина 2: git flow release finish 0.1.2 --notag; main (d0c1962) і develop відправлено; гілки release/0.1.2 на GitHub уже не було. Справжній запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45844358441 (зелений). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.2: тег v0.1.2, AudioCaptor.exe 4925440 байт, sha256 27455ba60d00065392a8e9c0ae4badc291e4b8f3cb4e140d1aebfbb359e233b4, не попередній; опис = розділ 0.1.2 без розривів посеред речень.
Не перевірено: AC #4 (завантажений exe, About, пробний запис WAV) і AC #5 (Excavator, scoop update) чекають дозволу власника.

2026-11-26 (AC #4): exe завантажено з релізу в C:\Temp\ac-0.1.2-check, sha256 збігається з GitHub (27455ba6...e233b4), розмір 4925440, FileVersion 0.1.2. Копія запущена з власними AudioCaptor-data і AudioCaptor-webview; «About» показує v0.1.2; запис Mix кілька секунд дав mix_2026-11-26_17-28-54.wav (1,65 МБ, RIFF/WAVE, 2 канали, 48 кГц, дані ненульові). За рішенням власника відтворення на слух не перевіряли, достатньо заголовка. %LOCALAPPDATA%\com.audiocaptor.app не змінювалася (останні зміни до перевірки). Тестову копію закрито.
Перед AC #5: у scoop-bucket маніфест ще 0.1.1, встановлено 0.1.1.

2026-11-26 (AC #5): ручний Excavator (https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/45849950392) зелений; коміт «audiocaptor: Update to version 0.1.2», маніфест 0.1.2, hash 27455ba6... збігається з релізом. scoop update audiocaptor (0.1.1 -> 0.1.2) пройшов: hash exe OK, FileVersion 0.1.2. AudioCaptor-data (persist) без змін: settings.json 716 байт, sha256 266FFBED...97A до й після, ті самі logs і Recordings (mic_2026-11-26_09-25-52.wav). Нічого в ній не змінювалось; нову версію не запускали.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випуск 0.1.2 опубліковано 26.11.2026: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.1.2 (тег v0.1.2 на d0c1962 створив Release workflow; AudioCaptor.exe 4 925 440 байт, sha256 27455ba6…; опис = розділ 0.1.2 CHANGELOG). Увійшли: тека даних WebView2 поруч з exe (TASK-25), збереження нечитабельного settings.json (TASK-32), одноразове читання назви «About» (TASK-19) і зміни залежностей без помітних для користувачів наслідків (TASK-26). TASK-23 відкладено.

Перевірки: `just check` і `just build`; повний `just smoke` пройшов 30 із 31 кроку (збій запису на кроці 3 — відомий DRAFT-10, повторний `just smoke record` 11/11) і підтвердив TASK-19; пробний запуск https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45842013298 і справжній https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45844358441 зелені. Завантажений exe показує 0.1.2; пробний WAV перевірено за заголовком (48 кГц, стерео, дані ненульові), на слух не слухали — власник вирішив, що заголовка досить. Excavator (https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/45849950392) оновив маніфест до 0.1.2; `scoop update audiocaptor` (0.1.1 → 0.1.2) лишив AudioCaptor-data без змін.
<!-- SECTION:FINAL_SUMMARY:END -->

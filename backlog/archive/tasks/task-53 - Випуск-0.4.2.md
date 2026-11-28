---
id: TASK-53
title: Випуск 0.4.2
status: Done
assignee:
  - '@claude'
created_date: '2026-11-28T04:52:50Z'
updated_date: '2026-11-28T18:23:45Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-9
dependencies:
  - TASK-52
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 15500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Патч-випуск: WAV, перерваний збоєм програми, відкривається з правильною тривалістю (TASK-48); запис, довший за 6 год 12 хв, лишається одним файлом RF64 (TASK-50); гарячу клавішу, яку не можна задати, NVDA називає, а стара лишається (TASK-49); Tab ходить головним вікном по колу без зайвих слів NVDA (TASK-51); гаряча клавіша захоплюється однаково за будь-якої розкладки (TASK-52). Формат `settings.json` не змінюється.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.4.1 (TASK-41). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.4.2 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 Повний `just smoke` проходить на збірці випуску (відомий нестійкий крок DRAFT-10 — після повтору свого сценарію)
- [x] #3 `CHANGELOG.md` має розділ 0.4.2 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [x] #4 Реліз завершено командою `git flow release finish 0.4.2 --notag`: `main` і `develop` містять реліз, а тег `v0.4.2` створив Release workflow
- [x] #5 GitHub Release v0.4.2 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.2
- [x] #6 Маніфест Scoop показує 0.4.2 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
2026-11-28 (сесія TASK-53), частина 1: гілка release/0.4.2, версія 0.4.2 у package.json, Cargo.toml, tauri.conf.json, Cargo.lock; CHANGELOG: розділ [0.4.2] — 2026-11-28 з п'яти нотаток [Unreleased] без змін + за вибором власника пункт «Changed» про повернення на 0.4.1 (RF64), новий порожній [Unreleased], посилання [0.4.2]; коміт 249a4c8. just check (154 тести) і just build зелені; exe 5438976 байт (0.4.1 — 5438464, +512).
Повний just smoke: 113 кроків OK, 1 впав — відомий нестійкий record крок 4 (DRAFT-10); за згодою власника повтор `just smoke record`: 17 з 17, 41 з 41 перевірок NVDA. Решта сценаріїв (launch, nodevice, dialogs, tray, settings, convert, mp3, flac) зелені з першого разу.
Пробний Release на release/0.4.2: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46115210605 (build зелений, publish пропущено).

2026-11-28 (сесія TASK-53), частина 2 з дозволу власника: git stash (backlog/), git flow release finish 0.4.2 --notag --no-fetch --no-push, git stash pop; main = 249a4c8 (той самий коміт, що пройшов пробний запуск), develop = f2bc3bf; обидві відправлено. Гілки release/0.4.2 на GitHub не було (запис про видалення: remote ref does not exist), бо git flow її не пушив... (її я відправляв сам перед пробним запуском; finish прибрав її на GitHub).
Справжній Release: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46115956940 (build і publish зелені). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.2: не попередній, AudioCaptor.exe 5437440 байт, sha256 a99f7dc3d25a2c99374195e0635535b4cdece722291c1bbd4838eb960c7b9451; опис = розділ 0.4.2.
AC #5: exe завантажено в C:\Temp\ac-0.4.2-check (хеш збігається, FileVersion 0.4.2), власна AudioCaptor-data. About (F1, UI Automation) показує v0.4.2. Тестовий запис WAV (mix, 48 кГц, 2 канали) близько 7 с: ffprobe duration 6.72 с. Тестову теку видалено.
AC #6: Excavator вручну https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46116928482 (зелений), маніфест 0.4.2, hash a99f7… збігається. scoop update audiocaptor 0.4.1 -> 0.4.2: exe у Scoop має хеш релізу, FileVersion 0.4.2. SHA256 усіх 6 файлів C:\scoop\persist\audiocaptor до й після однакові (fc: no differences). Нову версію зі Scoop на даних власника не запускали.

2026-11-28, головна сесія: звіт звірено з GitHub (реліз не попередній, тег `v0.4.2` на `249a4c8` = `main`, пробний і справжній Release та Excavator зелені, хеш у маніфесті Scoop = хеш `AudioCaptor.exe` у релізі, опис релізу = розділ 0.4.2 у `CHANGELOG.md`), з протоколами smoke (exe зібрано після коміту з версією) і з відповідями власника в сесії випуску; `just check` на `develop` повторено (154 тести). Уточнення до нотатки частини 2: гілку `release/0.4.2` сесія відправила на GitHub для пробного запуску, а `git flow release finish` сам прибрав її там, тож окрема команда лише сказала, що гілки вже немає. Закрито за вибором власника.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випущено AudioCaptor 0.4.2 — патч із п'ятьма виправленнями: WAV, перерваний збоєм програми, відкривається з правильною тривалістю (TASK-48); запис, довший за 6 год 12 хв, лишається одним файлом RF64 (TASK-50); гарячу клавішу, яку не можна задати, NVDA називає, а стара лишається (TASK-49); гаряча клавіша захоплюється однаково за будь-якої розкладки (TASK-52); Tab ходить головним вікном по колу без зайвих слів NVDA (TASK-51). Версія 0.4.2 стоїть у `package.json`, `Cargo.toml`, `tauri.conf.json` і `Cargo.lock`. `CHANGELOG.md` має розділ 0.4.2 (Fixed і, за вибором власника, Changed — примітка про повернення до 0.4.1, яка не конвертує RF64); опис релізу — той самий розділ. Випуск завершено `git flow release finish 0.4.2 --notag`, `main` і `develop` відправлено, тег `v0.4.2` створив Release workflow. Перший випуск із власним записом WAV замість `hound`: exe 5 437 440 байтів, майже як 0.4.1 (5 438 464).

Перевірки: `just check`, `just build`; повний `just smoke` — 113 кроків з 114 (впав відомий нестійкий крок 4 сценарію `record`, DRAFT-10), повтор `record` — 17 з 17. Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46115210605; справжній: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46115956940; реліз: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.2. Завантажений exe у тестовій теці: About показує v0.4.2, тестовий запис WAV на 7 с ffprobe читає (6,72 с). Excavator: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46116928482, маніфест 0.4.2 із правильним хешем; `scoop update audiocaptor` 0.4.1 → 0.4.2, дані власника без змін (ті самі файли й SHA256). Головна сесія звірила реліз, запуски, маніфест і опис релізу й повторила `just check` на `develop`.

Не перевірено: версію зі Scoop на даних власника не запускали; власник може сам перевірити Tab по колу, налаштування й запис.
<!-- SECTION:FINAL_SUMMARY:END -->

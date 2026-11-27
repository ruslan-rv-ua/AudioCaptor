---
id: TASK-41
title: Випуск 0.4.1
status: Done
assignee:
  - '@claude'
created_date: '2026-11-27T19:21:47Z'
updated_date: '2026-11-27T23:29:54Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-8
dependencies:
  - TASK-47
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 14500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Патч-випуск: Alt+S пояснює, чому запис не почався, коли не вибрано пристрою (TASK-38), а «Запис зупинено» не губиться, коли конвертація закінчується одразу (TASK-39); клавіші й меню браузера більше не перезавантажують вікно й не відкривають друку (TASK-44). Разом із ними йдуть зміни для безпечних і надійних випусків: перевірки збирають програму так само, як випуск (TASK-37), модулі inlang мають закріплені версії (TASK-40). Ще в патчі: час у журналі за годинником Windows (TASK-45) і нечитабельний `settings.json` не затирається пізніше (TASK-46). Підказки й дії діалогу налаштувань озвучуються (TASK-47). Формат `settings.json` не змінюється.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.4.0 (TASK-31). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.4.1 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.4.1 лише зі змінами, помітними користувачам; опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.4.1 --notag`: `main` і `develop` містять реліз, а тег `v0.4.1` створив Release workflow
- [x] #4 GitHub Release v0.4.1 містить `AudioCaptor.exe`; завантажений файл показує версію 0.4.1
- [x] #5 Маніфест Scoop показує 0.4.1 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
2026-11-27: TASK-36 (тест із NVDA не натискає клавіш у чужих вікнах) власник відклав на невизначений термін, тож її прибрано із залежностей, і 0.4.1 виходить без неї. Перевірка з NVDA під час випуску, як і раніше, потребує власника за ПК. Опис етапу 0.4.1 (m-8) ще згадує цей тест: CLI не має команди, щоб змінити опис етапу.

2026-11-27: додано TASK-44 (клавіші й меню браузера), винесену з TASK-43 за рішенням власника.

2026-11-27: за вибором власника в патч додано TASK-45 (колишня DRAFT-12) і TASK-46 (колишня DRAFT-19). Виконані й заархівовані TASK-37, TASK-38 і TASK-39 із переліку залежностей випали, бо CLI не приймає архівних задач у `--depends-on`.

2026-11-27: за вибором власника в патч додано TASK-47 (підказки в діалозі налаштувань). Виконані й заархівовані TASK-40, TASK-44, TASK-45 і TASK-46 із переліку залежностей випали, бо CLI не приймає архівних задач у `--depends-on`; тепер випуск чекає лише на TASK-47.

2026-11-28: код TASK-47 уже в `develop`, але сама задача відкрита: за вибором власника її AC #10 перевіряє повний `just smoke` під час цього випуску (зі згоди власника, власник за ПК). Після успішного повного `just smoke` головна сесія закриває TASK-47.

2026-11-28 (сесія TASK-41), частина 1: гілка release/0.4.1, версія 0.4.1 у package.json, Cargo.toml, tauri.conf.json, Cargo.lock; CHANGELOG: розділ [0.4.1] — 2026-11-28 з усіх шести нотаток [Unreleased] без змін, новий порожній [Unreleased], посилання [0.4.1]; коміт ddb8358. just check (142 тести) і just build зелені; локальний exe 5440000 байт (локальна 0.4.0 — 5431808), FileVersion 0.4.1.
just smoke повний з першого разу: 89 кроків з 89, 206 перевірок NVDA з 206, усі 9 сценаріїв (launch, record, nodevice, dialogs, tray, settings, convert, mp3, flac); dialogs — 35 з 35.
Пробний Release на release/0.4.1: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46079779674 (build зелений, publish пропущено).

2026-11-28 (сесія TASK-41), частина 2 з дозволу власника («Так, публікуй»): git stash (backlog/), git flow release finish 0.4.1 --notag --no-fetch --no-push, git stash pop на develop. main = ddb8358 (той самий коміт, що пройшов пробний запуск), develop = 0b56434 (злиття main); обидві відправлено. Гілки release/0.4.1 на GitHub уже не було (git push --delete: remote ref does not exist).
Справжній Release: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46081030569 (build і publish зелені). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.1: тег v0.4.1 на ddb8358, не попередній, AudioCaptor.exe 5438464 байти (0.4.0 — 5430784, +7680), sha256 61fc11145724d53fc345613eacf560fa2669d578a888bd8b39c03ec7afd2eb83; опис = розділ 0.4.1, кожен пункт одним рядком.
AC #4: exe завантажено в C:\Temp\ac-0.4.1-check (хеш збігається, FileVersion 0.4.1), власна порожня AudioCaptor-data. About (F1, UI Automation) показує v0.4.1 (перша спроба: F1 через 2 с після появи вікна не відкрив About; друга, через 5 с — так). %LOCALAPPDATA%\com.audiocaptor.app не змінилася (491 елемент, той самий найновіший час). Тестову теку видалено.
AC #5: Excavator вручну: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46082058355 (зелений), коміт bff1459 «audiocaptor: Update to version 0.4.1», маніфест 0.4.1, hash 61fc11… збігається. scoop update audiocaptor 0.4.0 -> 0.4.1: hash OK, FileVersion 0.4.1. AudioCaptor-data у C:\scoop\persist\audiocaptor без змін (ті самі 6 файлів і SHA256). Нову версію зі Scoop на даних власника не запускали.

2026-11-28, головна сесія: звіт звірено з GitHub (реліз не попередній, тег `v0.4.1` на `ddb8358` = `main`, обидва запуски Release і Excavator зелені, хеш у маніфесті Scoop = хеш `AudioCaptor.exe` у релізі, опис релізу = розділ 0.4.1 у `CHANGELOG.md`), з протоколом повного smoke (exe зібрано після коміту з версією) і з відповідями власника в сесії випуску; `just check` на `develop` повторено (142 тести). Гілку `release/0.4.1` на GitHub прибрав сам `git flow release finish`: без `--keepremote` він видаляє й гілку на GitHub, навіть із `--no-push` (DEVELOPMENT.md, крок 8 порядку випуску, це передбачає). Закрито за вибором власника.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випущено AudioCaptor 0.4.1 — патч із шістьма виправленнями: «Запис зупинено» не губиться, коли конвертація закінчується одразу (TASK-39); Alt+S пояснює, чому запис не почався без вибраного пристрою (TASK-38); клавіші й меню браузера більше не діють у вікні (TASK-44); час у журналі — за годинником Windows (TASK-45); нечитабельний `settings.json` не затирається першою зміною (TASK-46); діалог налаштувань озвучує підказки й дії (TASK-47). Версія 0.4.1 стоїть у `package.json`, `Cargo.toml`, `tauri.conf.json` і `Cargo.lock`. `CHANGELOG.md` має розділ 0.4.1 (Fixed); опис релізу — той самий розділ. Випуск завершено `git flow release finish 0.4.1 --notag`, `main` і `develop` відправлено, тег `v0.4.1` створив Release workflow. Це перший випуск після TASK-37 і TASK-40: exe більший за 0.4.0 лише на 7,5 КБ нового коду.

Перевірки: `just check`, `just build`; повний `just smoke` з першого разу — 89 кроків з 89, 206 перевірок NVDA з 206 (`dialogs` — 35 з 35, тож закрито й TASK-47). Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46079779674; справжній: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/46081030569; реліз: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.4.1 (AudioCaptor.exe, 5 438 464 байти). Завантажений exe у тестовій теці: About показує v0.4.1. Excavator: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/46082058355, маніфест 0.4.1 із правильним хешем; `scoop update audiocaptor` 0.4.0 → 0.4.1, дані власника без змін (ті самі файли й SHA256). Головна сесія звірила реліз, запуски, маніфест і опис релізу й повторила `just check` на `develop`.

Не перевірено: версію зі Scoop на даних власника не запускали. Власник може сам перевірити TASK-47: відкрити Settings і перейти Tab до списку Language — NVDA має прочитати «Change takes effect after restart».
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-13
title: Випуск 0.2.0
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:32:40Z'
updated_date: '2026-11-26T20:25:54Z'
labels:
  - release
  - model-sonnet
  - effort-medium
milestone: m-4
dependencies:
  - TASK-35
references:
  - DEVELOPMENT.md
  - CHANGELOG.md
priority: medium
type: task
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Випуск із новою можливістю: записи можна автоматично зберігати в Opus (TASK-28). Формат `settings.json` переходить на версію 6, тому в `CHANGELOG.md` має бути сказано, що відбувається з налаштуваннями під час оновлення і чи можна повернутися до 0.1.x з тим самим файлом.

Порядок випуску — у `DEVELOPMENT.md`, так само як для 0.1.2 (TASK-33). Push і запуск workflow потребують дозволу власника.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Версія 0.2.0 стоїть у `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` і `src-tauri/Cargo.lock`; `just check` проходить
- [x] #2 `CHANGELOG.md` має розділ 0.2.0 лише зі змінами, помітними користувачам, і каже, що відбувається з `settings.json` під час оновлення та повернення до 0.1.x; опис релізу на GitHub — той самий розділ
- [x] #3 Реліз завершено командою `git flow release finish 0.2.0 --notag`: `main` і `develop` містять реліз, а тег `v0.2.0` створив Release workflow
- [x] #4 GitHub Release v0.2.0 містить `AudioCaptor.exe`; завантажений файл показує версію 0.2.0, а пробний запис в Opus відтворюється
- [x] #5 Маніфест Scoop показує 0.2.0 після запуску Excavator; після `scoop update audiocaptor` профілі й налаштування користувача на місці
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
1. Формат settings.json: 0.2.0 при першому запуску переводить 5 -> 6 і записує файл; 0.1.x читає версію 6 (лише попередження в журналі), ігнорує поле формату, а збереживши налаштування, втрачає його.
2. git flow release start 0.2.0; версія 0.2.0 у package.json, Cargo.toml, tauri.conf.json; just check (Cargo.lock).
3. CHANGELOG: розділ [0.2.0] з датою, порожній [Unreleased], пункти про оновлення settings.json і повернення до 0.1.x, посилання внизу; коміт; just build.
4. З дозволу власника: повний just smoke.
5. З дозволу власника: push release/0.2.0 і пробний запуск Release.
6. Після дозволу: release finish --notag, push main і develop, справжній запуск, перевірка релізу (exe, About, запис Opus через mpv/FFmpeg), Excavator і scoop update.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-26: додано залежність від TASK-35 (вибір власника: 0.2.0 виходить із виправленням повторного Alt+I). Залежність від TASK-33 (випуск 0.1.2, виконано й заархівовано) при цьому випала: CLI не приймає архівних задач у `--depends-on`.

2026-11-26: TASK-34 скасовано (decision-9: програма не показує ліцензій сторонніх бібліотек), тож випуск 0.2.0 на неї не чекає. Залежність тепер лише одна, TASK-35. Виконана й заархівована TASK-28 із переліку випала, бо CLI не приймає архівних задач у `--depends-on`.

2026-11-26 (сесія TASK-13), частина 1: гілка release/0.2.0, версія 0.2.0 у package.json, Cargo.toml, tauri.conf.json, Cargo.lock; CHANGELOG: розділ [0.2.0] — 2026-11-26 з пунктами про перехід settings.json з версії 5 на 6 під час оновлення і про повернення до 0.1.x (0.1.x читає версію 6, лише пише попередження в журнал, ігнорує поле формату й, зберігаючи налаштування, втрачає його); коміт ce3fbfd. just check (111 тестів) і just build зелені, exe FileVersion 0.2.0.
Повний just smoke: 50 із 53 кроків; record крок 4 (DRAFT-10) і, вперше, tray крок 2 (Escape не сховав вікно) — подробиці в DRAFT-10. Повторний just smoke record tray: 15/15.
Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45883395873 (зелений, publish пропущено).

2026-11-26 (сесія TASK-13), частина 2 (дозвіл власника — «роби все аж поки не оновиш за допомогою scoop»): git flow release finish 0.2.0 --notag (перехід на main заважали незакомічені нотатки backlog/, тож їх тимчасово скопійовано в робочу теку сесії й повернуто на develop після finish; stash не використовувався). main = ce3fbfd, develop = a3df805 (злиття main), обидві відправлено; гілки release/0.2.0 на GitHub уже не було.
Справжній запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45888782854 (build і publish зелені). Реліз https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.2.0: тег v0.2.0 на ce3fbfd, не попередній, AudioCaptor.exe 5276672 байти, sha256 9cad22121a15b886810f77f396fc4b767a421fa294d9b52a1649714a801b48dc; опис = розділ 0.2.0, без розривів посеред речень.
AC #4: exe завантажено в C:\Temp\ac-0.2.0-check (хеш збігається, FileVersion 0.2.0), запущено з власною AudioCaptor-data; «About» показує v0.2.0; settings.json версії 5 з профілем Opus мовлення після старту має версію 6 і формат OpusSpeech. Запис системного звуку (тон 440 Гц, 7 с, старт і стоп глобальною Pause) дав loopback_….opus 28150 байт, WAV видалено після конвертації; FFmpeg: ogg/opus, 48 кГц, моно, 7,3 с, без помилок, max -5,6 дБ; mpv програв до кінця. %LOCALAPPDATA%\com.audiocaptor.app не змінилася. Alt+S, надіслані скриптом після закриття «About», до програми не дійшли (фокус), тому використано Pause; наприкінці Alt+F4 теж не дійшов, і тестову копію закрито примусово.
AC #5: Excavator вручну: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/45890469205 (зелений), коміт «audiocaptor: Update to version 0.2.0», маніфест 0.2.0, hash 9cad2… збігається. scoop update audiocaptor (0.1.2 -> 0.2.0): hash OK, FileVersion 0.2.0. AudioCaptor-data у persist без змін (ті самі 3 файли й хеші, settings.json 716 байт, версія 5, 1 профіль). Нову версію зі Scoop не запускали: її перший старт переведе settings.json на версію 6.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Випущено AudioCaptor 0.2.0: запис в Opus (TASK-28) і виправлення повторного Alt+I (TASK-35). Версія 0.2.0 стоїть у `package.json`, `Cargo.toml`, `tauri.conf.json` і `Cargo.lock`. `CHANGELOG.md` має розділ 0.2.0 із пунктами про перехід `settings.json` з версії 5 на 6 і про повернення до 0.1.x; опис релізу — той самий розділ. Випуск завершено `git flow release finish 0.2.0 --notag`, `main` і `develop` відправлено, тег `v0.2.0` створив Release workflow.

Перевірки: `just check` і `just build`; повний `just smoke` — 50 кроків з 53 (record, крок 4 — DRAFT-10; tray, крок 2 — уперше, теж записано в DRAFT-10), повторний `just smoke record tray` — 15 з 15. Пробний запуск: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45883395873; справжній: https://github.com/ruslan-rv-ua/AudioCaptor/actions/runs/45888782854; реліз: https://github.com/ruslan-rv-ua/AudioCaptor/releases/tag/v0.2.0 (AudioCaptor.exe, 5 276 672 байти). Завантажений exe у тестовій теці: About показує v0.2.0, тестовий `settings.json` перейшов з версії 5 на 6, пробний запис в Opus FFmpeg розкодовує без помилок, mpv програє до кінця. Excavator: https://github.com/ruslan-rv-ua/scoop-bucket/actions/runs/45890469205, маніфест 0.2.0; `scoop update audiocaptor` 0.1.2 → 0.2.0, дані власника без змін. Головна сесія повторила `just check` на `develop`.

Не перевірено: нову версію зі Scoop на даних власника не запускали; її перший старт переведе `settings.json` на версію 6.
<!-- SECTION:FINAL_SUMMARY:END -->

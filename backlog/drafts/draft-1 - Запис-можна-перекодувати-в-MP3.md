---
id: DRAFT-1
title: Запис можна перекодувати в MP3
status: Draft
assignee: []
created_date: '2026-11-23T11:30:07Z'
labels:
  - backend
  - audio
  - model-opus
  - effort-high
milestone: m-3
dependencies: []
priority: low
type: feature
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
MP3 — найпоширеніший формат для обміну записами: файли у 8–12 разів менші за WAV, і їх приймає будь-який сервіс чи програвач. Дослідження doc-1 радить додати MP3 після FLAC.

Чекає на рішення власника про ліцензію. Єдиний зрілий кодер — LAME (крейт `mp3lame-encoder`, прив'язка до бібліотеки на C). LAME поширюється за LGPL-2.1, а AudioCaptor — за MIT і збирається в один exe зі статичним зв'язуванням. За статичного зв'язування LGPL вимагає або надати об'єктні файли для перезбирання, або зв'язувати динамічно, тобто класти окрему DLL поруч з exe, що суперечить «одному файлу». Кодери MP3 на чистому Rust у квітні 2026 були незрілі (`shine-rs`, `oxideav-mp3`). Патенти на MP3 скінчилися 2017 року, тож сам формат вільний.

Що має вирішити власник:

- чи прийнятний LGPL-кодер у складі AudioCaptor і в якому вигляді (об'єктні файли чи DLL);
- або чекати на зрілий кодер на чистому Rust;
- чи потрібні інші формати зі втратами (Opus, OGG Vorbis); дослідження вважає їхні кодери в Rust незрілими.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Перевірки фронтенду проходять: `pnpm vite:build`, `pnpm check`
- [ ] #2 Перевірки Rust проходять: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] #3 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [ ] #4 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [ ] #5 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [ ] #6 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

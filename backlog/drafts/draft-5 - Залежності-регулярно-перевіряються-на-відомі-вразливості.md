---
id: DRAFT-5
title: Залежності регулярно перевіряються на відомі вразливості
status: Draft
assignee: []
created_date: '2026-11-23T11:30:23Z'
labels:
  - security
  - ci
  - model-sonnet
  - effort-low
milestone: m-3
dependencies: []
priority: low
type: chore
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
За сім місяців паузи в залежностях назбиралося 6 відомих вразливостей у Rust-крейтах і 33 в JS-пакетах (перевірка 2026-11-23), і ніщо про це не повідомило: ні CI, ні порядок випуску не запускають `cargo audit` чи `pnpm audit`.

Що має вирішити власник — один із варіантів:

- перевірка стає кроком порядку випуску (вручну або в Release workflow перед збіркою);
- окремий workflow за розкладом, який падає, коли з'являється нова вразливість;
- лишити як є й перевіряти за нагоди.
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

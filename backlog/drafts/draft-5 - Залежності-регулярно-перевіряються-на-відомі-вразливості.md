---
id: DRAFT-5
title: Залежності регулярно перевіряються на відомі вразливості
status: Draft
assignee: []
created_date: '2026-11-23T11:30:23Z'
updated_date: '2026-11-27T19:21:49Z'
labels:
  - security
  - ci
  - model-sonnet
  - effort-low
milestone: m-11
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25 (TASK-26): `cargo audit` 0.23.0 з локальною базою (без оновлення) на `src-tauri/Cargo.lock`: вразливостей немає, 6 попереджень — `chrono` 0.4.45 (unmaintained, RUSTSEC-2026-0543), `zbus_names`, `zvariant`, `zvariant_derive`, `proc-macro-error` (unmaintained) і `glib` 0.18.5 (unsound). Жоден із цих крейтів у збірку для Windows не потрапляє: `cargo tree -i <крейт> --target x86_64-pc-windows-msvc -e all` нічого не знаходить. Вони лише записані в Cargo.lock: Tauri тягне їх для Linux, а `chrono` після TASK-26 лишився там через `serde_with` 3.25.0 (остання версія), яку бере tauri-utils 2.10.4. Типова можливість `std` у `serde_with` згадує `chrono` як слабку необов'язкову залежність (`chrono_0_4?/clock`), а такі залежності Cargo записує в Cargo.lock, навіть коли нічого їх не вмикає (так само там лежить незібраний `jiff` 1.0.2). Тож перевірка, яка читає Cargo.lock, має відрізняти такі записи від того, що справді збирається: наприклад, падати лише на вразливостях, а попередження unmaintained і unsound показувати, або тримати список винятків для `cargo audit`.
<!-- SECTION:NOTES:END -->

---
id: TASK-1
title: Перевірки коду знову проходять, а версія Rust закріплена
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:28:13Z'
updated_date: '2026-11-23T11:53:35Z'
labels:
  - backend
  - ci
  - model-sonnet
  - effort-medium
milestone: m-0
dependencies: []
references:
  - src-tauri/src/audio/capture.rs
  - src-tauri/src/settings.rs
  - .github/workflows/ci.yml
priority: high
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Автоматична перевірка на GitHub (CI) червона з 2026-08-26: запуски 32940169820, 32943970973 і 32945514129 впали на кроці clippy. Код тоді не змінювався. Причина в тому, що CI щоразу встановлює найновіший Rust (`dtolnay/rust-toolchain@stable`), а Rust 1.98 додав до clippy правило `chunks_exact_to_as_chunks`. CI запускає clippy з `-D warnings`, тож кожне зауваження стає помилкою.

Перевірено локально 2026-11-23 з Rust 1.99.1:

- три зауваження `chunks_exact_to_as_chunks` у функції `bytes_to_f32` (`src-tauri/src/audio/capture.rs`, рядки 261, 265, 269);
- з кодом тестів (`--all-targets`, як вимагає Definition of Done) ще чотири зауваження `field_reassign_with_default` у тестах `src-tauri/src/settings.rs` (рядки 152, 190, 199, 276);
- інших зауважень немає, усі 64 тести проходять.

`bytes_to_f32` перетворює байти з аудіопристрою на семпли. Помилку з розширенням знака для 24 біт у ній виправлено в коміті 3624beb. Після змін функція має повертати ті самі значення. Тести є для 16 біт (додатний і від'ємний семпл, тиша), для 32 біт і для 24 біт (додатний семпл і тиша). Тесту для від'ємного 24-бітного семпла, тобто саме для виправленої помилки, немає.

Рішення власника від 2026-11-23: версію Rust закріпити у `rust-toolchain.toml` (1.99.1, як в axygen-totp і revisor), щоб новий випуск Rust не міг сам зробити перевірки червоними. Оновлення версії Rust відтоді — окрема свідома зміна.

Доки ця задача не виконана, пункт Definition of Done про перевірки Rust не може пройти в жодній іншій задачі.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Команда `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` завершується без зауважень
- [x] #2 Команда `cargo test --manifest-path src-tauri/Cargo.toml` проходить; тести `bytes_to_f32` для 16, 24 і 32 біт лишилися без змін
- [x] #3 У корені репозиторію є `rust-toolchain.toml` з точною версією 1.99.1 і компонентами `clippy` та `rustfmt`; коментар у файлі пояснює, навіщо версію закріплено
- [x] #4 Команда `cargo --version` у теці проєкту показує 1.99.1, тобто версію з `rust-toolchain.toml`
- [x] #5 Тест перевіряє від'ємний 24-бітний семпл у `bytes_to_f32` і падає на варіанті з помилкою, яку виправив коміт 3624beb
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Перевірка головної сесії, 2026-11-23 (коміт e2de8a4 на гілці `feature/task-1-checks-pass`):

- Змінено три файли: `rust-toolchain.toml`, `src-tauri/src/audio/capture.rs`, `src-tauri/src/settings.rs`. У `capture.rs` змінився лише спосіб поділу байтів на частини (`as_chunks` замість `chunks_exact`); обчислення семплів і тести не чіпано. У `settings.rs` змінено лише код тестів.
- Перевірки запущено наново на гілці: `pnpm vite:build` і `pnpm check` — 0 помилок; clippy з `--all-targets -- -D warnings` — без зауважень; `cargo test` — 64 з 64; `cargo --version` у теці проєкту — 1.99.1.
- Знахідка: тесту для від'ємного 24-бітного семпла немає. Коміт 3624beb виправив помилку, але тесту не додав. Опис задачі помилково казав, що цей випадок покрито тестами; опис виправлено. Для цього випадку однаковість результатів підтверджено лише читанням коду: обчислення для 24 біт не змінилося.
- CI на GitHub для цієї гілки не запускався: вручну запустити його ще не можна, це додасть TASK-4.

2026-11-23: власник погодив розширення задачі — тест для від'ємного 24-бітного семпла додається в цю ж гілку (критерій #5). Запуск CI через pull request власник не замовляв: CI перевірить TASK-4.

Повторна перевірка головної сесії, 2026-11-23, перед завершенням гілки:

- Коміт 11c4a60 додає два тести для від'ємних 24-бітних семплів (−4194304 дає приблизно −0,5; байти `FF FF FF`, тобто −1, дають −1/8388608). Код функції між e2de8a4 і 11c4a60 не змінився.
- Ті самі перевірки запущено окремо проти старого варіанта з помилкою (`i32::from_le_bytes([c[0], c[1], c[2], 0]) >> 8`): обидва тести падають, бо результат додатний (0,0059 і 0,0078).
- Коміт ae4f04d (головна сесія): у вимогах `DEVELOPMENT.md` замість «stable toolchain» сказано, що версію Rust закріплено в `rust-toolchain.toml`.
- На вершині гілки: clippy з `--all-targets -- -D warnings` без зауважень; `cargo test` — 66 з 66; `pnpm vite:build` і `pnpm check` — 0 помилок. Після завершення гілки дерево файлів у `develop` те саме, а `cargo --version` у теці проєкту показує 1.99.1.
- Гілку завершено командою `git flow feature finish task-1-checks-pass --no-fetch --no-push` (злиття без окремого коміту злиття).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Перевірки коду знову проходять, а версія Rust більше не змінюється сама.

- Сім зауважень clippy прибрано. У `bytes_to_f32` (`src-tauri/src/audio/capture.rs`) байти діляться на частини через `as_chunks` замість `chunks_exact`; обчислення семплів не змінилося. Чотири тести в `src-tauri/src/settings.rs` створюють `Settings` одним виразом.
- Версію Rust закріплено в `rust-toolchain.toml`: 1.99.1 з компонентами `clippy` і `rustfmt`. Вимоги в `DEVELOPMENT.md` про це кажуть.
- Додано два тести для від'ємних 24-бітних семплів. Саме цей випадок виправив коміт 3624beb, але тестів для нього не було. Проти старого варіанта з помилкою обидва тести падають.

Перевірено: clippy з `--all-targets -- -D warnings` без зауважень; `cargo test` — 66 з 66; `pnpm vite:build` і `pnpm check` — 0 помилок; `cargo --version` у теці проєкту — 1.99.1. Коміти e2de8a4, 11c4a60 і ae4f04d; гілку `feature/task-1-checks-pass` завершено в `develop`.

Пункти Definition of Done про тексти інтерфейсу, перевірку з NVDA і CHANGELOG не мали до чого застосовуватися: задача не змінює ні інтерфейсу, ні поведінки програми.

Не перевірено: CI на GitHub із закріпленою версією Rust. Вручну запустити його ще не можна; це перевірить TASK-4, де є критерії про версію Rust із `rust-toolchain.toml` і про зелений ручний запуск.
<!-- SECTION:FINAL_SUMMARY:END -->

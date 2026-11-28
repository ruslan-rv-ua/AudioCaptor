---
id: TASK-61
title: Логіку інтерфейсу покрито тестами
status: To Do
assignee: []
created_date: '2026-11-23T11:30:23Z'
updated_date: '2026-11-28T18:46:12Z'
labels:
  - frontend
  - testing
  - model-sonnet
  - effort-medium
milestone: m-11
dependencies: []
priority: low
type: enhancement
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Частина на Rust має 154 тести, а вебчастина — жодного: `pnpm check` перевіряє лише типи. При цьому у вебчастині є логіка, від якої залежить поведінка:

- готовність до запису й підказки (`canRecord`, `needsMic`, `needsLoopback`, `readinessHint` у `src/lib/stores/recording.svelte.ts`) повторюють правила з Rust (`OutputMode::needs_mic` і `needs_loopback`). Перевірка якості коду у квітні 2026 знайшла помилку саме в такому повторенні: режим «Окремі файли» було пропущено в перевірці гарячої клавіші;
- назви гарячої клавіші з фізичних клавіш (`src/lib/utils/hotkeyName.ts`, TASK-52).

Помилки в цих правилах зараз помічає лише людина з NVDA. Рішення власника (2026-11-28): задача в етапі «Супровід».
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Вебчастина має тестовий рушій (Vitest) як залежність для розробки, і тести запускаються однією командою
- [ ] #2 Тести перевіряють `canRecord`, `needsMic`, `needsLoopback` і `readinessHint` для всіх шести режимів, без мікрофона й без пристрою системного звуку, і їхні відповіді збігаються з правилами Rust (`OutputMode::needs_mic`, `needs_loopback`)
- [ ] #3 Тести перевіряють `hotkeyKeyName` і `refusedKeyName`: літери й цифри за будь-якої розкладки, слова для розділових знаків, цифрову клавіатуру з Num Lock і без нього, клавіші, яких не можна задати
- [ ] #4 `just check` запускає ці тести, і CI (`.github/workflows/ci.yml`) — теж
- [ ] #5 `DEVELOPMENT.md` пояснює, як запускати тести вебчастини й куди додавати нові
<!-- AC:END -->

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
2026-11-28, головна сесія: чернетку DRAFT-4 перетворено на задачу за рішенням власника, етап «Супровід». Питання з чернетки, чи запускати тести разом з іншими локальними перевірками, записано як AC #4 (так); власник може це змінити. Задача додає залежність для розробки, тож перед закриттям потрібен CI, запущений вручну.
<!-- SECTION:NOTES:END -->

---
id: TASK-6
title: >-
  Залежності оновлено в межах поточних версій, відомі вразливості закрито або
  пояснено
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:30:58Z'
updated_date: '2026-11-23T20:09:29Z'
labels:
  - backend
  - frontend
  - security
  - model-opus
  - effort-medium
milestone: m-0
dependencies:
  - TASK-1
  - TASK-14
references:
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - package.json
  - pnpm-lock.yaml
priority: medium
type: chore
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Залежності востаннє оновлювалися у квітні 2026 і ніколи не перевірялися на відомі вразливості. Перевірка 2026-11-23 показала таке.

**Rust.** `cargo audit` 0.23.0 проти `src-tauri/Cargo.lock` (599 крейтів): 6 вразливостей у 5 крейтах і 20 попереджень. Ці крейти компілюються в `AudioCaptor.exe`.

| Крейт | Версія | Запис | Виправлено у |
|---|---|---|---|
| cookie | 0.18.1 | RUSTSEC-2026-0594 | 0.18.3 |
| quick-xml | 0.38.4 | RUSTSEC-2026-0194, RUSTSEC-2026-0195 | 0.41.0 |
| ringbuf | 0.4.8 | RUSTSEC-2026-0293 | 0.5.2 |
| rkyv | 0.7.46 | RUSTSEC-2026-0235 | 0.8.17 |
| url | 2.5.8 | RUSTSEC-2026-0592 | 2.5.9 |

`ringbuf` — пряма залежність: через її буфери звук іде від потоків захоплення до мікшера (`audio/capture.rs`, `audio/mixer.rs`). Виправлення є лише у версії 0.5.2 і новіших, тобто з несумісними змінами API. Вразливість стосується випадку, коли падає `Drop` елемента; у буферах AudioCaptor лежать `f32`. Решта чотири крейти приходять через Tauri та його залежності.

Серед попереджень — `chrono` (RUSTSEC-2026-0543, «unmaintained»), теж пряма залежність: нею формується мітка часу в назві файлу запису (`lib.rs`).

`cargo update --dry-run` пропонує сумісні оновлення, зокрема `tauri` 2.10.3 → 2.12.7, `tauri-build` 2.5.6 → 2.7.6 і плагіни: `dialog` 2.7.0 → 2.8.4, `global-shortcut` 2.3.1 → 2.4.1, `log` 2.8.0 → 2.10.2, `single-instance` 2.4.0 → 2.5.5.

**JS.** `pnpm audit`: 33 вразливості (2 low, 17 moderate, 14 high), наприклад `vite` 8.0.3 (GHSA-9jrq-w75r-8gcw, виправлено у 8.1.6) і `devalue` через `svelte` (GHSA-wf3x-273g-mvxv, виправлено у 5.9.3). Майже все це інструменти збірки, які в exe не потрапляють; сам Svelte потрапляє. `pnpm outdated` показує сумісні оновлення: `vite` → 8.3.8, `svelte` → 5.59.1, `svelte-check` → 4.7.8, `@sveltejs/vite-plugin-svelte` → 7.3.1, `@tauri-apps/api` → 2.12.5, `@tauri-apps/cli` → 2.12.6, `@tauri-apps/plugin-dialog` → 2.8.4.

Пакети Tauri з боку Rust (`tauri`, плагіни) і з боку JS (`@tauri-apps/api`, плагіни) мають лишатися на однакових мінорних версіях, тому обидві частини — в одній задачі.

Поза межами задачі — переходи на нові мажорні версії (DRAFT-2) і питання, чи робити таку перевірку регулярною (DRAFT-5). Виняток — `ringbuf`, якщо без переходу вразливість не закрити.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Вивід `cargo audit` і `pnpm audit` до і після змін збережено в нотатках задачі
- [x] #2 Кожну з шести вразливостей у Rust-крейтах або закрито оновленням, або в нотатках пояснено, чому вона не стосується AudioCaptor
- [x] #3 `pnpm audit` не показує вразливостей рівня high; кожну, що лишилася, перелічено в нотатках із причиною
- [x] #4 Кожне попередження `cargo audit`, що лишилося, перелічено в нотатках із причиною
- [x] #5 Крейт `tauri` і пакет `@tauri-apps/api` мають однакову мінорну версію, а `pnpm tauri build --no-bundle` не скаржиться на розбіжність версій
- [x] #6 Після оновлень `pnpm tauri build --no-bundle` дає `AudioCaptor.exe`, який запускається, а пробний запис у режимі «Мікс» дає WAV, що відтворюється
- [x] #7 У `CHANGELOG.md` запис є лише тоді, коли закрита вразливість справді стосувалася програми або оновлення змінило її поведінку
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
1. Базовий результат до оновлень: exe з `just build`, прогін `just smoke` з дозволу власника, вивід `cargo audit` і `pnpm audit` — у нотатки.
2. Rust: `cargo update` у межах поточних версій (зокрема `tauri` 2.12 і плагіни). `ringbuf` 0.4.8 → 0.5.2 (власник погодив 2026-11-23): різниця між версіями — перехід крейта на Rust 2024 і саме виправлення, код AudioCaptor не змінюється.
3. JS: `pnpm update` у межах діапазонів з `package.json`; `@tauri-apps/api` і `@tauri-apps/cli` — на тій самій мінорній версії, що крейт `tauri`, `@tauri-apps/plugin-dialog` — що крейт `tauri-plugin-dialog`. Якщо високі вразливості в транзитивних пакетах лишаться — закрити їх найменшим кроком у межах сумісних версій.
4. Для кожної вразливості й попередження, що лишилися, записати причину в нотатки (чи потрапляє крейт або пакет у `AudioCaptor.exe`).
5. Перевірки: `just check`; `pnpm tauri build --no-bundle` без скарг на розбіжність версій; запуск exe і пробний запис у режимі «Мікс» (WAV відтворюється); `just smoke` з дозволу власника — не гірше за базовий результат.
6. CHANGELOG — лише якщо закрита вразливість справді стосувалася програми або змінилася її поведінка.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Базовий результат до оновлень (2026-11-23)

Exe зібрано `just build` о 20:25 з develop 8046d66 (без скарг на версії). `just smoke` о 20:38 з дозволу власника: 21 крок OK, 5 упали, перевірок NVDA 29 з 34, клавіатура зайнята 66 с. Упали відомі вади: повернення фокусу після чотирьох діалогів (TASK-15) і повторний запуск програми, схованої в трей (TASK-9). Той самий результат, що й у прогонах TASK-14.

### `cargo audit` до (cargo-audit 0.23.0, `src-tauri/Cargo.lock`, 599 крейтів; по рядку на запис)

Підсумок: `error: 6 vulnerabilities found!`, `warning: 20 allowed warnings found`.

```text
vulnerabilities: 6
- cookie 0.18.1 — RUSTSEC-2026-0594 — Private cookie nonces can repeat in processes that fork — fix >=0.18.3
- quick-xml 0.38.4 — RUSTSEC-2026-0194 — Quadratic run time when checking a start tag for duplicate attribute names — fix >=0.41.0
- quick-xml 0.38.4 — RUSTSEC-2026-0195 — Unbounded namespace-declaration allocation in `NsReader` enables memory-exhaustion denial of service — fix >=0.41.0
- ringbuf 0.4.8 — RUSTSEC-2026-0293 — Double free / use-after-free in `Consumer::skip` and `Consumer::clear` when an element's `Drop` panics — fix >=0.5.2
- rkyv 0.7.46 — RUSTSEC-2026-0235 — Insufficient archive validation can cause out-of-bounds reads in archives containing Rc/Arc — fix >=0.8.17
- url 2.5.8 — RUSTSEC-2026-0592 — Stack overflow in `Url::origin()` on nested `blob:` URLs — fix >=2.5.9
warnings: 20
- unmaintained: chrono 0.4.44 — RUSTSEC-2026-0543 — chrono is unmaintained
- unmaintained: fxhash 0.2.1 — RUSTSEC-2025-0057 — fxhash - no longer maintained
- unmaintained: pin-utils 0.1.0 — RUSTSEC-2026-0441 — `pin-utils` is unmaintained
- unmaintained: proc-macro-error 1.0.4 — RUSTSEC-2024-0370 — proc-macro-error is unmaintained
- unmaintained: unic-char-property 0.9.0 — RUSTSEC-2025-0081 — `unic-char-property` is unmaintained
- unmaintained: unic-char-range 0.9.0 — RUSTSEC-2025-0075 — `unic-char-range` is unmaintained
- unmaintained: unic-common 0.9.0 — RUSTSEC-2025-0080 — `unic-common` is unmaintained
- unmaintained: unic-ucd-ident 0.9.0 — RUSTSEC-2025-0100 — `unic-ucd-ident` is unmaintained
- unmaintained: unic-ucd-version 0.9.0 — RUSTSEC-2025-0098 — `unic-ucd-version` is unmaintained
- unmaintained: zbus_names 4.3.1 — RUSTSEC-2026-0371 — `zbus_names` is unmaintained
- unmaintained: zvariant 5.10.0 — RUSTSEC-2026-0370 — `zvariant` is unmaintained
- unmaintained: zvariant_derive 5.10.0 — RUSTSEC-2026-0372 — `zvariant_derive` is unmaintained
- unsound: anyhow 1.0.102 — RUSTSEC-2026-0190 — Unsoundness in `Error::downcast_mut()` — fix >=1.0.103
- unsound: event-listener 5.4.1 — RUSTSEC-2026-0221 — `event-listener` allows `!Send` tags to cross thread boundaries via `StackSlot` — fix >=5.4.2
- unsound: glib 0.18.5 — RUSTSEC-2024-0429 — Unsoundness in `Iterator` and `DoubleEndedIterator` impls for `glib::VariantStrIter` — fix >=0.20.0
- unsound: rand 0.7.3 — RUSTSEC-2026-0097 — Rand is unsound with a custom logger using `rand::rng()` — fix >=0.10.1 | <0.10.0, >=0.9.3 | <0.9.0, >=0.8.6
- unsound: rand 0.8.5 — RUSTSEC-2026-0097 — Rand is unsound with a custom logger using `rand::rng()` — fix >=0.10.1 | <0.10.0, >=0.9.3 | <0.9.0, >=0.8.6
- unsound: rustix 1.1.4 — RUSTSEC-2026-0486 — xattr functions mark unwritten bytes as initialized when given a zero-length buffer — fix >=1.1.6
- unsound: tokio 1.50.0 — RUSTSEC-2026-0386 — `AsyncFd::new` and related constructors violate I/O safety — fix >=1.51.6, <1.52.0 | >=1.53.3
- yanked: async-executor 1.14.0
```

### `pnpm audit` до (по рядку на запис; via — прямий пакет, через який він приходить)

Підсумок: `33 vulnerabilities found`, `Severity: 2 low | 17 moderate | 14 high`.

```text
33 vulnerabilities found. Severity: 2 low | 17 moderate | 14 high | 0 critical
- high: devalue 5.6.4 — GHSA-77vg-94rm-hx3p — Svelte devalue: DoS via sparse array deserialization — fix >=5.8.1 — via svelte
- high: devalue 5.6.4 — GHSA-j22f-vq7h-c4qm — devalue: `stringify`/`uneval` serialize shared memory — fix >=5.9.3 — via svelte
- high: devalue 5.6.4 — GHSA-mcm9-63f2-9j32 — devalue: Repeated primitive strings cause quadratic expansion in uneval — fix >=5.9.3 — via svelte
- high: devalue 5.6.4 — GHSA-r9w8-h9r3-54w4 — devalue: Custom ArrayBuffer revivers can bypass typed-array allocation validation — fix >=5.9.3 — via svelte
- high: kysely 0.28.15 — GHSA-pv5w-4p9q-p3v2 — Kysely: JSON-path traversal injection via unsanitized path-leg metacharacters in `JSONPathBuilder.key()` / `.at()` — fix >=0.28.17 — via @inlang/paraglide-js
- high: nanoid 3.3.11 — GHSA-28wg-ghj8-5hjv — nanoid: non-secure generators can loop indefinitely with negative size — fix >=3.3.16 — via vite
- high: nanoid 3.3.11 — GHSA-2v37-7h3g-55p8 — nanoid: custom generators can loop indefinitely when size is zero — fix >=3.3.18 — via vite
- high: nanoid 3.3.11 — GHSA-xwg4-73v4-xw9w — nanoid: Integer Overflow or Wraparound — fix >=3.3.12 — via vite
- high: postcss 8.5.8 — GHSA-6g55-p6wh-862q — PostCSS: Arbitrary file read and information disclosure via attacker-controlled sourceMappingURL in CSS comments — fix >=8.5.12 — via vite
- high: postcss 8.5.8 — GHSA-r28c-9q8g-f849 — PostCSS: Path Traversal in Previous Source Map Auto-Loading (sourceMappingURL) leads to Arbitrary .map File Disclosure — fix >=8.5.18 — via vite
- high: source-map-js 1.2.1 — GHSA-68fv-2mgg-jv7q — source-map-js allows event-loop denial of service through indexed source-map section offsets — fix >=1.2.2 — via vite
- high: vite 8.0.3 — GHSA-v2wj-q39q-566r — Vite: `server.fs.deny` bypassed with queries — fix >=8.0.5 — via vite
- high: vite 8.0.3 — GHSA-p9ff-h696-f583 — Vite Vulnerable to Arbitrary File Read via Vite Dev Server WebSocket — fix >=8.0.5 — via vite
- high: vite 8.0.3 — GHSA-587h-7wmj-h5pm — launch-editor: `fs.existsSync()` existence check can be bypassed on Windows — fix >=8.1.6 — via vite
- moderate: devalue 5.6.4 — GHSA-9rgm-9g3h-6x36 — Svelte devalue: DoS via malformed input — fix >=5.9.2 — via svelte
- moderate: devalue 5.6.4 — GHSA-hx4r-w6wj-j8fg — devalue: Residual sparse-array CPU amplification in uneval — fix >=5.9.3 — via svelte
- moderate: devalue 5.6.4 — GHSA-4q55-j62x-fr9h — devalue: Malformed null-prototype object keys bypass __proto__ rejection via property-key coercion — fix >=5.9.3 — via svelte
- moderate: postcss 8.5.8 — GHSA-qx2v-qp2m-jg93 — PostCSS has XSS via Unescaped </style> in its CSS Stringify Output — fix >=8.5.10 — via vite
- moderate: postcss 8.5.8 — GHSA-fxqj-rqcc-2cmp — PostCSS: incomplete fix of GHSA-6g55-p6wh-862q — attacker-controlled sourceMappingURL reads arbitrary .map files when `from` is unset — fix >=8.5.23 — via vite
- moderate: svelte 5.55.1 — GHSA-f3cj-j4f6-wq85 — Svelte: SSR XSS via Insecure Promise Serialization in hydratable — fix >=5.55.7 — via svelte
- moderate: svelte 5.55.1 — GHSA-rcqx-6q8c-2c42 — Svelte Vulnerable to XSS via DOM Clobbering of Internal Framework State — fix >=5.55.7 — via svelte
- moderate: svelte 5.55.1 — GHSA-9rmh-mm8f-r9h6 — Svelte: ReDoS in `<svelte:element>` Tag Validation — fix >=5.55.7 — via svelte
- moderate: svelte 5.55.1 — GHSA-pr6f-5x2q-rwfp — Svelte SSR vulnerable to cross-site scripting via spread attributes — fix >=5.55.7 — via svelte
- moderate: uuid 10.0.0 — GHSA-w5hq-g745-h8pq — uuid: Missing buffer bounds check in v3/v5/v6 when buf is provided — fix >=11.1.1 — via @inlang/paraglide-js
- moderate: uuid 13.0.0 — GHSA-w5hq-g745-h8pq — uuid: Missing buffer bounds check in v3/v5/v6 when buf is provided — fix >=13.0.1 — via @inlang/paraglide-js
- moderate: vite 8.0.3 — GHSA-4w7w-66w2-5vf9 — Vite Vulnerable to Path Traversal in Optimized Deps `.map` Handling — fix >=8.0.5 — via vite
- moderate: vite 8.0.3 — GHSA-v6wh-96g9-6wx3 — launch-editor: NTLMv2 hash disclosure via UNC path handling on Windows — fix >=8.0.16 — via vite
- moderate: vite 8.0.3 — GHSA-698v-4r3x-3vcv — launch-editor: escapeCmdArgs() omits % escaping on Windows — fix >=8.1.6 — via vite
- moderate: vite 8.0.3 — GHSA-r825-w8mm-5pxw — launch-editor: Editor CLI flag misinterpretation causing arbitrary command execution for some editors — fix >=8.1.6 — via vite
- moderate: vite 8.0.3 — GHSA-rq7h-c2jc-7f22 — Vite: Files outside `server.fs.allow` could be served when project paths match system paths on non-Windows machine — fix >=8.1.6 — via vite
- moderate: vite 8.0.3 — GHSA-fx2h-pf6j-xcff — vite: `server.fs.deny` bypass on Windows alternate paths — fix >=8.0.16 — via vite
- low: devalue 5.6.4 — GHSA-wf3x-273g-mvxv — devalue: Sparse arrays emitted by uneval cause eager allocation when evaluated — fix >=5.9.3 — via svelte
- low: vite 8.0.3 — GHSA-9jrq-w75r-8gcw — Vite: Cross origin script execution through Vite custom middleware — fix >=8.1.6 — via vite
```

## Оновлення (2026-11-23)

**Rust** — `cargo update` у межах поточних версій; у `Cargo.toml` змінено лише `ringbuf = "0.5.2"` (було `"0.4"`; власник погодив перехід 2026-11-23). У `Cargo.lock` 599 → 553 крейти.

- `tauri` 2.10.3 → 2.12.7, `tauri-build` 2.5.6 → 2.7.6, `tauri-runtime-wry` 2.10.1 → 2.12.5, `wry` 0.54.4 → 0.57.3, `tao` 0.34.8 → 0.37.3, `tray-icon` 0.21.3 → 0.25.1, `muda` 0.17.1 → 0.20.0, `global-hotkey` 0.7.0 → 0.8.0, `webview2-com` 0.38.2 → 0.39.1.
- Плагіни: `dialog` 2.7.0 → 2.8.4, `global-shortcut` 2.3.1 → 2.4.1, `log` 2.8.0 → 2.10.2, `single-instance` 2.4.0 → 2.5.5 (`fs` 2.5.0 → 2.7.1 — через `dialog`).
- `ringbuf` 0.4.8 → 0.5.3. Cargo взяв найновішу 0.5.x, а не 0.5.2, про яку питали власника. Порівняно з 0.5.2 вона ще робить арифметику індексів стійкою до переповнення й перебудовує обгортку з кешем індексів (через неї працюють `HeapProd`/`HeapCons`). Виклики, якими користується програма (`push_slice`, `pop_slice`, `occupied_len`), поводяться так само; код програми не змінено. Між 0.4.8 і 0.5.2 — лише перехід крейта на Rust 2024 і саме виправлення.
- Пряма залежність `windows` 0.59 лишилася (перехід на 0.62 — DRAFT-2), тож у збірці дві версії `windows`, як і раніше.

**JS** — `pnpm update` у межах діапазонів; `package.json` тепер вимагає щонайменше встановлені версії (`^2` → `^2.12.5` тощо).

- `@tauri-apps/api` 2.10.1 → 2.12.5, `@tauri-apps/cli` 2.10.1 → 2.12.6, `@tauri-apps/plugin-dialog` 2.7.0 → 2.8.4.
- `svelte` 5.55.1 → 5.59.1, `vite` 8.0.3 → 8.3.8, `@sveltejs/vite-plugin-svelte` 7.0.0 → 7.3.2, `svelte-check` 4.4.5 → 4.7.8, `@inlang/paraglide-js` 2.15.1 → 2.29.3, `@tsconfig/svelte` 5.0.8 → 5.0.9, `@types/node` 20.19.39 → 20.19.43; `typescript` лишився 5.8.3.

**Мінорні версії Tauri збігаються:** `tauri` 2.12 = `@tauri-apps/api` 2.12 = `@tauri-apps/cli` 2.12; `tauri-plugin-dialog` 2.8 = `@tauri-apps/plugin-dialog` 2.8. `pnpm tauri build --no-bundle` не скаржиться на розбіжність версій (лише давнє попередження про ідентифікатор `com.audiocaptor.app`, що закінчується на `.app`, — воно стосується macOS і було й до оновлення).

### `cargo audit` після (src-tauri/Cargo.lock, 553 крейти)

Підсумок: вразливостей немає, `warning: 6 allowed warnings found`.

```text
vulnerabilities: 0
warnings: 6
- unmaintained: chrono 0.4.45 — RUSTSEC-2026-0543 — chrono is unmaintained
- unmaintained: proc-macro-error 1.0.4 — RUSTSEC-2024-0370 — proc-macro-error is unmaintained
- unmaintained: zbus_names 4.3.5 — RUSTSEC-2026-0371 — `zbus_names` is unmaintained
- unmaintained: zvariant 5.15.1 — RUSTSEC-2026-0370 — `zvariant` is unmaintained
- unmaintained: zvariant_derive 5.15.1 — RUSTSEC-2026-0372 — `zvariant_derive` is unmaintained
- unsound: glib 0.18.5 — RUSTSEC-2024-0429 — Unsoundness in `Iterator` and `DoubleEndedIterator` impls for `glib::VariantStrIter` — fix >=0.20.0
```

### Шість вразливостей Rust

- **cookie** RUSTSEC-2026-0594 — закрито оновленням до 0.18.3. Програму й так не зачіпало: вада лише в процесах, що роблять `fork()` на Unix (на Windows його немає), і лише для приватних cookie (`PrivateJar`), яких AudioCaptor не використовує.
- **quick-xml** RUSTSEC-2026-0194 і RUSTSEC-2026-0195 — закрито оновленням до 0.42.0 (через `plist` 1.10.4). У Windows-збірку quick-xml не потрапляв і раніше: `plist` — залежність Tauri лише для macOS; `cargo tree --target x86_64-pc-windows-msvc -i quick-xml` нічого не показує.
- **ringbuf** RUSTSEC-2026-0293 — закрито переходом на 0.5.3. Програму й так не зачіпало: вада спрацьовує, лише коли деструктор елемента панікує під час `Consumer::skip`/`clear`; у буферах лише `f32` (деструктора немає), `skip`/`clear` програма не викликає, а реліз зібрано з `panic = "abort"`.
- **rkyv** RUSTSEC-2026-0235 — зник із `Cargo.lock`: `tauri-plugin-log` 2.10 більше не тягне `byte-unit` → `rust_decimal`. Компільованим rkyv не був і раніше: це необов'язкова залежність `rust_decimal`, яку `Cargo.lock` перелічував через синтаксис «слабких» ознак (`rkyv?/std`); `cargo tree` для всіх цілей і всіх видів залежностей її не показував.
- **url** RUSTSEC-2026-0592 — закрито оновленням до 2.5.9. Крейт є в exe (Tauri, wry), але вада спрацьовує лише в `Url::origin()` на навмисно вкладених адресах `blob:` з недовіреного джерела; вебвʼю програми показує лише власний інтерфейс із вбудованих файлів.

### Попередження `cargo audit`, що лишилися (6)

Перевірено `cargo tree --target x86_64-pc-windows-msvc -e normal,build -i <крейт>`.

- **chrono** 0.4.45 — unmaintained (RUSTSEC-2026-0543). Пряма залежність, у exe: мітка часу в назві файлу запису (`lib.rs`, один рядок). Виправленої версії немає й не буде; відомих вразливостей немає. Заміну вже записано в DRAFT-2.
- **glib** 0.18.5 — unsound (RUSTSEC-2024-0429) і **proc-macro-error** 1.0.4 — unmaintained (RUSTSEC-2024-0370): стек GTK для Linux (`tray-icon` → `libappindicator` → `gtk` → `glib`; `glib-macros` → `proc-macro-error`). У Windows-збірку не потрапляють; виправлення є лише в `glib` 0.20, тобто в нових мажорних версіях Tauri і gtk-rs.
- **zbus_names** 4.3.5, **zvariant** 5.15.1, **zvariant_derive** 5.15.1 — unmaintained (RUSTSEC-2026-0371, -0370, -0372): D-Bus для Linux через `tauri-plugin-single-instance`. У Windows-збірку не потрапляють; виправленої версії немає.

Зникли 14 попереджень: `anyhow`, `event-listener`, `rand` 0.7 і 0.8, `rustix`, `tokio`, `async-executor` (yanked), `fxhash`, `pin-utils`, п'ять `unic-*` — оновлено або вилучено з дерева.

### `pnpm audit` після

`No known vulnerabilities found` (0; було 33: 2 low, 17 moderate, 14 high). Тепер `devalue` 5.9.4, `kysely` 0.28.20, `nanoid` 3.3.20, `postcss` 8.5.33, `source-map-js` 1.2.4, `uuid` 14.0.3, `svelte` 5.59.1, `vite` 8.3.8 — усі не нижче виправлених версій.

Чи стосувалися вони програми: у exe з цих пакетів потрапляє лише рантайм Svelte. Вади Svelte стосуються SSR (у програми його немає), `<svelte:element>` з недовіреною назвою тегу і вставленого чужого HTML — програма не використовує ні `<svelte:element>`, ні `{@html}`; `devalue` у бандл не потрапляє. Решта (`vite`, `postcss`, `nanoid`, `source-map-js`, `kysely`, `uuid`) — інструменти збірки й розробки; вади `vite` стосуються dev-сервера (`just dev`) на машині розробника.

## Перевірки після оновлень (2026-11-23)

Коміт `38b7203` у гілці `feature/task-6-deps` (локальний, не запушено).

- `just check` пройшов: `pnpm vite:build`, `pnpm check` (svelte-check: 240 файлів, 0 помилок, 0 попереджень), clippy з `-D warnings`, `cargo test` (66 тестів), `cargo fmt --check`.
- `just build` (`pnpm tauri build --no-bundle`) о 20:47: `AudioCaptor.exe` зібрано, скарг на розбіжність версій немає. Розмір exe: 4 827 136 → 4 944 896 байт.
- Пробний запис у режимі «Мікс» (21:02): копія нового exe у тимчасовій теці з власними `AudioCaptor-data` і даними WebView2. Мікрофон і пристрій відтворення — типові в Windows (Realtek). Запис почато й зупинено кнопками через UI Automation, під час запису в динаміках звучав тон. Результат — `mix_2026-11-23_21-02-47.wav`, 1 107 884 байти: `ffprobe` показує pcm_s16le, 48 кГц, 2 канали, 5,77 с. `ffmpeg` декодував увесь файл (553 920 відліків, середня гучність −15,2 дБ, пік −4,3 дБ), тож файл відтворюється і не тихий. Копія файлу — `%TEMP%\audiocaptor-mixtest.wav`. Перша спроба о 20:51 теж дала WAV (5,78 с), але файл зник до повної перевірки: тест NVDA видалив спільну тимчасову теку.
- `just smoke` о 20:52 з дозволу власника: 21 крок OK, 5 упали, перевірок NVDA 29 з 34, клавіатура зайнята 65 с. Результат кожного кроку й список невдалих кроків збігаються з базовим прогоном (20:38): повернення фокусу після чотирьох діалогів (TASK-15) і повторний запуск зі трею (TASK-9). Дані WebView2 користувача (`EBWebView`) не змінилися. Отже, результат не гірший за базовий.

### Трей до й після оновлення

`tray-icon` 0.21.3 → 0.25.1 відкриває меню трею на відпусканні кнопки, а не на натисканні (0.23.1), і після меню надсилає `WM_NULL`. Тест NVDA трею не перевіряє. Тому окремим одноразовим скриптом, з дозволу власника, порівняно старий exe (зібраний з develop 8046d66 у тимчасовій теці) і новий. Значок в обох випадках серед прихованих піктограм: Win+B, Enter на «Відобразити приховані піктограми», значок AudioCaptor перший.

Результати однакові для обох exe (прогін 21:56–21:57):

- Enter на значку: вікно програми з'являється, фокус на «Minimize to tray», меню не відкривається. NVDA: «AudioCaptor | AudioCaptor | AudioCaptor, основне орієнтир | Minimize to tray, кнопка».
- Клавіша контекстного меню на значку: відкривається меню трею, вікно не з'являється. NVDA: «невідомо | Контекстне меню, меню».
- Лівий клік мишею: те саме, що Enter.
- Правий клік мишею: те саме, що клавіша контекстного меню.

Отже, для користувача поводження трею не змінилося.

Два давні нюанси, однакові до й після:
- коли меню трею відкривається, NVDA спершу каже «невідомо» (передній план переходить до прихованого безіменного вікна `tray_icon_app` з крейта `tray-icon`), а перший пункт меню не має фокусу;
- після показу вікна з трею фокус стоїть на «Minimize to tray», а не на списку профілів.

### CHANGELOG

Запису немає (критерій 7). Жодна із закритих вразливостей не стосувалася програми (див. пояснення вище). Помітного для користувача поводження оновлення не змінило: тест NVDA дав той самий результат, трей поводиться однаково, запис «Мікс» працює. Tauri 2.12 перейшов на `windows` 0.62 і `webview2-com` 0.39, що прибирає підтримку Windows 7 (за журналом змін Tauri), але програма й так вимагає Windows 10 або новішу (README).

### Що лишилося й не перевірено

- `cargo audit`: 6 попереджень (див. вище) — без вразливостей; заміна `chrono` — у DRAFT-2.
- Не перевірено: вихід із Windows або вимкнення під час запису. `tao` 0.37.0 тепер завершує процес на `WM_ENDSESSION`, а не падає пізніше; чи лишається WAV із незавершеним заголовком (найімовірніше, як і раніше), не перевірялося.
- Збірка фронтенду повільніша: крок Paraglide у `vite build` — близько 3,6 с (уся збірка до оновлення — 0,7 с). Він завантажує модулі inlang з CDN без закріпленої версії, тепер це DRAFT-7.

### Нові записи беклогу (не закомічено)

- TASK-21 — тест із NVDA стартує з дозволу або після паузи й попереджає звуком (побажання власника під час цієї задачі).
- DRAFT-7 — модулі inlang для збірки не завантажуються з CDN без закріпленої версії.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Залежності оновлено в межах поточних версій: Tauri 2.12 з плагінами, ringbuf 0.5.3, Svelte 5.59, Vite 8.3, Paraglide 2.29. `cargo audit`: 6 вразливостей → 0, лишилося 6 попереджень (5 лише для Linux, chrono — у DRAFT-2). `pnpm audit`: 33 → 0. Жодна закрита вразливість програму не зачіпала, тому запису в CHANGELOG немає.

Перевірки: `just check` проходить (повторено в головній сесії); CI на develop c5e07fb зелений (запуск 45255388791); `just smoke` до й після дає однаковий результат (21 з 26, відомі вади TASK-15 і TASK-9); трей поводиться однаково в старому й новому exe; запис «Мікс» дає справний WAV.

Не перевірено: вихід із Windows або вимкнення під час запису. Нові записи: TASK-21, DRAFT-7.
<!-- SECTION:FINAL_SUMMARY:END -->

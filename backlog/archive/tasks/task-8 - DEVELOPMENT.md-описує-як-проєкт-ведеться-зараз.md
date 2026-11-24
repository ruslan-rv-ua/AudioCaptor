---
id: TASK-8
title: DEVELOPMENT.md описує, як проєкт ведеться зараз
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T11:31:02Z'
updated_date: '2026-11-24T12:55:58Z'
labels:
  - docs
  - model-sonnet
  - effort-medium
milestone: m-0
dependencies:
  - TASK-3
  - TASK-4
  - TASK-5
references:
  - DEVELOPMENT.md
  - CLAUDE.md
priority: medium
type: docs
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`DEVELOPMENT.md` написано для версії 0.1.0 і частково виправлено в серпні 2026. Станом на 2026-11-23 він розходиться з тим, як проєкт ведеться:

- Документ англійський, але розділи «Release Process», «Оновлення Scoop-маніфесту» і «Required Secrets» написано українською (їх додав PR #3).
- Порядок випуску не згадує git flow. Насправді випуск — це `git flow release start`, версія в трьох файлах (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) і в `src-tauri/Cargo.lock`, розділ у `CHANGELOG.md`, `git flow release finish --notag`, push гілок `main` і `develop`, запуск Release на `main`, а потім Excavator у `ruslan-rv-ua/scoop-bucket`. Перевірка якості коду у квітні 2026 окремо зауважила, що версію легко забути в одному з файлів.
- Немає жодного слова про Backlog.md, правила роботи з `CLAUDE.md`, `just check` і закріплену версію Rust.
- У вимогах стоїть «Node.js 18+», тоді як CI користується Node 22.
- Розділ про значки каже, що вони генеруються «з одного файлу 1024×1024», а команда бере `src-tauri/icons/128x128.png`.

Розділи про CI і про пробний запуск Release оновлюють TASK-4 і TASK-5; ця задача зводить документ докупи й перевіряє його цілком.

До початку роботи власник вирішує, якою мовою буде документ: у quick-snippets `DEVELOPMENT.md` український, а README там і тут — двома мовами.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Документ написано однією мовою; якою саме — вирішив власник, і це записано в нотатках
- [x] #2 Розділ про випуск описує всі кроки від `git flow release start` до оновлення Scoop, зокрема три файли з версією, `Cargo.lock` і прапорець `--notag`
- [x] #3 Розділ про CI відповідає файлам `ci.yml` і `release.yml`
- [x] #4 Документ каже, де шукати задачі, рішення й дослідження (Backlog.md), і називає `CLAUDE.md`, `just check` та `rust-toolchain.toml`
- [x] #5 Вимоги до інструментів (Rust, Node, pnpm) збігаються з тим, що використовують CI і `rust-toolchain.toml`
- [x] #6 Кожну команду з документа виконано, і вона працює; виняток — команди, що публікують реліз
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
1. Звірити факти з файлами: justfile, package.json, rust-toolchain.toml, .github/workflows/ci.yml і release.yml, tauri.conf.json, Cargo.toml, структура тек, CHANGELOG.md.
2. Переписати DEVELOPMENT.md українською одним документом: вимоги до інструментів (як у CI і rust-toolchain.toml), як ведеться проєкт (CLAUDE.md, Backlog.md, git flow, just check), повний порядок випуску від git flow release start до оновлення Scoop, CI за ci.yml і release.yml, значки (128x128.png). Розділи TASK-4, TASK-5, TASK-14, TASK-21 і «Доступність» з посиланням на doc-3 зберегти за змістом.
3. Перевірити README.md і README_UK.md лише на суперечності щодо збирання й розробки.
4. Виконати кожну команду з документа (крім тих, що публікують або змінюють спільний стан; запуск workflow і just smoke лише з дозволу власника), записати результати.
5. Запустити just check і записати результати в нотатки.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Рішення власника (2026-11-24)

Мова документа — українська: `DEVELOPMENT.md` пишеться українською, як у quick-snippets і в Backlog. Назви команд, файлів, параметрів і фрагменти коду лишаються як є. Правило записано в `CLAUDE.md`.

## Виконання (робоча сесія, 2026-11-24)

Гілка `feature/task-8-development-md`, коміти b7597c0, dc12ef3, bf1c50e. Гілку не відправлено й не завершено. Файли backlog (ця задача, нова DRAFT-9, нотатка в DRAFT-7) не закомічено: це для головної сесії.

### Що змінилося в DEVELOPMENT.md

- Увесь документ українською (рішення власника); команди, файли, параметри й код — як були.
- Новий розділ «Як ведеться проєкт»: де задачі, чернетки, рішення, дослідження й етапи (Backlog.md і команди `backlog`), правила з `CLAUDE.md`, git flow (git-flow-next), `just check`, `rust-toolchain.toml`, мови текстів, `CHANGELOG.md`.
- «Що потрібно встановити» відповідає CI: Node 22 (Vite 8 і `@sveltejs/vite-plugin-svelte` 7 вимагають Node 20.19+, 22.12+ або 24+; було «18+»); pnpm 10 з `packageManager` (`npm install -g pnpm@10`: без `@10` npm ставить pnpm 12.23.0); rustup і `rust-toolchain.toml`; Microsoft C++ Build Tools; just більше не «необов'язковий», і йому потрібен `sh` у PATH (без `sh` рецепт не запускається: «could not find the shell `sh`», перевірено); git-flow-next, Backlog.md, gh.
- Повний порядок випуску: `git flow release start` → версія в трьох файлах + `src-tauri/Cargo.lock` (його оновлює `just check`) → розділ `CHANGELOG.md` і посилання → коміт → `git flow release finish --notag` → `git push origin main develop` → пробний запуск на `main` → справжній → `gh release view` → Excavator. Описано параметри Release (`version`, `dry_run`, `prerelease`) і хеш SHA256 у підсумку запуску.
- CI за `ci.yml` і `release.yml`; ручний запуск і `gh run list`.
- Значки: команда бере `128x128.png`, тож усе, більше за 128 пікселів, виходить розмитим; оригіналу 1024×1024 у робочому дереві немає (DRAFT-9); `public/favicon.png` команда не створює.
- Портативний режим: що поки пишеться поза `AudioCaptor-data` (TASK-20, DRAFT-6); посилання на decision-1 і decision-2.
- Режим розробки: exe називається `audio-captor.exe`, дані — у `src-tauri/target/debug/AudioCaptor-data/`; перед `just dev` треба закрити свій AudioCaptor.
- Архітектура: схему з рамок замінено текстовим ланцюжком (її зручніше слухати з NVDA); таблиця IPC названа «основні команди» з посиланням на `generate_handler!`; про довгий натиск — зупинка після 500 мс утримання, не чекаючи відпускання.
- Розділ «Перевірка з NVDA» (TASK-14, TASK-21) лишився слово в слово. Факти з розділів TASK-4 і TASK-5 (CI, пробний запуск, захист від повторної версії, Scoop, секрети) збережено; до розділу про Scoop додано, що Excavator за розкладом фактично запускався о 12:41–14:56 UTC (17–23 листопада), хоча в його файлі стоїть 04:20 UTC.
- `README.md` і `README_UK.md` про збирання й розробку лише посилаються на DEVELOPMENT.md («інструкції зі збирання та огляд архітектури»); суперечностей немає, файли не змінено. `CHANGELOG.md` не змінено: це документація для розробників.

### Команди з документа (AC #6)

Виконано, працюють:

- `backlog task list --plain`, `backlog task 8 --plain`, `backlog task 2 --plain` (архівна), `backlog draft list --plain`, `backlog decision list --plain`, `backlog decision view decision-1 --plain`, `backlog doc list --plain`, `backlog doc view doc-3 --plain`, `backlog milestone list --plain`, `backlog search "NVDA" --plain`, `backlog task --help` — код 0.
- `git-flow overview`; `git-flow feature finish --help`, `bugfix start --help`, `bugfix finish --help`, `release start --help`, `release finish --help`: прапорці з документа (`--no-fetch`, `--no-push`, `--notag`) є. `git flow --help` справді не працює (код 128).
- `git flow feature start task-8-development-md` — створив цю гілку.
- `rustup toolchain install` — 1.99.1 уже стоїть, код 0.
- `pnpm install`, `just install` — змін немає, код 0.
- `just` — показує список рецептів.
- `just vite-dev` і `pnpm vite dev --port 1420` — сторінка AudioCaptor, HTTP 200; сервер зупинено. Перший старт чекав на Paraglide близько 32 с (нотатка в DRAFT-7).
- `cargo fmt --manifest-path src-tauri/Cargo.toml` — змін немає.
- `just check` — пройшов: svelte-check 0 помилок і 0 попереджень, clippy, 66 тестів, fmt.
- `just build` (80 с), `pnpm tauri build --no-bundle` (79 с), `just build-fast` (72 с), `pnpm tauri build --no-bundle -- --profile release-fast` (21 с) — код 0, exe 4,7 МБ і 11,7 МБ.
- `pnpm tauri icon src-tauri/icons/128x128.png` — переписала 51 файл у `src-tauri/icons/` (зокрема `Square*Logo.png`, `StoreLogo.png`, ICO, ICNS; розміри ті самі), `public/favicon.png` не чіпала, нових файлів немає. Файли повернуто через `git checkout`, хеші збігаються з git. Окремий прогін у тимчасову теку: `icon.png` помітно розмитий порівняно з нинішнім.
- `just smoke dialogs -PauseSec 10` (з дозволу власника) — 8 кроків OK, 4 не пройшли: відомі кроки повернення фокусу після діалогів (TASK-15); код 1; клавіатура зайнята 31 с; дані WebView2 власника не змінилися. Повний `just smoke` сьогодні запускала TASK-21 (22 з 26).
- `just dev` (з дозволу власника; рецепт виконує `pnpm tauri dev`) — збирання 54 с, програма запустилася, власник бачив вікно. Копія працювала з `WEBVIEW2_USER_DATA_FOLDER` у тимчасовій теці, тож `EBWebView` власника не змінився (457 елементів, остання зміна 2026-04-20). Мій скрипт перевірки шукав процес `AudioCaptor.exe`, а в цьому режимі exe називається `audio-captor.exe`, тож вікно було відкрите близько 6 хв і закрилося примусово (taskkill дерева процесів); цей факт додано в документ.
- `gh workflow run ci.yml --ref develop` (з дозволу власника) — запуск 45394438205, success, 3 хв.
- `gh workflow run release.yml --ref develop -f version=0.1.0 -f dry_run=true` (з дозволу власника) — запуск 45394443732, success, 12 хв: усі кроки зелені, `publish` пропущено, попередження «Tag v0.1.0 already exists. Fine for a dry run…», як і описано в документі.
- `gh run list --workflow ci.yml -L 3`, `gh run list --workflow release.yml -L 3`, `gh release view v0.1.0` — працюють.
- `just clean` (з дозволу власника, останнім кроком) — видалив 26 144 файли (19,5 ГБ), теки `src-tauri/target` не стало. Після нього з порожньої теки: `just build` — 167 с, exe 4,7 МБ; `just check` — 94 с, svelte-check 0 помилок і 0 попереджень, clippy, 66 тестів, fmt. Обидва — код 0.
- Твердження про `Cargo.lock` перевірено в тимчасовому клоні в теці сесії: після зміни версії в `src-tauri/Cargo.toml` `cargo metadata` сам записав нову версію `audio-captor` у `Cargo.lock`. Клон видалено.

Не виконано (публікують реліз або змінюють спільний стан), перевірено читанням `release.yml`, довідки й налаштувань git flow:

- `git flow release start 0.1.1`, зміна версії, коміт, `git flow release finish 0.1.1 --notag`, `git push origin main develop`, `gh workflow run release.yml --ref main -f version=0.1.1 -f dry_run=false`.
- `gh workflow run release.yml --ref main -f version=0.1.1 -f dry_run=true`: на `main` поки старі `release.yml` (без `dry_run`) і `ci.yml` (з запуском на push у `main`), а версії 0.1.1 ще немає в коді. Нові workflow потраплять у `main` разом із першим релізом, тож у порядку випуску пробний запуск іде після push. Ту саму команду перевірено на `develop` з версією 0.1.0.
- `gh release view v0.1.1` — релізу ще немає; команду перевірено на v0.1.0.
- `npm install -g pnpm@10` — лише з `--dry-run` (поставив би 10.34.9 замість 10.32.1), щоб не міняти глобальний pnpm власника.
- `pnpm tauri dev` окремо — власник вирішив, що досить `just dev`.

Не перевірено наживо:

- що копія для розробки лише показує вікно вже запущеного AudioCaptor і закривається (з коду: `tauri-plugin-single-instance`, той самий ідентифікатор `com.audiocaptor.app`; AudioCaptor власника не був запущений);
- що `git flow release finish` оновлює `develop` від `main` (з налаштувань: `gitflow.branch.develop.autoupdate true`, `git-flow overview` показує «develop → main [auto-update]», довідка `finish` згадує оновлення дочірніх гілок);
- що rustup ставить версію з `rust-toolchain.toml` сам (`rustup set auto-install` типово `enable`; версія вже стояла).

### Знайдене мимохідь

- DRAFT-9 (нова): вихідний логотип 1024×1024 знову лежить у репозиторії; він є в історії git (afcff98).
- DRAFT-7: додано нотатку про ~32 с першого старту `vite dev`.
- Excavator у `ruslan-rv-ua/scoop-bucket` (інший репозиторій): розклад 04:20 UTC, а запуски за розкладом ідуть приблизно о 13:00 UTC; у документі записано, як є.
- `just dev`: tauri CLI під час збирання пише «file watcher error: EBUSY … target\debug\deps\…»; на роботу це не вплинуло.

### Definition of Done

- #1, #2: `just check` проходить (двічі: на зібраному раніше й з порожньої теки після `just clean`; після dc12ef3 змінювався лише DEVELOPMENT.md).
- #3, #4: не застосовуються (інтерфейс не змінювався).
- #5: DEVELOPMENT.md оновлено; README без змін (суперечностей немає).
- #6: CHANGELOG не змінювався: зміни лише для розробників.

## Перевірка головною сесією (2026-11-24)

- Звірено з кодом: редакція Rust 2021, `packageManager` pnpm@10.32.1, профілі `release` і `release-fast`, рецепти `justfile`, структура тек, `mainBinaryName`. Розбіжностей немає. README.md і README_UK.md лише посилаються на DEVELOPMENT.md, суперечностей немає. `just check` проходить.
- З рішення власника порядок випуску змінено (коміт 7ef2dde): пробний запуск Release іде на гілці `release/<версія>`, відправленій на GitHub, ще до `git flow release finish`; після завершення гілку випуску видаляють на GitHub (`git push origin --delete release/<версія>`). Так помилка, яку знайде пробний запуск, не потрапить у `main`. Міркування з нотаток вище про пробний запуск на `main` після push цим замінено.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
`DEVELOPMENT.md` переписано українською (рішення власника) і зведено з тим, як проєкт ведеться зараз: новий розділ «Як ведеться проєкт» (Backlog.md, `CLAUDE.md`, git flow, `just check`, `rust-toolchain.toml`, мови й тексти), вимоги до інструментів як у CI (Node 22, pnpm 10, `sh` у PATH для just), повний порядок випуску від `git flow release start` до Scoop (три файли з версією й `Cargo.lock`, `--notag`, пробний запуск на гілці випуску до злиття в `main`), розділ про значки з попередженням про розмиті великі значки, назва й тека exe в режимі розробки. Розділи про CI, Release і тест із NVDA збережено за змістом.

Перевірки: команди з документа виконано або перелічено з причиною в нотатках (не запускалися лише команди, що публікують реліз чи змінюють GitHub); CI й пробний Release на `develop` зелені; `just smoke dialogs` дав відомі 4 збої TASK-15; `just check` проходить, зокрема після `just clean`.

Нові записи: DRAFT-9 (повернути вихідний логотип 1024×1024), нотатка до DRAFT-7 (повільний перший старт Paraglide). Не перевірено наживо: поступку копії для розробки вже запущеній програмі, оновлення `develop` під час `git flow release finish`, автоматичне встановлення Rust через rustup.
<!-- SECTION:FINAL_SUMMARY:END -->

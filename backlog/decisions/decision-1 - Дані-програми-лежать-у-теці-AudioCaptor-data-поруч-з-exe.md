---
id: decision-1
title: Дані програми лежать у теці AudioCaptor-data поруч з exe
date: '2026-11-23T11:22:51Z'
status: accepted
---
## Context

<!-- SECTION:CONTEXT:BEGIN -->
До версії 0.1.0 програма створювала `settings.json`, `Recordings/` і `logs/` просто поруч з exe. Це мало три вади:

- поруч з exe лежали три окремі елементи замість одного;
- маніфест Scoop мусив зберігати між оновленнями три елементи (`"persist": ["settings.json", "logs", "Recordings"]`), а для файлу `settings.json` Scoop робить жорстке посилання і потребує `pre_install`, щоб не створити теку замість файлу;
- кожен новий вид даних означав би зміну маніфесту.

Рішення ухвалено 2026-04-20 і втілено в коміті ffb233c, ще до першого випуску. Запис звірено з кодом 2026-11-23.
<!-- SECTION:CONTEXT:END -->

## Decision

<!-- SECTION:DECISION:BEGIN -->
Усі дані програми лежать в одній теці `AudioCaptor-data/` поруч з exe:

- `settings.json` — налаштування;
- `Recordings/` — тека записів за замовчуванням;
- `logs/` — журнали.

Шлях до теки дає одна функція: `portable::data_dir()` у `src-tauri/src/portable.rs`. З неї беруть шлях налаштування (`settings.rs`), запис і журнали (`lib.rs`) та пункт меню трея «Відкрити папку записів» (`tray.rs`). Теку й типовий `settings.json` програма створює під час першого запуску (`portable::ensure_dirs()`).
<!-- SECTION:DECISION:END -->

## Consequences

<!-- SECTION:CONSEQUENCES:BEGIN -->
- Резервна копія чи перенесення програми — це exe і одна тека.
- Маніфест Scoop зберігає одну теку: `"persist": "AudioCaptor-data"`. Новий вид даних не потребує зміни маніфесту.
- Назва теки стала частиною формату: користувачі версії 0.1.0 і Scoop уже мають дані саме в `AudioCaptor-data/`, тому перейменування потребувало б перенесення даних.
- Не вирішено й не заплановано: режим без портативності (дані в `%AppData%`) і файл-позначка на кшталт `AudioCaptor-data/portable.txt`. У дослідженні це лишилося відкритими питаннями.
<!-- SECTION:CONSEQUENCES:END -->

## Alternatives

<!-- SECTION:ALTERNATIVES:BEGIN -->
- `data/` — найпоширеніша назва в портативних програмах (VS Code portable, foobar2000, mpv). Відхилено: назва надто загальна, з неї не видно, якій програмі належить тека, і вона може збігтися з чужою `data/`.
- Лишити три елементи поруч з exe. Відхилено через жорстке посилання на `settings.json` у Scoop і зміну маніфесту на кожен новий вид даних.
<!-- SECTION:ALTERNATIVES:END -->


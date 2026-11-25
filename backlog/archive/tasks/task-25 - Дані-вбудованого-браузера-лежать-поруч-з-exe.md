---
id: TASK-25
title: Дані вбудованого браузера лежать поруч з exe
status: Done
assignee:
  - '@claude'
created_date: '2026-11-23T18:20:09Z'
updated_date: '2026-11-25T01:00:10Z'
labels:
  - backend
  - model-sonnet
  - effort-medium
milestone: m-2
dependencies: []
priority: medium
type: task
ordinal: 10700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Знайшов тест із NVDA (TASK-14) 2026-11-23. Дані вбудованого браузера WebView2 (кеш і його власний стан) програма тримає поза `AudioCaptor-data`: Tauri за замовчуванням кладе профіль браузера в `%LOCALAPPDATA%\com.audiocaptor.app`, спільний для всіх копій AudioCaptor на комп'ютері. Це суперечить духу рішення decision-1 (усі дані портативної програми — поруч з exe) і означає, що тестова копія й ваша робоча ділять один профіль браузера.

У quick-snippets це вирішено так: профіль браузера лежить у `quick-snippets-webview/` поруч з exe, окремо від теки даних, щоб Scoop не зберігав кеш між оновленнями, а резервні копії теки даних його не захоплювали.

Рішення власника (2026-11-25): профіль браузера лежить в окремій теці поруч з exe, як у quick-snippets, окремо від `AudioCaptor-data`. Після оновлення через Scoop профіль починається з нуля: зникають кеш і підказки автозаповнення.

Старий профіль у `%LOCALAPPDATA%` програма в жодному разі не видаляє: ним можуть користуватися інші копії.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Дані вбудованого браузера (WebView2) програма тримає в окремій теці поруч з exe, а не в `%LOCALAPPDATA%\com.audiocaptor.app` і не в `AudioCaptor-data`
- [x] #2 Після запуску, запису й виходу з програми в `%LOCALAPPDATA%\com.audiocaptor.app` нічого не з'являється і не змінюється, а старого профілю браузера там програма не видаляє
- [x] #3 `just smoke` проходить, і тестова копія тримає дані браузера поруч зі своїм exe
- [x] #4 У `CHANGELOG.md` є запис про нову теку з поясненням, що стару теку `EBWebView` у `%LOCALAPPDATA%\com.audiocaptor.app` можна видалити, якщо старіші версії програми більше не потрібні
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Усі локальні перевірки проходять: `just check`
- [x] #2 Новий або змінений текст інтерфейсу є англійською й українською: у `messages/en.json` і `messages/uk.json`, а підписи меню трея — у `src-tauri/src/tray.rs`
- [x] #3 Зміни в інтерфейсі, фокусі, діалогах чи оголошеннях перевірено з клавіатури і з NVDA (якщо задача їх містить)
- [x] #4 Документацію оновлено там, де змінилася поведінка чи порядок роботи (README.md, README_UK.md, DEVELOPMENT.md)
- [x] #5 CHANGELOG.md оновлено: лише зміни, помітні користувачам (без подробиць про інструменти, тести, CI чи збірку)
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. `src-tauri/src/portable.rs`: функція `webview_dir()` — тека `AudioCaptor-webview` поруч з exe; тести: поруч з exe і поза `AudioCaptor-data`.
2. `src-tauri/tauri.conf.json`: `"create": false` для головного вікна. `src-tauri/src/lib.rs`: на самому початку `setup` вікно створюється з тієї самої конфігурації (`WebviewWindowBuilder::from_config`, як це робив Tauri: усі параметри без змін, зокрема типовий `focus` true з TASK-18) і з `data_directory(portable::webview_dir())`. Тоді Tauri не бере `%LOCALAPPDATA%\com.audiocaptor.app` для профілю браузера і не створює цю теку. `"dataDirectory"` у конфігурації та змінна `WEBVIEW2_USER_DATA_FOLDER` не підходять (перевірено в коді Tauri 2.12.7).
3. `scripts/smoke-nvda.ps1`: без `WEBVIEW2_USER_DATA_FOLDER`; тестова копія сама створює `AudioCaptor-webview\EBWebView` поруч зі своїм exe, тест це перевіряє, а звіт показує стан цієї теки. Перевірка, що в `%LOCALAPPDATA%\com.audiocaptor.app` нічого не змінилося, лишається.
4. Документація: README.md і README_UK.md (склад даних, речення про нову теку), DEVELOPMENT.md («Початок роботи», «Портативний режим», розділ про `just smoke`), CHANGELOG.md (`### Changed` у `[Unreleased]`, один рядок).
5. `just check`, `just build`; з дозволу власника `just smoke launch`, потім повний `just smoke`.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-11-25: власник переніс чернетку у віху 0.2.0 (m-2), щоб вона не затримувала випуск 0.1.1.

2026-11-25, робоча сесія TASK-25.

Дослідження (Tauri 2.12.7, tauri-utils 2.10.4 з `Cargo.lock`, `@tauri-apps/cli` 2.12.6):
- Висновки quick-snippets (Tauri 2.10) тут теж правдиві. Без явної теки даних Tauri на Windows бере для профілю браузера `app_local_data_dir()`, тобто `%LOCALAPPDATA%\com.audiocaptor.app`, і створює її (`tauri/src/manager/webview.rs`). `"dataDirectory"` у налаштуваннях вікна Tauri розв'язує всередині `%LOCALAPPDATA%\<мітка вікна>\` (`webview/mod.rs`, `from_config`). Зі змінною `WEBVIEW2_USER_DATA_FOLDER` Tauri однаково створює `%LOCALAPPDATA%\com.audiocaptor.app`.
- Нове в Tauri 2.12: параметр `app > appDirectoriesOverride`. У формі `{ "localData": "AudioCaptor-webview" }` він переносить лише теку локальних даних Tauri поруч з exe (відносний шлях рахується від теки exe), а саме цю теку Tauri бере для профілю WebView2. Інших тек Tauri програма не використовує: свої дані бере з `portable::data_dir()`, журнал пише в `AudioCaptor-data\logs` (TASK-20). Плагіни dialog, global-shortcut і single-instance файлів не пишуть. Схема `@tauri-apps/cli` 2.12.6 цей параметр знає.
- Тож вікно не треба переносити в код, як у quick-snippets. Tauri і далі створює його з `tauri.conf.json` тим самим викликом, що й раніше (`WebviewWindowBuilder::from_config(...).build()` у `setup` в `app.rs`), з усіма параметрами й типовим `focus` true (ранній фокус із TASK-18). Коду Rust не змінено.

Зміни на гілці `feature/task-25-webview-next-to-exe`:
- 642507e: `src-tauri/tauri.conf.json` — `"appDirectoriesOverride": { "localData": "AudioCaptor-webview" }`. CHANGELOG.md: один рядок у `### Changed` розділу `[Unreleased]`, з поясненням, що стару теку `EBWebView` можна видалити, коли версії 0.1.1 і старіші більше не потрібні. README.md і README_UK.md: склад даних, абзац про нову теку, речення в розділі про Scoop. DEVELOPMENT.md: «Початок роботи», «Портативний режим», «Що тест не чіпає».
- 8f763c2: `scripts/smoke-nvda.ps1` — без `WEBVIEW2_USER_DATA_FOLDER`. Тека тесту створюється без `AudioCaptor-webview`, тож тестова копія створює її сама, а `Start-App` після кожного старту перевіряє, що там є `EBWebView`. Звіт показує стан цієї теки. Тест не проходить і тоді, коли `%LOCALAPPDATA%\com.audiocaptor.app` до прогону не було, а після нього вона є.

Перевірки:
- `just check` (02:20) пройшов: svelte-check — 0 помилок і 0 попереджень, clippy, 66 тестів, `cargo fmt --check`.
- `just build`: exe о 02:23, рядок `AudioCaptor-webview` у ньому є.
- `just smoke launch` (02:37): 2 кроки з 2, 8 перевірок NVDA з 8. Старт: «AudioCaptor | документ | додаток | Recording profile, комбінований список, Smoke system audio, згорнуто». Тестова копія створила `AudioCaptor-webview\EBWebView` поруч зі своїм exe (226 файлів і тек). `%LOCALAPPDATA%\com.audiocaptor.app\EBWebView` не змінилася (457 файлів і тек, остання зміна 2026-04-20 17:54:08).
- Повний `just smoke` (02:47): 29 кроків з 30, 54 перевірки з 56. Не пройшов лише крок 3 сценарію `record`, і це DRAFT-10: о 02:46:56 помилка `VBufBackend_t::terminate` для закритого процесу WebView2 і зависання NVDA на 0,71 с між `launch` і `record`. На старті `record` вікно прочитано через UI Automation («tauri.localhost – веб-вміст, регіон»), а Alt+S дав «Recording started | недоступно | документ | додаток | Recording controls, групування | Pause recording (Alt+P), кнопка». Сценарії `dialogs` і `tray` пройшли; `%LOCALAPPDATA%\com.audiocaptor.app` без змін.
- `just smoke record` окремо (02:50): 11 кроків з 11, 25 перевірок з 25, крок 3 чистий, зависань NVDA немає. Запуск, запис, пауза, продовження, зупинка й вихід минули без змін у `%LOCALAPPDATA%\com.audiocaptor.app` (`EBWebView` і `logs`).
- Кожен прогін тест запитував у власника в чаті й запускав лише після «так». Звіти: `task-25-check.txt`, `task-25-smoke-launch.txt`, `task-25-smoke-full.txt`, `task-25-smoke-record.txt` у спільній теці нотаток сесій.

Не перевірено:
- Що тека `%LOCALAPPDATA%\com.audiocaptor.app` не з'являється, коли її не було: на цьому комп'ютері вона є (дані версії 0.1.1), а чіпати її не можна. Нова перевірка тесту спрацює там, де теки немає. За кодом Tauri 2.12.7 її ніщо не створює.
- `just dev` (профіль у `src-tauri/target/debug/AudioCaptor-webview`) і справжнє оновлення через Scoop. Маніфест Scoop змін не потребує: `persist` зберігає лише `AudioCaptor-data`, а `AudioCaptor-webview` лягає в теку версії.

Помічено мимохідь: у сценарії `launch` (02:37 і 02:47) перехід Tab на «Minimize to tray» обидва рази звучав «документ | додаток, пробіл | додаток | Minimize to tray, кнопка». Це новий варіант DRAFT-11, дописано туди.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Дані вбудованого браузера (WebView2) тепер лежать у теці `AudioCaptor-webview` поруч з exe, а не в `%LOCALAPPDATA%\com.audiocaptor.app\EBWebView`.

Як: у `src-tauri/tauri.conf.json` — `app > appDirectoriesOverride > localData: "AudioCaptor-webview"` (Tauri 2.12; відносний шлях Tauri рахує від теки exe, це видно в `tauri/src/path/mod.rs`). Саме цей каталог Tauri дає WebView2 як теку даних, коли вебвікно не має власної (`manager/webview.rs`). На відміну від quick-snippets, вікно лишилося в `tauri.conf.json`: усі його параметри, зокрема типовий `focus` (TASK-18), не змінилися, а код Rust не змінювався. Інших каталогів Tauri програма не використовує.

Також: `scripts/smoke-nvda.ps1` більше не ставить `WEBVIEW2_USER_DATA_FOLDER` і перевіряє, що тестова копія сама створила `AudioCaptor-webview\EBWebView` поруч зі своїм exe; тест не проходить, якщо `%LOCALAPPDATA%\com.audiocaptor.app` з'являється або змінюється. Оновлено `README.md`, `README_UK.md`, `DEVELOPMENT.md` і `CHANGELOG.md` (Changed, одним рядком).

Перевірки: `just check` проходить (у робочій сесії й повторно в головній); `just smoke launch` — 2 з 2 (назва один раз, без читання через UI Automation); `just smoke record` — 11 з 11, `%LOCALAPPDATA%\com.audiocaptor.app` не змінилася; повний `just smoke` — 29 з 30: не пройшов лише крок 3 сценарію `record` через зависання NVDA з DRAFT-10 (ушосте; до цієї зміни воно теж траплялося).

Не перевірено: що `%LOCALAPPDATA%\com.audiocaptor.app` не з'являється, коли її немає (на цьому комп'ютері вона є, а за кодом Tauri її нічого не створює); справжнє оновлення через Scoop (маніфест змін не потребує); `just dev`; JAWS і Narrator. Окремого запуску CI не робили: змінилося налаштування програми, а не збирання; пробний Release перед 0.2.0 збере його на GitHub.

Гілку feature/task-25-webview-next-to-exe завершено в develop.
<!-- SECTION:FINAL_SUMMARY:END -->

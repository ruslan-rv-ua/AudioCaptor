# Дослідження: Зворотний зв'язок при неможливості почати запис

**Дата:** 2026-03-31  
**Автор:** AI Research  
**Статус:** Завершене дослідження

---

## 1. Проблема

Коли користувач натискає кнопку «Start» або глобальну гарячу клавішу для початку запису, а запис з тих чи інших причин неможливий (наприклад, не обрано мікрофон), програма реагує **після спроби**: бекенд повертає помилку, яка відображається у вікні як текстове повідомлення `role="alert"`.

### Недоліки поточного підходу

1. **Реактивний, а не проактивний** — кнопка доступна для натискання навіть коли запис очевидно неможливий. Користувач дізнається про проблему тільки після спроби.
2. **Не працює для фонових гарячих клавіш** — коли вікно не у фокусі і запис ініціюється через гарячу клавішу, `role="alert"` повідомлення оновлюється у DOM, але:
   - Скрінрідер не зачитає `aria-live` регіон, оскільки вікно не має фокусу (обмеження платформи).
   - Візуально повідомлення не видно, бо вікно згорнуте/не у фокусі.
3. **Немає аудіо-зворотного зв'язку** — при помилці через гарячу клавішу користувач не отримує жодного сигналу.

### Які умови перешкоджають запису

З аналізу `start_recording_inner()` в `src-tauri/src/lib.rs`:

| Умова | Повідомлення |
|-------|-------------|
| Режим Microphone або Mix, мікрофон не обрано | `DEVICE_NOT_FOUND: Microphone device required` |
| Режим Loopback або Mix, loopback не обрано | `DEVICE_NOT_FOUND: Loopback device required` |
| Вже йде запис | `Already recording` |
| Помилка ініціалізації capture/mixer | Динамічна помилка runtime |

Перші дві ситуації — **повністю передбачувані** на стороні фронтенду. Третя вже обробляється (кнопка Start видна тільки в стані Idle). Четверта — непередбачувані runtime помилки.

---

## 2. Аналіз поточної реалізації

### 2.1. Кнопка Start (RecordControls.svelte)

```svelte
{#if recordingState === "Idle"}
  <button type="button" onclick={onstart} aria-label="Start recording (Alt+S)" class="btn btn-start">
    {#if altPressed}<u>S</u>tart{:else}Start{/if}
  </button>
{/if}
```

Кнопка завжди активна в стані Idle. Немає перевірки готовності до запису.

### 2.2. Відображення помилки (App.svelte)

```svelte
{#if recording.error}
  <div class="error" role="alert" aria-live="assertive">
    {recording.error}
  </div>
{/if}
```

Помилка з'являється **після** невдалої спроби. Скрінрідер зачитає її тільки при фокусі на вікні.

### 2.3. Потік для гарячої клавіші (hotkey.rs)

```rust
RecordingState::Idle => {
    if let Err(e) = do_start_recording(app) {
        log::error!("Hotkey start failed: {e}");
        let _ = app.emit("recording-error", e);
    }
}
```

Помилка емітується як подія → фронтенд встановлює `recordingError` → відображає у DOM. Але якщо вікно не у фокусі, ефекту нуль.

### 2.4. Звукова система (sounds.rs)

Наразі підтримує три звуки: `Start`, `Pause`, `Stop`. Звуки програються через окремий потік на основі `rodio` — надійний механізм, що працює незалежно від стану вікна.

---

## 3. Досліджені варіанти рішення

### Варіант A: Проактивний UI — неактивна кнопка + інструкція (базова версія пропозиції)

**Суть:** обчислювати `canRecord` на фронтенді та деактивувати кнопку, коли запис неможливий.

**UI:**
- Кнопка Start стає `disabled`, коли пристрої для обраного режиму не сконфігуровані.
- Біля кнопки з'являється текстова підказка: "Оберіть мікрофон" або "Оберіть пристрій системного звуку" тощо.

**Для скрінрідера:**
- Зі стандартним HTML `disabled` кнопка **вилучається з порядку табуляції** (tab order) — скрінрідер-користувачі можуть не знайти кнопку взагалі, або вона не зачитається при Tab-навігації. Це проблема.

**Плюси:**
- Простота реалізації.
- Запобігає помилці до її виникнення.
- Видима інструкція для зрячих користувачів.

**Мінуси:**
- HTML `disabled` ховає кнопку від деяких скрінрідерів (NVDA в browse mode все ж зачитує disabled кнопки, але при фокус-навігації через Tab — ні).
- Не вирішує проблему гарячої клавіші.

---

### Варіант B: `aria-disabled` замість HTML `disabled` + `aria-describedby` (покращена версія A)

**Суть:** використовувати `aria-disabled="true"` замість HTML `disabled`. Це тримає кнопку у фокус-порядку, скрінрідер зачитує її як "Start recording, dimmed" або "Start recording, unavailable", а через `aria-describedby` — пояснює причину.

**UI (RecordControls.svelte):**
```svelte
<button
  type="button"
  onclick={canRecord ? onstart : undefined}
  aria-disabled={!canRecord}
  aria-describedby={!canRecord ? "start-hint" : undefined}
  aria-label="Start recording (Alt+S)"
  class="btn btn-start"
  class:disabled={!canRecord}
>
  Start
</button>
{#if !canRecord}
  <p id="start-hint" class="hint">{readinessHint}</p>
{/if}
```

**Що почує скрінрідер при фокусі на кнопці:**
> "Start recording, button, dimmed. Оберіть мікрофон для початку запису."

**Важливо (WAI-ARIA APG, Button Pattern):**
> "When the action associated with a button is unavailable, the button has `aria-disabled` set to `true`."

Це офіційна рекомендація W3C для недоступних кнопок.

**Перевага `aria-disabled` над `disabled`:**

| Характеристика | `disabled` | `aria-disabled="true"` |
|---|---|---|
| У фокус-порядку (Tab) | ❌ Ні | ✅ Так |
| Скрінрідер може знайти | Частково (browse mode) | ✅ Завжди |
| Оголошує "dimmed/unavailable" | Так | Так |
| Блокує onclick автоматично | Так | ❌ Треба вручну |
| `aria-describedby` зачитується | Так (якщо знайде) | ✅ Так (при фокусі) |

**Плюси:**
- Кнопка доступна для виявлення скрінрідером у всіх режимах навігації.
- Ясне пояснення причини через `aria-describedby`.
- Відповідає WAI-ARIA APG.

**Мінуси:**
- Не вирішує проблему гарячої клавіші.
- Потрібна ручна перевірка `onclick`.

---

### Варіант C: Попереджувальний звук для гарячої клавіші

**Суть:** додати новий тип звуку `SoundKind::Warning` (або `Error`) до звукового рушія. Коли гаряча клавіша натиснута і запис неможливий — грати цей звук.

**Реалізація (hotkey.rs):**
```rust
RecordingState::Idle => {
    if let Err(e) = do_start_recording(app) {
        log::error!("Hotkey start failed: {e}");
        // Відтворити попереджувальний звук
        play_warning_sound(app);
        let _ = app.emit("recording-error", e);
    }
}
```

Або ще краще — **проактивна перевірка** перед спробою запису:
```rust
RecordingState::Idle => {
    let settings = settings::read_settings();
    if !can_start_recording(&settings) {
        play_warning_sound(app);
        return; // навіть не пробувати
    }
    if let Err(e) = do_start_recording(app) { ... }
}
```

**Звук:** короткий різкий тон (200-400 мс), чітко відрізняється від start/pause/stop звуків. Щось на кшталт "бап-бап" або один низький тон.

**Плюси:**
- Працює навіть коли вікно не у фокусі — звук грає незалежно від UI.
- Зрозумілий фідбек для скрінрідер-користувачів.
- Природно вписується в існуючу звукову систему.

**Мінуси:**
- Сам по собі не пояснює *що саме* не так. Користувач знає що "щось не так", але не знає що саме.

---

### Варіант D: Текст інструкції прямо на кнопці

**Суть:** замість "Start" писати "Start — оберіть мікрофон".

**Приклад:**
```svelte
<button aria-disabled="true">
  Start — select microphone
</button>
```

**Що почує скрінрідер:**
> "Start — select microphone, button, dimmed"

**Плюси:**
- Все пояснення в одному елементі.
- Не потрібен окремий текст підказки.

**Мінуси:**
- **Змішує дію та інструкцію** — не відповідає WAI-ARIA рекомендації "Convey function or purpose, not diagnostics". Кнопка повинна називатись відповідно до своєї дії.
- Довгий текст на кнопці виглядає некрасиво.
- Коли є кілька проблем одночасно (немає ні мікрофону, ні loopback), текст стає надмірно довгим.
- **Погана практика:** W3C APG рекомендує використовувати `aria-describedby` для додаткової інформації, а не включати її в ім'я кнопки.

**Висновок:** Не рекомендується.

---

### Варіант E: Windows Toast-повідомлення (Tauri Notification Plugin)

**Суть:** при помилці через гарячу клавішу показувати системне toast-повідомлення Windows.

**Технічна можливість:**
- Tauri v2 має `tauri-plugin-notification`.
- На Windows працює повноцінно, але **тільки для встановлених додатків** (у dev-режимі показує "PowerShell" як ім'я/іконку).
- Скрінрідери (Narrator, NVDA) зазвичай зачитують Windows toast-повідомлення.

**Плюси:**
- Видно і чутно навіть коли вікно не у фокусі.
- Скрінрідер зачитає toast.
- Може містити детальну інформацію.

**Мінуси:**
- **Вимагає дозволу на повідомлення** — користувач може відмовити або вимкнути.
- **Надто нав'язливо** для частої операції. Toast-повідомлення призначені для важливих/рідкісних подій, а не для повторюваних помилок конфігурації.
- Toast з'являється через кілька секунд (системна затримка) — поганий UX для миттєвого зворотного зв'язку.
- **Не проактивний** — все ще реактивний підхід (помилка після спроби).
- Додає нову залежність.
- У dev-режимі показує "PowerShell" — погано для тестування.

**Висновок:** Уникати як основний механізм. Можна розглянути як додатковий канал у майбутньому, але не для цієї задачі.

---

### Варіант F: Статусна панель готовності

**Суть:** у вікні завжди відображається стан готовності: "Готово до запису" або "Потрібно: оберіть мікрофон".

**Плюси:**
- Завжди видима інформація.
- Ненав'язливо.

**Мінуси:**
- Займає місце на екрані.
- Не вирішує проблему гарячої клавіші.
- Дублює функціональність hint-тексту при кнопці (Варіант B).

**Висновок:** Не має достатніх переваг над Варіантом B.

---

## 4. Порівняльна таблиця

| Критерій | A: disabled | B: aria-disabled | C: Звук | D: Текст на кнопці | E: Toast | F: Статус |
|---|---|---|---|---|---|---|
| Проактивний UI | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ |
| Скрінрідер: фокус-доступність | ❌ | ✅ | N/A | ✅ | ✅ | ✅ |
| Скрінрідер: пояснення причини | ⚠️ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Працює без фокусу вікна | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Миттєвий зворотний зв'язок | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Відповідність WAI-ARIA | ⚠️ | ✅ | N/A | ❌ | N/A | ✅ |
| Не вимагає дозволів | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Простота реалізації | Легко | Легко | Середньо | Легко | Складно | Легко |

---

## 5. Рекомендоване рішення: Комбінація B + C

**Найкраще рішення — поєднання Варіантів B і C**, яке забезпечує:
1. **Проактивний UI** (Варіант B) — запобігає помилці через кнопку.
2. **Аудіо-зворотний зв'язок** (Варіант C) — забезпечує фідбек для гарячої клавіші.
3. **Існуючий `role="alert"` — збережено** як fallback для непередбачуваних runtime помилок.

### 5.1. Зміни у фронтенді

#### Обчислення стану готовності

У `App.svelte` (або в store):

```typescript
let canRecord = $derived(() => {
  const mode = recording.outputMode;
  if (mode === "Microphone" || mode === "Mix") {
    if (!recording.selectedMic) return false;
  }
  if (mode === "Loopback" || mode === "Mix") {
    if (!recording.selectedLoopback) return false;
  }
  return true;
});

let readinessHint = $derived(() => {
  const mode = recording.outputMode;
  const needsMic = (mode === "Microphone" || mode === "Mix") && !recording.selectedMic;
  const needsLoopback = (mode === "Loopback" || mode === "Mix") && !recording.selectedLoopback;
  
  if (needsMic && needsLoopback) return "Select microphone and system audio device";
  if (needsMic) return "Select a microphone";
  if (needsLoopback) return "Select a system audio device";
  return "";
});
```

#### Компонент RecordControls

Передавати `canRecord` і `readinessHint` як props:

```svelte
<script lang="ts">
  interface Props {
    recordingState: RecordingState;
    canRecord: boolean;
    readinessHint: string;
    onstart: () => void;
    onpause: () => void;
    onresume: () => void;
    onstop: () => void;
  }
</script>

{#if recordingState === "Idle"}
  <button
    type="button"
    onclick={canRecord ? onstart : undefined}
    aria-disabled={!canRecord}
    aria-describedby={!canRecord ? "start-hint" : undefined}
    aria-label="Start recording (Alt+S)"
    class="btn btn-start"
    class:disabled={!canRecord}
  >
    {#if altPressed}<u>S</u>tart{:else}Start{/if}
  </button>
  {#if !canRecord}
    <p id="start-hint" class="hint" role="note">{readinessHint}</p>
  {/if}
{/if}
```

CSS для disabled-стану:
```css
.btn-start.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: #86efac; /* світліший зелений */
}
```

### 5.2. Зміни у бекенді

#### Новий звук SoundKind::Warning

У `sounds.rs`:
```rust
const WARNING_SOUND: &[u8] = include_bytes!("../sounds/warning.ogg");

pub enum SoundKind {
    Start,
    Pause,
    Stop,
    Warning,  // Новий
}

// У match:
SoundKind::Warning => WARNING_SOUND,
```

#### Файл звуку

Створити `src-tauri/sounds/warning.ogg` — короткий (200-300 мс) різкий звук, що чітко відрізняється від інших. Наприклад, низькочастотний подвійний тон або характерний "buzz".

#### Проактивна перевірка в hotkey.rs

```rust
RecordingState::Idle => {
    // Проактивно перевірити конфігурацію перед спробою
    let settings = settings::read_settings();
    let mode = &settings.output_mode;
    let needs_mic = mode == "Microphone" || mode == "Mix" || mode == "microphone" || mode == "mix";
    let needs_loopback = mode == "Loopback" || mode == "Mix" || mode == "loopback" || mode == "mix";
    
    let mic_missing = needs_mic && settings.selected_mic.is_none();
    let loopback_missing = needs_loopback && settings.selected_loopback.is_none();
    
    if mic_missing || loopback_missing {
        // Грати попереджувальний звук — працює навіть без фокусу вікна
        if let Some(state) = app.try_state::<SharedState>() {
            if let Ok(s) = state.lock() {
                play_sound(&s, sounds::SoundKind::Warning);
            }
        }
        return;
    }
    
    if let Err(e) = do_start_recording(app) {
        log::error!("Hotkey start failed: {e}");
        // Для непередбачуваних помилок — теж грати warning
        if let Some(state) = app.try_state::<SharedState>() {
            if let Ok(s) = state.lock() {
                play_sound(&s, sounds::SoundKind::Warning);
            }
        }
        let _ = app.emit("recording-error", e);
    }
}
```

### 5.3. Збережений fallback

Існуючий `<div class="error" role="alert">` в `App.svelte` зберігається для runtime помилок, які не можна передбачити (наприклад, пристрій відключено в момент запису).

---

## 6. Потік взаємодії після впровадження

### Сценарій 1: Кнопка у вікні, немає мікрофону

1. Режим: Mix, мікрофон не обрано.
2. Кнопка "Start" видна, але візуально приглушена (opacity 0.5).
3. Під кнопкою текст: "Select a microphone".
4. **Скрінрідер при фокусі на кнопці:** "Start recording, button, dimmed. Select a microphone."
5. Натискання кнопки/Enter/Space — нічого не відбувається (onclick не призначений).
6. ✅ Користувач одразу знає що робити.

### Сценарій 2: Гаряча клавіша, вікно не у фокусі, немає мікрофону

1. Користувач натискає глобальну гарячу клавішу.
2. Бекенд перевіряє конфігурацію → мікрофон відсутній.
3. **Лунає попереджувальний звук** — користувач одразу розуміє, що запис не почався.
4. Запис не починається, стан не змінюється.
5. ✅ Чіткий зворотний зв'язок без залежності від вікна.

### Сценарій 3: Все сконфігуровано, runtime помилка

1. Мікрофон обрано, кнопка активна.
2. Користувач натискає "Start".
3. Під час ініціалізації capture виникає помилка (пристрій відключено).
4. З'являється `role="alert"` повідомлення з деталями помилки.
5. Якщо через гарячу клавішу — додатково лунає warning звук.
6. ✅ Fallback механізм працює для непередбачуваних ситуацій.

### Сценарій 4: Все сконфігуровано, успішний запис

1. Пристрої обрано, кнопка активна.
2. Натискання кнопки або гарячої клавіші → запис починається → звук Start.
3. ✅ Поведінка не змінюється для нормального сценарію.

---

## 7. Підсумок рекомендацій

| Компонент | Зміна | Пріоритет |
|---|---|---|
| `RecordControls.svelte` | `aria-disabled` + `aria-describedby` + hint text | Високий |
| `App.svelte` | Обчислення `canRecord` та `readinessHint` | Високий |
| `sounds.rs` | Додати `SoundKind::Warning` | Високий |
| `sounds/warning.ogg` | Створити звуковий файл попередження | Високий |
| `hotkey.rs` | Проактивна перевірка + warning звук | Високий |
| `lib.rs` | Допоміжна функція `play_warning_sound` | Середній |
| Існуючий `role="alert"` | Зберегти як fallback для runtime помилок | — (не змінювати) |

---

## 8. Джерела

- [WAI-ARIA APG: Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/) — "When the action associated with a button is unavailable, the button has `aria-disabled` set to `true`."
- [WAI-ARIA APG: Providing Accessible Names and Descriptions](https://www.w3.org/WAI/ARIA/apg/practices/names-and-descriptions/) — використання `aria-describedby` для пояснення.
- [W3C WAI: User Notification](https://www.w3.org/WAI/tutorials/forms/notifications/) — патерни для зворотного зв'язку.
- [Tauri v2: Notification Plugin](https://v2.tauri.app/plugin/notification/) — обмеження на Windows.
- Аналіз коду AudioCaptor: `src-tauri/src/lib.rs`, `hotkey.rs`, `sounds.rs`, `RecordControls.svelte`, `App.svelte`.

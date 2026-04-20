# Структура даних портативного додатка та іменування exe

> **Тип задачі:** 🏗️ Архітектура / 📦 Вибір підходу
> **Стек:** Rust, Tauri v2, Svelte 5, Windows desktop
> **Дата:** 2026-04-20

## TL;DR

> Ідея з окремою папкою для даних — **правильна і доцільна**. Обрано `AudioCaptor-data/` за бажанням автора. Всередині: `settings.json`, `Recordings/`, `logs/`. Scoop-маніфест спрощується до `"persist": "AudioCaptor-data"` — один junction замість трьох. Ім'я exe `audio-captor.exe` — **нестандартне** для Windows GUI; Tauri v2 перейменовує бінарник за `productName`, зафіксовано `mainBinaryName: "AudioCaptor"`.

---

## Аналіз проблеми

### Поточний стан (до змін)

`portable.rs` створював файли/папки безпосередньо поряд з exe:

```
AudioCaptor.exe
settings.json
Recordings/
logs/
```

Scoop-маніфест: `"persist": ["settings.json", "logs", "Recordings"]`

### Проблеми

1. **Захаращення кореневої папки** — три окремих елементи поряд з exe
2. **Scoop: 3 junction/hard link** замість одного
3. **Hard link для файлів** (settings.json) — Scoop вимагає `pre_install` щоб не створити директорію замість файлу
4. **Зміна маніфесту при додаванні** нового елементу даних

---

## Розглянуті рішення

### Варіант A: `data/` (єдина папка)

Стандарт індустрії: VSCode portable, foobar2000, mpv.
- ✅ Найпоширеніший патерн
- ⚠️ Занадто загальне ім'я

**Рівень впевненості:** ✅ підтверджено

### Варіант B: `AudioCaptor-data/` (обраний)

- ✅ Однозначно зрозуміло, до якого додатка належить
- ✅ Не конфліктує з іншими `data/` папками
- ⚠️ Нестандартне ім'я, але працює ідентично

**Рівень впевненості:** ✅ підтверджено

### Варіант C: Залишити як є

- ❌ Проблема з hard link для settings.json в Scoop
- ❌ Кожне додавання → зміна маніфесту

---

## Порівняльна таблиця

| Критерій | A: `data/` | B: `AudioCaptor-data/` | C: Поточний |
|---|---|---|---|
| Scoop junction points | 1 | 1 | 3 (+ hard link) |
| Додавання нових даних | Без зміни маніфесту | Без зміни маніфесту | Зміна маніфесту |
| Бекап/портабельність | Одна папка | Одна папка | Три елементи |
| Впізнаваність | ✅ VSCode стандарт | ⚠️ Унікальне | ❌ Немає root |

---

## Рекомендація

**Обраний варіант:** B — `AudioCaptor-data/`

**Структура:**
```
AudioCaptor.exe
AudioCaptor-data/
├── settings.json
├── Recordings/
└── logs/
```

---

## Ім'я exe

| Аспект | `audio-captor.exe` | `AudioCaptor.exe` |
|---|---|---|
| Конвенція Windows GUI | ❌ Нестандартно | ✅ PascalCase |
| Task Manager | `audio-captor.exe` | `AudioCaptor.exe` |
| Tauri поведінка | Cargo default | Перейменовується за productName |

**Рішення:** Зафіксовано `"mainBinaryName": "AudioCaptor"` в `tauri.conf.json`.

---

## Реалізовані зміни

| Файл | Зміна |
|---|---|
| `portable.rs` | Додано `data_dir()` → `<exe>/AudioCaptor-data/`; `ensure_dirs()` використовує `data_dir()` |
| `settings.rs` | `settings_path()` → `data_dir()/settings.json` |
| `lib.rs` | Recording output, `get_recordings_dir()`, log plugin → `data_dir()` |
| `tray.rs` | Open recordings → `data_dir()` |
| `tauri.conf.json` | Додано `"mainBinaryName": "AudioCaptor"` |
| `bucket/audiocaptor.json` | `"persist": "AudioCaptor-data"` |

---

## Відкриті питання

- [ ] Чи варто підтримувати non-portable режим (`%AppData%`) на майбутнє?
- [ ] Чи потрібен прапор `AudioCaptor-data/portable.txt` для визначення режиму?

---

## Джерела

- [Scoop Wiki — Persistent data](https://github.com/ScoopInstaller/Scoop/wiki/Persistent-data) — механіка persist
- [Scoop Wiki — App Manifests](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests) — формат маніфесту
- [VSCode Portable Mode](https://code.visualstudio.com/docs/editor/portable) — патерн `data/`
- [PortableApps.com Format 3.9](https://portableapps.com/development/portableapps.com_format) — стандарт `Data/`
- [Tauri v2 Config Reference](https://tauri.app/reference/config/) — `productName`, `mainBinaryName`

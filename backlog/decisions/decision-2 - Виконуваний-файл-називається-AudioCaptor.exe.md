---
id: decision-2
title: Виконуваний файл називається AudioCaptor.exe
date: '2026-11-23T11:23:16Z'
status: accepted
---
## Context

<!-- SECTION:CONTEXT:BEGIN -->
Пакет Cargo називається `audio-captor`, тому за замовчуванням бінарний файл дістав би назву `audio-captor.exe`. Для віконної програми Windows така назва незвична: у теці й у Диспетчері завдань користувач очікує `AudioCaptor.exe`. Tauri v2 називає бінарний файл за `productName`, але це неявна поведінка. Рішення ухвалено 2026-04-20 (коміт ffb233c). Запис звірено з кодом 2026-11-23.
<!-- SECTION:CONTEXT:END -->

## Decision

<!-- SECTION:DECISION:BEGIN -->
Назву виконуваного файлу зафіксовано явно: `"mainBinaryName": "AudioCaptor"` у `src-tauri/tauri.conf.json`. Збірка дає `src-tauri/target/release/AudioCaptor.exe`.
<!-- SECTION:DECISION:END -->

## Consequences

<!-- SECTION:CONSEQUENCES:BEGIN -->
- На цю назву спираються Release workflow (файл релізу `AudioCaptor.exe`) і маніфест Scoop у репозиторії `ruslan-rv-ua/scoop-bucket`: поля `bin` і `shortcuts` та адреса `.../releases/download/v$version/AudioCaptor.exe` в `autoupdate`. Зміна назви зламає оновлення через Scoop, доки маніфест не виправлено.
- Назва пакета Cargo лишається `audio-captor`, а бібліотеки — `audio_captor_lib`.
<!-- SECTION:CONSEQUENCES:END -->

## Alternatives

<!-- SECTION:ALTERNATIVES:BEGIN -->
- `audio-captor.exe`, назва за замовчуванням від Cargo. Відхилено: незвична для віконної програми Windows.
<!-- SECTION:ALTERNATIVES:END -->


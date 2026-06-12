# yplayer

Консольный проигрыватель Яндекс Музыки на Rust с TUI (ratatui) и libmpv.

## Особенности

- **TUI интерфейс** — vim-стиль навигации (j/k/h/l, Space, Enter, Tab)
- **libmpv** — высококачественное воспроизведение через нативные биндинги
- **Кэш треков** — скачивание в `~/Library/Caches/yplayer/tracks/` для офлайн-прослушивания
- **Библиотека** — «Мне нравится», плейлисты (с загрузкой треков), поиск
- **Очередь воспроизведения** — shuffle (Fisher-Yates), repeat (off/one/all), next/prev
- **Прогресс-бар** — время, громкость, статус shuffle/repeat
- **Авторизация** — OAuth через браузер (вставка токена в терминал)

## Требования

- macOS (Apple Silicon / Intel) или Linux
- Rust 1.75+
- `mpv` (через Homebrew: `brew install mpv`)
- Токен Яндекс Музыки (OAuth)

## Установка

```bash
# Клонирование и сборка
git clone https://github.com/yourname/yplayer
cd yplayer
cargo build --release

# Бинарник будет в target/release/yplayer
```

## Настройка

При первом запуске откроется браузер для OAuth-авторизации:
1. Войдите в Яндекс и разрешите доступ
2. Скопируйте `access_token` из адресной строки (после `#access_token=...`)
3. Вставьте в терминал и нажмите Enter

Токен сохраняется в `~/Library/Application Support/yplayer/config.toml`.

## Горячие клавиши

| Клавиша | Действие |
|---------|----------|
| `j` / `k` | Вверх / вниз |
| `h` / `l` | Назад / вперёд (sidebar ↔ контент) |
| `Enter` | Воспроизвести / открыть плейлист |
| `Space` | Play / Pause |
| `n` / `p` | Next / Prev |
| `+` / `-` | Громкость + / - |
| `Ctrl+d` / `Ctrl+u` | Seek +5s / -5s |
| `s` | Shuffle on/off |
| `r` | Repeat: off → all → one |
| `/` | Поиск |
| `z` / `L` | Now Playing |
| `a` | Play All (весь список) |
| `Tab` / `Shift+Tab` | Следующий / предыдущий вид библиотеки |
| `Esc` | Назад / выход из поиска |
| `q` / `Ctrl+c` | Выход |

## Структура проекта

```
src/
├── api/           # Яндекс Музыка API клиент
│   ├── auth.rs    # OAuth Device Flow → ручной ввод токена
│   ├── client.rs  # HTTP запросы, download info, lyrics
│   └── models.rs  # YTrack, YAlbum, YPlaylist, YArtist...
├── app/           # Приложение
│   ├── config.rs  # TOML конфиг + токен
│   ├── keybindings.rs # Action enum + key mapping
│   └── state.rs   # AppState, Screen, Focus
├── player/        # Аудио движок
│   ├── mpv.rs     # libmpv-sys обёртка (unsafe)
│   ├── player.rs  # Player + кэш + очередь
│   ├── queue.rs   # PlaybackQueue (shuffle perm, repeat)
│   └── cache.rs   # TrackCache (файловый кэш)
└── ui/            # TUI (ratatui)
    ├── screens/   # Library, PlaylistDetail, Search, NowPlaying, Queue
    └── components/player_bar.rs # Нижняя панель прогресса
```

## Архитектура

- **Player** — синглтон, владеет `MpvPlayer`, `PlaybackQueue`, `TrackCache`
- **MpvPlayer** — прямые C-биндинги к `libmpv`, `loadfile replace`, event polling
- **PlaybackQueue** — Fisher-Yates shuffle при включении, сохраняет порядок
- **TrackCache** — `~/Library/Caches/yplayer/tracks/{id}.mp3`, играет локальный файл
- **YandexClient** — reqwest + OAuth заголовки, парсинг XML download-info (MD5 подпись)

## Полезные команды

```bash
# Очистка кэша
# В плеере: пока нет UI, можно удалить вручную
rm -rf ~/Library/Caches/yplayer

# Сброс токена
rm ~/Library/Application\ Support/yplayer/config.toml

# Запуск из любой директории (после cargo install)
cargo install --path .
yplayer
```

## Известные ограничения

- Только треки (нет радио, подкастов)
- Обложки не отображаются (планируется через ratatui-image + sixel)
- Нет редактирования плейлистов
- Текст песен — только через отдельный экран (не реализован)

## Лицензия

MIT OR Apache-2.0
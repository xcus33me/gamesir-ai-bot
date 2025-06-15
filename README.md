# 🐓 GameSir AI Bot 🐓

Профессиональный петушок-бот искусственный интеллект слой 2

## 🐔 Сommands

| Команда | Описание |
|---------|----------|
| `!join` | Подключиться к голосовому каналу |
| `!play <URL или название>` | Воспроизвести музыку |
| `!stop` | Остановить воспроизведение |
| `!skip` | Пропустить текущий трек |
| `!queue` | Показать очередь |
| `!leave` | Покинуть голосовой канал |

## 📋 Dependency

### System dependency:
- **Rust 1.87+** - для компиляции
- **libopus-dev** - кодек для Discord аудио
- **ffmpeg** - обработка аудио/видео
- **yt-dlp** - загрузка с YouTube


## 🐳 Docker 

### Быстрый запуск:

```bash
# 1. Клонировать репозиторий
git clone <your-repo-url>
cd gamesir-ai-bot

# 2. Создать .env файл
cp .env.example .env
# Отредактировать .env и добавить токены

# 3. Запустить через Docker Compose
docker-compose up -d

# 4. Просмотр логов
docker-compose logs -f
```

### Управление:
```bash
docker-compose up -d      # Запуск в фоне
docker-compose down       # Остановка
docker-compose restart    # Перезапуск
docker-compose logs -f    # Логи в реальном времени
```
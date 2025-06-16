#!/bin/bash

# Usage: ./deploy.sh user@server.com

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SERVER=$1
if [ -z "$SERVER" ]; then
    echo -e "${RED}Использование: ./deploy.sh user@server.com${NC}"
    exit 1
fi

echo -e "${GREEN}🚀 Начинаю деплой на $SERVER${NC}"

echo -e "${YELLOW}📦 Сборка проекта для Linux...${NC}"
cargo build --release --target x86_64-unknown-linux-gnu

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Ошибка сборки${NC}"
    exit 1
fi

DEPLOY_DIR="deploy"
rm -rf $DEPLOY_DIR
mkdir -p $DEPLOY_DIR

echo -e "${YELLOW}📋 Подготовка файлов...${NC}"
cp target/x86_64-unknown-linux-gnu/release/gamesir-ai-bot $DEPLOY_DIR/
cp .env.example $DEPLOY_DIR/
chmod +x $DEPLOY_DIR/gamesir-ai-bot

cat > $DEPLOY_DIR/server-install.sh << 'EOF'
#!/bin/bash

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🔧 Установка зависимостей на сервере...${NC}"

# Обновляем пакеты
sudo apt update

# Устанавливаем только runtime зависимости
sudo apt install -y \
    libopus0 \
    ffmpeg \
    python3-pip \
    ca-certificates

# Устанавливаем yt-dlp
echo -e "${YELLOW}📥 Установка yt-dlp...${NC}"
sudo pip3 install --break-system-packages yt-dlp

# Создаем символическую ссылку
sudo ln -sf /usr/local/bin/yt-dlp /usr/local/bin/youtube-dl

# Создаем systemd service
echo -e "${YELLOW}⚙️ Создание systemd service...${NC}"
BOT_DIR="/opt/gamesir-ai-bot"
sudo mkdir -p $BOT_DIR

# Копируем файлы
sudo cp gamesir-ai-bot $BOT_DIR/
sudo cp .env.example $BOT_DIR/.env
sudo chown -R $USER:$USER $BOT_DIR

# Создаем service файл
sudo tee /etc/systemd/system/gamesir-ai-bot.service > /dev/null << EOL
[Unit]
Description=GameSir AI Discord Bot
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$BOT_DIR
ExecStart=$BOT_DIR/gamesir-ai-bot
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOL

# Перезагружаем systemd и включаем автозапуск
sudo systemctl daemon-reload
sudo systemctl enable gamesir-ai-bot

echo -e "${GREEN}✅ Установка завершена!${NC}"
echo -e "${YELLOW}📝 Не забудьте:${NC}"
echo "1. Отредактировать $BOT_DIR/.env с вашими токенами"
echo "2. Запустить бота: sudo systemctl start gamesir-ai-bot"
echo "3. Проверить статус: sudo systemctl status gamesir-ai-bot"
echo "4. Смотреть логи: journalctl -u gamesir-ai-bot -f"
EOF

chmod +x $DEPLOY_DIR/server-install.sh

echo -e "${YELLOW}📤 Загрузка на сервер...${NC}"
scp -r $DEPLOY_DIR/* $SERVER:~/

echo -e "${YELLOW}🔧 Запуск установки на сервере...${NC}"
ssh $SERVER 'chmod +x server-install.sh && ./server-install.sh'

echo -e "${GREEN}🎉 Деплой завершен!${NC}"
echo -e "${YELLOW}📋 Следующие шаги:${NC}"
echo "1. ssh $SERVER"
echo "2. nano /opt/gamesir-ai-bot/.env  # добавить токены"
echo "3. sudo systemctl start gamesir-ai-bot"
echo "4. sudo systemctl status gamesir-ai-bot"

rm -rf $DEPLOY_DIR 
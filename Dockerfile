FROM rust:1.87-bookworm as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libopus-dev \
    build-essential \
    autoconf \
    automake \
    libtool \
    m4 \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY commands/ ./commands/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libopus0 \
    ffmpeg \
    python3 \
    python3-pip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --break-system-packages yt-dlp

RUN ln -s /usr/local/bin/yt-dlp /usr/local/bin/youtube-dl

RUN useradd -m -u 1000 bot

COPY --from=builder /app/target/release/gamesir-ai-bot /usr/local/bin/gamesir-ai-bot

RUN chmod +x /usr/local/bin/gamesir-ai-bot

USER bot
WORKDIR /home/bot

EXPOSE 8080

CMD ["gamesir-ai-bot"] 
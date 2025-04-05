# build image
FROM rust:1.85.1 as build
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target cargo build --release && cp target/release/rs-bot /rs-bot

# app image
FROM rust:1.85.1-slim-bookworm as bot

RUN apt-get update && apt-get install -y \
    libssl3 \
    libglib2.0-0 \
    libnss3 \
    libgconf-2-4 \
    libxss1 \
    libasound2 \
    libu2f-udev \
    libatk-bridge2.0-0 \
    libgtk-3-0 \
    libdbus-glib-1-2 \
    chromium \
    chromium-driver \
    && apt-get clean

RUN chromium --version && chromedriver --version

COPY --from=build /rs-bot /rs-bot
CMD sh -c "chromedriver --port=9515 --verbose --no-sandbox --headless --disable-dev-shm-usage & /rs-bot"

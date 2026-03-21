# SplitSmart

SplitSmart is a Telegram Bot plus Telegram Mini App for fair group expense splitting during a trip.

The bot posts an `Open SplitSmart 👇` message into the chat, with a Telegram Web App button and a fallback browser button that both point to:

`https://<PUBLIC_BASE_URL>/app?chat_id=<chat_id>`

The backend is server-authoritative:

- participants exist only after they open the Mini App and complete signed Telegram bootstrap
- all identity and membership checks happen on the backend
- expense calculations use integer cents only
- the active trip state is scoped to one open session per chat

## Features

- opt-in participant registration per Telegram chat
- shared spend creation with payer selection
- `ABS` split mode for exact amounts
- `PERCENT` split mode with deterministic largest-remainder rounding
- live balances and greedy settlement suggestions
- `/report` bot command for the current session
- `/reset` bot command and API flow that posts the current report, closes the session, and opens a new one while keeping participants

## Environment Variables

Required:

- `SPLIT_SMART_BOT_TOKEN`: primary bot token source
- `PUBLIC_BASE_URL`: public HTTPS base URL, for example `https://example.ngrok-free.app`
- `SQLITE_PATH`: SQLite database path, for example `data/splitsmart.db`
- `TELEGRAM_BOT_USERNAME`: bot username without `@`

Optional compatibility fallback:

- `BOT_TOKEN`: used only if `SPLIT_SMART_BOT_TOKEN` is unset

## Local Setup

1. Create a public HTTPS tunnel. Telegram Mini Apps require HTTPS.

Using `ngrok`:

```bash
ngrok http 8080
```

Using `cloudflared`:

```bash
cloudflared tunnel --url http://localhost:8080
```

2. Export the environment variables:

```bash
export SPLIT_SMART_BOT_TOKEN=123456:your-bot-token
export PUBLIC_BASE_URL=https://your-public-domain.example
export SQLITE_PATH=data/splitsmart.db
export TELEGRAM_BOT_USERNAME=split_smart_bot
```

3. Start the app:

```bash
cargo run
```

4. The bot and HTTP server run in the same process. The HTTP server listens on `0.0.0.0:8080`.

## Database

SQLite migrations live in [migrations/0001_init.sql](/Users/sild/Projects/Personal/chat_bots/tg_split_smart_bot/migrations/0001_init.sql).

Startup behavior:

- creates the SQLite parent directory if needed
- enables SQLite foreign keys
- runs migrations automatically

## Telegram BotFather Setup

1. Create the bot in BotFather and copy the token.
2. Set the bot username and export it as `TELEGRAM_BOT_USERNAME`.
3. Configure the Mini App domain in BotFather to match `PUBLIC_BASE_URL`.
4. Add the bot to the target group or supergroup.
5. Start the bot in a private chat if you also want to test the private-chat flow.

## Telegram Mini App Flow

When the bot is added to a chat, or when `/start` is used, the bot sends `Open SplitSmart 👇` with two buttons:

- Telegram Web App button
- fallback URL button

The bot then attempts to pin that message. If pinning fails, the app logs the failure and continues.

Users become participants only when they open the Mini App from Telegram and the backend successfully validates `Telegram.WebApp.initData`.

## API Overview

- `POST /api/bootstrap`
- `POST /api/spends`
- `POST /api/report`
- `POST /api/reset`

All JSON API routes require signed Telegram Mini App `init_data`. A plain browser fallback can open `/app`, but backend actions are rejected without valid signed Telegram data.

## Quality Gates

The project is intended to pass:

```bash
cargo +nightly fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-targets --all-features
```

## Important Telegram Limitations

- Telegram bots cannot fetch an arbitrary full group member list. SplitSmart only knows about users who opened the Mini App and registered themselves.
- The fallback browser button may open outside Telegram and therefore may not include signed `initData`. Backend actions reject missing or invalid signed data.
- Reset authorization relies on Telegram `getChatMember`, not on frontend claims.

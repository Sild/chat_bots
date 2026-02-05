# SplitSmart (Telegram Bot + Mini App)

Production-grade Telegram bot and WebApp for splitting group costs.

## Environment

- `BOT_TOKEN` - Bot token from BotFather
- `PUBLIC_BASE_URL` - Public HTTPS base URL (e.g. `https://your-domain.tld`)
- `SQLITE_PATH` - Path to SQLite database file (e.g. `data/splitsmart.db`)
- `WEBAPP_URL` - WebApp base URL (defaults to `${PUBLIC_BASE_URL}/app`)
- `TELEGRAM_BOT_USERNAME` - Bot username (without `@`)

## Local development

1. Run ngrok to expose the HTTP server:

```bash
ngrok http 8080
```

2. Set env vars:

```bash
export BOT_TOKEN=...
export PUBLIC_BASE_URL=https://<your-ngrok-domain>
export SQLITE_PATH=data/splitsmart.db
export WEBAPP_URL=${PUBLIC_BASE_URL}/app
export TELEGRAM_BOT_USERNAME=split_smart_bot
```

3. Start the app:

```bash
cargo run
```

## Telegram setup

- In BotFather, enable Mini App/WebApp and set the allowed WebApp domain to your HTTPS domain.
- Add the bot to a group or start a private chat.
- The bot posts an "Open SplitSmart" button to launch the WebApp. The fallback browser button works without Telegram auth, but the backend rejects API calls without valid `initData`.

## SQLx notes

- Migrations are in `migrations/` and run on startup.
- For compile-time query checks, set `DATABASE_URL` to your SQLite URL and run:

```bash
cargo sqlx prepare -- --all-targets
```

Then build with `SQLX_OFFLINE=true` (and optionally `--features sqlx-offline`) for offline checks.

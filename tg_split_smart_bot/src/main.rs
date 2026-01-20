mod new_group_handler;

use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Me},
};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    storage_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Entry {
    user_id: i64,
    user_name: String,
    number: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddRequest {
    user_id: i64,
    user_name: String,
    number: i64,
}

async fn ensure_storage(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Create with header for readability
        let mut wtr = WriterBuilder::new().from_path(path)?;
        // header
        let _ = wtr.write_record(["user_id", "user_name", "number"])?;
        wtr.flush()?;
    }
    Ok(())
}

fn read_entries(path: &PathBuf) -> Result<Vec<Entry>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut rdr = ReaderBuilder::new().from_path(path)?;
    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: Entry = result?;
        entries.push(record);
    }
    Ok(entries)
}

fn write_entry(path: &PathBuf, entry: &Entry) -> Result<()> {
    let file_exists = path.exists();
    let mut wtr = WriterBuilder::new().from_path(path)?;
    if !file_exists {
        let _ = wtr.write_record(["user_id", "user_name", "number"])?;
    }
    wtr.serialize(entry)?;
    wtr.flush()?;
    Ok(())
}

async fn axum_add(State(state): State<Arc<AppState>>, Json(req): Json<AddRequest>) -> impl IntoResponse {
    let entry = Entry { user_id: req.user_id, user_name: req.user_name, number: req.number };
    match write_entry(&state.storage_path, &entry) {
        Ok(_) => (StatusCode::OK, Json(entry)).into_response(),
        Err(e) => {
            warn!(error=?e, "Failed to write entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

async fn axum_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match read_entries(&state.storage_path) {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            warn!(error=?e, "Failed to read entries");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

#[derive(Serialize)]
struct UsersResponse {
    users: Vec<(i64, String)>,
}

async fn axum_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match read_entries(&state.storage_path) {
        Ok(entries) => {
            let mut uniq = Vec::<(i64, String)>::new();
            for e in entries {
                if !uniq.iter().any(|(id, _)| *id == e.user_id) {
                    uniq.push((e.user_id, e.user_name));
                }
            }
            Json(UsersResponse { users: uniq }).into_response()
        }
        Err(e) => {
            warn!(error=?e, "Failed to read users");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

async fn axum_index() -> impl IntoResponse {
    Html(include_str!("../frontend/index.html")).into_response()
}

async fn run_server(state: Arc<AppState>) -> Result<()> {
    let cors = tower_http::cors::CorsLayer::very_permissive();
    let app = Router::new()
        .route("/", get(axum_index))
        .route("/api/add", post(axum_add))
        .route("/api/list", get(axum_list))
        .route("/api/users", get(axum_users))
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!(%addr, "Starting axum server");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn run_bot(state: Arc<AppState>) -> Result<()> {
    let token = dotenvy::var("SPLIT_SMART_BOT_TOKEN").expect("SPLIT_SMART_BOT_TOKEN not set");
    let bot = Bot::new(token);
    let miniapp_url = dotenvy::var("SPLIT_SMART_BOT_URL").unwrap_or_else(|_| "".to_string());
    let miniapp_url_mcm = miniapp_url.clone();

    info!(miniapp_url = %miniapp_url, "Bot started, configured miniapp URL");
    if miniapp_url.is_empty() {
        warn!("SPLIT_SMART_BOT_URL is empty; web_app button will have invalid URL");
    }

    // Handler for when bot is added to a group via my_chat_member update
    let my_chat_member_handler = Update::filter_my_chat_member().endpoint(move |bot: Bot, upd: teloxide::types::ChatMemberUpdated| {
        let miniapp_url = miniapp_url_mcm.clone();
        async move {
            let chat = &upd.chat;
            info!(chat_id = %chat.id, old_status = ?upd.old_chat_member.status(), new_status = ?upd.new_chat_member.status(), is_bot = upd.new_chat_member.user.is_bot, "my_chat_member update received");
            let was_out = matches!(upd.old_chat_member.status(), teloxide::types::ChatMemberStatus::Left | teloxide::types::ChatMemberStatus::Banned);
            let is_in = matches!(upd.new_chat_member.status(), teloxide::types::ChatMemberStatus::Member | teloxide::types::ChatMemberStatus::Administrator);
            let is_bot = upd.new_chat_member.user.is_bot;
            if was_out && is_in && is_bot {
                info!(chat_id = %chat.id, "Detected bot added to chat; sending SplitSmart button");
                match url::Url::parse(&miniapp_url) {
                    Ok(url) => {
                        let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::web_app(
                            "SplitSmart",
                            teloxide::types::WebAppInfo { url },
                        )]]);
                        if let Err(e) = bot
                            .send_message(chat.id, "Чтобы разделить счёт — откройте SplitSmart 👇")
                            .reply_markup(keyboard)
                            .await
                        {
                            warn!(error=?e, chat_id = %chat.id, "Failed to send SplitSmart button on my_chat_member");
                        }
                    }
                    Err(e) => {
                        warn!(error=?e, url=%miniapp_url, "Invalid miniapp URL; cannot send web_app button");
                    }
                }
            } else {
                info!(chat_id = %chat.id, was_out, is_in, is_bot, "Bot addition conditions not met; skipping send");
            }
            Ok::<(), teloxide::RequestError>(())
        }
    });

    // Existing message handler extended to also react on new_chat_members when bot is added
    let message_handler = Update::filter_message().branch(dptree::endpoint(move |bot: Bot, _me: Me, msg: Message| {
        let state = state.clone();
        let miniapp_url = miniapp_url.clone();
        async move {
            // If new chat members were added and one is a bot (us), send the mini app entry
            if let Some(new_members) = msg.new_chat_members() {
                let bot_added = new_members.iter().any(|u| u.is_bot);
                info!(chat_id = %msg.chat.id, bot_added, members = ?new_members, "message.new_chat_members received");
                if bot_added {
                    match url::Url::parse(&miniapp_url) {
                        Ok(url) => {
                            let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::web_app(
                                "SplitSmart",
                                teloxide::types::WebAppInfo { url },
                            )]]);
                            if let Err(e) = bot
                                .send_message(msg.chat.id, "Чтобы разделить счёт — откройте SplitSmart 👇")
                                .reply_markup(keyboard)
                                .await
                            {
                                warn!(error=?e, chat_id = %msg.chat.id, "Failed to send SplitSmart button on new_chat_members");
                            }
                            return Ok::<(), teloxide::RequestError>(())
                        }
                        Err(e) => {
                            warn!(error=?e, url=%miniapp_url, "Invalid miniapp URL in message handler");
                        }
                    }
                }
            }

            if let Some(text) = msg.text() {
                if text.starts_with("/start") {
                    let url = url::Url::parse(&miniapp_url).unwrap();
                    let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::web_app(
                        "Open Mini App",
                        teloxide::types::WebAppInfo { url },
                    )]]);
                    bot.send_message(msg.chat.id, "Welcome! Use the buttons or commands.").reply_markup(keyboard).await.ok();
                    return Ok::<(), teloxide::RequestError>(())
                }
                if let Some(rest) = text.strip_prefix("/add ") {
                    if let Ok(number) = rest.trim().parse::<i64>() {
                        let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);
                        let user_name = msg.from().map(|u| u.full_name()).unwrap_or_else(|| "unknown".to_string());
                        let entry = Entry { user_id, user_name, number };
                        if let Err(e) = write_entry(&state.storage_path, &entry) {
                            warn!(error=?e, "Failed to write entry");
                            bot.send_message(msg.chat.id, "Failed to save").await.ok();
                        } else {
                            bot.send_message(msg.chat.id, format!("Saved {}", number)).await.ok();
                        }
                        return Ok::<(), teloxide::RequestError>(())
                    } else {
                        bot.send_message(msg.chat.id, "Usage: /add {number}").await.ok();
                        return Ok::<(), teloxide::RequestError>(())
                    }
                }
                if text == "/list" {
                    match read_entries(&state.storage_path) {
                        Ok(entries) => {
                            if entries.is_empty() {
                                bot.send_message(msg.chat.id, "No data").await.ok();
                            } else {
                                let mut s = String::new();
                                for e in entries { s.push_str(&format!("{}: {}\n", e.user_name, e.number)); }
                                bot.send_message(msg.chat.id, s).await.ok();
                            }
                        }
                        Err(_) => { bot.send_message(msg.chat.id, "Failed to read").await.ok(); }
                    }
                    return Ok::<(), teloxide::RequestError>(())
                }
                if text == "/users" {
                    match read_entries(&state.storage_path) {
                        Ok(entries) => {
                            let mut uniq = Vec::<(i64, String)>::new();
                            for e in entries {
                                if !uniq.iter().any(|(id, _)| *id == e.user_id) { uniq.push((e.user_id, e.user_name)); }
                            }
                            let s = uniq.into_iter().map(|(_, name)| name).collect::<Vec<_>>().join("\n");
                            bot.send_message(msg.chat.id, if s.is_empty() { "No users".into() } else { s }).await.ok();
                        }
                        Err(_) => { bot.send_message(msg.chat.id, "Failed to read").await.ok(); }
                    }
                    return Ok::<(), teloxide::RequestError>(())
                }
            }
            Ok::<(), teloxide::RequestError>(())
        }
    }));

    let handler = dptree::entry()
        .branch(my_chat_member_handler)
        .branch(message_handler);

    info!("Starting bot polling");
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let storage_path = PathBuf::from("data/storage.csv");
    ensure_storage(&storage_path).await?;
    let state = Arc::new(AppState { storage_path });

    // Run both server and bot
    let s1 = run_server(state.clone());
    let s2 = run_bot(state.clone());
    tokio::try_join!(s1, s2)?;
    Ok(())
}

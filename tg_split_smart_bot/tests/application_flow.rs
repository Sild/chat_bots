use hmac::{Hmac, Mac};
use pretty_assertions::assert_eq;
use serde_json::json;
use sha2::Sha256;
use sqlx::sqlite::SqlitePoolOptions;
use tg_split_smart_bot::application::{AddSpendCommand, SplitInput, SplitSmartApplication};
use tg_split_smart_bot::infra::db::Database;
use url::form_urlencoded::Serializer;

type HmacSha256 = Hmac<Sha256>;

const BOT_TOKEN: &str = "123456:test-token";
const CHAT_ID: i64 = -1001234567890;

#[tokio::test]
async fn test_bootstrap_creates_chat_session_and_participant() {
    let application = test_application().await;
    let init_data = signed_init_data(CHAT_ID, 101, "Alice", Some("alice"), None);
    let auth = application
        .authenticate_chat_request(CHAT_ID, &init_data)
        .unwrap();

    let first = application.bootstrap(&auth).await.unwrap();
    assert_eq!(first.snapshot.chat.chat_id, CHAT_ID);
    assert_eq!(first.snapshot.session.id, 1);
    assert_eq!(first.snapshot.participants.len(), 1);
    assert_eq!(
        first.registration_message,
        Some("User Alice registered".to_string())
    );

    let second = application.bootstrap(&auth).await.unwrap();
    assert_eq!(second.snapshot.session.id, 1);
    assert_eq!(second.snapshot.participants.len(), 1);
    assert_eq!(second.registration_message, None);
}

#[tokio::test]
async fn test_add_spend_abs_updates_balances_and_transfers() {
    let application = seeded_three_participant_application().await;
    let auth = application
        .authenticate_chat_request(
            CHAT_ID,
            &signed_init_data(CHAT_ID, 101, "Alice", Some("alice"), None),
        )
        .unwrap();

    let result = application
        .add_spend(AddSpendCommand {
            auth,
            total: "12.40".to_string(),
            mode: "ABS".to_string(),
            payer_user_id: 101,
            splits: vec![
                SplitInput {
                    user_id: 101,
                    value: "0.00".to_string(),
                },
                SplitInput {
                    user_id: 202,
                    value: "5.10".to_string(),
                },
                SplitInput {
                    user_id: 303,
                    value: "7.30".to_string(),
                },
            ],
        })
        .await
        .unwrap();

    assert_eq!(result.snapshot.spends_count, 1);
    assert_eq!(result.snapshot.balances[0].net_cents, 1_240);
    assert_eq!(result.snapshot.balances[1].net_cents, -510);
    assert_eq!(result.snapshot.balances[2].net_cents, -730);
    assert_eq!(result.snapshot.transfers.len(), 2);
    assert_eq!(result.snapshot.transfers[0].from_user_id, 303);
    assert_eq!(result.snapshot.transfers[0].to_user_id, 101);
    assert_eq!(result.snapshot.transfers[0].amount_cents, 730);
    assert_eq!(result.snapshot.transfers[1].from_user_id, 202);
    assert_eq!(result.snapshot.transfers[1].to_user_id, 101);
    assert_eq!(result.snapshot.transfers[1].amount_cents, 510);
}

#[tokio::test]
async fn test_add_spend_percent_sums_exactly_to_total() {
    let application = seeded_three_participant_application().await;
    let auth = application
        .authenticate_chat_request(
            CHAT_ID,
            &signed_init_data(CHAT_ID, 202, "Bob", Some("bob"), None),
        )
        .unwrap();

    let result = application
        .add_spend(AddSpendCommand {
            auth,
            total: "1.00".to_string(),
            mode: "PERCENT".to_string(),
            payer_user_id: 202,
            splits: vec![
                SplitInput {
                    user_id: 101,
                    value: "33.33".to_string(),
                },
                SplitInput {
                    user_id: 202,
                    value: "33.33".to_string(),
                },
                SplitInput {
                    user_id: 303,
                    value: "33.34".to_string(),
                },
            ],
        })
        .await
        .unwrap();

    let total_allocated: i64 = result
        .snapshot
        .balances
        .iter()
        .map(|balance| balance.net_cents)
        .sum();
    assert_eq!(total_allocated, 0);
    assert_eq!(result.snapshot.spends_count, 1);
}

#[tokio::test]
async fn test_add_spend_rejects_unknown_payer_and_duplicate_users() {
    let application = seeded_three_participant_application().await;
    let auth = application
        .authenticate_chat_request(
            CHAT_ID,
            &signed_init_data(CHAT_ID, 101, "Alice", Some("alice"), None),
        )
        .unwrap();

    let payer_error = application
        .add_spend(AddSpendCommand {
            auth: auth.clone(),
            total: "10.00".to_string(),
            mode: "ABS".to_string(),
            payer_user_id: 999,
            splits: vec![
                SplitInput {
                    user_id: 101,
                    value: "3.34".to_string(),
                },
                SplitInput {
                    user_id: 202,
                    value: "3.33".to_string(),
                },
                SplitInput {
                    user_id: 303,
                    value: "3.33".to_string(),
                },
            ],
        })
        .await
        .unwrap_err();
    assert_eq!(
        payer_error.to_string(),
        "payer must be a registered participant"
    );

    let duplicate_error = application
        .add_spend(AddSpendCommand {
            auth,
            total: "10.00".to_string(),
            mode: "ABS".to_string(),
            payer_user_id: 101,
            splits: vec![
                SplitInput {
                    user_id: 101,
                    value: "5.00".to_string(),
                },
                SplitInput {
                    user_id: 101,
                    value: "5.00".to_string(),
                },
                SplitInput {
                    user_id: 303,
                    value: "0.00".to_string(),
                },
            ],
        })
        .await
        .unwrap_err();
    assert_eq!(duplicate_error.to_string(), "split users must be unique");
}

#[tokio::test]
async fn test_reset_closes_old_session_opens_new_one_and_keeps_participants() {
    let application = seeded_three_participant_application().await;
    let auth = application
        .authenticate_chat_request(
            CHAT_ID,
            &signed_init_data(CHAT_ID, 101, "Alice", Some("alice"), None),
        )
        .unwrap();

    application
        .add_spend(AddSpendCommand {
            auth: auth.clone(),
            total: "12.00".to_string(),
            mode: "ABS".to_string(),
            payer_user_id: 101,
            splits: vec![
                SplitInput {
                    user_id: 101,
                    value: "4.00".to_string(),
                },
                SplitInput {
                    user_id: 202,
                    value: "4.00".to_string(),
                },
                SplitInput {
                    user_id: 303,
                    value: "4.00".to_string(),
                },
            ],
        })
        .await
        .unwrap();

    let snapshot = application.reset_for_member(&auth).await.unwrap();
    assert_eq!(snapshot.session.id, 2);
    assert_eq!(snapshot.participants.len(), 3);
    assert_eq!(snapshot.spends_count, 0);
    assert!(
        snapshot
            .balances
            .iter()
            .all(|balance| balance.net_cents == 0)
    );
}

async fn seeded_three_participant_application() -> SplitSmartApplication {
    let application = test_application().await;
    bootstrap_user(&application, 101, "Alice", Some("alice")).await;
    bootstrap_user(&application, 202, "Bob", Some("bob")).await;
    bootstrap_user(&application, 303, "Carol", Some("carol")).await;
    application
}

async fn bootstrap_user(
    application: &SplitSmartApplication,
    user_id: i64,
    first_name: &str,
    username: Option<&str>,
) {
    let init_data = signed_init_data(CHAT_ID, user_id, first_name, username, None);
    let auth = application
        .authenticate_chat_request(CHAT_ID, &init_data)
        .unwrap();
    application.bootstrap(&auth).await.unwrap();
}

async fn test_application() -> SplitSmartApplication {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    SplitSmartApplication::new(Database::new(pool), BOT_TOKEN.to_string())
}

fn signed_init_data(
    chat_id: i64,
    user_id: i64,
    first_name: &str,
    username: Option<&str>,
    last_name: Option<&str>,
) -> String {
    let user = json!({
        "id": user_id,
        "first_name": first_name,
        "last_name": last_name,
        "username": username,
    });
    let chat = json!({
        "id": chat_id,
        "type": "supergroup",
        "title": "SplitSmart Test Chat",
    });

    let mut pairs = vec![
        ("auth_date".to_string(), "1700000000".to_string()),
        ("chat".to_string(), serde_json::to_string(&chat).unwrap()),
        ("query_id".to_string(), format!("query-{user_id}")),
        ("user".to_string(), serde_json::to_string(&user).unwrap()),
    ];
    pairs.sort_by(|left, right| left.0.cmp(&right.0));

    let data_check_string = pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").unwrap();
    secret_mac.update(BOT_TOKEN.as_bytes());
    let secret = secret_mac.finalize().into_bytes();

    let mut hash_mac = HmacSha256::new_from_slice(&secret).unwrap();
    hash_mac.update(data_check_string.as_bytes());
    let hash = hex::encode(hash_mac.finalize().into_bytes());

    let mut serializer = Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(&key, &value);
    }
    serializer.append_pair("hash", &hash);
    serializer.finish()
}

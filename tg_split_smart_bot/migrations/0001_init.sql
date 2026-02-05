CREATE TABLE IF NOT EXISTS chats (
    chat_id INTEGER PRIMARY KEY,
    chat_type TEXT NOT NULL,
    title TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS participants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    username TEXT,
    first_name TEXT,
    last_name TEXT,
    registered_at TEXT NOT NULL,
    UNIQUE(chat_id, user_id)
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS spends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    creator_user_id INTEGER NOT NULL,
    payer_user_id INTEGER NOT NULL,
    total_cents INTEGER NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('ABS','PERCENT'))
);

CREATE TABLE IF NOT EXISTS allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spend_id INTEGER NOT NULL,
    participant_user_id INTEGER NOT NULL,
    share_cents INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_participants_chat ON participants(chat_id);
CREATE INDEX IF NOT EXISTS idx_sessions_chat ON sessions(chat_id);
CREATE INDEX IF NOT EXISTS idx_spends_session ON spends(session_id);
CREATE INDEX IF NOT EXISTS idx_allocations_spend ON allocations(spend_id);

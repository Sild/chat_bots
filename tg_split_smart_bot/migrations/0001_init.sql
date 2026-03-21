CREATE TABLE IF NOT EXISTS chats (
    chat_id INTEGER PRIMARY KEY,
    chat_type TEXT NOT NULL,
    title TEXT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS participants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    username TEXT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NULL,
    registered_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE,
    UNIQUE(chat_id, user_id)
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS spends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    creator_user_id INTEGER NOT NULL,
    payer_user_id INTEGER NOT NULL,
    total_cents INTEGER NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('ABS', 'PERCENT')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spend_id INTEGER NOT NULL,
    participant_user_id INTEGER NOT NULL,
    share_cents INTEGER NOT NULL,
    FOREIGN KEY (spend_id) REFERENCES spends(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_participants_chat_user ON participants(chat_id, user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_chat_ended_at ON sessions(chat_id, ended_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_one_active_per_chat
    ON sessions(chat_id)
    WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_spends_session_created_at ON spends(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_allocations_spend_participant ON allocations(spend_id, participant_user_id);

CREATE TABLE IF NOT EXISTS inboxes (
    name TEXT PRIMARY KEY,
    is_locked BOOLEAN NOT NULL DEFAULT 0,
    password_hash TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_user_activity_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_received_at DATETIME,
    locked_at DATETIME
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    inbox_name TEXT NOT NULL,
    sender TEXT NOT NULL,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '(No Subject)',
    snippet TEXT NOT NULL DEFAULT '',
    body_text TEXT,
    body_html TEXT,
    raw_headers TEXT NOT NULL,
    detected_otp TEXT,
    detected_links TEXT,
    is_read BOOLEAN NOT NULL DEFAULT 0,
    is_favorite BOOLEAN NOT NULL DEFAULT 0,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    received_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inbox_name) REFERENCES inboxes(name) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes INTEGER NOT NULL,
    disk_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_messages_inbox ON messages(inbox_name, received_at DESC);
CREATE INDEX IF NOT EXISTS idx_inbox_user_active ON inboxes(last_user_activity_at);


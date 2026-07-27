CREATE TABLE IF NOT EXISTS installations (
    install_id TEXT PRIMARY KEY,
    secret_hash TEXT NOT NULL,
    install_secret TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS registrations (
    install_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    secret_hash TEXT NOT NULL,
    fcm_token TEXT,
    firebase_app_id TEXT NOT NULL,
    firebase_project_id TEXT NOT NULL,
    firebase_api_key TEXT NOT NULL,
    cert_sha1 TEXT,
    app_version INTEGER,
    app_version_name TEXT,
    target_sdk INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (install_id, app_id)
);

CREATE TABLE IF NOT EXISTS fcm_sessions (
    install_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    registration_data TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (install_id, app_id)
);

CREATE TABLE IF NOT EXISTS acked_messages (
    install_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    persistent_id TEXT NOT NULL,
    acked_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (install_id, app_id, persistent_id)
);

CREATE TABLE IF NOT EXISTS unified_push_registrations (
    install_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    connector_token TEXT NOT NULL,
    endpoint_token TEXT NOT NULL UNIQUE,
    vapid_pubkey TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (install_id, connector_token)
);

CREATE TABLE IF NOT EXISTS outbox (
    -- AUTOINCREMENT is load-bearing. The client cursor protocol assumes ids
    -- are never reused, so a rowid reissued after deletion would silently
    -- swallow or replay messages on every attached device.
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    install_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    connector_token TEXT,
    persistent_id TEXT,
    payload BLOB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS outbox_fcm_persistent_id
    ON outbox(install_id, app_id, persistent_id)
    WHERE persistent_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS outbox_socket_order
    ON outbox(install_id, id);

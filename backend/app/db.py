from __future__ import annotations

import os
import sqlite3
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator


SCHEMA = """
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    conversation_type TEXT NOT NULL,
    title TEXT NOT NULL,
    avatar_data TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_message_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    edited_at TEXT,
    reply_to TEXT,
    connector_id TEXT NOT NULL,
    platform_message_id TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_conversations_last_message
    ON conversations(last_message_at);
CREATE INDEX IF NOT EXISTS idx_messages_conversation
    ON messages(conversation_id, created_at);
CREATE TABLE IF NOT EXISTS telegram_accounts (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    phone TEXT,
    username TEXT,
    connected INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
"""


def database_path() -> Path:
    configured = os.getenv("SOCIALS_DATABASE_PATH")
    if configured:
        return Path(configured).expanduser()
    return Path.home() / ".local" / "share" / "socials" / "socials.db"


class Database:
    def __init__(self, path: Path | None = None) -> None:
        self.path = path or database_path()
        self.path.parent.mkdir(parents=True, exist_ok=True)

    @contextmanager
    def connection(self) -> Iterator[sqlite3.Connection]:
        connection = sqlite3.connect(self.path)
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys = ON")
        try:
            yield connection
            connection.commit()
        finally:
            connection.close()

    def initialize(self) -> None:
        with self.connection() as connection:
            connection.executescript(SCHEMA)
            columns = {
                row["name"]
                for row in connection.execute("PRAGMA table_info(conversations)")
            }
            if "avatar_data" not in columns:
                connection.execute("ALTER TABLE conversations ADD COLUMN avatar_data TEXT")

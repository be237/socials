from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any

from .db import Database
from .models import conversation_from_row, message_from_row


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


class Repository:
    def __init__(self, database: Database) -> None:
        self.database = database

    def ensure_default_user(self) -> str:
        with self.database.connection() as connection:
            row = connection.execute(
                "SELECT id FROM users WHERE email = ?",
                ("user@socials.local",),
            ).fetchone()
            if row:
                return row["id"]
            user_id = str(uuid.uuid4())
            timestamp = now()
            connection.execute(
                """INSERT INTO users
                   (id, username, email, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?)""",
                (user_id, "user", "user@socials.local", timestamp, timestamp),
            )
            return user_id

    def list_conversations(self) -> list[dict[str, Any]]:
        with self.database.connection() as connection:
            rows = connection.execute(
                """SELECT * FROM conversations
                   ORDER BY COALESCE(last_message_at, created_at) DESC"""
            ).fetchall()
            return [conversation_from_row(row) for row in rows]

    def get_telegram_account(self) -> dict[str, Any] | None:
        with self.database.connection() as connection:
            row = connection.execute(
                "SELECT phone, username, connected, updated_at FROM telegram_accounts WHERE id = 1"
            ).fetchone()
            if row is None:
                return None
            return {
                "phone": row["phone"],
                "username": row["username"],
                "connected": bool(row["connected"]),
                "updated_at": row["updated_at"],
            }

    def save_telegram_account(self, phone: str, username: str) -> None:
        with self.database.connection() as connection:
            connection.execute(
                """INSERT INTO telegram_accounts (id, phone, username, connected, updated_at)
                   VALUES (1, ?, ?, 1, ?)
                   ON CONFLICT(id) DO UPDATE SET phone = excluded.phone,
                   username = excluded.username, connected = 1, updated_at = excluded.updated_at""",
                (phone, username, now()),
            )

    def disconnect_telegram_account(self) -> None:
        with self.database.connection() as connection:
            connection.execute(
                """UPDATE telegram_accounts SET connected = 0, updated_at = ?
                   WHERE id = 1""",
                (now(),),
            )

    def seed_demo_data(self) -> None:
        with self.database.connection() as connection:
            if connection.execute("SELECT 1 FROM conversations LIMIT 1").fetchone():
                return

        user_id = self.ensure_default_user()
        demo_data = (
            ("Telegram - Alice", (("Alice", "Hey, tu es la ?"), ("Vous", "Oui ! Quoi de neuf ?"))),
            ("Discord - Dev Team", (("Marc", "Le build est passe !"), ("Sophie", "Super, je merge"))),
            ("Gmail - Newsletter", (("Newsletter", "Votre resume tech de la semaine"),)),
        )
        for title, messages in demo_data:
            conversation = self.create_conversation(title, "private", user_id)
            for sender, content in messages:
                self.create_message(conversation["id"], content, sender)

    def get_conversation(self, conversation_id: str) -> dict[str, Any] | None:
        with self.database.connection() as connection:
            row = connection.execute(
                "SELECT * FROM conversations WHERE id = ?", (conversation_id,)
            ).fetchone()
            return conversation_from_row(row) if row else None

    def find_telegram_conversation(self, chat_id: int) -> dict[str, Any] | None:
        with self.database.connection() as connection:
            row = connection.execute(
                "SELECT * FROM conversations WHERE title LIKE ? LIMIT 1",
                (f"TG:{chat_id}:%",),
            ).fetchone()
            return conversation_from_row(row) if row else None

    def update_conversation_avatar(self, conversation_id: str, avatar_data: str) -> None:
        with self.database.connection() as connection:
            connection.execute(
                "UPDATE conversations SET avatar_data = ?, updated_at = ? WHERE id = ?",
                (avatar_data, now(), conversation_id),
            )

    def find_message_by_platform_id(self, platform_message_id: str) -> bool:
        with self.database.connection() as connection:
            return connection.execute(
                "SELECT 1 FROM messages WHERE platform_message_id = ? LIMIT 1",
                (platform_message_id,),
            ).fetchone() is not None

    def create_conversation(
        self, title: str, conversation_type: str, user_id: str
    ) -> dict[str, Any]:
        conversation_id = str(uuid.uuid4())
        timestamp = now()
        with self.database.connection() as connection:
            connection.execute(
                """INSERT INTO conversations
                   (id, user_id, conversation_type, title, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                (
                    conversation_id,
                    user_id,
                    conversation_type,
                    title,
                    timestamp,
                    timestamp,
                ),
            )
        return self.get_conversation(conversation_id)  # type: ignore[return-value]

    def list_messages(self, conversation_id: str) -> list[dict[str, Any]]:
        with self.database.connection() as connection:
            rows = connection.execute(
                """SELECT * FROM messages
                   WHERE conversation_id = ? ORDER BY created_at ASC""",
                (conversation_id,),
            ).fetchall()
            return [message_from_row(row) for row in rows]

    def create_message(
        self,
        conversation_id: str,
        content: str,
        connector_id: str,
        sender_id: str | None = None,
        platform_message_id: str | None = None,
        status: str = "sent",
    ) -> dict[str, Any]:
        message_id = str(uuid.uuid4())
        timestamp = now()
        with self.database.connection() as connection:
            exists = connection.execute(
                "SELECT 1 FROM conversations WHERE id = ?", (conversation_id,)
            ).fetchone()
            if not exists:
                raise ValueError("Conversation introuvable")
            connection.execute(
                """INSERT INTO messages
                   (id, conversation_id, sender_id, content, message_type, status,
                    created_at, connector_id, platform_message_id)
                   VALUES (?, ?, ?, ?, 'text', ?, ?, ?, ?)""",
                (
                    message_id,
                    conversation_id,
                    sender_id or str(uuid.uuid4()),
                    content,
                    status,
                    timestamp,
                    connector_id,
                    platform_message_id or message_id,
                ),
            )
            connection.execute(
                """UPDATE conversations
                   SET updated_at = ?, last_message_at = ? WHERE id = ?""",
                (timestamp, timestamp, conversation_id),
            )
            row = connection.execute(
                "SELECT * FROM messages WHERE id = ?", (message_id,)
            ).fetchone()
            return message_from_row(row)

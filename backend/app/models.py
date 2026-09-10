from __future__ import annotations

from datetime import datetime
from typing import Any

from pydantic import BaseModel, Field


class ConversationCreate(BaseModel):
    title: str = Field(min_length=1, max_length=200)
    conversation_type: str = "private"


class MessageCreate(BaseModel):
    conversation_id: str
    content: str = Field(min_length=1, max_length=10000)
    sender_name: str = "Vous"


class ChannelConnect(BaseModel):
    token: str = Field(min_length=10, max_length=512)


class TelegramPhoneRequest(BaseModel):
    phone: str = Field(min_length=8, max_length=32)


class TelegramCodeRequest(BaseModel):
    code: str = Field(min_length=3, max_length=16)
    password: str | None = Field(default=None, max_length=256)


def conversation_from_row(row: Any) -> dict[str, Any]:
    return {
        "id": row["id"],
        "user_id": row["user_id"],
        "conversation_type": row["conversation_type"],
        "title": row["title"],
        "avatar_url": row["avatar_data"],
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
        "last_message_at": row["last_message_at"],
    }


def message_from_row(row: Any) -> dict[str, Any]:
    return {
        "id": row["id"],
        "conversation_id": row["conversation_id"],
        "sender_id": row["sender_id"],
        "content": row["content"],
        "message_type": row["message_type"],
        "status": row["status"],
        "created_at": row["created_at"],
        "edited_at": row["edited_at"],
        "reply_to": row["reply_to"],
        "connector_id": row["connector_id"],
        "platform_message_id": row["platform_message_id"],
    }

from __future__ import annotations

import asyncio
import logging
import os
from typing import Any

import httpx

from .events import EventBus
from .repositories import Repository

logger = logging.getLogger(__name__)


class TelegramPoller:
    def __init__(self, repository: Repository, events: EventBus, token: str) -> None:
        self.repository = repository
        self.events = events
        self.token = token
        self.offset = 0
        self.client = httpx.AsyncClient(
            base_url=f"https://api.telegram.org/bot{token}", timeout=40
        )

    async def close(self) -> None:
        await self.client.aclose()

    async def validate(self) -> None:
        response = await self.client.get("/getMe")
        response.raise_for_status()
        payload = response.json()
        if not payload.get("ok"):
            raise RuntimeError("Telegram a refusé le token du bot")
        logger.info("Telegram connecté: @%s", payload["result"].get("username", ""))

    async def run(self) -> None:
        await self.validate()
        while True:
            try:
                response = await self.client.get(
                    "/getUpdates",
                    params={"offset": self.offset, "timeout": 30},
                )
                response.raise_for_status()
                payload = response.json()
                if not payload.get("ok"):
                    raise RuntimeError("Telegram getUpdates a échoué")
                for update in payload["result"]:
                    self.offset = update["update_id"] + 1
                    await self.handle_update(update)
            except asyncio.CancelledError:
                raise
            except Exception:
                logger.exception("Erreur de polling Telegram")
                await asyncio.sleep(5)

    async def handle_update(self, update: dict[str, Any]) -> None:
        message = update.get("message")
        if not message or not message.get("text"):
            return
        chat = message["chat"]
        sender = message.get("from", {})
        sender_label = (
            f"@{sender['username']}"
            if sender.get("username")
            else " ".join(
                value
                for value in (sender.get("first_name", ""), sender.get("last_name", ""))
                if value
            )
        )
        chat_id = int(chat["id"])
        conversation = self.repository.find_telegram_conversation(chat_id)
        if conversation is None:
            conversation = self.repository.create_conversation(
                f"TG:{chat_id}:{sender_label or chat.get('type', 'Telegram')}",
                "private",
                self.repository.ensure_default_user(),
            )
            await self.events.publish(
                {"event": "ConversationCreated", "conversation_id": conversation["id"]}
            )

        saved = self.repository.create_message(
            conversation["id"],
            message["text"],
            f"Telegram:{sender_label or chat_id}",
            sender_id=f"telegram:{sender.get('id', 'unknown')}",
            platform_message_id=str(message["message_id"]),
            status="delivered",
        )
        await self.events.publish(
            {
                "event": "MessageReceived",
                "message": saved,
                "conversation_id": conversation["id"],
            }
        )

    async def send_message(self, chat_id: int, content: str) -> None:
        response = await self.client.post(
            "/sendMessage", json={"chat_id": chat_id, "text": content}
        )
        response.raise_for_status()
        if not response.json().get("ok"):
            raise RuntimeError("Telegram a refusé le message")


def telegram_token() -> str | None:
    token = os.getenv("TELEGRAM_BOT_TOKEN", "").strip()
    return token or None

from __future__ import annotations

import base64
import os
from pathlib import Path

from telethon import TelegramClient
from telethon.errors import SessionPasswordNeededError

from .events import EventBus
from .repositories import Repository


class TelegramUserSession:
    def __init__(self) -> None:
        api_id = os.getenv("TELEGRAM_API_ID", "").strip()
        api_hash = os.getenv("TELEGRAM_API_HASH", "").strip()
        if not api_id or not api_hash:
            raise RuntimeError(
                "Telegram nécessite TELEGRAM_API_ID et TELEGRAM_API_HASH côté serveur"
            )
        session_dir = Path(
            os.getenv(
                "SOCIALS_SESSION_DIR",
                str(Path.home() / ".local" / "share" / "socials"),
            )
        ).expanduser()
        session_dir.mkdir(parents=True, exist_ok=True)
        self.client = TelegramClient(
            str(session_dir / "telegram-user"), int(api_id), api_hash
        )
        self.phone_code_hash: str | None = None

    async def start_phone_login(self, phone: str) -> None:
        await self.client.connect()
        result = await self.client.send_code_request(phone)
        self.phone_code_hash = result.phone_code_hash

    async def finish_login(self, phone: str, code: str, password: str | None) -> str:
        if not self.phone_code_hash:
            raise RuntimeError("Aucun code Telegram en attente")
        try:
            await self.client.sign_in(
                phone=phone,
                code=code,
                phone_code_hash=self.phone_code_hash,
            )
        except SessionPasswordNeededError:
            if not password:
                raise
            await self.client.sign_in(password=password)
        if not await self.client.is_user_authorized():
            raise RuntimeError("Connexion Telegram non autorisée")
        self.phone_code_hash = None
        user = await self.client.get_me()
        return getattr(user, "username", None) or getattr(user, "first_name", "Telegram")

    async def is_authorized(self) -> bool:
        await self.client.connect()
        return await self.client.is_user_authorized()

    async def logout(self) -> None:
        await self.client.log_out()
        self.phone_code_hash = None

    async def sync_dialogs(self, repository: Repository, events: EventBus) -> int:
        if not await self.client.is_user_authorized():
            raise RuntimeError("La session Telegram n'est pas autorisée")

        synced = 0
        user_id = repository.ensure_default_user()
        async for dialog in self.client.iter_dialogs(limit=100):
            if not dialog.is_user and not dialog.is_group and not dialog.is_channel:
                continue
            entity = dialog.entity
            chat_id = int(entity.id)
            display_name = dialog.name or "Telegram"
            conversation = repository.find_telegram_conversation(chat_id)
            if conversation is None:
                conversation = repository.create_conversation(
                    f"TG:{chat_id}:{display_name}", "private", user_id
                )
                await events.publish({
                    "event": "ConversationCreated",
                    "conversation": conversation,
                })
            if not conversation["avatar_url"]:
                avatar = await self.client.download_profile_photo(entity, file=bytes)
                if avatar:
                    encoded = base64.b64encode(avatar).decode("ascii")
                    mime = "image/jpeg"
                    repository.update_conversation_avatar(
                        conversation["id"], f"data:{mime};base64,{encoded}"
                    )

            async for message in self.client.iter_messages(entity, limit=50, reverse=True):
                if not message.message:
                    continue
                platform_id = f"telegram-user:{chat_id}:{message.id}"
                if repository.find_message_by_platform_id(platform_id):
                    continue
                saved = repository.create_message(
                    conversation["id"],
                    message.message,
                    display_name,
                    sender_id=f"telegram-user:{getattr(message.sender, 'id', 'unknown')}",
                    platform_message_id=platform_id,
                    status="delivered" if not message.out else "sent",
                )
                synced += 1
                await events.publish({
                    "event": "MessageReceived",
                    "message": saved,
                    "conversation_id": conversation["id"],
                })
        return synced

    async def close(self) -> None:
        await self.client.disconnect()

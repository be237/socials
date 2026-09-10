from __future__ import annotations

import asyncio
import logging
import os
from contextlib import asynccontextmanager
from typing import Any

from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from .db import Database
from .events import EventBus
from .models import (
    ChannelConnect,
    ConversationCreate,
    MessageCreate,
    TelegramCodeRequest,
    TelegramPhoneRequest,
)
from .repositories import Repository
from .telegram import TelegramPoller, telegram_token
from .telegram_user import TelegramUserSession

load_dotenv()
logging.basicConfig(level=logging.INFO)

database = Database()
repository = Repository(database)
events = EventBus()
telegram: TelegramPoller | None = None
telegram_task: asyncio.Task[None] | None = None
telegram_user: TelegramUserSession | None = None
telegram_phone: str | None = None
telegram_sync_task: asyncio.Task[None] | None = None
telegram_sync_error: str | None = None
telegram_last_sync_at: str | None = None


async def sync_telegram_forever() -> None:
    global telegram_sync_error, telegram_last_sync_at
    while True:
        try:
            if telegram_user is None:
                return
            telegram_sync_error = None
            await telegram_user.sync_dialogs(repository, events)
            from datetime import datetime, timezone

            telegram_last_sync_at = datetime.now(timezone.utc).isoformat()
        except asyncio.CancelledError:
            raise
        except Exception as error:
            telegram_sync_error = str(error)
            logging.getLogger(__name__).exception(
                "Synchronisation Telegram échouée"
            )
        await asyncio.sleep(5)


@asynccontextmanager
async def lifespan(_: FastAPI):
    global telegram, telegram_task, telegram_user, telegram_sync_task
    database.initialize()
    repository.ensure_default_user()
    repository.seed_demo_data()
    account = repository.get_telegram_account()
    if account and account["connected"]:
        try:
            candidate = TelegramUserSession()
            if await candidate.is_authorized():
                telegram_user = candidate
                telegram_sync_task = asyncio.create_task(sync_telegram_forever())
            else:
                repository.disconnect_telegram_account()
                await candidate.close()
        except Exception:
            logging.getLogger(__name__).exception(
                "Impossible de restaurer la session Telegram"
            )
    token = telegram_token()
    if token:
        telegram = TelegramPoller(repository, events, token)
        telegram_task = asyncio.create_task(telegram.run())
    else:
        logging.getLogger(__name__).warning(
            "TELEGRAM_BOT_TOKEN absent: intégration Telegram désactivée"
        )
    yield
    if telegram_task:
        telegram_task.cancel()
        await asyncio.gather(telegram_task, return_exceptions=True)
    if telegram_sync_task:
        telegram_sync_task.cancel()
        await asyncio.gather(telegram_sync_task, return_exceptions=True)
    if telegram:
        await telegram.close()
    if telegram_user:
        await telegram_user.close()


app = FastAPI(title="Socials API", version="0.2.0", lifespan=lifespan)
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "tauri://localhost",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/api/health")
async def health() -> dict[str, Any]:
    return {"success": True, "data": "ok", "error": None}


@app.get("/api/channels")
async def get_channels() -> dict[str, Any]:
    connected = False
    if telegram_user is not None:
        try:
            connected = await telegram_user.is_authorized()
        except Exception:
            connected = False
    return {
        "success": True,
        "data": [{
            "id": "telegram",
            "name": "Telegram",
            "connected": connected or telegram is not None,
            "syncing": telegram_sync_task is not None and not telegram_sync_task.done(),
            "sync_error": telegram_sync_error,
            "last_sync_at": telegram_last_sync_at,
            "account": repository.get_telegram_account(),
        }],
        "error": None,
    }


@app.post("/api/channels/telegram")
async def connect_telegram(request: ChannelConnect) -> dict[str, Any]:
    global telegram, telegram_task
    candidate = TelegramPoller(repository, events, request.token.strip())
    try:
        await candidate.validate()
    except Exception as error:
        await candidate.close()
        raise HTTPException(status_code=400, detail=f"Connexion Telegram refusée: {error}") from error

    if telegram_task:
        telegram_task.cancel()
        await asyncio.gather(telegram_task, return_exceptions=True)
    if telegram:
        await telegram.close()
    telegram = candidate
    telegram_task = asyncio.create_task(telegram.run())
    return {
        "success": True,
        "data": {"id": "telegram", "name": "Telegram", "connected": True},
        "error": None,
    }


@app.post("/api/channels/telegram/phone")
async def telegram_phone_login(request: TelegramPhoneRequest) -> dict[str, Any]:
    global telegram_user, telegram_phone
    if telegram_user is None:
        try:
            telegram_user = TelegramUserSession()
        except RuntimeError as error:
            raise HTTPException(status_code=503, detail=str(error)) from error
    try:
        await telegram_user.start_phone_login(request.phone.strip())
    except Exception as error:
        raise HTTPException(status_code=400, detail=f"Impossible d'envoyer le code: {error}") from error
    telegram_phone = request.phone.strip()
    return {
        "success": True,
        "data": {"step": "code", "message": "Code envoyé dans Telegram"},
        "error": None,
    }


@app.post("/api/channels/telegram/code")
async def telegram_code_login(request: TelegramCodeRequest) -> dict[str, Any]:
    global telegram_sync_task, telegram_sync_error, telegram_last_sync_at
    if telegram_user is None or telegram_phone is None:
        raise HTTPException(status_code=400, detail="Commence par saisir ton numéro")
    try:
        username = await telegram_user.finish_login(
            telegram_phone, request.code.strip(), request.password
        )
    except Exception as error:
        if error.__class__.__name__ == "SessionPasswordNeededError":
            return {
                "success": True,
                "data": {"step": "password", "message": "Mot de passe 2FA requis"},
                "error": None,
            }
        raise HTTPException(status_code=400, detail=f"Connexion refusée: {error}") from error
    if telegram_sync_task:
        telegram_sync_task.cancel()
        await asyncio.gather(telegram_sync_task, return_exceptions=True)

    telegram_sync_task = asyncio.create_task(sync_telegram_forever())
    repository.save_telegram_account(telegram_phone, username)
    return {
        "success": True,
        "data": {
            "step": "connected",
            "name": username,
            "message": "Telegram est connecté",
        },
        "error": None,
    }


@app.post("/api/channels/telegram/disconnect")
async def disconnect_telegram() -> dict[str, Any]:
    global telegram_user, telegram_phone, telegram_sync_task
    if telegram_sync_task:
        telegram_sync_task.cancel()
        await asyncio.gather(telegram_sync_task, return_exceptions=True)
        telegram_sync_task = None
    if telegram_user:
        try:
            await telegram_user.logout()
        finally:
            await telegram_user.close()
            telegram_user = None
    telegram_phone = None
    global telegram_sync_error, telegram_last_sync_at
    telegram_sync_error = None
    telegram_last_sync_at = None
    repository.disconnect_telegram_account()
    return {
        "success": True,
        "data": {"id": "telegram", "name": "Telegram", "connected": False},
        "error": None,
    }


@app.get("/api/conversations")
async def get_conversations() -> dict[str, Any]:
    return {"success": True, "data": repository.list_conversations(), "error": None}


@app.post("/api/conversations")
async def create_conversation(request: ConversationCreate) -> dict[str, Any]:
    conversation = repository.create_conversation(
        request.title,
        request.conversation_type,
        repository.ensure_default_user(),
    )
    await events.publish(
        {"event": "ConversationCreated", "conversation": conversation}
    )
    return {"success": True, "data": conversation, "error": None}


@app.get("/api/conversations/{conversation_id}")
async def get_conversation(conversation_id: str) -> dict[str, Any]:
    conversation = repository.get_conversation(conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation introuvable")
    return {"success": True, "data": conversation, "error": None}


@app.get("/api/conversations/{conversation_id}/messages")
async def get_messages(conversation_id: str) -> dict[str, Any]:
    if repository.get_conversation(conversation_id) is None:
        raise HTTPException(status_code=404, detail="Conversation introuvable")
    return {
        "success": True,
        "data": repository.list_messages(conversation_id),
        "error": None,
    }


@app.post("/api/messages")
async def send_message(request: MessageCreate) -> dict[str, Any]:
    conversation = repository.get_conversation(request.conversation_id)
    if conversation is None:
        raise HTTPException(status_code=404, detail="Conversation introuvable")
    saved = repository.create_message(
        request.conversation_id, request.content, request.sender_name
    )
    if telegram and conversation["title"].startswith("TG:"):
        chat_id = int(conversation["title"].split(":", 2)[1])
        await telegram.send_message(chat_id, request.content)
    await events.publish(
        {
            "event": "MessageSent",
            "message": saved,
            "conversation_id": request.conversation_id,
        }
    )
    return {"success": True, "data": saved, "error": None}


@app.websocket("/api/ws")
async def websocket(websocket: WebSocket) -> None:
    await websocket.accept()
    queue = events.subscribe()
    try:
        while True:
            await websocket.send_json(await queue.get())
    except WebSocketDisconnect:
        pass
    finally:
        events.unsubscribe(queue)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "app.main:app",
        host=os.getenv("SOCIALS_HOST", "127.0.0.1"),
        port=int(os.getenv("SOCIALS_PORT", "3000")),
        reload=False,
    )

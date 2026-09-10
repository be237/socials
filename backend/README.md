# Socials backend Python

Backend local-first de Socials avec FastAPI, SQLite, WebSocket et polling Telegram.

```bash
cd backend
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env
python -m app.main
```

`TELEGRAM_BOT_TOKEN` est optionnel. Sans cette variable, l'API et l'interface
fonctionnent mais le mode bot Telegram reste désactivé.

Pour la connexion utilisateur sans bot, configure une fois côté installation
`TELEGRAM_API_ID` et `TELEGRAM_API_HASH`. L'utilisateur final saisit uniquement
son numéro Telegram puis le code reçu dans l'application Telegram.

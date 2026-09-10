# Socials

Une plateforme de messagerie unifiee qui centralise les conversations de plusieurs services,
avec un backend Python local-first et une interface React/Tauri.

## Architecture

```
Socials
├── backend/app           — FastAPI, SQLite, bus d'evenements, Telegram
├── frontend/src          — Interface React + TypeScript
└── frontend/src-tauri    — Fenetre desktop Tauri
```

## Stack technique

| Composant | Technologie | Cout |
|-----------|-------------|------|
| Backend | Python + FastAPI | 0 EUR |
| Frontend | React + TypeScript | 0 EUR |
| Desktop | Tauri | 0 EUR |
| DB locale | SQLite | 0 EUR |
| Temps reel | WebSocket | 0 EUR |

## Demarrer

```bash
# Installer les dependances backend
cd backend
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Lancer l'API sur http://localhost:3000
python -m app.main

# Dans un autre terminal, lancer l'interface sur http://localhost:5173
cd frontend
npm install
npm run dev
```

## API Endpoints

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | /api/health | Verification sante |
| POST | /api/users | Creer un utilisateur |
| GET | /api/users/:id | Obtenir un utilisateur |
| GET | /api/conversations | Lister les conversations |
| GET | /api/conversations/:id | Obtenir une conversation |
| GET | /api/conversations/:id/messages | Obtenir les messages |
| POST | /api/messages | Envoyer un message |

## Structure du projet

```
Socials/
├── backend/
│   ├── app/main.py              (API FastAPI et WebSocket)
│   ├── app/repositories.py      (persistance SQLite)
│   └── app/telegram.py          (polling Telegram)
├── frontend/
│   ├── src/App.tsx              (interface React)
│   └── src-tauri/               (enveloppe desktop Tauri)
```

## Politique de gratuité

- Aucun service payant obligatoire
- Priorite aux outils open source / gratuits
- Pas de cloud obligatoire
- Pas de SaaS obligatoire
- Tout fonctionne en local

## License

MIT

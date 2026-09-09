# Socials

Une plateforme de messagerie unifiee qui centralise les conversations de plusieurs services.

## Architecture

```
Socials
├── Core (Rust)           — Moteur central, entites, evenements
├── Persistence (SQLite)  — Stockage local des donnees
├── Connecteurs           — Adaptateurs pour chaque plateforme
│   └── Telegram          — Connecteur Telegram Bot API
├── Server (Axum)         — API REST interne
└── UI (a venir)          — Interface Tauri + React
```

## Stack technique

| Composant | Technologie | Cout |
|-----------|-------------|------|
| Langage | Rust | 0 EUR |
| Build | Cargo | 0 EUR |
| DB locale | SQLite | 0 EUR |
| ORM | SQLx | 0 EUR |
| Async | Tokio | 0 EUR |
| API | Axum | 0 EUR |
| Desktop | Tauri (a venir) | 0 EUR |
| UI | React + TypeScript (a venir) | 0 EUR |

## Demarrer

```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Lancer le serveur
cargo run -p socials-server

# Tests
cargo test
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
├── Cargo.toml                    (workspace)
├── crates/
│   ├── core/                     (moteur central)
│   │   └── src/
│   │       ├── entities/         (User, Account, Contact, etc.)
│   │       ├── events/           (bus d'evenements)
│   │       ├── connectors/       (trait Connector)
│   │       ├── services/         (CoreService)
│   │       └── repositories/     (traits de persistence)
│   ├── persistence/              (implementation SQLite)
│   │   └── src/
│   │       ├── sqlite.rs         (connexion DB)
│   │       └── repository/       (implementations des repositories)
│   ├── server/                   (API Axum)
│   │   └── src/
│   │       ├── api.rs            (endpoints REST)
│   │       ├── lib.rs
│   │       └── main.rs           (point d'entree)
│   ├── shared/                   (types communs)
│   └── app/                      (application desktop)
├── connectors/
│   └── telegram/                 (connecteur Telegram)
├── migrations/                   (schemas SQL)
└── docs/
    ├── evolution_projet.md       (historique des decisions)
    └── specification_core_v0.1.md (specification du Core)
```

## Politique de gratuité

- Aucun service payant obligatoire
- Priorite aux outils open source / gratuits
- Pas de cloud obligatoire
- Pas de SaaS obligatoire
- Tout fonctionne en local

## Documentation

- `docs/evolution_projet.md` — Historique complet des decisions
- `docs/specification_core_v0.1.md` — Specification detaillee du Core

## License

MIT

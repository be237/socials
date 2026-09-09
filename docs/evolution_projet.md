# EVOLUTION DU PROJET SOCIALS

## Contexte permanent — Toutes les decisions prises

Ce fichier retrace l'integralite des decisions architecturales et techniques du projet.
Il sert de reference pour tout developpeur travaillant sur Socials.

---

## 1. Origine du projet (9 septembre 2026)

Le projet est né d'une conversation autour de plusieurs idees :
- Supprimer des photos du net → impossible universellement
- Publication automatique sur reseaux sociaux
- **Centralisation des messages** → idee retenue

L'idee centrale : **une seule application qui rassemble les conversations de plusieurs services et permet d'y repondre depuis une interface unique.**

---

## 2. Positionnement du produit

**« Une seule boîte de réception pour toutes vos conversations. »**

Le produit n'est PAS :
- Un agregateur d'applications encapsulees
- Un outil pour developpeurs (SDK, plugins tiers)

Le produit EST :
- Un **moteur de conversations unifiees**
- Une application simple pour l'utilisateur final
- Modulaire en interne, transparente pour l'utilisateur

---

## 3. Architecture fondamentale

### Principe
**« Modulaire pour nous, simple pour l'utilisateur. »**

L'utilisateur installe une application. Il active/desactive des services depuis Parametres → Comptes connectes.

### Architecture en couches

```
                 APPLICATION
                      |
              +-------v-------+
              |      UI       |
              | Tauri + React |
              +-------+-------+
                      |
                 API interne
                      |
              +-------v-------+
              |     CORE      |
              |    Rust       |
              +-------+-------+
                      |
        +-------------+-------------+
        |             |             |
   Connecteurs     Modules       Services
        |             |             |
   Telegram        Search        Database
   Discord         AI            Sync
   Gmail           etc.          Storage
   WhatsApp
   Instagram
```

### Ce qu'on garde
- Core independant des plateformes
- Connecteurs modulaires
- Modules internes (Search, AI, etc.)
- Systeme d'evenements
- Capacites par connecteur
- Architecture multi-comptes
- Modele universel des conversations

### Ce qu'on ne fait PAS
- SDK public
- Marketplace de plugins
- Installation de plugins par l'utilisateur
- Architecture pour developpeurs externes
- Systeme complexe de compatibilite entre SDK

---

## 4. Politique de gratuité (contrainte architecturale)

### Principes
- **Aucun service payant obligatoire**
- Priorite aux outils open source / gratuits
- Pas de cloud obligatoire
- Pas de SaaS obligatoire
- Pas d'API IA payante dans le Core
- Connecteurs : API officielles accessibles gratuitement
- Toute dependance externe evaluee selon : cout, licence, limites, possibilite de remplacement
- Le Core ne depend jamais directement d'un fournisseur commercial
- Fonctionnalites payantes = optionnelles uniquement

### Stack validee (tout gratuit)

| Element | Choix | Cout |
|---------|-------|------|
| Langage | Rust | 0 EUR |
| Build | Cargo | 0 EUR |
| Desktop | Tauri | 0 EUR |
| UI | React + TypeScript | 0 EUR |
| CSS | Tailwind CSS | 0 EUR |
| API interne | Axum | 0 EUR |
| DB locale | SQLite | 0 EUR |
| ORM | SQLx | 0 EUR |
| Async | Tokio | 0 EUR |
| Serialisation | Serde | 0 EUR |
| IDs | UUID | 0 EUR |

### Architecture economique V1

```
PC
|
+-- Socials
+-- Core
+-- SQLite
+-- Connecteurs
+-- Interface
```

Tout fonctionne sur la machine de l'utilisateur. Pas de serveur cloud obligatoire.

Le cloud pourra eventuellement venir plus tard pour la synchronisation multi-appareils,
mais ce sera une fonction supplementaire, pas une dependance fondamentale.

### APIs externes — a verifier plateforme par plateforme

| Plateforme | API | Cout | Statut |
|-----------|-----|------|--------|
| Telegram | Bot API | Gratuit | OK |
| Discord | Discord API | Gratuit | OK |
| Gmail | Gmail API | Gratuit (quota) | OK |
| WhatsApp | Business API | Payant | A evaluer |
| Instagram | Graph API | Restreint | A evaluer |

---

## 5. Cibles et phases de developpement

### Public cible
- Personnes qui utilisent plusieurs services de communication
- Veulent eviter de jongler entre plusieurs applications
- Puis : professionnels, equipes, entreprises

### Plateformes par phase

**PHASE 1** — Prouver le moteur universel :
- Telegram
- Discord
- Email

**PHASE 2** — Grandes messageries (selon APIs gratuites) :
- WhatsApp (si API gratuite disponible)
- Messenger
- Instagram (si API accessible)

**PHASE 3** — Professionnels :
- Slack
- autres services professionnels

**PHASE 4** — Extensions futures :
- AI
- traduction
- automatisation
- CRM
- statistiques
- workflows

---

## 6. Modele de donnees conceptuel

### Entites fondamentales

| Entite | Description |
|--------|-------------|
| User | Utilisateur de Socials |
| Account | Compte connecte (ex: Telegram #1) |
| Contact | Identite unifiee d'une personne |
| Identity | Lien entre Contact et plateforme |
| Conversation | Conversation universelle (privee, groupe, channel, email) |
| Message | Message unifie |
| Attachment | Piece jointe |
| Connector | Adaptateur entre plateforme et Core |
| ConnectorCapabilities | Capacites declarees par un connecteur |
| Event | Evenement du bus d'evenements |
| Repository | Abstraction de persistence |
| Permissions | Droits des modules |
| Synchronization | Gestion sync offline-first |

### Modele universel du message

Le Core manipule des `UniversalMessage`, pas des `WhatsAppMessage` ou `TelegramMessage`.

Chaque message contient :
- id universel
- conversation_id
- sender_id
- content
- message_type
- created_at
- edited_at
- status
- reply_to
- attachments
- reactions
- **source** : connector_id / account_id / platform_message_id
- metadata specifique eventuelle

Le lien avec l'identifiant de la plateforme est conserve par le connecteur.

### Modele universel de conversation

Peut representer :
- conversation privee
- groupe
- channel
- thread
- email
- ticket / support
- communaute
- bot

---

## 7. Systeme de connecteurs

### Principe
Chaque plateforme est un **connecteur** independant.

### Flux

**Entrant :**
```
Plateforme → Connector → Message universel → Core
```

**Sortant :**
```
Core → Message universel → Connector → Plateforme
```

### Capacites par connecteur

Chaque connecteur declare ce qu'il peut faire :
- receive_messages
- send_messages
- images / files / voice / video
- reactions
- edit_message / delete_message
- typing_indicator
- read_receipts
- etc.

L'interface s'adapte automatiquement aux capacites disponibles.

### Identite unifiee

Une meme personne peut exister sur plusieurs plateformes :
```
Jean
+-- WhatsApp : +237...
+-- Telegram : @jean123
+-- Instagram : @jeanxxx
```

Le logiciel peut proposer la fusion : « Ces deux comptes semblent appartenir a la meme personne. Fusionner ? »

---

## 8. Systeme d'evenements

Le Core fonctionne autour d'un bus d'evenements :

```
MESSAGE_RECEIVED → CORE → UI + Notification + Automation + ...
```

Evenements :
- MESSAGE_RECEIVED / SENT / EDITED / DELETED
- CONVERSATION_CREATED / UPDATED
- CONTACT_CREATED / MERGED
- ACCOUNT_CONNECTED / DISCONNECTED
- SYNC_STARTED / COMPLETED / FAILED

Permet d'ajouter plus tard : regles automatiques, notifications intelligentes, IA, workflows.

---

## 9. Choix du public initial

**Commencer par Desktop / PC.**

Raisons :
- Beaucoup de conversations
- Plusieurs comptes
- Recherche globale
- Gestion des pieces jointes
- Raccourcis clavier
- Glisser-deposer
- Multi-fenêtres
- Notifications

Mais l'architecture ne doit pas etre « PC uniquement » :

```
              UNIVERSAL CORE
                    |
          +---------+---------+
          |                   |
    Desktop Client       Mobile Client
          |                   |
   Linux / Windows       Android / iOS
```

Meme Core, clients differents.

---

## 10. Specification Core v0.1

**Objectif** : Produire une specification detaillee des 16 elements du Core avant toute ligne de code.

**Pourquoi** : Eviter une reecriture du cœur dans quelques mois parce que la premiere
implementation serait trop fortement liee a Telegram, l'interface desktop ou une bibliotheque particuliere.

**16 elements a specifier :**
1. User
2. Account
3. Contact
4. Identity
5. Conversation
6. Message
7. Attachment
8. Connector (trait)
9. ConnectorCapabilities
10. Event
11. Repository
12. Synchronization
13. Permissions
14. API interne
15. Versioning
16. Extension

**Regle** : « Le projet doit être un framework de communication auquel on peut ajouter
ou retirer des capacites, et non une application monolithique qui contient WhatsApp +
Telegram + Instagram en dur. »

---

## 11. Arborescence cible du projet

```
Socials/
+-- Cargo.toml
+-- README.md
+-- LICENSE
+-- crates/
|   +-- core/
|   |   +-- entities/
|   |   +-- events/
|   |   +-- capabilities/
|   |   +-- connectors/
|   |   +-- services/
|   |   +-- repositories/
|   |   +-- lib.rs
|   +-- shared/
|   +-- server/
|   |   +-- api/
|   |   +-- websocket/
|   |   +-- main.rs
|   +-- app/
|       +-- main.rs
+-- connectors/
|   +-- telegram/
|   +-- discord/
|   +-- gmail/
|   +-- ...
+-- migrations/
+-- docs/
    +-- evolution_projet.md      <- ce fichier
    +-- specification_core_v0.1.md
    +-- architecture/
```

**Regle** : Le Core ne depend jamais d'un connecteur concret.

---

## 12. Regles de developpement

### Regle fondamentale
**Penser long terme ne signifie pas tout developper maintenant.**

Les interfaces et contrats sont concus pour le long terme, mais l'implementation reste progressive.

### Ordre recommande
1. Architecture → Core → Universal Models
2. Connector API → Event System
3. Persistence → Sync abstraction
4. Telegram → API → Desktop UI
5. Discord / Gmail → autres connecteurs
6. Mobile / Web / Cloud / Extensions

### Regle de dependance
**Une dependance ne doit pas contaminer inutilement toute l'architecture.**

| Crate | Dependances |
|-------|-------------|
| core | serde, uuid, chrono, tokio |
| shared | serde, uuid, chrono |
| server | axum, tokio, serde |
| app | dependance vers core |
| telegram | teloxide |
| persistence | sqlx (uniquement dans la couche DB) |

### Dependance = interfacage
Chaque couche communique via des interfaces, pas via des dependances directes.

### Pas de code métier dans les connecteurs
Un connecteur ne contient que la logique d'adaptation entre une plateforme et le modèle universel.

### Tests
Chaque element doit pouvoir etre teste independemment.

---

## 13. Decisions en attente

- [ ] WhatsApp : API Business payante → faut-il l'inclure en Phase 2 ?
- [ ] Instagram : API restreinte → accessibilite reelle ?
- [ ] Desktop : Tauri confirme ou Electron a evaluer ?
- [ ] Mobile : Flutter ou React Native ?
- [ ] License du projet : MIT ? Apache 2.0 ? Autre ?

---

## 14. Prochaine etape concrete

**Etape 1 — Fondations :**

1. Installer Rust (rustup)
2. Creer le workspace Cargo
3. Initialiser les crates (core, shared, server, app, telegram)
4. Ecrire `docs/specification_core_v0.1.md`
5. Verifier que `cargo build` et `cargo test` passent

**Livrable :** Structure de projet + specification Core v0.1 + crates qui compilent.
Aucun comportement métier complexe.

**Etape 2 — Persistence SQLite :**

1. Creer le crate persistence avec SQLx + SQLite
2. Creer les migrations SQL (schema de la DB)
3. Implementer les repositories :
   - UserRepository
   - AccountRepository
   - ContactRepository
   - IdentityRepository
   - ConversationRepository
   - MessageRepository
   - AttachmentRepository
4. Verifier que `cargo build` et `cargo test` passent

**Livrable :** Persistence fonctionnelle avec SQLite.
Les repositories sont des implementations concretes des traits definis dans le Core.

**Statut : Termine.**

**Etape 3 — Bus d'evenements :**

1. Definir le systeme d'evenements dans le Core
2. Creer le bus d'evenements (EventBus)
3. Implementer le Publisher/Subscriber
4. Integrer le bus avec les repositories
5. Verifier que `cargo build` et `cargo test` passent

**Livrable :** Bus d'evenements fonctionnel avec :
- Event enum avec toutes les variantes
- EventEnvelope pour porter les metadonnees
- EventBus avec publish/subscribe/unsubscribe
- Tests unitaires

**Statut : Termine.**

**Etape 4 — Connecteur Telegram :**

1. Mettre a jour le connecteur Telegram avec teloxide
2. Implementer la reception de messages
3. Implementer l'envoi de messages
4. Ajouter la configuration du bot token
5. Verifier que `cargo build` et `cargo test` passent

**Livrable :** Connecteur Telegram fonctionnel avec :
- TelegramConnector avec nouvelle methode `new(token)`
- Implementation du trait Connector
- Conversion des messages teloxide en messages universels
- Tests unitaires

**Statut : Termine.**

**Etape 5 — API Axum :**

1. Integrer le bus d'evenements avec les repositories
2. Creer un service Core qui lie tout ensemble
3. Creer l'API Axum avec les endpoints REST
4. Creer un main.rs fonctionnel
5. Verifier que `cargo build` et `cargo test` passent

**Livrable :** API REST fonctionnelle avec :
- CoreService qui integre les repositories et le bus d'evenements
- Endpoints REST : users, conversations, messages
- Structure prete pour etre connectee a SQLite

**Statut : Termine.**

---

## 15. Etat actuel du projet

### Fichiers crees

```
Socials/
+-- Cargo.toml
+-- crates/
|   +-- core/
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- entities/
|   |       |   +-- user.rs, account.rs, contact.rs
|   |       |   +-- identity.rs, conversation.rs
|   |       |   +-- message.rs, attachment.rs
|   |       +-- events/mod.rs, bus.rs
|   |       +-- connectors/mod.rs
|   |       +-- services/mod.rs
|   |       +-- repositories/mod.rs
|   +-- shared/
|   +-- persistence/
|   |   +-- src/
|   |       +-- lib.rs, sqlite.rs
|   |       +-- repository/ (7 repositories)
|   +-- server/
|       +-- src/
|           +-- lib.rs, main.rs, api.rs
+-- connectors/
|   +-- telegram/
+-- migrations/
|   +-- 001_initial_schema.sql
+-- frontend/
|   +-- src-tauri/          (Tauri v2)
|   |   +-- Cargo.toml, tauri.conf.json
|   |   +-- src/lib.rs, main.rs
|   +-- src/
|   |   +-- App.tsx         (UI React)
|   |   +-- index.css, main.tsx
|   +-- package.json, vite.config.ts
+-- docs/
    +-- evolution_projet.md
    +-- specification_core_v0.1.md
```

### Verification

- `cargo build` : OK
- `cargo test` : OK (4 tests passing)

### Stack utilisee

| Element | Choix | Cout |
|---------|-------|------|
| Langage | Rust 1.98.1 | 0 EUR |
| Build | Cargo | 0 EUR |
| DB locale | SQLite via SQLx | 0 EUR |
| ORM | SQLx | 0 EUR |
| Async | Tokio | 0 EUR |
| Serialisation | Serde | 0 EUR |
| IDs | UUID | 0 EUR |
| Desktop | Tauri v2 | 0 EUR |
| UI | React + TypeScript + Tailwind | 0 EUR |
| Frontend build | Vite | 0 EUR |
| API | Axum | 0 EUR |

### Lancement

```bash
# Dev mode (frontend + backend separes)
cargo run -p socials-server    # API sur :3000
cd frontend && npm run dev     # UI sur :5173

# Production (Tauri desktop app)
cargo tauri dev                # Lance tout automatiquement
```

### Persistence SQLite

La base de donnees est stockee dans `~/.local/share/socials/socials.db` :
- Les conversations et messages sont sauvegardes
- Les donnees de demo sont inserees au premier lancement
- Les donnees survivent aux redemarrages

### API Endpoints

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | /api/health | Verification sante |
| GET | /api/conversations | Lister les conversations |
| POST | /api/conversations | Creer une conversation |
| GET | /api/conversations/:id | Obtenir une conversation |
| GET | /api/conversations/:id/messages | Obtenir les messages |
| POST | /api/messages | Envoyer un message |

---

*Ce fichier sera mis a jour a chaque decision importante du projet.*
*Derniere mise a jour : 9 septembre 2026*

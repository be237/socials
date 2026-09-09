# Specification Core v0.1 — Socials

## Objectif

Document de reference definissant les 16 elements fondamentaux du Core.
Ce document doit etre valide avant toute implementation Complexe.

**Regle** : Le Core ne depend jamais d'un connecteur concret.
Le projet est un framework de communication, pas une application monolithique.

---

## 1. User

**Utilisateur de Socials.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| username | String | Nom d'utilisateur |
| email | String | Email de connexion |
| created_at | DateTime | Date de creation |
| updated_at | DateTime | Derniere modification |

**Regles :**
- Un User possede zero ou plusieurs Accounts
- Un User possede zero ou plusieurs Contacts
- Les donnees sont isolees par User (pas de partage)

---

## 2. Account

**Compte connecte d'un utilisateur sur une plateforme.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| user_id | UUID | Reference vers User |
| connector_name | String | Nom du connecteur (ex: "telegram") |
| platform_account_id | String | ID du compte sur la plateforme |
| display_name | String | Nom affiche |
| is_connected | bool | Compte actif ou non |
| access_token | Option | Token d'acces (chiffre) |
| refresh_token | Option | Token de rafraichissement |
| created_at | DateTime | Date de creation |
| updated_at | DateTime | Derniere modification |

**Regles :**
- Un User peut avoir plusieurs Accounts du meme connecteur (ex: 2 Telegram)
- Le token doit etre stocke de facon securisee (chiffrement)
- Un Account est lie a un seul Connector

---

## 3. Contact

**Identite unifiee d'une personne.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| user_id | UUID | Reference vers User |
| display_name | String | Nom affiche |
| avatar_url | Option | URL de l'avatar |
| created_at | DateTime | Date de creation |
| updated_at | DateTime | Derniere modification |

**Regles :**
- Un Contact peut etre lie a plusieurs Identities
- Un Contact peut etre fusionne avec un autre (conservation des Identities)
- Le display_name est le nom unifie pour l'interface

---

## 4. Identity

**Lien entre un Contact et un compte sur une plateforme.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| contact_id | UUID | Reference vers Contact |
| account_id | UUID | Reference vers Account |
| platform_user_id | String | ID de l'utilisateur sur la plateforme |
| platform_username | Option | Username sur la plateforme |
| created_at | DateTime | Date de creation |

**Regles :**
- Une Identity est liee a un Contact et a un Account
- Plusieurs Identities peuvent etre liees au meme Contact
- La fusion de Contacts deplace les Identities vers le Contact garde

---

## 5. Conversation

**Conversation universelle.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| user_id | UUID | Reference vers User |
| conversation_type | Enum | Type de conversation |
| title | String | Titre / nom |
| created_at | DateTime | Date de creation |
| updated_at | DateTime | Derniere modification |
| last_message_at | Option | Date du dernier message |
| metadata | JSON | Donnees specifiques a la plateforme |

**Types de conversations :**
- Private — conversation 1:1
- Group — groupe de discussion
- Channel — canal de diffusion
- Thread — fil de discussion
- Email — conversation email
- Bot — conversation avec un bot

**Regles :**
- Une Conversation est liee a un User
- Une Conversation contient zero ou plusieurs Messages
- Les participants sont geres via les Messages (sender_id)

---

## 6. Message

**Message unifie.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant universel |
| conversation_id | UUID | Reference vers Conversation |
| sender_id | UUID | Reference vers Contact ou User |
| content | String | Contenu du message |
| message_type | Enum | Type de message |
| status | Enum | Statut du message |
| created_at | DateTime | Date de creation |
| edited_at | Option | Date de modification |
| reply_to | Option | Reference vers un autre Message |
| connector_id | String | ID du connecteur source |
| platform_message_id | String | ID du message sur la plateforme |
| metadata | JSON | Donnees specifiques |

**Types de messages :**
- Text — texte brut
- Image — image
- File — fichier
- Voice — message vocal
- Video — video
- Sticker — sticker
- Other — autre

**Statuts :**
- Pending — en attente d'envoi
- Sent — envoye
- Delivered — delivre
- Read — lu
- Failed — echec

**Regles :**
- Le platform_message_id est conserve par le connecteur
- Le connector_id identifie la plateforme source
- Le Core ne manipule que les id universels

---

## 7. Attachment

**Piece jointe d'un message.**

| Champ | Type | Description |
|-------|------|-------------|
| id | UUID | Identifiant unique |
| message_id | UUID | Reference vers Message |
| attachment_type | Enum | Type de piece jointe |
| filename | String | Nom du fichier |
| url | Option | URL de telechargement |
| size | Option | Taille en octets |
| mime_type | Option | Type MIME |
| created_at | DateTime | Date de creation |

**Types :**
- Image — image
- File — fichier
- Voice — vocal
- Video — video
- Sticker — sticker
- Other — autre

**Regles :**
- Un Message peut avoir zero ou plusieurs Attachments
- Le telechargement est gere par le connecteur
- Le stockage local est gere par le service Storage

---

## 8. Connector (trait)

**Interface entre une plateforme et le Core.**

```rust
#[async_trait]
pub trait Connector: Send + Sync {
    /// Nom unique du connecteur
    fn name(&self) -> &str;

    /// Capacites du connecteur
    fn capabilities(&self) -> ConnectorCapabilities;

    /// Envoyer un message
    async fn send_message(&self, message: &Message) -> Result<(), Error>;

    /// Recevoir les messages
    async fn receive_messages(&self) -> Result<Vec<Message>, Error>;

    /// Synchroniser les conversations
    async fn sync_conversations(&self) -> Result<Vec<Conversation>, Error>;

    /// Obtenir les contacts
    async fn get_contacts(&self) -> Result<Vec<Contact>, Error>;

    /// Telecharger une piece jointe
    async fn download_attachment(&self, url: &str) -> Result<Vec<u8>, Error>;
}
```

**Regles :**
- Un Connector ne connait que son interface
- Le Core ne connait pas les details de l'implementation
- Chaque connecteur est un crate independant
- Un connecteur peut etre active/desactive par l'utilisateur

---

## 9. ConnectorCapabilities

**Capacites declarees par un connecteur.**

| Capacite | Type | Description |
|----------|------|-------------|
| receive_messages | bool | Peut recevoir des messages |
| send_messages | bool | Peut envoyer des messages |
| images | bool | Supporte les images |
| files | bool | Supporte les fichiers |
| voice | bool | Supporte les messages vocaux |
| video | bool | Supporte les videos |
| reactions | bool | Supporte les reactions |
| edit_message | bool | Peut modifier un message |
| delete_message | bool | Peut supprimer un message |
| typing_indicator | bool | Peut afficher l'indicateur de frappe |
| read_receipts | bool | Peut envoyer/recevoir les accusés de lecture |
| sync | bool | Supporte la synchronisation |

**Regles :**
- L'interface s'adapte aux capacites disponibles
- Une capacite non supportee est masquee dans l'UI
- Les capacites sont declares a l'installation du connecteur

---

## 10. Event

**Evenement du bus d'evenements.**

```rust
pub enum Event {
    // Messages
    MessageReceived { message: Message },
    MessageSent { message: Message },
    MessageEdited { message: Message },
    MessageDeleted { message_id: UUID },

    // Conversations
    ConversationCreated { conversation: Conversation },
    ConversationUpdated { conversation: Conversation },

    // Contacts
    ContactCreated { contact: Contact },
    ContactMerged { source: UUID, target: UUID },

    // Accounts
    AccountConnected { account: Account },
    AccountDisconnected { account_id: UUID },

    // Sync
    SyncStarted { account_id: UUID },
    SyncCompleted { account_id: UUID },
    SyncFailed { account_id: UUID, error: String },
}
```

**Bus d'evenements :**
- Les producteurs emettent des evenements
- Les consommateurs s'abonnent aux evenements
- Le producteur ne connait pas ses consommateurs
- Les evenements permettent d'ajouter des modules sans modifier le Core

**Consommateurs potentiels :**
- UI (mise a jour de l'affichage)
- Notifications (notification system)
- Search (indexation)
- Automation (regles automatiques)
- AI (analyse)

---

## 11. Repository

**Abstraction de persistence.**

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UUID) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn create(&self, user: &User) -> Result<User>;
    async fn update(&self, user: &User) -> Result<User>;
    async fn delete(&self, id: UUID) -> Result<()>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn find_by_conversation(&self, conversation_id: UUID) -> Result<Vec<Message>>;
    async fn find_by_id(&self, id: UUID) -> Result<Option<Message>>;
    async fn create(&self, message: &Message) -> Result<Message>;
    async fn update(&self, message: &Message) -> Result<Message>;
    async fn delete(&self, id: UUID) -> Result<()>;
}
```

**Regles :**
- Le Repository est une abstraction, pas une implementation SQL
- SQLite est l'implementation par defaut
- On peut remplacer SQLite par PostgreSQL sans changer le Core
- Les Repositories sont definis par entite

---

## 12. Synchronization

**Gestion de la synchronisation offline-first.**

```
Remote (Plateforme)
      ^
      |
  Sync Engine
      |
  Local Database (SQLite)
```

**Modes :**
- Online — connexion active, sync en temps reel
- Offline — pas de connexion, donnees locales
- Syncing — synchronisation en cours
- Pending — messages en attente d'envoi

**Regles :**
- Les donnees locales sont toujours disponibles
- Les messages envoyes hors ligne passent en Pending
- La reconnexion declenche la synchronisation
- Les conflits sont resolus par last-write-wins (V1)
- La deduplication est geree via platform_message_id

---

## 13. Permissions

**Droits des modules et connecteurs.**

| Permission | Description |
|------------|-------------|
| READ_MESSAGES | Lire les messages |
| SEND_MESSAGES | Envoyer des messages |
| READ_CONTACTS | Lire les contacts |
| READ_MEDIA | Lire les media |
| DELETE_MESSAGES | Supprimer des messages |
| MANAGE_ACCOUNTS | Gerer les comptes |
| SYNC | Synchroniser les donnees |

**Regles :**
- Chaque module recoit uniquement les permissions necessaires
- Un module AI pourrait recevoir READ_MESSAGES mais pas SEND_MESSAGES
- Les permissions sont definies par le Core
- Les modules ne peuvent pas s'auto-attribuer de permissions

---

## 14. API Interne

**Contrat entre l'application et le Core.**

L'API interne est un ensemble de services exposes via Axum.

```rust
// Services exposes
pub struct CoreApi {
    pub users: UserService,
    pub accounts: AccountService,
    pub contacts: ContactService,
    pub conversations: ConversationService,
    pub messages: MessageService,
    pub connectors: ConnectorService,
}
```

**Endpoints potentiels :**
- GET /conversations — lister les conversations
- GET /conversations/:id — obtenir une conversation
- GET /conversations/:id/messages — obtenir les messages
- POST /conversations/:id/messages — envoyer un message
- GET /contacts — lister les contacts
- GET /accounts — lister les comptes connectes
- POST /accounts/:id/connect — connecter un compte
- POST /accounts/:id/disconnect — deconnecter un compte

**Regles :**
- L'API est interne, pas publique
- Elle communique via HTTP REST et WebSocket
- L'UI (Tauri) consomme cette API
- Pas de SDK public

---

## 15. Versioning

**Regles de versionnage.**

| Element | Format | Description |
|---------|--------|-------------|
| Core | semver | Version majeure.minore.patch |
| Connectors | semver independant | Chaque connecteur a sa propre version |
| API interne | version dans l'URL | /api/v1/... |

**Regles :**
- Un ancien connecteur ne doit pas casser le Core
- Le Core peut evoluer sans casser les connecteurs
- Les breaking changes incrementent la version majeure
- Les nouvelles fonctionnalites incrementent la version minore

---

## 16. Extension

**Comment ajouter un nouveau connecteur.**

**Etapes :**
1. Creer un nouveau crate dans `connectors/nom/`
2. Implementer le trait `Connector`
3. Definir les `ConnectorCapabilities`
4. Ajouter le crate au workspace
5. Tester les fonctionnalites de base

**Structure d'un connecteur :**
```
connectors/
└── nom/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

**Exemple de minimal :**
```rust
use socials_core::connectors::{Connector, ConnectorCapabilities};
use async_trait::async_trait;

pub struct NomConnector;

#[async_trait]
impl Connector for NomConnector {
    fn name(&self) -> &str { "nom" }
    fn capabilities(&self) -> ConnectorCapabilities { /* ... */ }
    async fn send_message(&self, _: &Message) -> Result<()> { /* ... */ }
    async fn receive_messages(&self) -> Result<Vec<Message>> { /* ... */ }
}
```

**Regles :**
- Un connecteur ne contient que la logique d'adaptation
- Pas de code metier dans les connecteurs
- Les connecteurs peuvent etre actives/desactives
- Le Core ne connait pas les connecteurs concrets

---

## Politique de gratuité

Tout le Core et les connecteurs doivent etre utilisables gratuitement.

| Composant | Cout |
|-----------|------|
| Core | 0 EUR |
| SQLite | 0 EUR |
| Telegram connector | API Bot gratuite |
| Discord connector | API gratuite |
| Gmail connector | API avec quota gratuit |
| WhatsApp connector | A evaluer (API payante ?) |
| Instagram connector | A evaluer (API restreinte ?) |

**Principe : Socials doit fonctionner sans aucun service payant.**

---

*Specification v0.1 — 9 septembre 2026*
*Document de reference pour l'implementation du Core.*

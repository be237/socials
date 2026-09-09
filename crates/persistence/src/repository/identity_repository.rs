use async_trait::async_trait;
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;
use socials_core::entities::identity::Identity;
use socials_core::repositories::{IdentityRepository, Error};

pub struct SqliteIdentityRepository {
    pool: SqlitePool,
}

impl SqliteIdentityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_identity(row: sqlx::sqlite::SqliteRow) -> Identity {
    Identity {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        contact_id: Uuid::parse_str(row.get::<&str, _>("contact_id")).unwrap(),
        account_id: Uuid::parse_str(row.get::<&str, _>("account_id")).unwrap(),
        platform_user_id: row.get("platform_user_id"),
        platform_username: row.get("platform_username"),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
    }
}

#[async_trait]
impl IdentityRepository for SqliteIdentityRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Identity>, Error> {
        let row = sqlx::query("SELECT * FROM identities WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_identity))
    }

    async fn find_by_contact_id(&self, contact_id: Uuid) -> Result<Vec<Identity>, Error> {
        let rows = sqlx::query("SELECT * FROM identities WHERE contact_id = ?")
            .bind(contact_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_identity).collect())
    }

    async fn find_by_account_id(&self, account_id: Uuid) -> Result<Vec<Identity>, Error> {
        let rows = sqlx::query("SELECT * FROM identities WHERE account_id = ?")
            .bind(account_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_identity).collect())
    }

    async fn create(&self, identity: &Identity) -> Result<Identity, Error> {
        sqlx::query("INSERT INTO identities (id, contact_id, account_id, platform_user_id, platform_username, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(identity.id.to_string())
            .bind(identity.contact_id.to_string())
            .bind(identity.account_id.to_string())
            .bind(&identity.platform_user_id)
            .bind(&identity.platform_username)
            .bind(identity.created_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(identity.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM identities WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

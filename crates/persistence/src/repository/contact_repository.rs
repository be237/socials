use async_trait::async_trait;
use socials_core::entities::contact::Contact;
use socials_core::repositories::{ContactRepository, Error};
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteContactRepository {
    pool: SqlitePool,
}

impl SqliteContactRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_contact(row: sqlx::sqlite::SqliteRow) -> Contact {
    Contact {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        user_id: Uuid::parse_str(row.get::<&str, _>("user_id")).unwrap(),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
    }
}

#[async_trait]
impl ContactRepository for SqliteContactRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Contact>, Error> {
        let row = sqlx::query("SELECT * FROM contacts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_contact))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Contact>, Error> {
        let rows = sqlx::query("SELECT * FROM contacts WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_contact).collect())
    }

    async fn create(&self, contact: &Contact) -> Result<Contact, Error> {
        sqlx::query("INSERT INTO contacts (id, user_id, display_name, avatar_url, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact.id.to_string())
            .bind(contact.user_id.to_string())
            .bind(&contact.display_name)
            .bind(&contact.avatar_url)
            .bind(contact.created_at.to_rfc3339())
            .bind(contact.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(contact.clone())
    }

    async fn update(&self, contact: &Contact) -> Result<Contact, Error> {
        sqlx::query(
            "UPDATE contacts SET display_name = ?, avatar_url = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&contact.display_name)
        .bind(&contact.avatar_url)
        .bind(contact.updated_at.to_rfc3339())
        .bind(contact.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(contact.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

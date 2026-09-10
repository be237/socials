use async_trait::async_trait;
use socials_core::entities::user::User;
use socials_core::repositories::{Error, UserRepository};
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let row = sqlx::query(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let user = row.map(|row| User {
            id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
            username: row.get("username"),
            email: row.get("email"),
            created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
                .unwrap()
                .with_timezone(&chrono::Utc),
        });

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let row = sqlx::query(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        let user = row.map(|row| User {
            id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
            username: row.get("username"),
            email: row.get("email"),
            created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
                .unwrap()
                .with_timezone(&chrono::Utc),
        });

        Ok(user)
    }

    async fn create(&self, user: &User) -> Result<User, Error> {
        sqlx::query("INSERT INTO users (id, username, email, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&user.email)
            .bind(user.created_at.to_rfc3339())
            .bind(user.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(user.clone())
    }

    async fn update(&self, user: &User) -> Result<User, Error> {
        sqlx::query("UPDATE users SET username = ?, email = ?, updated_at = ? WHERE id = ?")
            .bind(&user.username)
            .bind(&user.email)
            .bind(user.updated_at.to_rfc3339())
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

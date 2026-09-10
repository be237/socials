use async_trait::async_trait;
use socials_core::entities::account::Account;
use socials_core::repositories::{AccountRepository, Error};
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteAccountRepository {
    pool: SqlitePool,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_account(row: sqlx::sqlite::SqliteRow) -> Account {
    Account {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        user_id: Uuid::parse_str(row.get::<&str, _>("user_id")).unwrap(),
        connector_name: row.get("connector_name"),
        platform_account_id: row.get("platform_account_id"),
        display_name: row.get("display_name"),
        is_connected: row.get::<bool, _>("is_connected"),
        access_token: row.get("access_token"),
        refresh_token: row.get("refresh_token"),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, Error> {
        let row = sqlx::query("SELECT * FROM accounts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_account))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, Error> {
        let rows = sqlx::query("SELECT * FROM accounts WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_account).collect())
    }

    async fn find_by_connector(
        &self,
        user_id: Uuid,
        connector: &str,
    ) -> Result<Vec<Account>, Error> {
        let rows = sqlx::query("SELECT * FROM accounts WHERE user_id = ? AND connector_name = ?")
            .bind(user_id.to_string())
            .bind(connector)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_account).collect())
    }

    async fn create(&self, account: &Account) -> Result<Account, Error> {
        sqlx::query("INSERT INTO accounts (id, user_id, connector_name, platform_account_id, display_name, is_connected, access_token, refresh_token, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(account.id.to_string())
            .bind(account.user_id.to_string())
            .bind(&account.connector_name)
            .bind(&account.platform_account_id)
            .bind(&account.display_name)
            .bind(account.is_connected)
            .bind(&account.access_token)
            .bind(&account.refresh_token)
            .bind(account.created_at.to_rfc3339())
            .bind(account.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(account.clone())
    }

    async fn update(&self, account: &Account) -> Result<Account, Error> {
        sqlx::query("UPDATE accounts SET connector_name = ?, platform_account_id = ?, display_name = ?, is_connected = ?, access_token = ?, refresh_token = ?, updated_at = ? WHERE id = ?")
            .bind(&account.connector_name)
            .bind(&account.platform_account_id)
            .bind(&account.display_name)
            .bind(account.is_connected)
            .bind(&account.access_token)
            .bind(&account.refresh_token)
            .bind(account.updated_at.to_rfc3339())
            .bind(account.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(account.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

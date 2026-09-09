use std::sync::Arc;
use tokio::sync::RwLock;
use socials_core::events::bus::EventBus;
use socials_core::services::CoreService;
use socials_persistence::{
    Database,
    SqliteUserRepository,
    SqliteAccountRepository,
    SqliteContactRepository,
    SqliteConversationRepository,
    SqliteMessageRepository,
};
use socials_server::api::{AppState, seed_demo_data, create_router};
use socials_server::telegram::TelegramBot;

#[tokio::main]
async fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let data_dir = std::path::PathBuf::from(home).join(".local/share/socials");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("socials.db");

    let database = Database::new(&db_path)
        .await
        .expect("Failed to initialize SQLite database");

    let user_repo = Arc::new(SqliteUserRepository::new(database.pool.clone()));
    let account_repo = Arc::new(SqliteAccountRepository::new(database.pool.clone()));
    let contact_repo = Arc::new(SqliteContactRepository::new(database.pool.clone()));
    let conv_repo = Arc::new(SqliteConversationRepository::new(database.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(database.pool.clone()));
    let event_bus = Arc::new(EventBus::new());

    let core = Arc::new(CoreService::new(
        user_repo,
        account_repo,
        contact_repo,
        conv_repo,
        msg_repo,
        event_bus,
    ));

    // Initialize Telegram bot
    let telegram_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .unwrap_or_else(|_| "8916975203:AAHPB8QBNUQl92ELeGoGqDEe1meEA9YRt1s".to_string());

    let telegram = Arc::new(RwLock::new(None));

    match TelegramBot::new(telegram_token).await {
        Ok(bot) => {
            *telegram.write().await = Some(bot);
            println!("Telegram bot connected!");

            // Start polling in background
            let core_clone = core.clone();
            let telegram_clone = telegram.clone();
            tokio::spawn(async move {
                loop {
                    if let Some(bot) = telegram_clone.write().await.as_mut() {
                        if let Err(e) = bot.poll_updates(&core_clone).await {
                            eprintln!("Telegram poll error: {}", e);
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });
        }
        Err(e) => {
            eprintln!("Failed to connect Telegram bot: {}", e);
        }
    }

    // Seed demo data only if no conversations exist
    seed_demo_data(&core).await;

    let state = AppState { core, telegram };
    let app = create_router(state);

    println!("Socials API running on http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server failed");
}

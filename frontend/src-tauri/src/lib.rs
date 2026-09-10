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
use socials_server::api::{AppState, AuthState, seed_demo_data, create_router};
use socials_server::telegram::TelegramBot;
use socials_telegram_user::TelegramUserClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
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

                // Initialize Telegram user client (MTProto)
                let mut telegram_user = TelegramUserClient::new(&data_dir);
                if let Err(e) = telegram_user.connect().await {
                    eprintln!("Telegram user client: {}", e);
                }

                // Initialize Telegram bot (fallback, only if TELEGRAM_BOT_TOKEN is set)
                let telegram_bot = Arc::new(RwLock::new(None));
                if let Ok(telegram_token) = std::env::var("TELEGRAM_BOT_TOKEN") {
                    match TelegramBot::new(telegram_token).await {
                        Ok(bot) => {
                            *telegram_bot.write().await = Some(bot);
                            println!("Telegram bot connected (fallback mode)!");
                        }
                        Err(e) => {
                            eprintln!("Failed to connect Telegram bot: {}", e);
                        }
                    }
                }

                let auth_state = AuthState {
                    telegram: Arc::new(RwLock::new(telegram_user)),
                };

                seed_demo_data(&core).await;

                let state = AppState {
                    core,
                    telegram: telegram_bot,
                    auth: auth_state,
                };
                let router = create_router(state);

                let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                    .await
                    .expect("Failed to bind");

                tokio::spawn(async move {
                    axum::serve(listener, router).await.expect("Server failed");
                });

                use tauri::Emitter;
                let _ = app_handle.emit("server-ready", ());
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

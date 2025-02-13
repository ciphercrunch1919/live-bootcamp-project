use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use secrecy::Secret;

use auth_service::{
    app_state::AppState,
    get_postgres_pool, get_redis_client,
    services::{
        data_stores::{PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore},
        mock_email_client::MockEmailClient,
    },
    utils::{
        constants::{prod, DATABASE_URL, REDIS_HOST_NAME}, 
        tracing::init_tracing
    },
    Application,
};

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing"); // Updated!

    
    let pg_pool = configure_postgresql().await;
    let redis_client = Arc::new(RwLock::new(configure_redis()));

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_client.clone())));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_client)));

    let email_client = Arc::new(MockEmailClient);

    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
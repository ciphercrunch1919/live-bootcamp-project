use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState, get_postgres_pool, get_redis_client, services::data_stores::{
        HashmapTwoFACodeStore, RedisBannedTokenStore, MockEmailClient, PostgresUserStore
    }, utils::constants::{prod, DATABASE_URL, REDIS_HOST_NAME}, Application
};

#[tokio::main]
async fn main() {
    // We will use this PostgreSQL pool in the next task! 
    let pg_pool = configure_postgresql().await;
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));

    let redis_client = Arc::new(RwLock::new(configure_redis()));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore ::new(redis_client)));
    
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));

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
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database! 
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
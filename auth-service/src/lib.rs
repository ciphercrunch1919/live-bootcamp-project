pub mod routes;
pub mod services;
mod domain;
pub mod app_state;

use axum::{routing::post, serve::Serve, Router};
use tower_http::services::ServeDir;
use std::error::Error;
use routes::{login, logout, signup, verify_2fa, verify_token};
use std::sync::Arc;
use tokio::sync::RwLock;
use app_state::AppState;

use services::hashmap_user_store::HashmapUserStore;
pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

pub type UserStoreType = Arc<RwLock<HashmapUserStore>>;

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-2fa", post(verify_2fa))
            .route("/logout", post(logout))
            .route("/verify-token", post(verify_token))
            .with_state(app_state.into());

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);
        
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }

}
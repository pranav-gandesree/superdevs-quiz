
mod keypair;
mod create_token;
mod mint_token;

use keypair::{hello, generate_keypair};

use axum::{
    routing::{get, post},
    Router,
};

use crate::{create_token::create_token, mint_token::mint_token};



#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token));


    println!("Hello Solana from axum!");

    let port = std::env::var("PORT").unwrap_or("3000".into());
    let address = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

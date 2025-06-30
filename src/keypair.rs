
use axum::{response::IntoResponse, response::Json};
// use axum::extract::Query;
use serde::{ Serialize};
use solana_sdk::{
    pubkey::{ Pubkey}, 
    signature::Keypair, 
    signer::Signer
};
use bs58;

#[derive(Serialize)]
struct Data {
    pubkey: String,
    secret: String,
}

#[derive(Serialize)]
struct MyResponse {
    success: bool,
    data: Data,
}

pub async fn hello() -> impl IntoResponse {
    println!("hi from axum");
    "Hello from Axum!"
}

pub async fn generate_keypair() -> impl IntoResponse {
    let keypair = Keypair::new();
    let pubkey: Pubkey = keypair.pubkey();
    let secret = keypair.to_bytes();
    
    let response = MyResponse {
        success: true,
        data: Data {
            pubkey: pubkey.to_string(), // This gives base58 encoded pubkey
            secret: bs58::encode(&secret).into_string(), // Base58 encode the secret key
        }
    };
    
    Json(response)
}

use axum::{
    Json,
    http::StatusCode,
};
use serde::{Deserialize};
use serde_json::{json, Value};
use solana_sdk::signature::{Keypair, Signer};
use base64::Engine;

#[derive(Deserialize)]
pub struct MessageSignRequest {
    text: Option<String>,
    private_key: Option<String>,
}

fn create_error_response(status: StatusCode, error_msg: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "success": false,
            "error": error_msg
        }))
    )
}

fn validate_input_text(input: &Option<String>) -> Result<&String, (StatusCode, Json<Value>)> {
    match input {
        None => Err(create_error_response(
            StatusCode::BAD_REQUEST, 
            "Text field is required"
        )),
        Some(content) if content.trim().is_empty() => Err(create_error_response(
            StatusCode::BAD_REQUEST, 
            "Text content cannot be empty"
        )),
        Some(valid_content) => Ok(valid_content),
    }
}

fn validate_private_key(key: &Option<String>) -> Result<&String, (StatusCode, Json<Value>)> {
    match key {
        None => Err(create_error_response(
            StatusCode::BAD_REQUEST, 
            "Private key field is required"
        )),
        Some(key_value) if key_value.trim().is_empty() => Err(create_error_response(
            StatusCode::BAD_REQUEST, 
            "Private key cannot be empty"
        )),
        Some(valid_key) => Ok(valid_key),
    }
}

fn decode_base58_key(encoded_key: &str) -> Result<Vec<u8>, (StatusCode, Json<Value>)> {
    bs58::decode(encoded_key)
        .into_vec()
        .map_err(|_| create_error_response(
            StatusCode::BAD_REQUEST, 
            "Invalid private key encoding"
        ))
}

fn validate_key_length(key_bytes: &[u8]) -> Result<(), (StatusCode, Json<Value>)> {
    if key_bytes.len() != 64 {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST, 
            "Private key must be 64 bytes long"
        ));
    }
    Ok(())
}

fn create_keypair_from_bytes(raw_bytes: &[u8]) -> Result<Keypair, (StatusCode, Json<Value>)> {
    Keypair::try_from(raw_bytes)
        .map_err(|_| create_error_response(
            StatusCode::BAD_REQUEST, 
            "Cannot create keypair from provided private key"
        ))
}

fn build_success_response(signed_data: &[u8], wallet_pubkey: &str, original_text: &str) -> Json<Value> {
    let encoded_signature = base64::engine::general_purpose::STANDARD.encode(signed_data);
    
    Json(json!({
        "success": true,
        "result": {
            "signed_message": encoded_signature,
            "wallet_address": wallet_pubkey,
            "original_text": original_text
        }
    }))
}

#[axum::debug_handler]
pub async fn process_message_signing(
    Json(request_data): Json<MessageSignRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
    // Extract and validate the text to be signed
    let text_to_sign = validate_input_text(&request_data.text)?;
    
    // Extract and validate the private key
    let raw_private_key = validate_private_key(&request_data.private_key)?;
    
    // Decode the base58 encoded private key
    let decoded_key_bytes = decode_base58_key(raw_private_key)?;
    
    // Ensure the key has the correct length
    validate_key_length(&decoded_key_bytes)?;
    
    // Generate the keypair from the raw bytes
    let wallet_keypair = create_keypair_from_bytes(&decoded_key_bytes)?;
    
    // Perform the actual message signing
    let message_signature = wallet_keypair.sign_message(text_to_sign.as_bytes());
    
    // Extract the public key from the keypair
    let wallet_address = wallet_keypair.pubkey();
    let encoded_wallet_address = bs58::encode(wallet_address.to_bytes()).into_string();
    
    // Build and return the success response
    Ok(build_success_response(
        message_signature.as_ref(),
        &encoded_wallet_address,
        text_to_sign
    ))
}
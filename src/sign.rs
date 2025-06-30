use axum::{
    Json,
    http::StatusCode,
    extract,
};
use serde::{Deserialize};
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signature, Signer}};
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
    
   
    let text_to_sign = validate_input_text(&request_data.text)?;
    
    
    let raw_private_key = validate_private_key(&request_data.private_key)?;
    
    
    let decoded_key_bytes = decode_base58_key(raw_private_key)?;
    

    validate_key_length(&decoded_key_bytes)?;
    
   
    let wallet_keypair = create_keypair_from_bytes(&decoded_key_bytes)?;
    

    let message_signature = wallet_keypair.sign_message(text_to_sign.as_bytes());
    
   
    let wallet_address = wallet_keypair.pubkey();
    let encoded_wallet_address = bs58::encode(wallet_address.to_bytes()).into_string();
    
  
    Ok(build_success_response(
        message_signature.as_ref(),
        &encoded_wallet_address,
        text_to_sign
    ))
}




#[derive(Deserialize)]
pub struct SignatureVerificationRequest {
    text: Option<String>,
    signed_data: Option<String>,
    wallet_address: Option<String>,
}

struct ValidationError {
    status: StatusCode,
    message: String,
}

impl ValidationError {
    fn new(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    fn to_response(self) -> (StatusCode, Json<Value>) {
        (
            self.status,
            Json(json!({
                "success": false,
                "error": self.message
            }))
        )
    }
}

fn extract_text_content(text_input: &Option<String>) -> Result<&String, ValidationError> {
    match text_input {
        None => Err(ValidationError::new("Text field is mandatory")),
        Some(content) if content.trim().is_empty() => {
            Err(ValidationError::new("Text content must not be empty"))
        }
        Some(valid_text) => Ok(valid_text),
    }
}

fn extract_signature_data(sig_input: &Option<String>) -> Result<&String, ValidationError> {
    match sig_input {
        None => Err(ValidationError::new("Signature field is mandatory")),
        Some(sig_data) if sig_data.trim().is_empty() => {
            Err(ValidationError::new("Signature data must not be empty"))
        }
        Some(valid_signature) => Ok(valid_signature),
    }
}

fn extract_wallet_address(addr_input: &Option<String>) -> Result<&String, ValidationError> {
    match addr_input {
        None => Err(ValidationError::new("Wallet address field is mandatory")),
        Some(addr_data) if addr_data.trim().is_empty() => {
            Err(ValidationError::new("Wallet address must not be empty"))
        }
        Some(valid_address) => Ok(valid_address),
    }
}

fn parse_wallet_address(encoded_address: &str) -> Result<Pubkey, ValidationError> {
    let address_bytes = bs58::decode(encoded_address)
        .into_vec()
        .map_err(|_| ValidationError::new("Wallet address encoding is invalid"))?;

    Pubkey::try_from(address_bytes.as_slice())
        .map_err(|_| ValidationError::new("Cannot parse wallet address"))
}

fn parse_signature_bytes(encoded_signature: &str) -> Result<Signature, ValidationError> {
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded_signature)
        .map_err(|_| ValidationError::new("Signature encoding is invalid"))?;

    Signature::try_from(sig_bytes.as_slice())
        .map_err(|_| ValidationError::new("Cannot parse signature data"))
}

fn perform_signature_verification(
    signature_obj: &Signature,
    wallet_pubkey: &Pubkey,
    original_text: &str,
) -> bool {
    signature_obj.verify(wallet_pubkey.as_ref(), original_text.as_bytes())
}

fn create_verification_response(
    verification_result: bool,
    original_text: &str,
    wallet_addr: &str,
) -> Json<Value> {
    Json(json!({
        "success": true,
        "result": {
            "is_verified": verification_result,
            "original_text": original_text,
            "wallet_address": wallet_addr
        }
    }))
}

#[axum::debug_handler]
pub async fn authenticate_message_signature(
    extract::Json(request_payload): extract::Json<SignatureVerificationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
    
    let text_content = extract_text_content(&request_payload.text)
        .map_err(|e| e.to_response())?;
    
    
    let signature_data = extract_signature_data(&request_payload.signed_data)
        .map_err(|e| e.to_response())?;
    
   
    let wallet_addr_str = extract_wallet_address(&request_payload.wallet_address)
        .map_err(|e| e.to_response())?;
    

    let parsed_wallet_addr = parse_wallet_address(wallet_addr_str)
        .map_err(|e| e.to_response())?;
    

    let parsed_signature = parse_signature_bytes(signature_data)
        .map_err(|e| e.to_response())?;

    let verification_outcome = perform_signature_verification(
        &parsed_signature,
        &parsed_wallet_addr,
        text_content,
    );
    

    Ok(create_verification_response(
        verification_outcome,
        text_content,
        wallet_addr_str,
    ))
}
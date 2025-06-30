use axum::{Json, http::StatusCode, extract};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, system_instruction, system_program};
use spl_token::instruction;
use base64::Engine;




#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: Option<String>,
    pub to: Option<String>,
    pub lamports: Option<u64>,
}

#[derive(Serialize)]
pub struct AccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
} 


#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub destination: Option<String>,
    pub mint: Option<String>,
    pub owner: Option<String>,
    pub amount: Option<u64>,
}



pub async fn send_solana(
    extract::Json(payload): extract::Json<SendSolRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
    // Validate from field
    let from_str = match &payload.from {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: from"
                }))
            ));
        }
        Some(from) if from.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "From address cannot be empty"
                }))
            ));
        }
        Some(from) => from,
    };

    // Validate to field
    let to_str = match &payload.to {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: to"
                }))
            ));
        }
        Some(to) if to.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "To address cannot be empty"
                }))
            ));
        }
        Some(to) => to,
    };

    // Validate lamports field
    let lamports = match payload.lamports {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: lamports"
                }))
            ));
        }
        Some(0) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Amount must be greater than 0"
                }))
            ));
        }
        Some(amt) => amt,
    };

    let from = match bs58::decode(from_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid from public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid from public key format"
                }))
            ));
        }
    };

    let to = match bs58::decode(to_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid to public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid to public key format"
                }))
            ));
        }
    };

    // Create SOL transfer instruction
    let instruction = system_instruction::transfer(
        &from,
        &to,
        lamports,
    );

    // Convert accounts to required format
    let accounts: Vec<AccountMeta> = instruction.accounts.iter().map(|meta| AccountMeta {
        pubkey: bs58::encode(meta.pubkey.to_bytes()).into_string(),
        is_signer: meta.is_signer,
        is_writable: meta.is_writable,
    }).collect();

    let response = json!({
        "success": true,
        "data": {
            "program_id": bs58::encode(system_program::ID.to_bytes()).into_string(),
            "accounts": accounts,
            "instruction_data": base64::engine::general_purpose::STANDARD.encode(&instruction.data)
        }
    });

    Ok(Json(response))
}

pub async fn send_token(
    extract::Json(payload): extract::Json<SendTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
    // Validate destination field
    let destination_str = match &payload.destination {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: destination"
                }))
            ));
        }
        Some(dest) if dest.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Destination address cannot be empty"
                }))
            ));
        }
        Some(dest) => dest,
    };

    // Validate mint field
    let mint_str = match &payload.mint {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: mint"
                }))
            ));
        }
        Some(mint) if mint.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Mint address cannot be empty"
                }))
            ));
        }
        Some(mint) => mint,
    };

    // Validate owner field
    let owner_str = match &payload.owner {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: owner"
                }))
            ));
        }
        Some(owner) if owner.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Owner address cannot be empty"
                }))
            ));
        }
        Some(owner) => owner,
    };

    // Validate amount field
    let amount = match payload.amount {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: amount"
                }))
            ));
        }
        Some(0) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Amount must be greater than 0"
                }))
            ));
        }
        Some(amt) => amt,
    };

    let destination = match bs58::decode(destination_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid destination public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid destination public key format"
                }))
            ));
        }
    };

    let source = match bs58::decode(mint_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid source public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid source public key format"
                }))
            ));
        }
    };

    let owner = match bs58::decode(owner_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid owner public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid owner public key format"
                }))
            ));
        }
    };

    // Create token transfer instruction
    let instruction = instruction::transfer(
        &spl_token::ID,
        &source,  // source token account
        &destination,  // destination token account
        &owner,  // owner of source account
        &[],  // signer seeds
        amount,
    ).map_err(|_| (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "Failed to create token transfer instruction"
        }))
    ))?;

    // Convert accounts to required format
    let accounts: Vec<AccountMeta> = instruction.accounts.iter().map(|meta| AccountMeta {
        pubkey: bs58::encode(meta.pubkey.to_bytes()).into_string(),
        is_signer: meta.is_signer,
        is_writable: meta.is_writable,
    }).collect();

    let response = json!({
        "success": true,
        "data": {
            "program_id": bs58::encode(spl_token::ID.to_bytes()).into_string(),
            "accounts": accounts,
            "instruction_data": base64::engine::general_purpose::STANDARD.encode(&instruction.data)
        }
    });

    Ok(Json(response))
} 
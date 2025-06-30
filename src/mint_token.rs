use axum::{
    Json,
    http::StatusCode,
    extract,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction;
use base64::Engine;

#[derive(Deserialize)]
pub struct MintTokenRequest {
    mint: Option<String>,
    destination: Option<String>,
    authority: Option<String>,
    amount: Option<u64>,
}


#[derive(Serialize, Debug, Deserialize)]
pub struct AccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

pub async fn mint_token(
    extract::Json(payload): extract::Json<MintTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
   
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


    let authority_str = match &payload.authority {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: authority"
                }))
            ));
        }
        Some(auth) if auth.trim().is_empty() => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Authority address cannot be empty"
                }))
            ));
        }
        Some(auth) => auth,
    };

    
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

    let mint = match bs58::decode(mint_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid mint public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid mint public key format"
                }))
            ));
        }
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

    let authority = match bs58::decode(authority_str).into_vec() {
        Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid authority public key"
                    }))
                ));
            }
        },
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid authority public key format"
                }))
            ));
        }
    };

    
    let instruction = instruction::mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &authority,
        &[],
        amount,
    ).map_err(|_| (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "Failed to create mint-to instruction"
        }))
    ))?;

    
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

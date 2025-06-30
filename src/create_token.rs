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
pub struct CreateTokenRequest {
    mint_authority: Option<String>,
    mint: Option<String>,
    decimals: Option<u8>,
}



#[derive(Serialize)]
pub struct AccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}



pub async fn create_token(
    extract::Json(payload): extract::Json<CreateTokenRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    
    
    //     None => {
    //         return Err((
    //             StatusCode::BAD_REQUEST,
    //             Json(json!({
    //                 "success": false,
    //                 "error": "Missing required field: mint_authority"
    //             }))
    //         ));
    //     }
    //     Some(authority) => {
    //         match bs58::decode(authority).into_vec() {
    //             // Ok(bytes) => match Pubkey::try_from(bytes.as_slice()) {
    //                 Some(authority) => match authority.parse::<Pubkey>() {
    //                 Ok(pubkey) => pubkey,
    //                 Err(_) => {
    //                     return Err((
    //                         StatusCode::BAD_REQUEST,
    //                         Json(json!({
    //                             "success": false,
    //                             "error": "Invalid mint authority public key"
    //                         }))
    //                     ));
    //                 }
    //             },
    //             Err(_) => {
    //                 return Err((
    //                     StatusCode::BAD_REQUEST,
    //                     Json(json!({
    //                         "success": false,
    //                         "error": "Invalid mint authority public key format"
    //                     }))
    //                 ));
    //             }
    //         }
    //     }
    // };
    let mint_authority = match &payload.mint_authority {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: mint_authority"
                }))
            ));
        }
        Some(authority_str) => match authority_str.parse::<Pubkey>() {
            Ok(pubkey) => pubkey,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid mint authority public key"
                    }))
                ));
            }
        },
    };

   
let mint = match &payload.mint {
    None => {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Missing required field: mint"
            }))
        ));
    }
    Some(mint_str) => match mint_str.parse::<Pubkey>() {
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
};

    let decimals = match payload.decimals {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Missing required field: decimals"
                }))
            ));
        }
        Some(decimals) => decimals,
    };


    let instruction = instruction::initialize_mint(
        &spl_token::ID,
        &mint,
        &mint_authority,
        Some(&mint_authority),
        decimals,
    ).map_err(|_| (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "Failed to create initialize mint instruction"
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

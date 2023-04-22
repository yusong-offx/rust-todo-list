use lazy_static::lazy_static;
use serde_json::json;
use std::{time::{SystemTime, UNIX_EPOCH, Duration}, ops::Add};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{
    encode, decode, Header, Validation, EncodingKey, DecodingKey
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::{
    dev::ServiceRequest, HttpMessage
};
use myerror::ServerError;


#[allow(dead_code)]
const JWT_EXP_MIN: u64 = 15;

lazy_static! {
    static ref TOKEN_VALIDATOR: Validation = {
        let mut validator = Validation::default();
        validator.leeway = 0;
        validator
    };

    static ref FOR_DECODE: DecodingKey = DecodingKey::from_secret(SECRET_KEY.as_ref());
    static ref FOR_ENCODE: EncodingKey = EncodingKey::from_secret(SECRET_KEY.as_ref());
}

const SECRET_KEY: &str = "jwt_scret_key";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaim {
    pub user_id: i32,
    exp: usize
}

pub async fn jwt_validator(
    req: ServiceRequest,
    _credentials: BearerAuth
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    // Decode Token
    match verify_token(_credentials.token()).await {
        // Token decode error
        Err(e) => Err((actix_web::error::ErrorUnauthorized(json!(e)), req)),
        // Valid
        Ok(v) => {
            // Compare jwt's user_id and path's user_id
            // Get jwt's user_id
            let jwt_user_id = v.user_id;
            
            // Get path's user_id
            let path_user_id = req.match_info().query("user_id").as_bytes();

            // Compare
            if path_user_id == jwt_user_id.to_string().as_bytes() {
                req.extensions_mut().insert(v);
                return Ok(req)
            }
            Err((actix_web::error::ErrorUnauthorized("NotFound"), req))
        },
    }
}


pub async fn generate_token(user_id: i32) -> Result<String, ServerError> {
    // Make Custom Claim
    let claims = JwtClaim{
            user_id: user_id,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap()
                .add(Duration::from_secs(60 * JWT_EXP_MIN))
                .as_secs() as usize
        };

    // Make Signed Token
    let signed_token = encode(
            &Header::default(), 
            &claims, 
            &FOR_ENCODE
        )
        .map_err(|e|
            ServerError::InternalServerError { msg: "JWT generate error", detail: e.to_string() }
        )?;

    Ok(signed_token)
}

pub async fn verify_token(token: &str) -> Result<JwtClaim, ServerError> {
    let token = decode::<JwtClaim>(
            &token, 
            &FOR_DECODE, 
            &TOKEN_VALIDATOR
        )
        .map_err(|e|
            ServerError::InternalServerError { msg: "JWT verfy error", detail: e.to_string() }
        )?;

    Ok(token.claims)
}
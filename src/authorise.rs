use jsonwebtoken::{decode, DecodingKey, Algorithm, Validation, decode_header};
use reqwest;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,                 
    pub iat: usize,     
    pub aud: String, 
    pub iss: String,      
    pub sub: String,
    pub auth_time: usize,
}

pub async fn authorise(token: &str, project_id: &str) -> Result<Claims, Response> {
    fn unauthorised() -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
    let algorithm = Algorithm::RS256;
    let validation = Validation::new(algorithm);
    let header = decode_header(token).map_err(|_| unauthorised())?;
    if header.alg != algorithm {
        return Err(unauthorised());
    }
    let decoding_key = get_decoding_key(&header.kid.ok_or(unauthorised())?).await.ok_or(unauthorised())?;
    let decoded_token = decode::<Claims>(token, &decoding_key, &validation).map_err(|_| unauthorised())?;
    let issuer = format!("https://securetoken.google.com/{}", project_id);
    let claims = decoded_token.claims;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = now.as_secs() as usize;
    //expiration date must be in the future, issued-at-time must be in the past,
    //authentication-time must be in the past, aud must be equal to project_id, issuer must be 
    //"https://securetoken.google.com/<projectId>", subject must be a non empty string(user id)
    let safety_gap = 1; //add another second as safety gap
    if claims.exp <= secs || claims.iat >= secs + safety_gap || claims.auth_time >= secs + safety_gap || project_id != claims.aud || claims.iss != issuer || claims.sub.is_empty() {
        return Err(unauthorised());
    }
    Ok(claims)
}

async fn get_decoding_key(kid: &String) -> Option<DecodingKey> {
    let response = reqwest::get("https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com").await.unwrap();
    let keys: HashMap<String, String> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    let public_key =  keys.get::<String>(kid);
    return match public_key {
        Some(key) => Some(DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()),
        None => None,
    }
}
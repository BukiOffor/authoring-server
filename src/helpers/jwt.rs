use axum::{
    Json, RequestPartsExt,
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::{
    error::ModuleError,
    helpers::dto::auth::{AuthBodyDto, JwtPayloadDto},
};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
}

pub async fn generate_token(payload: JwtPayloadDto) -> Result<Json<AuthBodyDto>, ModuleError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(100))
        .expect("valid timestamp")
        .timestamp() as usize;

    let refresh_expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(120))
        .expect("valid timestamp")
        .timestamp() as usize;

    let mut claims = Claims {
        user_id: payload.id.clone(),
        exp: expiration,
    };

    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| ModuleError::TokenCreation)?;

    claims.exp = refresh_expiration;

    let refresh_token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| ModuleError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBodyDto::new(token, refresh_token, payload.id)))
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ModuleError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|e| ModuleError::CouldNotExtractToken(e.to_string()))?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| ModuleError::InvalidToken)?;
        Ok(token_data.claims)
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub async fn auth_middleware(
    Claims { .. }: Claims,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    Ok(next.run(req).await)
}

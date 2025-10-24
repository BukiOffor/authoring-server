use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("{0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("{0}")]
    InternalError(String),

    #[error("User does not have access to this service")]
    PermissionDenied,

    #[error("{0}")]
    ParseError(String),

    #[error("TokenCreationError")]
    TokenCreation,

    #[error("Provided Token is Invalid")]
    InvalidToken,

    #[error("Missing Credentials")]
    MissingCredentials,

    #[error("{0}")]
    CouldNotExtractToken(String),

    #[error("Incorrect Username or Password")]
    WrongCredentials,

    #[error("{0}")]
    ItemNotFound(String),

    #[error("{0}")]
    Error(String),

    #[error("{0}")]
    ConversionError(String),

    #[error("{0}")]
    InvalidOtp(&'static str),
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
    pub status_code: u32,
}

impl Default for ErrorMessage {
    fn default() -> Self {
        Self {
            message: "Something went wrong, Please contact Adminstrator".to_string(),
            status_code: 500,
        }
    }
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ErrorMessage {
    pub fn build(&mut self, message: String, status_code: u32) -> Self {
        Self {
            message,
            status_code,
        }
    }
}

impl IntoResponse for ModuleError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidToken | Self::InvalidOtp(_) => {
                let message = ErrorMessage::default().build(self.to_string(), 401);
                (axum::http::StatusCode::UNAUTHORIZED, axum::Json(message)).into_response()
            }
            Self::PermissionDenied => {
                let message = ErrorMessage::default().build(self.to_string(), 403);
                (axum::http::StatusCode::FORBIDDEN, axum::Json(message)).into_response()
            }
            Self::ItemNotFound(_) => {
                let message = ErrorMessage::default().build(self.to_string(), 404);
                (axum::http::StatusCode::NOT_FOUND, axum::Json(message)).into_response()
            }
            Self::WrongCredentials => {
                let message = ErrorMessage::default().build(self.to_string(), 401);
                (axum::http::StatusCode::UNAUTHORIZED, axum::Json(message)).into_response()
            }
            Self::DieselError(_) | Self::InternalError(_) | Self::TokenCreation => {
                let message = ErrorMessage::default().build(self.to_string(), 500);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(message),
                )
                    .into_response()
            }
            Self::ParseError(_) | Self::CouldNotExtractToken(_) | Self::Error(_) => {
                let message = ErrorMessage::default().build(self.to_string(), 400);
                (axum::http::StatusCode::BAD_REQUEST, axum::Json(message)).into_response()
            }
            _ => {
                let message = ErrorMessage::default().build(self.to_string(), 500);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(message),
                )
                    .into_response()
            }
        }
    }
}

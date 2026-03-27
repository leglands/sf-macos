//! sf-remote — Remote SF instance management
//! HTTP API client + SSE consumer + OAuth2 PKCE + Keychain

pub mod instance;
pub mod sse;
pub mod oauth;
pub mod keychain;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("Keychain error: {0}")]
    Keychain(String),
    #[error("{0}")]
    Other(String),
}

pub type RemoteResult<T> = Result<T, RemoteError>;

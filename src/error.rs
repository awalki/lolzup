use thiserror::Error;

#[derive(Error, Debug)]
pub enum LolzUpError {
    #[error("Network error, lzt problem?")]
    Network(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(#[from] std::env::VarError),

    #[error("Scope error: {0}")]
    Scope(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::error::Error),

    #[error("Bump error: {0}")]
    Bump(String),

    #[error("Telegram (Frankenstein) error: {0}")]
    Telegram(#[from] frankenstein::Error),
}

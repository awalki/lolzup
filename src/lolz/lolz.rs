use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use frankenstein::reqwest;
use frankenstein::reqwest::Client;
use jsonwebtoken::TokenData;
use jsonwebtoken::dangerous::insecure_decode;
use serde::{Deserialize, Serialize};

use crate::error::LolzUpError;

#[derive(Deserialize, Serialize)]
pub struct Bump {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub next_available_time: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize)]
pub struct Permissions {
    pub bump: Bump,
}

#[derive(Deserialize, Serialize)]
pub struct Thread {
    pub thread_id: i64,
    pub thread_title: String,
    pub permissions: Permissions,
}

#[derive(Deserialize, Serialize)]
pub struct GetThreadResponse {
    pub thread: Thread,
}

#[derive(Deserialize)]
struct Claims {
    scope: String,
}

const LOLZ_API_URL: &str = "https://prod-api.lolz.live/threads/";

#[async_trait]
pub trait LolzClient {
    async fn bump_thread(&self, thread_id: i64) -> Result<reqwest::StatusCode, LolzUpError>;
    async fn get_thread_by_id(&self, thread_id: i64) -> Result<GetThreadResponse, LolzUpError>;
}

pub struct LolzHttpClient {
    token: String,
    http: Arc<reqwest::Client>,
}

#[async_trait]
impl LolzClient for LolzHttpClient {
    async fn bump_thread(&self, thread_id: i64) -> Result<reqwest::StatusCode, LolzUpError> {
        let threads_url = format!("{}/bump", thread_id);

        let req = self
            .http
            .post(format!("{}{}", LOLZ_API_URL, threads_url))
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(req.status())
    }

    async fn get_thread_by_id(&self, thread_id: i64) -> Result<GetThreadResponse, LolzUpError> {
        let req = self
            .http
            .get(format!("{}{}", LOLZ_API_URL, thread_id))
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(req.json::<GetThreadResponse>().await?)
    }
}

impl LolzHttpClient {
    pub fn new(token: String) -> Result<Self, LolzUpError> {
        let data: TokenData<Claims> = insecure_decode(&token)?;

        let scopes_str = data.claims.scope;

        let required_scope = ["read", "post"];

        if required_scope.iter().any(|s| !scopes_str.contains(s)) {
            return Err(LolzUpError::Scope("Missing scope, why: https://nztcdn.com/files/b07d7e62-c351-4e69-8e04-53f44ea5ba43.webp".to_string()));
        }

        Ok(Self {
            token: token.into(),
            http: Arc::from(Client::default()),
        })
    }
}

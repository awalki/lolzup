use frankenstein::reqwest;
use frankenstein::reqwest::Client;
use jsonwebtoken::TokenData;
use jsonwebtoken::dangerous::insecure_decode;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone)]
pub struct Lolz {
    client: Client,
    token: String,
}

#[derive(Deserialize)]
struct Claims {
    scope: String,
}

const LOLZ_API_URL: &str = "https://prod-api.lolz.live/threads/";

impl Lolz {
    pub fn new(token: String) -> Self {
        // secure: scope is not secure information, pohui?
        let data: TokenData<Claims> =
            insecure_decode(&token).expect("provided token is not a jwt token");
        let scopes_str = data.claims.scope;

        let ok = scopes_str.contains(&"read") && scopes_str.contains(&"read");

        if !ok {
            panic!("select 'read' and 'post' scopes before creating a token")
        }

        Self {
            client: Client::default(),
            token: token.into(),
        }
    }
    pub async fn bump_thread(&self, thread_id: i64) -> Result<reqwest::StatusCode, reqwest::Error> {
        let threads_url = format!("{}/bump", thread_id);

        let req = self
            .client
            .post(format!("{}{}", LOLZ_API_URL, threads_url))
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(req.status())
    }
    pub async fn get_thread(&self, thread_id: i64) -> Result<Value, reqwest::Error> {
        let req = self
            .client
            .get(format!("{}{}", LOLZ_API_URL, thread_id))
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(req.json().await?)
    }
}

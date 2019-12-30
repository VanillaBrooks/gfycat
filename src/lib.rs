mod error;

use serde::Deserialize;
use serde_json;
use std::time;

const ENDPOINT: &str = "https://api.gfycat.com/v1/";
type ClientType = reqwest::Client;

/// Direct response from gfycat http request
#[derive(Deserialize, Debug)]
struct TokenResponse {
    token_type: TokenType,
    expires_in: u64,
    access_token: String,
}

impl TokenResponse {
    fn to_api(self, client: ClientType) -> Result<Api, error::AuthError> {
        let expire = time::Duration::from_secs(self.expires_in);
        let instant_expire = match time::Instant::now().checked_add(expire) {
            Some(expiration) => expiration,
            None => return Err(error::AuthError::Expiration),
        };

        Ok(Api {
            token_type: self.token_type,
            expiration: instant_expire,
            token: self.access_token,
            client: client,
        })
    }
}

/// Return types enumerated for future compatability + memory space
#[derive(Debug, Deserialize)]
enum TokenType {
    #[serde(rename = "bearer")]
    Bearer,
}

/// Api handler for gfycat
#[derive(Debug)]
pub struct Api {
    token_type: TokenType,
    expiration: time::Instant,
    token: String,
    client: ClientType,
}
impl Default for Api {
    fn default() -> Self {
        Api {
            token_type: TokenType::Bearer,
            expiration: time::Instant::now(),
            token: "".into(),
            client: reqwest::Client::new(),
        }
    }
}

impl Api {
    /// create a new api handler
    pub async fn new(client_id: String, client_secret: String) -> Result<Api, error::AuthError> {
        let client = reqwest::Client::new();

        let form = serde_json::json!{
            {
                "client_id": client_id,
                "client_secret": client_secret,
                "grant_type": "client_credentials",

            }
        };

        let response = client
            .post("https://api.gfycat.com/v1/oauth/token")
            .json(&form)
            .send()
            .await?
        .json::<TokenResponse>()
        .await?;

        Ok(response.to_api(client)?)
    }
}

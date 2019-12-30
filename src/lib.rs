mod error;

use surf;
use serde_json;
use serde::Deserialize;
use std::time;

/// Direct response from gfycat http request
#[derive(Deserialize, Debug)]
struct TokenResponse {
    token_type: TokenType,
    expires_in: u64,
    access_token: String
}

impl TokenResponse {
    fn to_api(self) -> Result<Api, error::AuthError> {
        let expire = time::Duration::from_secs(self.expires_in);
        let instant_expire = 
        match time::Instant::now().checked_add(expire) {
            Some(expiration) => expiration,
            None => return Err(error::AuthError::Expiration)
        };

        Ok(Api {
            token_type: self.token_type,
            expiration: instant_expire,
            token: self.access_token
        })
    }
}

/// Return types enumerated for future compatability + memory space
#[derive(Debug, Deserialize)]
enum TokenType{
    #[serde(rename="bearer")]
    Bearer
}

/// Api handler for gfycat
#[derive(Debug)]
pub struct Api {
    token_type: TokenType,
    expiration: time::Instant,
    token: String
}

impl Api {
    pub async fn new(client_id: String, client_secret: String) -> Result<Api,error::AuthError> {
        let client = surf::Client::new();

        let form = serde_json::json!{
            {
                "client_id": client_id,
                "client_secret": client_secret,
                "grant_type": "client_credentials",

            }
        };

        let response : TokenResponse= client.post("https://api.gfycat.com/v1/oauth/token")
            .body_json(&form)?
            .await?
            .body_json()
            .await?;

        Ok(response.to_api()?)

    }
}
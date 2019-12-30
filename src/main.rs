use gfycat;

use serde::{Deserialize, Serialize};
use tokio;

#[tokio::main]
async fn main() {
    let api = gfycat::LoadCredentials::new(std::path::Path::new("config.json")).unwrap();
    let gc = gfycat::Api::from_credentials(&api).await.unwrap();

    let a = gc.email_verified().await;
    // let a =  gc.user_exists("@sypher0115").await;
    dbg! {a};
}

use gfycat;

use serde::{Deserialize, Serialize};
use tokio;

#[derive(Deserialize, Debug)]
struct ApiInfo {
    #[serde(rename = "id")]
    client_id: String,
    #[serde(rename = "secret")]
    client_secret: String,
}
impl Default for ApiInfo {
    fn default() -> Self {
        let read = std::fs::File::open("config.json").unwrap();
        let read = serde_json::from_reader(read);
        if let Err(e) = read {
            dbg! {e};
            panic! {"err"}
        } else {
            read.unwrap()
        }
    }
}

#[tokio::main]
async fn main() {
    let api = ApiInfo::default();
    let gc = gfycat::Api::new(api.client_id, api.client_secret).await;

    dbg! {gc};
}

use gfycat;

use serde::{Deserialize, Serialize};
use async_std;

#[derive(Deserialize, Debug)]
struct ApiInfo {
    #[serde(rename="id")]
    client_id: String,
    #[serde(rename="secret")]
    client_secret: String
}
impl Default for ApiInfo{
    fn default() -> Self {
        let read = std::fs::File::open("config.json").unwrap();
        let read = serde_json::from_reader(read);
        if let Err(e) = read {
            dbg!{e};
            panic!{"err"}
        }
        else{
            read.unwrap()
        }
    }
}

async fn async_main() {
    let api = ApiInfo::default();
    let gc = gfycat::Api::new(api.client_id, api.client_secret).await;
    
    dbg!{gc};

}

fn  main() {
async_std::task::block_on(async_main());
    
}
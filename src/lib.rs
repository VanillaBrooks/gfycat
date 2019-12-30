pub mod error;
use tokio;

use serde::Deserialize;
use serde_json;
use std::time;

const ENDPOINT: &str = "https://api.gfycat.com/v1/";
type ClientType = reqwest::Client;
type ApiResult<T> = Result<T, error::ApiError>;

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
            token: "Bearer ".to_owned() + &self.access_token,
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
    // creds: &'a LoadCredentials
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
    pub async fn new(client_id: &str, client_secret: &str) -> Result<Api, error::AuthError> {
        let client = reqwest::Client::new();

        let form = serde_json::json! {
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

    pub async fn from_credentials(credentials: &LoadCredentials) -> Result<Api, error::AuthError> {
        Self::new(&credentials.client_id, &credentials.client_secret).await
    }

    /// Check to see if the OAuth2 autorization needs to be refreshed.
    /// Usually the tokens must be refreshed every hour
    fn need_reauthoirze(&self) -> bool {
        self.expiration > time::Instant::now()
    }

    /// Reauthorize the tokens with your provided credentials
    fn reauthorize(&mut self) -> Result<(), error::AuthError> {
        unimplemented! {}
    }

    /// Checks if username exists. `username` should be prefixed with an "@"
    pub async fn user_exists(&self, username: &str) -> Result<bool, error::ApiError> {
        let endpoint = ENDPOINT.to_owned() + "users/" + username;

        let response = self
            .client
            .get(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(false), // username not available
            404 => Ok(true),  // username available
            401 => Err(error::ApiError::Unauthorized),
            422 => Err(error::ApiError::InvalidValue),
            _ => Err(error::ApiError::Unknown),
        }
    }

    // FIXME not sure how to go about this authr
    pub async fn email_verified(&self) -> ApiResult<bool> {
        // let endpoint = concat!{ENDPOINT, "/users/", username};
        let endpoint = ENDPOINT.to_owned() + "me/email_verified";
        dbg! {&endpoint};

        let response = self
            .client
            .get(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?;

        match response.status().as_u16() {
            404 => Ok(false),
            200 => Ok(true),
            401 => Err(error::ApiError::Unauthorized),
            _ => Err(error::ApiError::Unknown),
        }
    }

    /// Send a verification email to the user.
    // FIXME: this returns 500 which is not covered in the docs
    pub async fn send_email_verification(&self) -> ApiResult<()> {
        let endpoint = ENDPOINT.to_owned() + "me/send_verification_email";

        let response = self
            .client
            .post(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?;

        dbg! {response.status()};

        match response.status().as_u16() {
            400 => Err(error::ApiError::Unknown),
            404 => Err(error::ApiError::MissingEmail),
            401 => Err(error::ApiError::Unauthorized),
            _ => Err(error::ApiError::Unknown),
        }
    }

    pub async fn reset_password(&self, email: &str) -> ApiResult<()> {
        let endpoint = ENDPOINT.to_owned() + "users/";

        let json = serde_json::json! {
            {
                "value": email,
                "action": "send_password_reset_email"
            }
        };

        let response = self
            .client
            .patch(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?;

        dbg! {response.status()};

        match response.status().as_u16() {
            404 => Err(error::ApiError::InvalidValue),
            400 => Err(error::ApiError::InvalidValue),
            422 => Err(error::ApiError::MissingEmail),
            _ => Err(error::ApiError::Unknown),
        }
    }

    /// Get all user details based on the user's id
    pub async fn user_details(&self, user_id: u64) -> ApiResult<User> {
        let endpoint = ENDPOINT.to_owned() + "users/" + &user_id.to_string();

        let response = self
            .client
            .get(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?
            .json::<User>()
            .await?;

        Ok(response)
    }

    /// Get authenticated user details
    pub async fn self_details(&self) -> ApiResult<SelfUser> {
        let endpoint = ENDPOINT.to_owned() + "me";

        dbg! {&endpoint};

        let mut response = self
            .client
            .get(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?
            .json::<SelfUser>()
            .await?;

        Ok(response)
    }

    pub async fn update_details(&self, operations: UpdateOperations) -> ApiResult<()> {
        unimplemented! {}
    }

    pub async fn profile_image(&self, bytes: &[u8]) -> ApiResult<()> {
        unimplemented! {}
    }

    pub async fn create_account(&self, info: CreateUser) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn follow_user(&self, username: &str) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn unfollow_user(&self, username: &str) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn check_following(&self, username: &str) -> ApiResult<bool> {
        unimplemented! {}
    }
    pub async fn list_following(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }
    pub async fn list_followers(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }

    //
    // User feeds
    //
    pub async fn published(&self, user_id: u64) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }
    pub async fn private_feed(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }
    pub async fn timeline(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }

    //
    // User Folders
    //

    pub async fn all_folders(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
        // all other methods will be done via methods on the object
    }

    //
    // Bookmarks
    //

    pub async fn bookmark_folders(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
        // all other methods will be done via methods on the object
    }
    pub async fn bookmark_folders_id(&self, bookmark_id: u64) -> ApiResult<Vec<String>> {
        unimplemented! {}
        // missing features are methods on objects
    }

    //
    // Albums
    //

    pub async fn self_albums(&self) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }
    pub async fn get_album_contents(&self, user_id: u64, album_id: u64) -> ApiResult<Vec<String>> {
        unimplemented! {}
    }
    pub async fn albums_by_link(&self, user_id: u64, link: &str) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn self_album_id(&self, user_id: u64, album_id: u64) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn create_album(&self, user_id: u64, album_id: u64) -> ApiResult<()> {
        unimplemented! {}
    }
    pub async fn move_album_to_folder(&self, user_id: u64, album_id: u64) -> ApiResult<()> {
        unimplemented! {}
    }

    // skipped some

    //
    // Getting gfycats
    //

    pub async fn info(&self, gfy_id: &str) -> ApiResult<GfyItem> {
        let endpoint = ENDPOINT.to_owned() + "gfycats/" + &gfy_id.to_string();

        let response = self
            .client
            .get(&endpoint)
            .header("Autorization", &self.token)
            .send()
            .await?
            .json::<GfycatInfo>()
            .await?;

        Ok(response.gfy_item)
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct GfycatInfo {
    #[serde(rename = "gfyItem")]
    gfy_item: GfyItem,
}

#[derive(Deserialize, Debug, Default)]
pub struct GfyItem {
    #[serde(rename = "gfyId")]
    pub gfy_id: String,
    #[serde(rename = "gfyName")]
    pub gfy_name: String,
    #[serde(rename = "gfyNumber")]
    pub gfy_number: String,
    #[serde(rename = "webmUrl")]
    pub webm_url: String,
    #[serde(rename = "gifUrl")]
    pub gif_url: String,
    #[serde(rename = "mobileUrl")]
    pub mobile_url: String,
    #[serde(rename = "mobilePosterUrl")]
    pub mobile_poster_url: String,
    #[serde(rename = "miniUrl")]
    pub mini_url: String,
    #[serde(rename = "posterUrl")]
    pub poster_url: String,
    #[serde(rename = "thumb100PosterUrl")]
    pub thumb_100_poster_url: String,
    #[serde(rename = "max5mbGif")]
    pub five_mb_gif: String,
    #[serde(rename = "max2mbGif")]
    pub two_mb_gif: String,
    #[serde(rename = "max1mbGif")]
    pub one_mb_gif: String,
    #[serde(rename = "gif100px")]
    pub _100px_gif: String,
    pub width: u64,
    pub height: u64,
    #[serde(rename = "avgColor")]
    pub avg_color: String,
    #[serde(rename = "frameRate")]
    pub fame_rate: f64,
    #[serde(rename = "numFrames")]
    pub num_frames: f64,
    #[serde(rename = "mp4Size")]
    pub mp4_size: u32,
    #[serde(rename = "webmSize")]
    pub webm_size: u32,
    #[serde(rename = "gifSize")]
    pub gif_size: Option<u32>,
    pub source: u32,
    #[serde(rename = "createDate")]
    pub create_date: u32,
    pub nsfw: String,
    #[serde(rename = "mp4Url")]
    pub mp4_url: String,
    pub likes: String,
    pub published: u32,
    pub dislikes: String,
    #[serde(rename = "extraLemmas")]
    extra_lemmas: String,
    pub md5: Option<String>,
    pub views: u32,
    pub tags: Vec<String>,
    #[serde(rename = "userName")]
    pub username: String,
    pub title: String,
    description: String,
    #[serde(rename = "languageText")]
    pub language_text: String,
    #[serde(rename = "languageCategories")]
    pub language_categories: Option<Vec<String>>,
    pub subreddit: Option<String>,
    #[serde(rename = "redditId")]
    pub reddit_id: Option<String>,
    #[serde(rename = "redditIdText")]
    pub reddit_id_text: Option<String>,
    #[serde(rename = "domainWhitelist")]
    pub domain_whitelist: Vec<String>,
}

// gth": "3153",
//         "connection": "keep-alive",
//     },
// }
// [src\lib.rs:329] response.text().await = Ok(
//     "{\"gfyItem\":{\"tags\":[\"new york\",\"timelapse\"],\"languageCategories\":[\"new york\",\"
//     timelapse\"],\"domainWhitelist\":[],\"geoWhitelist\":[],\"published\":1,\"nsfw\":\"0\",\"gat
//      ekeeper\":0,\"mp4Url\":\"https://giant.gfycat.com/AccomplishedFondKingsnake.mp4\",\"gifUrl\
//      ":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-size_restricted.gif\",\"webmUrl\":\"https://giant.gfycat.com/AccomplishedFondKingsnake.webm\",\"webpUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake.webp\",\"mobileUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.mp4\",\"mobilePosterUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.jpg\",\"extraLemmas\":\"\",\"thumb100PosterUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.jpg\",\"miniUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.mp4\",\"gif100px\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-max-1mb.gif\",\"miniPosterUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.jpg\",\"max5mbGif\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-size_restricted.gif\",\"title\":\"NYC Timelapse\",\"max2mbGif\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-small.gif\",\"max1mbGif\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-max-1mb.gif\",\"posterUrl\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-poster.jpg\",\"languageText\":\"\",\"views\":25705,\"userName\":\"egster\",\"description\":\"\",\"hasTransparency\":false,\"hasAudio\":false,\"likes\":\"1\",\"dislikes\":\"0\",\"gfyNumber\":\"6742951\",\"gfyId\":\"accomplishedfondkingsnake\",\"gfyName\":\"AccomplishedFondKingsnake\",\"avgColor\":\"#252A28\",\"rating\":\"G\",\"gfySlug\":\"new-york\",\"width\":1920,\"height\":1080,\"frameRate\":30.03003,\"numFrames\":200.0,\"mp4Size\":14234971,\"webmSize\":2329134,\"createDate\":1561075293,\"source\":1,\"content_urls\":{\"max2mbGif\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-small.gif\",\"size\":1817625,\"height\":169,\"width\":300},\"webp\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake.webp\",\"size\":1289824,\"height\":0,\"width\":0},\"max1mbGif\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-max-1mb.gif\",\"size\":910232,\"height\":158,\"width\":280},\"100pxGif\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-max-1mb.gif\",\"size\":910232,\"height\":158,\"width\":280},\"mobilePoster\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.jpg\",\"size\":28159,\"height\":360,\"width\":640},\"mp4\":{\"url\":\"https://giant.gfycat.com/AccomplishedFondKingsnake.mp4\",\"size\":14234971,\"height\":1080,\"width\":1920},\"webm\":{\"url\":\"https://giant.gfycat.com/AccomplishedFondKingsnake.webm\",\"size\":2329134,\"height\":1080,\"width\":1920},\"max5mbGif\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-size_restricted.gif\",\"size\":4234962,\"height\":250,\"width\":444},\"largeGif\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-size_restricted.gif\",\"size\":4234962,\"height\":250,\"width\":444},\"mobile\":{\"url\":\"https://thumbs.gfycat.com/AccomplishedFondKingsnake-mobile.mp4\",\"size\":620154,\"height\":360,\"width\":640}},\"userData\":{\"name\":\"\",\"profileImageUrl\":\"\",\"url\":\"https://gfycat.com/@egster\",\"username\":\"egster\",\"followers\":1,\"subscription\":0,\"following\":0,\"profileUrl\":\"\",\"views\":25670,\"verified\":false}}}",
// )

// C:\Users\Daniel\github\gfycat>
pub struct CreateUser;
pub struct UpdateOperations;

/// helper struct for loading credentials from json
#[derive(Deserialize, Debug)]
pub struct LoadCredentials {
    #[serde(rename = "id")]
    pub client_id: String,
    #[serde(rename = "secret")]
    pub client_secret: String,
}
impl LoadCredentials {
    pub fn new(path: &std::path::Path) -> Result<Self, error::AuthError> {
        let read = std::fs::File::open(path)?;
        let json = serde_json::from_reader(read)?;
        Ok(json)
    }
}

/// Information returend by Api.user_details()
#[derive(Debug, Deserialize, Default)]
pub struct User {
    userid: u64,
    username: String,
    description: String,
    #[serde(rename = "profileUrl")]
    profile_url: String,
    name: String,
    views: u64,
    email_verified: bool,
    url: String,
    #[serde(rename = "createDate")]
    create_date: u32,
    #[serde(rename = "profileImageUrl")]
    profile_image_url: String,
    verified: bool,
    followers: u32,
    following: u32,
}

/// Information returend by Api.user_details()
#[derive(Debug, Deserialize, Default)]
pub struct SelfUser {
    userid: u64,
    username: String,
    description: String,
    #[serde(rename = "profileUrl")]
    profile_url: String,
    name: String,
    views: u64,
    email_verified: bool,
    url: String,
    #[serde(rename = "createDate")]
    create_date: u32,
    #[serde(rename = "profileImageUrl")]
    profile_image_url: String,
    verified: bool,
    followers: u32,
    following: u32,
    #[serde(rename = "geoWhitelist")]
    geo_whitelist: String,
    #[serde(rename = "domainWhitelist")]
    domain_whitelist: String,
    #[serde(rename = "associatedProviders")]
    associated_providers: String,
    #[serde(rename = "iframeProfileImageVisible")]
    iframe_profile_Image_visible: String,
}

#[allow(dead_code)]
fn init_test() -> (tokio::runtime::Runtime, Api) {
    let tk = tokio::runtime::Runtime::new().unwrap();
    let cred = LoadCredentials::new(std::path::Path::new("config.json")).unwrap();
    let api = tk.block_on(Api::from_credentials(&cred)).unwrap();
    (tk, api)
}

// #[test]
// fn email_verified() {
//     let (tk, api) = init_test();
//     let left = tk
//         .block_on(api.email_verified())
//         .expect("could not call api");
//     assert_eq! {left,  true};
// }

// #[test]
// fn user_exists() {
//     let (tk, api) = init_test();
//     let left = tk
//         .block_on(api.user_exists("@sypher0115"))
//         .expect("could not call api");
//     assert_eq! {left,  true};
// }

// #[test]
// fn user_exists_false() {
//     let (tk, api) = init_test();
//     // not prefixed by @, will fail
//     let left = tk
//         .block_on(api.user_exists("sypher0115"))
//         .expect("could not call api");
//     assert_eq! {left,  false};
// }

// #[test]
// fn send_email_verification() {
//     let (tk, api) = init_test();
//     // not prefixed by @, will fail
//     let left = tk
//         .block_on(api.send_email_verification());
//     dbg!{&left};
//     assert!{left.is_ok()};
// }

// #[test]
// fn reset_password() {
//     let (tk, api) = init_test();
//     // not prefixed by @, will fail
//     let left = tk.block_on(api.reset_password("brooks@karlik.org"));
//     dbg! {&left};
//     assert! {left.is_ok()};
// }

// // TODO: write this test
// #[test]
// fn user_details() {
//     let (tk, api) = init_test();
//     // not prefixed by @, will fail
//     let left = tk.block_on(api.reset_password("brooks@karlik.org"));
//     dbg! {&left};
//     assert! {left.is_ok()};

// }

// #[test]
// fn self_details() {
//     let (tk, api) = init_test();
//     // not prefixed by @, will fail
//     let left = tk.block_on(api.self_details());
//     dbg! {&left};
//     assert! {left.is_ok()};
// }


#[test]
fn info_1() {
    let (tk, api) = init_test();
    // not prefixed by @, will fail
    let left = tk.block_on(api.info("cleartatteredbunny"));
    dbg! {&left};
    assert! {left.is_ok()};
}
#[test]
fn info_2() {
    let (tk, api) = init_test();
    // not prefixed by @, will fail
    let left = tk.block_on(api.info("paltryfrigidhalibut"));
    dbg! {&left};
    assert! {left.is_ok()};
}
#[test]
fn info_3() {
    let (tk, api) = init_test();
    // not prefixed by @, will fail
    let left = tk.block_on(api.info("exemplarytaneasteuropeanshepherd"));
    dbg! {&left};
    assert! {left.is_ok()};
}
#[test]
fn info_4() {
    let (tk, api) = init_test();
    // not prefixed by @, will fail
    let left = tk.block_on(api.info("denseslimafricanclawedfrog"));
    dbg! {&left};
    assert! {left.is_ok()};
}

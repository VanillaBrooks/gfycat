use reqwest;

macro_rules! from {
    ($root:path, $destination_enum:ident :: $path_:ident) => {
        impl From<$root> for $destination_enum {
            fn from(e: $root) -> Self {
                $destination_enum::$path_(e)
            }
        }
    };
}

#[derive(Debug)]
pub enum AuthError {
    Request(reqwest::Error),
    SerdeJson(serde_json::Error),
    IoError(std::io::Error),
    Expiration,
}

#[derive(Debug)]
pub enum ApiError {
    Request(reqwest::Error),
    SerdeJson(serde_json::Error),
    IoError(std::io::Error),
    InvalidValue,
    Unauthorized,
    Unknown,
    MissingEmail,
}
from! {reqwest::Error, AuthError::Request}
from! {serde_json::Error, AuthError::SerdeJson}
from! {std::io::Error, AuthError::IoError}

from! {reqwest::Error, ApiError::Request}
from! {serde_json::Error, ApiError::SerdeJson}
from! {std::io::Error, ApiError::IoError}

use surf;

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
pub enum AuthError{
    Request(surf::Exception),
    SerdeJson(serde_json::Error),
    IoError(std::io::Error),
    Expiration
}

from!{surf::Exception, AuthError::Request}
from!{serde_json::Error, AuthError::SerdeJson}
from!{std::io::Error, AuthError::IoError}

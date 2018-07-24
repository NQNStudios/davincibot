extern crate std;

extern crate rusqlite;
extern crate serde_json;

#[derive(Debug)]
pub enum Error {
    Rusqlite(rusqlite::Error),
    SerdeJson(serde_json::Error),
    DaVinci(String),
    ParseInt(std::num::ParseIntError),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        self::Error::Rusqlite(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        self::Error::SerdeJson(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        self::Error::ParseInt(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

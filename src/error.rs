extern crate std;
extern crate rusqlite;
extern crate serde_json;
extern crate edit_rs;

#[derive(Debug)]
pub enum Error {
    Rusqlite(rusqlite::Error),
    SerdeJson(serde_json::Error),
    DaVinci(String),
    ParseInt(std::num::ParseIntError),
    EditRS(edit_rs::Error),
    Utf8(std::str::Utf8Error),
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

impl From<edit_rs::Error> for Error {
    fn from(e: edit_rs::Error) -> Self {
        self::Error::EditRS(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        self::Error::Utf8(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

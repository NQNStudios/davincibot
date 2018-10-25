extern crate std;
extern crate rusqlite;
extern crate yaml_rust;
extern crate edit_rs;

#[derive(Debug)]
pub enum Error {
    Rusqlite(rusqlite::Error),
    DaVinci(String),
    ParseInt(std::num::ParseIntError),
    EditRS(edit_rs::Error),
    Utf8(std::str::Utf8Error),
    Yaml(yaml_rust::ScanError),
    IO(std::io::Error),
    None(std::option::NoneError),
}

impl From<std::option::NoneError> for Error {
    fn from(e: std::option::NoneError) -> Self {
        self::Error::None(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        self::Error::IO(e)
    }
}

impl From<yaml_rust::ScanError> for Error {
    fn from(e: yaml_rust::ScanError) -> Self {
        self::Error::Yaml(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        self::Error::Rusqlite(e)
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

extern crate rusqlite;

#[derive(Debug)]
pub enum DVError {
    Rusqlite(rusqlite::Error),
}

impl From<rusqlite::Error> for DVError {
    fn from(e: rusqlite::Error) -> Self {
        DVError::Rusqlite(e)
    }
}

pub type DVResult<T> = Result<T, DVError>;

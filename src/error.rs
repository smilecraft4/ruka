//! Ruka errors

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Ruka Generic error: {0}")]
    Generic(String),
}

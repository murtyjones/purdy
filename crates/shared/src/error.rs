use thiserror::Error;

#[derive(Error, Debug)]
pub enum NumberError {
    // TODO: Add more context
    #[error("invalid number conversion")]
    InvalidNumberConversion,
}
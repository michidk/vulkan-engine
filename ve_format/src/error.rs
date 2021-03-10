use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, FormatError>;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Serialization Error: {0}")]
    SerializationError(#[from] Box<bincode::ErrorKind>),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

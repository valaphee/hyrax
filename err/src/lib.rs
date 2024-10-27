#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not implemented")]
    Unimplemented,
    #[error("IO: {0}")]
    Io(IoError),
}

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("Read-only")]
    ReadOnly,
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not implemented")]
    Unimplemented,
    #[error("IO: {0}")]
    Io(IoError),
    #[error("FS: {0}")]
    Fs(FsError),
}

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("Read-only")]
    ReadOnly,
}

#[derive(thiserror::Error, Debug)]
pub enum FsError {
    #[error("Inconsistent")]
    Inconsistent,
    #[error("Index")]
    Index,
}

pub type Result<T> = core::result::Result<T, Error>;

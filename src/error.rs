use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Object Store Error")]
    Os(#[from] object_store::Error),
    #[error("{0}")]
    InvalidConfiguration(String),
    #[error("unknown error")]
    Unknown,
}

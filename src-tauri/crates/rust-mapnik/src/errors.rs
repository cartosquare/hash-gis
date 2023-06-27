use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapnikError {
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    StdError(#[from] std::io::Error),
}

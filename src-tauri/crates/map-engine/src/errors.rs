//! Library errors (using [`thiserror`]).
use gdal::errors::GdalError;
use ndarray::ShapeError;
// use rust_mapnik::errors::MapnikError;
use png::EncodingError;
use std::num::ParseIntError;
// use quick_xml::de::DeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapEngineError {
    #[error("TileError: {0}")]
    TileError(String),
    #[error("AffineError: {0}")]
    AffineError(String),
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    EncodingError(#[from] EncodingError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    GdalError(#[from] GdalError),
    #[error(transparent)]
    ShapeError(#[from] ShapeError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    // #[error(transparent)]
    // MapnikError(#[from] MapnikError),
    // #[error(transparent)]
    // DeError(#[from] DeError),
}

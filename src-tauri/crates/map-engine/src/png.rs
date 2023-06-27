//!Empty PNG image.
use crate::errors::MapEngineError;
use crate::raster::{pixels::driver::Driver, StyledPixels};
use crate::tiles::TILE_SIZE;
use lazy_static::lazy_static;
use ndarray::{Array, Array3};

/// Fully-transparent tile served when the requested tile does not intersect the map extent
pub fn empty_png() -> Result<Vec<u8>, MapEngineError> {
    let arr: Array3<u8> = Array::zeros((4, TILE_SIZE, TILE_SIZE));
    StyledPixels::new(arr, Driver::Generic).into_png()
}

lazy_static! {
    pub static ref EMPTY_PNG: Vec<u8> = empty_png().unwrap();
}

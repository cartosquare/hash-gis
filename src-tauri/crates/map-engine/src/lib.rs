/*!A library to work with tiled geospatial (raster) data.

This is the base library of a series of crates. It contains methods to:

* Work with XYZ tiles
* Read raster data intersecting a tile
* Style data to generate coloured PNG files

🚧 This is a work in progress so the API is unstable and likely to change as the related crates are
developed. 🚧

## Related crates

* [map-engine-server](<https://crates.io/crates/map-engine-server>): An HTTP tile server

## Example

```
use map_engine::{
    errors::MapEngineError,
    cmap::{Composite, viridis},
    raster::{Raster, RawPixels},
    tiles::Tile,
};
use std::path::PathBuf;

fn main() -> Result<(), MapEngineError> {
    let tile = Tile::new(304, 624, 10);
    let raster = Raster::new(PathBuf::from("src/tests/data/chile_optimised.tif"))?;

    // Note that these three things should support the same number of bands (in this case, 1).
    // At the moment we don't enforce this at compile time.
    // https://gitlab.com/spadarian/map_engine/-/issues/36
    let comp = Composite::new_gradient(0.0, 27412.0, &viridis);
    let bands = &[1];
    let na_value = vec![0.0];
    assert!(comp.n_bands() == bands.len());
    assert!(comp.n_bands() == na_value.len());

    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(bands), None)?;
    let styled = arr.style(comp, na_value)?;
    let png = styled.into_png()?;  // PNG as a byte sequence

    // The PNG can then be sent as an HTTP response, written to disk, etc.
    // Also, we have a method to write to disk:
    // styled.write_to_disk(std::path::Path::new("path_to_png"))?;

    Ok(())
}
```

## Intallation notes

At the moment, we only support Linux but the crate might work on macOS and Windows (let us know!).

We depend on [GDAL](http://gdal.org/) `>3.3` and you might need to [compile it yourself](https://gdal.org/download.html#development-source).

### Ubuntu

The [UbuntuGIS](https://launchpad.net/~ubuntugis/+archive/ubuntu/ppa) team has a recent version of GDAL (3.3.2) that might work for you.
*/
extern crate lazy_static;

#[cfg(test)]
mod tests;

pub mod affine;
pub mod cmap;
pub mod colour;
pub mod errors;
pub mod gdal;
pub mod mercator;
pub mod png;
pub mod raster;
pub mod vector;
pub mod tiles;
pub mod windows;

/// Maximum zoom level supported
pub const MAXZOOMLEVEL: u32 = 32;

/// Available tile formats to request.
///
/// At the moment, only PNG8 tiles are supported.
pub const SUPPORTED_FORMATS: &[&str] = &["png"];

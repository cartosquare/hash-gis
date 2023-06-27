# map-engine

[![Gitlab pipeline status](https://gitlab.com/spadarian/map_engine/badges/master/pipeline.svg)](https://gitlab.com/spadarian/map-engine)
[![Gitlab code coverage](https://gitlab.com/spadarian/map_engine/badges/master/coverage.svg)](https://gitlab.com/spadarian/map-engine)
[![Crates.io](https://img.shields.io/crates/v/map-engine.svg)](https://crates.io/crates/map-engine)
[![Documentation](https://docs.rs/map-engine/badge.svg)](https://docs.rs/map-engine)

A library to work with tiled geospatial (raster) data.

This is the base library of a series of crates. It contains methods to:

* Work with XYZ tiles
* Read raster data intersecting a tile
* Style data to generate coloured PNG files

🚧 This is a work in progress so the API is unstable and likely to change as the related crates are
developed. 🚧

### Related crates

* [map-engine-server](<https://crates.io/crates/map-engine-server>): An HTTP tile server

### Example

```rust
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

### Intallation notes

At the moment, we only support Linux but the crate might work on macOS and Windows (let us know!).

We depend on [GDAL](http://gdal.org/) `>3.3` and you might need to [compile it yourself](https://gdal.org/download.html#development-source).

#### Ubuntu

The [UbuntuGIS](https://launchpad.net/~ubuntugis/+archive/ubuntu/ppa) team has a recent version of GDAL (3.3.2) that might work for you.

//!Re-exports from [`gdal`] crate.
pub use gdal::Dataset;
pub use gdal::spatial_ref;
pub use gdal::vector::LayerAccess;
pub use gdal::raster::ResampleAlg;
pub use gdal::config;
pub use std::env;

pub fn gdal_initialize(gdal_data: String) {
    config::set_config_option("GDAL_DATA", &gdal_data).unwrap();
    env::set_var("PROJ_LIB", format!("{}/proj/data", gdal_data));
}
use super::style::Style;
use map_engine::{affine::GeoTransform, cmap::Composite, windows::Window, raster::SpatialInfo};
use serde::{Deserialize, Serialize};

/// Configurable setting for individual maps.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapSettings {
    /// Valid extent. If not provided, it defaults to the total image extent.
    pub extent: Option<Window>,
    /// Path to raster file.
    pub path: String,
    /// Name of the map. It is used as part of the URL to request tiles.
    pub name: String,
    // TODO: Make this private
    /// Raster GeoTransform. This is automatically, generated.
    pub geotransform: Option<GeoTransform>,
    /// Pixel value to make transparent.
    pub no_data_value: Option<Vec<f64>>,
    /// Style definition
    pub style: Option<Style>,
    /// Style definition for vector data
    pub xml: Option<String>,
    /// GDAL driver used to open the file
    pub driver_name: Option<String>,
    /// Spatial reference system
    //pub spatial_ref_code: Option<i32>,
    pub spatial_info: Option<SpatialInfo>,
    /// Spatial units
    pub spatial_units: Option<String>,
    /// wgs84 bounds
    pub bounds: Option<[f64; 4]>,
    /// Has overview or not
    pub has_overview: Option<bool>,
}

impl MapSettings {
    pub fn get_bands(&self) -> &Vec<isize> {
        self.style
            .as_ref()
            .expect("Style not available in MapSettings")
            .bands
            .as_ref()
            .expect("Bands not available in Style")
    }

    pub fn get_no_data_values(&self) -> &Vec<f64> {
        self.no_data_value
            .as_ref()
            .expect("no_data_values not available in MapSettings")
    }

    pub fn to_composite(&self) -> Composite {
        self.style
            .as_ref()
            .expect("Style not available in MapSettings")
            .into()
    }
}

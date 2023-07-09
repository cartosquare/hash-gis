//! Types and helpers to work with XYZ tiles.
use crate::{
    affine::GeoTransform, errors::MapEngineError, raster::Raster, windows::Window, MAXZOOMLEVEL,
    SUPPORTED_FORMATS,
};
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use std::cmp;
use std::f64::consts::PI;

const RE: f64 = 6378137.0;
const EPSILON: f64 = 1e-14;
// const LL_EPSILON: f64 = 1e-11;
/// Size of the Tile in pixels
pub const TILE_SIZE: usize = 256;

/// An XYZ web mercator tile
#[derive(Debug, PartialEq)]
pub struct Tile {
    /// Column index
    pub x: u32,
    /// Row index
    pub y: u32,
    /// Zoom level
    pub z: u32,
    /// Image extension
    ext: Option<String>,
}

impl Tile {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            x,
            y,
            z,
            ext: Some("png".to_string()),
        }
    }

    pub fn set_extension(&mut self, ext: &str) -> Result<(), MapEngineError> {
        if !SUPPORTED_FORMATS.contains(&ext) {
            return Err(MapEngineError::Msg(format!(
                "The extension {:?} is not yet supported",
                ext
            )));
        }
        self.ext = Some(ext.to_string());
        Ok(())
    }

    pub fn to_tuple(&self) -> (u32, u32, u32) {
        (self.x, self.y, self.z)
    }

    /// Return the coordinates (lat, long) of the upper-left tile corner
    pub fn ul(&self) -> (f64, f64) {
        let (xtile, ytile, zoom) = self.to_tuple();
        let z2: f64 = 2u32.pow(zoom).into();
        let lon_deg = (xtile as f64) / z2 * 360.0 - 180.0;
        let lat_rad = (PI * (1.0 - 2.0 * (ytile as f64) / z2)).sinh().atan();
        let lat_deg = lat_rad.to_degrees();
        (lon_deg, lat_deg)
    }

    /// Return the coordinates (mercator x, y) of the upper-left tile corner
    pub fn ul_xy(&self) -> (f64, f64) {
        let (lon_deg, lat_deg) = self.ul();
        xy(lon_deg, lat_deg)
    }

    /// Return the bounds (lat, lng) of the tile
    ///
    /// The order of the output is (min_lng_deg, max_lat_deg, max_lng_deg, min_lat_deg)
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let (xtile, ytile, zoom) = self.to_tuple();
        let z2: f64 = 2u32.pow(zoom).into();

        let min_lng_deg = (xtile as f64) / z2 * 360.0 - 180.0;
        let max_lat_rad = (PI * (1.0 - 2.0 * (ytile as f64) / z2)).sinh().atan();
        let max_lat_deg = max_lat_rad.to_degrees();

        let max_lng_deg = ((xtile + 1) as f64) / z2 * 360.0 - 180.0;
        let min_lat_rad = (PI * (1.0 - 2.0 * ((ytile + 1) as f64) / z2)).sinh().atan();
        let min_lat_deg = min_lat_rad.to_degrees();
        (min_lng_deg, max_lat_deg, max_lng_deg, min_lat_deg)
    }

    /// Return the bounds (mercator x, y) of the tile
    ///
    /// The order of the output is (min_lng_deg, max_lat_deg, max_lng_deg, min_lat_deg)
    pub fn bounds_xy(&self) -> (f64, f64, f64, f64) {
        let (min_lng_deg, max_lat_deg, max_lng_deg, min_lat_deg) = self.bounds();
        let (min_x, max_y) = xy(min_lng_deg, max_lat_deg);
        let (max_x, min_y) = xy(max_lng_deg, min_lat_deg);
        (min_x, max_y, max_x, min_y)
    }

    /// Return the vertices of the tile
    ///
    /// The order of the vertices is: upper-left, lower-left, lower-right and upper-right
    pub fn vertices(&self) -> [(f64, f64); 4] {
        let (min_x, max_y, max_x, min_y) = self.bounds();
        [
            (min_x, max_y), // Upper-left
            (max_x, max_y), // Upper-right
            (max_x, min_y), // Lower-right
            (min_x, min_y), // Lower-left
        ]
    }

    /// Return a tile from a lower zoom level that contains this tile
    pub fn zoom_out(&self, zoom: Option<u32>) -> Option<Self> {
        if self.z == 0 {
            return None;
        };
        let target_zoom = zoom.unwrap_or(self.z - 1);
        let mut return_tile = Tile::new(self.x, self.y, self.z);
        while return_tile.z > target_zoom {
            let (xtile, ytile, ztile) = (return_tile.x, return_tile.y, return_tile.z);
            let newx = if xtile % 2 == 0 {
                xtile / 2
            } else {
                (xtile - 1) / 2
            };
            let newy = if ytile % 2 == 0 {
                ytile / 2
            } else {
                (ytile - 1) / 2
            };
            let newz = ztile - 1;
            return_tile = Tile::new(newx, newy, newz);
        }
        Some(return_tile)
    }

    /// Return the tiles from a higher zoom level contained by this tile
    pub fn zoom_in(&self, zoom: Option<u32>) -> Option<Vec<Self>> {
        if self.z == MAXZOOMLEVEL {
            return None;
        }
        let target_zoom = zoom.unwrap_or(self.z + 1);
        let mut tiles = vec![Tile::new(self.x, self.y, self.z)];
        while tiles[0].z < target_zoom {
            tiles = tiles
                .iter()
                .map(|tile| {
                    [
                        Tile::new(tile.x * 2, tile.y * 2, tile.z + 1),
                        Tile::new(tile.x * 2 + 1, tile.y * 2, tile.z + 1),
                        Tile::new(tile.x * 2 + 1, tile.y * 2 + 1, tile.z + 1),
                        Tile::new(tile.x * 2, tile.y * 2 + 1, tile.z + 1),
                    ]
                })
                .flatten()
                .collect()
        }
        Some(tiles)
    }

    /// Find the `Tile` intersecting the coordinate at a given zoom level
    pub fn from_lat_lng(lng: f64, lat: f64, zoom: u32) -> Self {
        let (x, y) = _xy(lng, lat);
        let z2 = 2u32.pow(zoom);
        let xtile = if x <= 0.0 {
            0u32
        } else if x >= 1.0 {
            z2 - 1
        } else {
            ((x + EPSILON) * (z2 as f64)).floor() as u32
        };

        let ytile = if y <= 0.0 {
            0u32
        } else if y >= 1.0 {
            z2 - 1
        } else {
            ((y + EPSILON) * (z2 as f64)).floor() as u32
        };

        Self::new(xtile, ytile, zoom)
    }

    // pub fn to_window(&self, geo: &GeoTransform) -> Result<Window, MapEngineError> {
    //     let mercator = GlobalMercator::new(TILE_SIZE);
    //     let res = geo.geo[0];
    //     let orig_zoom = mercator.zoom_for_pixel_size(&res);
    //     let req_overview = 2f64.powf((orig_zoom as f64) - self.z as f64);
    //     let (lng, lat) = self.ul();
    //     let (x, y) = xy(lng, lat);
    //     let (row, col) = geo.rowcol(x + res / 2.0, y - res / 2.0)?;
    //     let tile_size = TILE_SIZE as f64;
    //     let out_size = (tile_size * req_overview).ceil() as usize;
    //     Ok(Window::new(col as isize, row as isize, out_size, out_size))
    // }

    /// Transform a `Tile` to a `Window`.
    ///
    /// # Arguments
    ///
    /// * `raster` - `Raster` used for the conversion.
    pub fn to_window(&self, raster: &Raster) -> Result<(Window, bool), MapEngineError> {
        let geo = raster.geo();
        let spatial_ref = raster.spatial_ref()?;

        let src_spatial_units = spatial_ref
            .linear_units_name()
            .unwrap_or_else(|_| "metre".to_string());

        let wgs84_crs = gdal::spatial_ref::SpatialRef::from_epsg(4326)?;

        let vertex_trans = gdal::spatial_ref::CoordTransform::new(&wgs84_crs, &spatial_ref)?;

        let vertices = self.vertices();
        let mut xs = [vertices[0].0, vertices[1].0, vertices[2].0, vertices[3].0];

        let mut ys = [vertices[0].1, vertices[1].1, vertices[2].1, vertices[3].1];
        let mut zs = [0.0f64; 4];

        // Transform vertices to raster CRS
        vertex_trans.transform_coords(&mut ys, &mut xs, &mut zs)?;

        let offset = get_vertices_offset(self.z, &src_spatial_units)?;

        let row_cols = get_row_cols(&xs, &ys, &offset, geo, &src_spatial_units);

        let is_skewed = (row_cols[0].0 != row_cols[1].0)
            || (row_cols[2].0 != row_cols[3].0 || (row_cols[0].1 != row_cols[3].1))
            || (row_cols[1].1 != row_cols[2].1);

        let win = Window::new(
            cmp::min(row_cols[0].1, row_cols[3].1) as isize,
            cmp::min(row_cols[0].0, row_cols[1].0) as isize,
            cmp::max(row_cols[1].1 - row_cols[0].1, row_cols[2].1 - row_cols[3].1) as usize + 1,
            cmp::max(row_cols[3].0 - row_cols[0].0, row_cols[2].0 - row_cols[1].0) as usize + 1,
        );

        Ok((win, is_skewed))
    }
}

fn get_row_cols(
    xs: &[f64],
    ys: &[f64],
    offset: &[(f64, f64)],
    geo: &GeoTransform,
    src_spatial_units: &str,
) -> Vec<(i32, i32)> {
    xs.iter()
        .zip(ys)
        .zip(offset)
        .map(|((x, y), (x_off, y_off))| {
            if src_spatial_units == "metre" {
                // TODO: Find out why x,y gets shifted to y,x
                geo.rowcol(y + y_off, x + x_off).unwrap()
            } else {
                geo.rowcol(x + x_off, y + y_off).unwrap()
            }
        })
        .collect()
}

fn get_vertices_offset(
    zoom: u32,
    src_spatial_units: &str,
) -> Result<[(f64, f64); 4], MapEngineError> {
    let mercator_crs = SpatialRef::from_epsg(3857)?;
    let wgs84_crs = gdal::spatial_ref::SpatialRef::from_epsg(4326).unwrap();
    let tile_res_trans = CoordTransform::new(&mercator_crs, &wgs84_crs).unwrap();
    let tile_z2: f64 = 2u32.pow(zoom).into();
    let tile_res = (2.0 * PI * RE / TILE_SIZE as f64) / tile_z2;
    let tile_res_m = ([tile_res], [tile_res], [0.0]);
    let mut tile_res_deg = ([tile_res], [tile_res], [0.0]);

    tile_res_trans
        .transform_coords(
            &mut tile_res_deg.0,
            &mut tile_res_deg.1,
            &mut tile_res_deg.2,
        )
        .unwrap();

    // Shift
    let offset_prop = 0.01;
    let offset = if src_spatial_units == "metre" {
        // NOTE: shift tuple
        [
            (
                -tile_res_m.0[0] * offset_prop,
                tile_res_m.1[0] * offset_prop,
            ),
            (
                -tile_res_m.0[0] * offset_prop,
                -tile_res_m.1[0] * offset_prop,
            ),
            (
                tile_res_m.0[0] * offset_prop,
                -tile_res_m.1[0] * offset_prop,
            ),
            (tile_res_m.0[0] * offset_prop, tile_res_m.1[0] * offset_prop),
        ]
    } else {
        [
            (
                tile_res_deg.1[0] * offset_prop,
                -tile_res_deg.0[0] * offset_prop,
            ),
            (
                -tile_res_deg.1[0] * offset_prop,
                -tile_res_deg.0[0] * offset_prop,
            ),
            (
                -tile_res_deg.1[0] * offset_prop,
                tile_res_deg.0[0] * offset_prop,
            ),
            (
                tile_res_deg.1[0] * offset_prop,
                tile_res_deg.0[0] * offset_prop,
            ),
        ]
    };
    Ok(offset)
}

fn xy(lng: f64, lat: f64) -> (f64, f64) {
    let x = RE * lng.to_radians();
    let y = if lat <= -90.0 {
        std::f64::NEG_INFINITY
    } else if lat >= 90.0 {
        std::f64::INFINITY
    } else {
        RE * (PI * 0.25 + lat.to_radians() * 0.5).tan().ln()
    };
    (x, y)
}

fn _xy(lng: f64, lat: f64) -> (f64, f64) {
    let x = lng / 360.0 + 0.5;
    let sinlat = lat.to_radians().sin();
    let y = 0.5 - 0.25 * (sinlat + 1.0).ln() / (1.0 - sinlat) / std::f64::consts::PI;
    (x, y)
}

// fn tiles(west: f64, south: f64, east: f64, north: f64, zoom: u32) -> impl Iterator<Item = Tile> {
//     let bboxes = if west > east {
//         let bbox_west = (-180.0, south, east, north);
//         let bbox_east = (west, south, 180.0, north);
//         vec![bbox_west, bbox_east]
//     } else {
//         vec![(west, south, east, north)]
//     };

//     bboxes
//         .iter()
//         .map(move |(mut w, mut s, mut e, mut n)| {
//             w = f64::max(-180.0, w);
//             s = f64::max(-85.051129, s);
//             e = f64::min(180.0, e);
//             n = f64::min(85.051129, n);
//             let u_tile = tile(w, n, zoom);
//             let lr_tile = tile(e - LL_EPSILON, s + LL_EPSILON, zoom);
//             let range_x = u_tile.x..=lr_tile.x;
//             let range_y = u_tile.y..=lr_tile.y;
//             iproduct!(range_x, range_y).map(move |(i, j)| Tile::new(i, j, zoom.clone()))
//         })
//         .flatten()
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mercator_xy() {
        let tile = Tile::new(1, 2, 3);
        let (lng, lat) = tile.ul();
        assert_eq!(xy(lng, lat), (-15028131.257091932, 10018754.171394626));
    }
}

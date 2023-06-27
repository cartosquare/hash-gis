//!Global Map Tiles in Spherical Mercator projection.
use crate::MAXZOOMLEVEL;
use std::f64::consts::PI;

pub struct GlobalMercator {
    pub tile_size: usize,
}

impl GlobalMercator {
    pub fn new(tile_size: usize) -> Self {
        GlobalMercator { tile_size }
    }

    pub fn resolution(&self, zoom: &u32) -> f64 {
        let initial_resolution = 2.0 * PI * 6378137.0 / (self.tile_size as f64);
        initial_resolution / (2u32.pow(*zoom) as f64)
    }

    pub fn zoom_for_pixel_size(&self, pixel_size: &f64) -> u32 {
        let mut zoom: u32 = 0;
        for i in 0..MAXZOOMLEVEL {
            if pixel_size > &self.resolution(&i) {
                zoom = i - 1;
                break;
            };
        }
        zoom
    }
}

impl Default for GlobalMercator {
    fn default() -> Self {
        Self::new(256)
    }
}

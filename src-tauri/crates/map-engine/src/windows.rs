//! Types and helpers to work with data windows.
use crate::affine::GeoTransform;
use crate::raster::SpatialInfo;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::ops::Mul;

/// A data window representing a section of a raster image
#[derive(Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Window {
    pub col_off: isize,
    pub row_off: isize,
    pub width: usize,
    pub height: usize,
}

impl Window {
    /// Create a new window.
    ///
    /// # Argument
    ///
    /// * `col_off` - Column pixel index of the upper-left corner
    /// * `row_off` - Row pixel index of the upper-left corner
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    pub fn new(col_off: isize, row_off: isize, width: usize, height: usize) -> Self {
        Window {
            col_off,
            row_off,
            width,
            height,
        }
    }

    /// Get the start and end pixel indices in the row and column space
    pub fn toranges(self) -> ((isize, isize), (isize, isize)) {
        (
            (self.row_off, self.row_off + self.height as isize),
            (self.col_off, self.col_off + self.width as isize),
        )
    }

    /// Check if the window covers 0 pixels
    pub fn is_zero(self) -> bool {
        self.height == 0 || self.width == 0
    }

    /// Check if both windows intersect
    pub fn intersects(&self, other: &Window) -> bool {
        intersection(&[*self, *other]).is_some()
    }

    /// Get a GeoTransform relative to this window
    pub fn geotransform(&self, geo: &GeoTransform) -> GeoTransform {
        let (x, y) = geo * (self.col_off, self.row_off);
        let c = geo.geo[2];
        let f = geo.geo[5];
        &GeoTransform::translation(x - c, y - f) * geo
    }

    /// Get the spatial bounds of the window
    pub fn bounds(&self, geo: &GeoTransform) -> (f64, f64, f64, f64) {
        let row_min = self.row_off;
        let row_max = row_min + self.height as isize;
        let col_min = self.col_off;
        let col_max = col_min + self.width as isize;

        let (left, bottom) = geo * (col_min, row_max);
        let (right, top) = geo * (col_max, row_min);
        (left, top, right, bottom)
    }

    pub fn bounds_lat_long(
        &self,
        spatial_info: &SpatialInfo,
        geo: &GeoTransform,
    ) -> (f64, f64, f64, f64) {
        let spatial_ref = spatial_info.to_spatial_ref().unwrap();
        let wgs84_crs = gdal::spatial_ref::SpatialRef::from_epsg(4326).unwrap();
        spatial_ref.set_axis_mapping_strategy(0);
        wgs84_crs.set_axis_mapping_strategy(0);
        let vertex_trans =
            gdal::spatial_ref::CoordTransform::new(&spatial_ref, &wgs84_crs).unwrap();
        let (left, top, right, bottom) = self.bounds(geo);
        let mut xs = [left, right];
        let mut ys = [top, bottom];
        let mut zs = [0.0f64; 2];
        vertex_trans
            .transform_coords(&mut xs, &mut ys, &mut zs)
            .unwrap();
        (xs[0], ys[0], xs[1], ys[1])
    }

    // pub fn from_slices(self, rows: (i32, i32), cols: (i32, i32), boudless: bool) -> Self {}
    // pub fn from_bounds(
    //     self,
    //     left: f64,
    //     bottom: f64,
    //     right: f64,
    //     top: f64,
    //     transform: GeoTransform,
    // ) -> Self {
    //     let (row_start, col_start) = transform.rowcol(left, top);
    //     let (row_stop, col_stop) = transform.rowcol(right, bottom);
    //     Self::new(
    //         row_start,
    //         col_start,
    //         col_stop - col_start,
    //         row_stop - row_start,
    //     )
    // }
}

impl Mul<f64> for Window {
    type Output = Window;

    fn mul(self, rhs: f64) -> Window {
        let extended_width = (self.width as f64 * rhs).ceil() as isize;
        let extended_height = (self.height as f64 * rhs).ceil() as isize;
        let col_shift = (extended_width - self.width as isize) / 2;
        let row_shift = (extended_width - self.width as isize) / 2;
        Window::new(
            self.col_off - col_shift,
            self.row_off - row_shift,
            extended_width as usize,
            extended_height as usize,
        )
    }
}

// Finds the intersection of 2 windows.
//
// This is a private function used as the inner block of the
// reduce part of `intersection`.
// Users of this API should use `intersection` function directly.
fn intersection2(w0: Window, w1: Window) -> Window {
    if w0.is_zero() || w1.is_zero() {
        return Window::default();
    }

    let v0 = cmp::max(w0.col_off, w1.col_off);
    let v1 = cmp::min(
        w0.col_off + w0.width as isize,
        w1.col_off + w1.width as isize,
    );
    if v0 >= v1 {
        return Window::default();
    }

    let h0 = cmp::max(w0.row_off, w1.row_off);
    let h1 = cmp::min(
        w0.row_off + w0.height as isize,
        w1.row_off + w1.height as isize,
    );
    if h0 >= h1 {
        return Window::default();
    }

    Window {
        col_off: v0,
        row_off: h0,
        width: (v1 - v0) as usize,
        height: (h1 - h0) as usize,
    }
}

/// Find the intersect between multiple [`Window`]s.
///
/// # Arguments
///
/// * `windows` - Windows to intersect.
pub fn intersection(windows: &[Window]) -> Option<Window> {
    match windows.iter().copied().reduce(intersection2) {
        None => None,
        Some(w) => {
            if w == Window::default() {
                None
            } else {
                Some(w)
            }
        }
    }
}

#[test]
fn test_intersection2() {
    let w0 = Window::new(0, 0, 10, 10);
    let w1 = Window::new(11, 11, 10, 10);
    assert_eq!(intersection2(w0, w1), Window::new(0, 0, 0, 0));

    let w1 = Window::new(11, 11, 10, 10);
    let w0 = Window::new(0, 0, 10, 10);
    assert_eq!(intersection2(w0, w1), Window::new(0, 0, 0, 0));

    let w0 = Window::new(0, 0, 10, 10);
    let w1 = Window::new(5, 5, 10, 10);
    assert_eq!(intersection2(w0, w1), Window::new(5, 5, 5, 5));

    let w0 = Window::new(5, 5, 10, 10);
    let w1 = Window::new(5, 5, 10, 10);
    assert_eq!(intersection2(w0, w1), Window::new(5, 5, 10, 10));

    let w0 = Window::new(5, 5, 10, 10);
    let w1 = Window::new(5, 5, 10, 10);
    assert_eq!(intersection2(w0, w1), Window::new(5, 5, 10, 10));

    let w0 = Window::new(0, 0, 0, 0);
    let w1 = Window::new(0, 0, 0, 0);
    assert_eq!(intersection2(w0, w1), Window::new(0, 0, 0, 0));
}

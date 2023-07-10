//! Types and helpers to work with raster images.
pub mod pixels;

use crate::{
    affine::GeoTransform,
    errors::MapEngineError,
    tiles::{Tile, TILE_SIZE},
    windows::intersection,
    windows::Window,
};
use gdal::{
    raster::{GdalType, RasterBand, ResampleAlg},
    spatial_ref::SpatialRef,
    Dataset,
    DriverManager,
};
use ndarray::{s, Array, Array2, Array3};
use num_traits::{Num, NumCast};
pub use pixels::{driver::MBTILES, RawPixels, StyledPixels};
use std::cmp;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpatialInfo {
    epsg_code: Option<i32>,
    proj4: Option<String>,
    wkt: Option<String>,
    esri: Option<String>,
}

impl SpatialInfo {
    pub fn from_spatial_ref(spatial_ref: &SpatialRef) -> SpatialInfo {
        let epsg_code: Option<i32> = match spatial_ref.auth_code() {
            Ok(data) => Some(data),
            _ => None,
        };
        let wkt: Option<String> =  match spatial_ref.to_wkt() {
            Ok(data) => Some(data),
            _ => None,
        };
        let proj4: Option<String> = match spatial_ref.to_proj4() {
            Ok(data) => Some(data),
            _ => None,
        };

        SpatialInfo {
            epsg_code,
            wkt,
            proj4,
            esri: None,
            }
    }

    pub fn to_spatial_ref(&self) -> Result<SpatialRef, MapEngineError> {
        if self.epsg_code.is_some() {
            Ok(SpatialRef::from_epsg(self.epsg_code.clone().unwrap() as u32)?)
        } else if self.proj4.is_some() {
            Ok(SpatialRef::from_proj4(&self.proj4.clone().unwrap())?)
        } else if self.wkt.is_some() {
            Ok(SpatialRef::from_wkt(&self.wkt.clone().unwrap())?)
        } else if self.esri.is_some() {
            Ok(SpatialRef::from_esri(&self.wkt.clone().unwrap())?)
        } else {
            Err(MapEngineError::Msg("Unknow spatial ref".into()))
        }
    }
}

/// A Raster image.
#[derive(Debug, Clone)]
pub struct Raster {
    path: PathBuf,
    geo: GeoTransform,
    spatial_info: SpatialInfo,
    driver_name: String,
    raster_count: isize,
    raster_size: (usize, usize),
    min_max: Vec<(f64, f64)>,
}

impl Raster {
    /// Crete a new `Raster`.
    ///
    /// This will open a [`Dataset`] and store some metadata into the `Raster` struct. This serves
    /// as a cache to avoid constantly reading from the file.
    pub fn new(path: PathBuf) -> Result<Self, MapEngineError> {
        let src = Dataset::open(&path)?;
        let geo = src.geo_transform()?;
        let geo = GeoTransform::from_gdal(&geo);
        let mut min_max: Vec<(f64, f64)> = vec![];
        for b in 1..=src.raster_count() {
            let band = src.rasterband(b)?;
            let minmax = band.compute_raster_min_max(true)?;
            let skip = (minmax.max - minmax.min) * 0.02;
            min_max.push((minmax.min + skip, minmax.max - skip));
        }

        Ok(Self {
            path,
            geo,
            spatial_info: SpatialInfo::from_spatial_ref(&src.spatial_ref()?),
            driver_name: src.driver().short_name(),
            raster_count: src.raster_count(),
            raster_size: src.raster_size(),
            min_max,
        })
    }

    /// Create a new `Raster` from an open [`Dataset`].
    ///
    /// Usually, you would want to use `Raster::new` but this method is available in case you
    /// already opened a `Dataset`.
    pub fn from_src(path: PathBuf, src: &Dataset) -> Result<Self, MapEngineError> {
        let geo = src.geo_transform()?;
        let geo = GeoTransform::from_gdal(&geo);
        let spatial_ref = src.spatial_ref()?;

        let mut min_max: Vec<(f64, f64)> = vec![];
        for b in 1..=src.raster_count() {
            let band = src.rasterband(b)?;
            let minmax = band.compute_raster_min_max(true)?;
            min_max.push((minmax.min, minmax.max));
        }

        Ok(Self {
            path,
            geo,
            spatial_info: SpatialInfo::from_spatial_ref(&spatial_ref),
            driver_name: src.driver().short_name(),
            raster_count: src.raster_count(),
            raster_size: src.raster_size(),
            min_max,
        })
    }

    /// Read a tile from raster file.
    ///
    /// # Arguments
    ///
    /// * `tile` - Tile to read.
    /// * `bands` - Bands to read (1-indexed).
    /// * `e_resample_alg` - Resample algorith to use in case interpolations are needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use gdal::raster::ResampleAlg;
    /// use map_engine::{errors::MapEngineError, raster::{Raster, RawPixels}, tiles::Tile};
    /// use std::path::PathBuf;
    ///
    /// fn main() -> Result<(), MapEngineError> {
    ///     let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    ///     let raster = Raster::new(path)?;
    ///     let tile = Tile::new(304, 624, 10);
    ///
    ///     let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), Some(ResampleAlg::Average))?;
    ///     assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    ///     Ok(())
    /// }
    /// ```
    pub fn read_tile<P>(
        &self,
        tile: &Tile,
        bands: Option<&[isize]>,
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<RawPixels<P>, MapEngineError>
    where
        P: GdalType + Copy + Num + NumCast,
        P: ndarray::ScalarOperand,
    {
        let src = Dataset::open(&self.path)?;
        let driver_name = self.driver_name();
        // let tile_size = TILE_SIZE as usize;
        let geo = self.geo();
        let (mut win, is_skewed) = tile.to_window(self)?;
        // println!("win: {:?}, is_skewed: {:?}", win, is_skewed);
        let tile_bounds_xy = tile.bounds_xy();
        // println!("tile_bounds_xy: {:?}", tile_bounds_xy);
        if is_skewed {
            win = win * 2f64.sqrt();
        };

        let all_bands: Vec<_> = (1..=self.raster_count()).collect();
        let mut bands = bands.unwrap_or(&all_bands);
        if driver_name == MBTILES {
            bands = &all_bands;
        }

        let mut container_arr = Array3::<P>::zeros((bands.len(), TILE_SIZE, TILE_SIZE));

        for (out_idx, band_index) in bands.iter().enumerate() {
            let band = src.rasterband(*band_index)?;

            let band_data = try_boundless(
                &src,
                &band,
                &win,
                geo,
                &self.spatial_info,
                tile_bounds_xy,
                is_skewed,
                e_resample_alg,
            );
            let band_data = if let Some(d) = band_data {
                d
            } else {
                try_overview(
                    &band,
                    &win,
                    // req_overview as f64,
                    geo,
                    &self.spatial_info,
                    tile_bounds_xy,
                    is_skewed,
                    e_resample_alg,
                )?
            };

            // println!("read band data : {:?}", band_data.dim());
            container_arr
                .slice_mut(s![out_idx, .., ..])
                .assign(&band_data);
        }

        // TODO: evaluate if we have to read this every time
        if driver_name == MBTILES {
            container_arr.swap_axes(0, 1);
            container_arr.swap_axes(1, 2);
        }
        Ok(RawPixels::new(container_arr, driver_name))
    }

    pub fn geo(&self) -> &GeoTransform {
        &self.geo
    }

    pub fn spatial_ref(&self) -> Result<SpatialRef, MapEngineError> {
        self.spatial_info.to_spatial_ref()
    }

    pub fn spatial_info(&self) -> SpatialInfo {
        self.spatial_info.clone()
    }

    pub fn driver_name(&self) -> &str {
        &self.driver_name
    }

    /// Get number of bands available in the file.
    pub fn raster_count(&self) -> isize {
        self.raster_count
    }

    pub fn raster_size(&self) -> (usize, usize) {
        self.raster_size
    }

    pub fn min_max(&self) -> Vec<(f64, f64)> {
        self.min_max.clone()
    }

    pub fn intersects(&self, tile: &Tile) -> Result<bool, MapEngineError> {
        let (raster_w, raster_h) = self.raster_size();
        let raster_win = Window::new(0, 0, raster_w, raster_h);
        Ok(intersection(&[raster_win, tile.to_window(self)?.0]).is_some())
    }
}

fn array_to_mem_dataset<N>(
    arr: Array2<N>,
    transform: &GeoTransform,
    spatial_info: &SpatialInfo,
    fname: &str,
) -> Result<Dataset, MapEngineError>
where
    N: Clone + GdalType + Copy,
{
    let shape = arr.shape();
    let count = 1;
    let height = shape[0];
    let width = shape[1];

    // let driver = gdal::Driver::get("MEM")?;
    let driver = DriverManager::get_driver_by_name("MEM")?;
    // let driver = gdal::Driver::get("GTiff")?;

    let mut dataset = driver
        .create_with_band_type::<N, _>(fname, height as isize, width as isize, count as isize)
        .unwrap();
    let gt = transform.to_tuple();
    let gt = [gt.0, gt.1, gt.2, gt.3, gt.4, gt.5];
    dataset.set_geo_transform(&gt)?;
    dataset.set_spatial_ref(&spatial_info.to_spatial_ref()?)?;

    let mut band = dataset.rasterband(1)?;

    let v: Vec<N> = Array::from_iter(arr.into_iter()).to_vec();
    let buff = gdal::raster::Buffer::<N>::new((height, width), v);
    band.write((0, 0), (height, width), &buff)?;
    drop(band);
    Ok(dataset)
}

fn reproject<N>(
    source: Array2<N>,
    src_transform: &GeoTransform,
    src_spatial_info: &SpatialInfo,
    destination: Array2<N>,
    dst_transform: &GeoTransform,
    dst_spatial_info: &SpatialInfo,
) -> Result<Array2<N>, MapEngineError>
where
    N: GdalType + Copy,
{
    let dst_shape = &destination.shape();
    let dst_shape = (dst_shape[0], dst_shape[1]);
    let src_dataset = array_to_mem_dataset(source, src_transform, src_spatial_info, "/tmp/src.tif")?;
    let dst_dataset = array_to_mem_dataset(destination, dst_transform, dst_spatial_info, "/tmp/dst.tif")?;
    gdal::raster::reproject(&src_dataset, &dst_dataset)?;

    let dst_band = dst_dataset.rasterband(1)?;

    dst_band
        .read_as_array::<N>(
            (0, 0),
            dst_shape,
            (TILE_SIZE, TILE_SIZE),
            Some(gdal::raster::ResampleAlg::NearestNeighbour),
        )
        .map_err(From::from)
}

fn read_and_reproject<N>(
    band: &RasterBand,
    win: &Window,
    geo: &GeoTransform,
    spatial_info: &SpatialInfo,
    tile_bounds_xy: (f64, f64, f64, f64),
    e_resample_alg: Option<ResampleAlg>,
) -> Result<Array2<N>, MapEngineError>
where
    N: GdalType + Copy + Num,
{
    let win_geo = win.geotransform(geo).to_gdal();

    let d = band.read_as::<N>(
        (win.col_off, win.row_off),
        (win.width as usize, win.height as usize),
        (win.width as usize, win.height as usize),
        e_resample_alg,
    )?;

    let arr = Array::from_iter(d.data).into_shape((win.width as usize, win.height as usize))?;

    let res_x = win_geo.geo[1];
    let res_y = win_geo.geo[5];
    let (min_x, max_y, max_x, min_y) = tile_bounds_xy;
    let mercator_geo = &GeoTransform::new(&[min_x, res_x, 0.0, max_y, 0.0, res_y]);
    let dst_cols = ((max_x - min_x) / res_x) as usize;
    let dst_rows = ((max_y - min_y) / -res_y) as usize;
    let dst_shape = (dst_cols, dst_rows);
    let dst_arr = Array2::<N>::zeros(dst_shape);
    reproject(arr, &win_geo, spatial_info, dst_arr, mercator_geo, &SpatialInfo { epsg_code: Some(3857), proj4: None, wkt: None, esri: None })
}

fn try_overview<N>(
    band: &RasterBand,
    win: &Window,
    // factor: f64,
    geo: &GeoTransform,
    spatial_info: &SpatialInfo,
    tile_bounds_xy: (f64, f64, f64, f64),
    is_skewed: bool,
    e_resample_alg: Option<ResampleAlg>,
) -> Result<Array2<N>, MapEngineError>
where
    N: GdalType + Copy + Num,
{
    if is_skewed {
        read_and_reproject(band, win, geo, spatial_info, tile_bounds_xy, e_resample_alg)
    } else {
        band.read_as_array::<N>(
            // (new_win.col_off, new_win.row_off),
            // (new_win.width as usize, new_win.height as usize),
            (win.col_off, win.row_off),
            (win.width as usize, win.height as usize),
            (TILE_SIZE, TILE_SIZE),
            e_resample_alg,
        )
        .map_err(From::from)
    }
}

// Read pixels within a Window
#[allow(clippy::too_many_arguments)]
fn try_boundless<N>(
    src: &Dataset,
    band: &RasterBand,
    win: &Window,
    geo: &GeoTransform,
    spatial_info: &SpatialInfo,
    tile_bounds_xy: (f64, f64, f64, f64),
    is_skewed: bool,
    e_resample_alg: Option<ResampleAlg>,
) -> Option<Array2<N>>
where
    N: GdalType + Copy + Num + NumCast,
    N: ndarray::ScalarOperand,
{
    let (raster_w, raster_h) = src.raster_size();
    // println!("src dim: {}x{}", raster_w, raster_h);
    let raster_win = Window::new(0, 0, raster_w, raster_h);
    let inter = intersection(&[raster_win, *win]);

    // println!("inter: {:?}", inter);
    if let Some(inter) = inter {
        if (inter.height >= win.height || inter.width >= win.width)
            && (win.col_off >= 0 && win.row_off >= 0)
            && (win.row_off + win.height as isize) < raster_win.height as isize
            && (win.col_off + win.width as isize) < raster_win.width as isize
        {
            // The image is larger than the tile. Return None to proceed normally (try_overview)
            return None;
        }
    };

    let mut container_arr = match band.no_data_value() {
        None => Array2::<N>::zeros((TILE_SIZE, TILE_SIZE)),
        Some(val) => {
            let val = N::from(val)?;
            Array2::<N>::ones((TILE_SIZE, TILE_SIZE)) * val
        }
    };

    if inter.is_none() {
        return Some(container_arr);
    };

    let inter = inter.unwrap();
    // The image is smaller than the tile
    let factor = (
        win.width as f64 / inter.width as f64,
        win.height as f64 / inter.height as f64,
    );
    // println!("factor: {:?}", factor);

    let data = if is_skewed {
        read_and_reproject(band, &inter, geo, spatial_info, tile_bounds_xy, e_resample_alg)
    } else {
        let into_shape = (
            (TILE_SIZE as f64 / factor.0).floor() as usize,
            (TILE_SIZE as f64 / factor.1).floor() as usize,
        );
        // println!("into shape: {:?}", into_shape);
        band.read_as_array::<N>(
            (inter.col_off, inter.row_off),
            (inter.width, inter.height),
            into_shape,
            e_resample_alg,
        )
        .map_err(From::from)
    }
    .unwrap_or_else(|_| panic!("Cannot read window {:?} from {:?}", inter, src));

    let col_off = if win.col_off < 0 {
        (TILE_SIZE as f64 * (win.col_off as f64 / win.width as f64) - 1.0).trunc() as isize
    } else {
        0
    };
    let row_off = if win.row_off < 0 {
        (TILE_SIZE as f64 * (win.row_off as f64 / win.height as f64) - 1.0).trunc() as isize
    } else {
        0
    };
    // println!("col_of x row_of: {}x{}", col_off, row_off);
    let (row_range, col_range) = if is_skewed {
        let row_range = ((TILE_SIZE - data.shape()[0]) as isize)
            ..cmp::min(row_off.abs() + data.shape()[0] as isize, TILE_SIZE as isize);
        let col_range = ((TILE_SIZE - data.shape()[1]) as isize)
            ..cmp::min(col_off.abs() + data.shape()[1] as isize, TILE_SIZE as isize);
        (row_range, col_range)
    } else {
        let row_range =
            row_off.abs()..cmp::min(row_off.abs() + data.shape()[0] as isize, TILE_SIZE as isize);
        let col_range =
            col_off.abs()..cmp::min(col_off.abs() + data.shape()[1] as isize, TILE_SIZE as isize);
        (row_range, col_range)
    };
    // println!("row_range: {:?}", row_range);
    // println!("col_range: {:?}", col_range);
    container_arr
        .slice_mut(s![row_range, col_range])
        .assign(&data);
    Some(container_arr)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_try_boundless() {
        let path = Path::new("src/tests/data/chile_optimised.tif");
        let src = Dataset::open(path).unwrap();
        let epsg_code = src.spatial_ref().unwrap().auth_code().unwrap();
        let geo = src.geo_transform().unwrap();
        let geo = GeoTransform::from_gdal(&geo);
        let band = src.rasterband(1).unwrap();
        // The whole image is 1/16 of the window size
        let win = Window::new(0, 0, 2048, 2048); // quad row=0,col=0
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Array2<f64> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();
        assert_eq!(arr.shape(), &[256, 256]);
        let expected = Array::from_iter([2251., 2242., 0., 2251., 2259., 0., 0., 0., 0.])
            .into_shape((3, 3))
            .unwrap();
        assert_eq!(arr.slice(s![62..65, 62..65]), expected);
        let win = Window::new(-1024, -512, 2048, 2048); // quad row=1,col=2
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Array2<f64> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();
        assert_eq!(arr.shape(), &[256, 256]);
        assert_eq!(arr.slice(s![126..129, 190..193]), expected);
        // Read bottom-right 1/4 of the image
        let win = Window::new(256, 256, 2048, 2048);
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Array2<f64> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();
        assert_eq!(arr.shape(), &[256, 256]);
        let expected = Array::from_iter([2251., 2242., 0., 2251., 2259., 0., 0., 0., 0.])
            .into_shape((3, 3))
            .unwrap();
        assert_eq!(arr.slice(s![30..33, 30..33]), expected);
        // Read bottom-left 1/4 of the image
        let win = Window::new(-1792, 256, 2048, 2048);
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Array2<f64> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();
        assert_eq!(arr.shape(), &[256, 256]);
        let expected = Array::from_iter([0., 4645., 4527., 0., 4706., 4297., 0., 0., 0.])
            .into_shape((3, 3))
            .unwrap();
        assert_eq!(arr.slice(s![30..33, 223..226]), expected);

        // These tiles don't exist but they simulate some cases
        let win = Window::new(0, -512, 512, 512 * 2);
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Option<Array2<f64>> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        );
        assert!(arr.is_some());

        let win = Window::new(0, 256, 512, 512 * 2);
        let tile_bounds_xy = win.bounds(&geo);
        let arr: Option<Array2<f64>> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        );
        assert!(arr.is_some());
    }

    #[test]
    fn test_non_intersecting_returns_no_data_tile() {
        let path = Path::new("src/tests/data/categorical_optimised.tif");
        let src = Dataset::open(path).unwrap();
        let epsg_code = src.spatial_ref().unwrap().auth_code().unwrap();
        let geo = src.geo_transform().unwrap();
        let geo = GeoTransform::from_gdal(&geo);
        let mut band = src.rasterband(1).unwrap();

        // A window with no intersecting area, should return an array of no_data_value
        let win = Window::new(2048, 2048, 4096, 4096);
        let tile_bounds_xy = win.bounds(&geo);

        band.set_no_data_value(Some(2f64)).unwrap();
        // Setting the return type to `Array2<f32>` makes the generic algorithm
        // return the expected type.
        let arr: Array2<f32> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            None,
        )
        .unwrap();

        // Simulate an array of the same dimensions and values
        let expected: Array2<f32> = Array2::ones((256, 256)) * 2f32;
        assert_eq!(arr, expected);

        // TODO: what's the story here? what do we want to achieve?
        // let _arr2 = arr.mapv(|v| {
        //     v.to_u8()
        //         .unwrap_or(band.no_data_value().unwrap_or(0.0) as u8)
        // });
    }

    #[test]
    fn test_generic_function_returns_expected_type() {
        let path = Path::new("src/tests/data/chile_optimised.tif");
        let src = Dataset::open(path).unwrap();
        let epsg_code = src.spatial_ref().unwrap().auth_code().unwrap();
        let geo = src.geo_transform().unwrap();
        let geo = GeoTransform::from_gdal(&geo);
        let band = src.rasterband(1).unwrap();
        let win = Window::new(-1792, 256, 2048, 2048);
        let tile_bounds_xy = win.bounds(&geo);

        let arr: Array2<f32> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();

        // Return types should be f32
        let expected = Array::from_iter([
            0.0f32, 4645.0f32, 4527.0f32, 0.0f32, 4706.0f32, 4297.0f32, 0.0f32, 0.0f32, 0.0f32,
        ])
        .into_shape((3, 3))
        .unwrap();
        assert_eq!(arr.slice(s![30..33, 223..226]), expected);

        let arr: Array2<u32> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();

        // Return types should be u32
        let expected = Array::from_iter([
            0u32, 4645u32, 4527u32, 0u32, 4706u32, 4297u32, 0u32, 0u32, 0u32,
        ])
        .into_shape((3, 3))
        .unwrap();
        assert_eq!(arr.slice(s![30..33, 223..226]), expected);

        // Following code does not compile!, type annotations are needed!
        // let arr: Array2<_> = try_boundless(&src, &band, &win, Some(ResampleAlg::Average)).unwrap();
    }

    #[test]
    fn test_band_type_does_not_fit_in_resulting_type() {
        let path = Path::new("src/tests/data/chile_optimised.tif");
        let src = Dataset::open(path).unwrap();
        let epsg_code = src.spatial_ref().unwrap().auth_code().unwrap();
        let geo = src.geo_transform().unwrap();
        let geo = GeoTransform::from_gdal(&geo);
        let band = src.rasterband(1).unwrap();
        let win = Window::new(-1792, 256, 2048, 2048);
        let tile_bounds_xy = win.bounds(&geo);

        let arr: Array2<u8> = try_boundless(
            &src,
            &band,
            &win,
            &geo,
            epsg_code,
            tile_bounds_xy,
            false,
            Some(ResampleAlg::Average),
        )
        .unwrap();

        // Return types should be u8
        // Using the `as` operator, and because particular u32 values do not fit in u8 they are
        // made whatever the amount of bytes from the smalles type represent.
        let expected = Array::from_iter([0u8, 255, 255, 0, 255, 255, 0, 0, 0])
            .into_shape((3, 3))
            .unwrap();

        // values not fitting in u8 will be made std::u8::MAX
        assert_eq!(arr.slice(s![30..33, 223..226]), expected);

        // and not "<big-value><big-type> as <small-type>"
        // returns any value, not necessarily <small-type>::MAX
        let not_expected = Array::from_iter([
            0u8,
            4645u32 as u8,
            4527u32 as u8,
            0u8,
            4706u32 as u8,
            4297u32 as u8,
            0u8,
            0u8,
            0u8,
        ])
        .into_shape((3, 3))
        .unwrap();

        assert_ne!(arr.slice(s![30..33, 223..226]), not_expected);
    }

    #[test]
    fn test_intersects() {
        let path = Path::new("src/tests/data/chile_optimised.tif");
        let raster = Raster::new(path.into()).unwrap();
        let tile1 = Tile::new(304, 624, 10);
        assert!(raster.intersects(&tile1).unwrap());
        let tile1 = Tile::new(303, 624, 10);
        assert!(!raster.intersects(&tile1).unwrap());
    }
}

//! Raw and Styled pixels.
use crate::errors::MapEngineError;
use crate::{
    cmap::{Composite, HandleGet},
    tiles::TILE_SIZE,
};
use gdal::raster::GdalType;
use ndarray::{Array, Array3, Axis};
use num_traits::{Num, NumCast};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Raw pixels read from a raster file.
pub struct RawPixels<P>
where
    P: GdalType + Copy + Num + NumCast,
    P: ndarray::ScalarOperand,
{
    data: Array3<P>,
    driver: Box<dyn driver::Style<P>>,
}

impl<P> RawPixels<P>
where
    P: GdalType + Copy + Num + NumCast,
    P: ndarray::ScalarOperand,
{
    pub(super) fn new(data: Array3<P>, driver: &str) -> Self {
        match driver {
            driver::MBTILES => {
                let driver = Box::new(driver::Mbtile {});

                Self { data, driver }
            }
            _ => {
                let driver = Box::new(driver::Generic {});
                Self { data, driver }
            }
        }
    }

    /// Apply a colour map to an array.
    ///
    /// # Arguments
    ///
    /// * `cmap` - A colour composite that maps pixel values to RGBA.
    /// * `no_data_values` - Pixel values to be set as fully transparent.
    pub fn style(
        self,
        cmap: Composite,
        no_data_values: Vec<f64>,
    ) -> Result<StyledPixels, MapEngineError>
    where
        P: std::fmt::Debug,
    {
        self.driver.style(&self, cmap, no_data_values)
    }

    pub fn as_array(&self) -> &Array3<P> {
        &self.data
    }
}

/// Pixels styled using the [`RawPixels::style`] method.
pub struct StyledPixels {
    data: Array3<u8>,
    driver: driver::Driver,
}

impl StyledPixels {
    pub fn new(data: Array3<u8>, driver: driver::Driver) -> Self {
        Self { data, driver }
    }

    pub fn into_png(self) -> Result<Vec<u8>, MapEngineError> {
        let mut buffer = Vec::<u8>::new();
        let mut w: BufWriter<&mut Vec<u8>> = BufWriter::new(buffer.as_mut());
        {
            let mut encoder = png::Encoder::new(&mut w, TILE_SIZE as u32, TILE_SIZE as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            match self.driver {
                driver::Driver::Generic => writer.write_image_data(&self.data.into_raw_vec())?,
                driver::Driver::Mbtile => {
                    writer.write_image_data(&self.data.into_iter().collect::<Vec<u8>>()[..])?
                }
            }
        }
        w.flush()?;
        drop(w);
        Ok(buffer)
    }

    /// Write a styled tile to disk.
    ///
    /// # Arguments
    ///
    /// * `out_path` - Path were to write the tile.
    pub fn write_to_disk(self, out_path: &Path) -> Result<(), MapEngineError> {
        let png_data = self.into_png()?;
        let mut file = File::create(out_path)?;
        file.write_all(&png_data[..])?;
        Ok(())
    }

    #[allow(dead_code)]
    fn as_array(&self) -> &Array3<u8> {
        &self.data
    }

    #[allow(dead_code)]
    fn into_array(self) -> Array3<u8> {
        self.data
    }
}

impl Default for StyledPixels {
    fn default() -> Self {
        Self::new(
            Array::zeros((4, TILE_SIZE, TILE_SIZE)),
            driver::Driver::Generic,
        )
    }
}

pub(crate) mod driver {
    use super::*;

    pub struct Mbtile;
    pub struct Generic;
    pub const MBTILES: &str = "MBTiles";
    pub enum Driver {
        Mbtile,
        Generic,
    }

    pub trait Style<P>
    where
        P: GdalType + Copy + Num + NumCast,
        P: ndarray::ScalarOperand,
    {
        fn style(
            &self,
            arr: &RawPixels<P>,
            cmap: Composite,
            no_data_values: Vec<f64>,
        ) -> Result<StyledPixels, MapEngineError>;
    }

    impl<P> Style<P> for Generic
    where
        P: GdalType + Copy + Num + NumCast,
        P: ndarray::ScalarOperand,
    {
        fn style(
            &self,
            raw: &RawPixels<P>,
            cmap: Composite,
            no_data_values: Vec<f64>,
        ) -> Result<StyledPixels, MapEngineError> {
            let arr_f64 = unsafe { raw.data.raw_view().cast::<f64>().deref_into_view() };
            let v = if cmap.is_contiguous() {
                arr_f64
                    .lanes(Axis(0))
                    .into_iter()
                    .map(|v| cmap.get(v.as_slice().unwrap(), Some(&no_data_values)))
                    .flatten()
                    .collect()
            } else {
                arr_f64
                    .lanes(Axis(0))
                    .into_iter()
                    .map(|v| cmap.get(&v.to_vec(), Some(&no_data_values)))
                    .flatten()
                    .collect()
            };
            let arr = unsafe {
                Array::from_shape_vec_unchecked((raw.data.shape()[1], raw.data.shape()[2], 4), v)
            };
            Ok(StyledPixels::new(arr, driver::Driver::Generic))
        }
    }

    impl<P> Style<P> for Mbtile
    where
        P: GdalType + Copy + Num + NumCast,
        P: ndarray::ScalarOperand,
    {
        fn style(
            &self,
            raw: &RawPixels<P>,
            _: Composite,
            _: Vec<f64>,
        ) -> Result<StyledPixels, MapEngineError> {
            Ok(StyledPixels::new(
                // TODO: Cast directly to u8 using unsafe rust
                raw.data.mapv(|v| v.to_u8().expect("Cannot parse to u8")),
                driver::Driver::Mbtile,
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cmap::viridis;
    use crate::raster::Raster;
    use crate::tiles::Tile;
    use gdal::Dataset;
    use ndarray::{arr3, s};
    use std::path::PathBuf;

    #[test]
    fn test_write() {
        let path = PathBuf::from("src/tests/data/chile_optimised.tif");
        let src = Dataset::open(&path).unwrap();
        let band = src.rasterband(1).unwrap();
        let no_data_value = band.no_data_value().unwrap_or(0.0);
        let (vmin, vmax) = (0.0, 27412.0); // Extracted from raster

        let raster = Raster::new(path).unwrap();
        let tile = Tile::new(304, 624, 10);
        let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
        let styled = arr
            .style(
                Composite::new_gradient(vmin, vmax, &viridis),
                vec![no_data_value],
            )
            .unwrap();
        let out_path = format!("/tmp/tile_{}_{}_{}.png", tile.x, tile.y, tile.z);
        let out_path = Path::new(&out_path);
        styled.write_to_disk(out_path).unwrap();
    }

    #[test]
    fn test_style_tile() {
        // let arr = RawPixels::new(arr3(&[[[0.0, 0.25], [0.5, 1.]], [[0.0, 0.0], [0.0, 0.0]]]));
        let arr = RawPixels::new(arr3(&[[[0.0, 0.25], [0.5, 1.]]]), "");
        let styled = arr
            .style(Composite::new_gradient(0.0, 1., &viridis), vec![0.25])
            .unwrap();
        assert_eq!(
            styled.as_array().slice(s![0, 0, ..]).to_vec(),
            [68, 1, 84, 255]
        );
        assert_eq!(styled.as_array()[[0, 1, 3]], 0); // 0.25 is transparent
    }

    #[test]
    fn test_style_rgb_tile() {
        let arr = RawPixels::new(
            arr3(&[
                [[1.0, 0.0], [0.0, 0.25]],
                [[0.0, 1.0], [0.0, 0.0]],
                [[0.0, 0.0], [1.0, 0.0]],
            ]),
            "",
        );
        let styled = arr
            .style(
                Composite::new_rgb(vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]),
                vec![0.25, 0.25, 0.25],
            )
            .unwrap();
        assert_eq!(
            styled.as_array().slice(s![0, 0, ..]).to_vec(),
            [255, 0, 0, 255]
        ); // Red pixel
        assert_eq!(
            styled.as_array().slice(s![0, 1, ..]).to_vec(),
            [0, 255, 0, 255]
        ); // Green pixel
        assert_eq!(
            styled.as_array().slice(s![1, 0, ..]).to_vec(),
            [0, 0, 255, 255]
        ); // Blue pixel
        assert_eq!(styled.as_array().slice(s![1, 1, ..]).to_vec()[3], 0); // Transparent pixel
    }

    #[test]
    #[should_panic]
    fn test_style_tile_with_gradient_fails() {
        let arr = RawPixels::new(arr3(&[[[1.0, 0.0], [0.0, 0.25]]]), "");
        // TODO: Should style panic, return a Result, or provide a good default as a fallback?
        arr.style(
            Composite::new_gradient(0.0, 1., &viridis),
            vec![0.25, 0.25, 0.25], // We should provide 1 value for a Gradient
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_style_rgb_tile_fails() {
        let arr = RawPixels::new(
            arr3(&[
                [[1.0, 0.0], [0.0, 0.25]],
                [[0.0, 1.0], [0.0, 0.0]],
                [[0.0, 0.0], [1.0, 0.0]],
            ]),
            "",
        );
        arr.style(
            Composite::new_rgb(vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]),
            vec![0.25], // We should provide 3 values for a RBG composite
        )
        .unwrap();
    }
}

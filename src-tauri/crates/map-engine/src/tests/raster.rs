use crate::raster::{Raster, RawPixels};
use crate::tiles::Tile;
use gdal::raster::ResampleAlg;
use ndarray::{s, Array, Array2, Array3};
use std::path::PathBuf;

#[test]
fn test_read_tile_float() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(304, 624, 10);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        3671., 3648., 3480., 3696., 3821., 3807., 3599., 3760., 3843.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_all_bands() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(304, 624, 10);
    let arr: RawPixels<f64> = raster.read_tile(&tile, None, None).unwrap();
    assert_eq!(arr.as_array().shape(), &[2, 256, 256]);
}

#[test]
fn test_read_tile_int() {
    let path = PathBuf::from("src/tests/data/categorical_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(304, 624, 10);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([9., 6., 7., 5., 1., 7., 5., 8., 7.])
        .into_shape((3, 3))
        .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_overview() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(152, 312, 9);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        2307., 2244., 2159., 2304., 2234., 2170., 2312., 2282., 2217.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_overview_no_metadata() {
    let path = PathBuf::from("src/tests/data/chile_no_meta.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(152, 312, 9);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        2307., 2244., 2159., 2304., 2234., 2170., 2312., 2282., 2217.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_overview_forced_resampling() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(152, 312, 9);
    let arr: RawPixels<f64> = raster
        .read_tile(&tile, Some(&[1]), Some(ResampleAlg::NearestNeighbour))
        .unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);

    // Expected if ResampleAlg::Average is being used
    // let expected = Array::from_iter([2307., 2244., 2159.,
    //                                   2304., 2234., 2170.,
    //                                   2312., 2282., 2217.]).into_shape((3, 3)).unwrap();
    // Expected if ResampleAlg::NearestNeighbour is being used
    let expected = Array::from_iter([
        2314., 2184., 2193., 2293., 2208., 2207., 2315., 2269., 2256.,
    ])
    .into_shape((3, 3))
    .unwrap();

    // Note that we can't force a resampling method if we have overviews.
    // GDAL always reads the overview!
    // Is is possible to use a flag to ignore them?
    assert_ne!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_no_overview() {
    let path = PathBuf::from("src/tests/data/chile_no_overview.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(152, 312, 9);
    let arr: RawPixels<f64> = raster
        .read_tile(&tile, Some(&[1]), Some(ResampleAlg::Average))
        .unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        2307., 2244., 2159., 2304., 2234., 2170., 2312., 2282., 2217.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_no_overview_forced_resampling() {
    let path = PathBuf::from("src/tests/data/chile_no_overview.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(152, 312, 9);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        2314., 2184., 2193., 2293., 2208., 2207., 2315., 2269., 2256.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_with_oversampling() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(608, 1248, 11);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([
        3195., 3674., 3674., 3459., 4019., 4019., 3459., 4019., 4019.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_tile_completely_out_of_range() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(303, 624, 10);
    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array3::<f64>::zeros((1, 256, 256));
    assert_eq!(arr.as_array(), expected);
}

#[test]
fn test_read_tile_larger_than_range() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    // The whole image is 1/4 of the window size.
    let tile = Tile::new(76, 156, 8);
    let arr: RawPixels<f64> = raster
        .read_tile(&tile, Some(&[1]), Some(ResampleAlg::Average))
        .unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([2283., 2218., 0., 2312.0, 2226., 0., 0., 0., 0.])
        .into_shape((3, 3))
        .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 126..129, 126..129]), expected);
    // The whole image is 1/16 of the window size
    let tile = Tile::new(38, 78, 7);
    let arr: RawPixels<f64> = raster
        .read_tile(&tile, Some(&[1]), Some(ResampleAlg::Average))
        .unwrap();
    assert_eq!(arr.as_array().shape(), &[1, 256, 256]);
    let expected = Array::from_iter([2251., 2242., 0., 2251., 2259., 0., 0., 0., 0.])
        .into_shape((3, 3))
        .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 62..65, 62..65]), expected);
}

#[test]
fn test_tile_are_different() {
    let path = PathBuf::from("src/tests/data/chile_optimised.tif");
    let raster = Raster::new(path).unwrap();
    let tile1 = Tile::new(608, 1248, 11);
    let arr: RawPixels<f64> = raster.read_tile(&tile1, Some(&[1]), None).unwrap();
    let expected = Array::from_iter([
        3195., 3674., 3674., 3459., 4019., 4019., 3459., 4019., 4019.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);

    let tile2 = Tile::new(608, 1249, 11);
    let arr: RawPixels<f64> = raster.read_tile(&tile2, Some(&[1]), None).unwrap();
    let expected = Array::from_iter([
        5129., 5151., 5151., 5143., 5171., 5171., 5143., 5171., 5171.,
    ])
    .into_shape((3, 3))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

#[test]
fn test_read_mbtiles() {
    let path = PathBuf::from("src/tests/data/chile_optimised.mbtiles");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(304, 624, 10);
    let arr: RawPixels<f64> = raster.read_tile(&tile, None, None).unwrap();
    let expected = Array::from_iter([
        71.0, 46.0, 124.0, 255.0, 71.0, 44.0, 123.0, 255.0, 71.0, 44.0, 123.0, 255.0, 70.0, 48.0,
        126.0, 255.0, 71.0, 47.0, 125.0, 255.0, 71.0, 47.0, 125.0, 255.0, 71.0, 47.0, 125.0, 255.0,
        70.0, 48.0, 126.0, 255.0, 70.0, 48.0, 126.0, 255.0,
    ])
    .into_shape((3, 3, 4))
    .unwrap();
    assert_eq!(arr.as_array().slice(s![253..256, 253..256, ..]), expected);
}

#[test]
fn test_transform_projection() {
    let path = PathBuf::from("src/tests/data/chile_32718.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(609, 1249, 11);
    let arr: RawPixels<u32> = raster.read_tile(&tile, Some(&[1]), None).unwrap();
    let expected = Array2::from_shape_vec(
        (3, 3),
        vec![3717, 3691, 3691, 3760, 3810, 3810, 3760, 3810, 3810],
    )
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);

    let path = PathBuf::from("src/tests/data/chile_32718.tif");
    let raster = Raster::new(path).unwrap();
    let tile = Tile::new(304, 624, 10);
    let arr: RawPixels<u32> = raster.read_tile(&tile, Some(&[1]), None).unwrap();

    let expected = Array2::from_shape_vec(
        (3, 3),
        vec![3582, 3564, 3489, 3675, 3697, 3672, 3677, 3753, 3811],
    )
    .unwrap();
    assert_eq!(arr.as_array().slice(s![0, 253..256, 253..256]), expected);
}

use crate::affine::GeoTransform;

const GDAL_GEO: [f64; 6] = [
    -75.71809698315035,
    0.0008983152841195215,
    0.0,
    -17.504571626353,
    0.0,
    -0.0008983152841195215,
];

#[test]
fn geotransform_inv() {
    let geo_transform = GeoTransform::from_gdal(&GDAL_GEO);

    let inv = geo_transform.inv().unwrap();
    #[allow(clippy::excessive_precision)]
    let expected_inv = GeoTransform::new(&[
        1113.1949079327355,
        0.0,
        84288.99999999999,
        0.0,
        -1113.1949079327355,
        -19486.0,
    ]);
    assert_eq!(inv, expected_inv);
}

#[test]
fn test_rowcol_and_xy() {
    // TODO: This should be from_gdal
    let geo_transform = GeoTransform::new(&GDAL_GEO);

    let xy = (-37.858599333933114, -8.753184128460619);

    assert_eq!(geo_transform.xy(0, 0), xy);
    assert_eq!(geo_transform.rowcol(xy.0, xy.1).unwrap(), (0, 0));
    assert_eq!(geo_transform.rowcol(-78.0, -23.0).unwrap(), (23917, 1));
}

#[test]
fn test_translation_scale_shear() {
    let trans = GeoTransform::translation(-75.71809698315035, -17.504571626353);
    let scale = GeoTransform::scale(0.0008983152841195215, -0.0008983152841195215);
    let shear = GeoTransform::shear(0.0, 0.0);
    // TODO: Make it more ergonomic (trans * scale * shear)
    let res = &(&trans * &scale) * &shear;
    assert_eq!(res.to_gdal().to_array(), GDAL_GEO);
}

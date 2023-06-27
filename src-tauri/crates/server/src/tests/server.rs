use crate::{
    endpoints::{get_tile, preview},
    favicon,
    mapsettings::MapSettings,
    state::State,
    style::Style,
    tests::test_helpers::create_config,
};
use map_engine::{affine::GeoTransform, png::empty_png, windows::Window};
use std::io::Write;
use tempfile::NamedTempFile;
use tide::http::{mime, StatusCode};
use tide_testing::TideTestingExt;

#[async_std::test]
async fn test_init_state_succeeds() {
    let conf_path = create_config(None, true);
    let expected = MapSettings {
        extent: Some(Window::new(0, 0, 512, 512)),
        path: "../map-engine/src/tests/data/chile_optimised.tif".to_string(),
        name: "chile_optimised".to_string(),
        geotransform: Some(GeoTransform::new(&[
            152.8740565703525,
            0.0,
            -8140237.7643,
            0.0,
            -152.8740565703525,
            -4383204.95,
        ])),
        no_data_value: Some(vec![0.0, 0.0]),
        style: Some(Style::default()),
        driver_name: Some("GTiff".to_string()),
        spatial_ref_code: Some(3857),
        spatial_units: Some("metre".to_string()),
    };
    let state = State::from_file(&conf_path).unwrap();
    assert_eq!(&expected, state.maps.get("chile_optimised").unwrap());
}

#[async_std::test]
async fn test_init_state_fails() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "invalid-json").unwrap();

    let state = State::from_file(file.path().to_str().unwrap());
    assert!(state.is_err());
}

#[async_std::test]
async fn test_get_tile() {
    let conf_path = create_config(None, false);
    let state = State::from_file(&conf_path).unwrap();
    std::fs::remove_file(conf_path).unwrap();
    let mut app = tide::with_state(state);

    app.at("/:map_name/:z/:x/:y").get(get_tile);

    let mut response = app.get("/missing/0/0/0.png").await.unwrap();
    assert_eq!(response.status(), StatusCode::NotFound);
    assert_eq!(
        response.body_string().await.unwrap(),
        "The map \"missing\" does not exist"
    );

    let mut response = app.get("/missing/0/0/0.jpg").await.unwrap();
    assert_eq!(response.status(), StatusCode::NotImplemented);
    assert_eq!(
        response.body_string().await.unwrap(),
        "The extension \"jpg\" is not yet supported"
    );

    // Not intersecting tile
    let mut response = app.get("/chile_optimised/11/607/1248.png").await.unwrap();
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.content_type(), Some(mime::PNG));
    let png_data = response.body_bytes().await.unwrap();
    let empty = empty_png().unwrap();
    assert_eq!(png_data, empty);

    // Ok
    let mut response = app.get("/chile_optimised/11/608/1248.png").await.unwrap();
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.content_type(), Some(mime::PNG));
    let png_data = response.body_bytes().await.unwrap();
    assert_ne!(png_data, empty);
}

#[async_std::test]
async fn test_get_mbtile() {
    let content = br#"
[
  {
    "name": "mbtile",
    "path": "../map-engine/src/tests/data/chile_optimised.mbtiles"
  }
]
"#;
    let conf_path = create_config(Some(content), false);
    let state = State::from_file(&conf_path).unwrap();
    std::fs::remove_file(conf_path).unwrap();
    let mut app = tide::with_state(state);

    app.at("/:map_name/:z/:x/:y").get(get_tile);

    // Not intersecting tile
    let mut response = app.get("/mbtile/11/607/124").await.unwrap();
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.content_type(), Some(mime::PNG));
    let png_data = response.body_bytes().await.unwrap();
    let empty = empty_png().unwrap();
    assert_eq!(png_data, empty);

    // Ok
    let mut response = app.get("/mbtile/10/304/624.png").await.unwrap();
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.content_type(), Some(mime::PNG));
    let png_data = response.body_bytes().await.unwrap();
    let empty = empty_png().unwrap();
    assert_ne!(png_data, empty);
}

#[async_std::test]
async fn test_preview() {
    let conf_path = create_config(None, false);
    let state = State::from_file(&conf_path).unwrap();
    std::fs::remove_file(conf_path).unwrap();
    let mut app = tide::with_state(state);

    app.at("/:map_name").get(preview);

    let mut response = app.get("/missing").await.unwrap();
    assert_eq!(response.status(), StatusCode::NotFound);
    assert_eq!(
        response.body_string().await.unwrap(),
        "The map \"missing\" does not exist"
    );

    // Ok
    let mut response = app.get("/chile_optimised").await.unwrap();
    assert_eq!(response.status(), StatusCode::Ok);
    assert_eq!(response.content_type(), Some(mime::HTML));
    let html_data = response.body_string().await.unwrap();

    assert!(html_data.contains(
        "[[-36.597889133177326,-73.12500000037612],[-37.1603165468431,-72.42187500037613]]"
    ));
    assert!(html_data.contains("true&&L.tileLayer"));
    assert!(html_data.contains("chile_optimised/{z}"));
}

#[async_std::test]
async fn test_favicon() {
    let conf_path = create_config(None, true);
    let state = State::from_file(&conf_path).unwrap();
    let mut app = tide::with_state(state);
    app.at("/favicon.ico").get(favicon);

    let response = app.get("/favicon.ico").await.unwrap();
    assert_eq!(response.status(), StatusCode::NotFound);
}

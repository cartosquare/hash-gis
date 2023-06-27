use crate::state::State;
use crate::mapsettings::MapSettings;
use map_engine::{png::EMPTY_PNG, raster::RawPixels, tiles::Tile};
use std::convert::Into;
use tide::{http::mime, Request, Response, StatusCode, Body};

/// Generate a tile given a XYZ URL.
pub async fn add_map(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let map_setting = req.body_json().await?;
    println!("map setting: {:?}", map_setting);

    let map = req.state().add_map(map_setting)?;
    let response = Response::builder(StatusCode::Ok)
        .content_type(mime::JSON)
        .body(Body::from_json(&map)?);
    Ok(response)
}
use crate::{state::State, mapsettings::MapSettings};
use std::convert::Into;
use tide::{http::mime, log::info, Request, Response, StatusCode, Body};

/// Generate a tile given a XYZ URL.
pub async fn add_map(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let map_setting: MapSettings = req.body_json().await?;
    info!("map setting: {:?}", map_setting);

    let map: MapSettings = match map_setting.geo_type.as_str() {
        "vector" =>  req.state().add_map_vector(map_setting)?,
        "raster" =>  req.state().add_map(map_setting)?,
        _ => return Ok(Response::builder(StatusCode::BadRequest).content_type(mime::PLAIN).body(String::from("invalid geo type")))
    };

    let response = Response::builder(StatusCode::Ok)
        .content_type(mime::JSON)
        .body(Body::from_json(&map)?);

    Ok(response)
}
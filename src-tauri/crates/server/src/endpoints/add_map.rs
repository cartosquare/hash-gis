use crate::state::State;
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


pub async fn add_map_vector(mut req: Request<State>) -> tide::Result<impl Into<Response>> {
    let xml = req.body_string().await?;
    println!("xml: {:?}", xml);

    let name = req.state().add_map_vector(xml)?;
    let response = Response::builder(StatusCode::Ok)
        .content_type(mime::JSON)
        .body(format!("\"id\":{}", name));
    Ok(response)
}
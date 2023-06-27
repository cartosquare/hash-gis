use crate::{mapsettings::MapSettings, state::State};
use tide::{http::mime, Request, Response, StatusCode};

/// Generate a webmap preview.
pub async fn preview(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let map_name = req.param("map_name")?;
    let req_map = req.state().get_map(map_name);
    if let Err(e) = req_map {
        return Ok(Response::builder(StatusCode::NotFound).body(e.to_string()));
    };
    let template = gen_template(&req_map.unwrap(), map_name).await?;
    Ok(Response::builder(StatusCode::Ok)
        .content_type(mime::HTML)
        .body(template))
}

async fn gen_template(req_map: &MapSettings, map_name: &str) -> tide::Result<String> {
    let geo = req_map
        .geotransform
        .as_ref()
        .expect("Map was not initialised");
    let spatial_ref_code = req_map.spatial_ref_code.expect("Map was not initialised");
    let ext = req_map.extent.expect("Map was not initialised");

    let (lat_max, long_min, lat_min, long_max) = ext.bounds_lat_long(spatial_ref_code, geo);

    let params = &[
        ("m", map_name),
        (
            "bo",
            &format!("[[{},{}],[{},{}]]", lat_max, long_min, lat_min, long_max),
        ),
        ("ba", "true"),
    ];

    let mut template = include_str!("../../template/preview.html").to_string();
    for (k, v) in params.iter() {
        let k = format!("{{{{{}}}}}", k);
        template = template.replace(&k, v);
    }
    Ok(template)
}

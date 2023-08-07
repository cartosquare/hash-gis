use crate::state::State;
use map_engine::{png::EMPTY_PNG, raster::RawPixels, tiles::Tile};
use std::convert::Into;
use tide::{http::mime, log::info, log::debug, Request, Response, StatusCode};

/// Generate a tile given a XYZ URL.
pub async fn get_tile(req: Request<State>) -> tide::Result<impl Into<Response>> {
    let (map_name, z, x, y, ext) = get_params(&req).await?;
    let mut tile = Tile::new(x, y, z);
    if let Err(e) = tile.set_extension(ext) {
        return Ok(Response::builder(StatusCode::NotImplemented).body(e.to_string()));
    };
    let req_map = req.state().get_map(map_name);
    if let Err(e) = req_map {
        return Ok(Response::builder(StatusCode::NotFound).body(e.to_string()));
    };

    // We already checked if the map exists, so it should be ok to unwrap
    let req_map = &req_map.unwrap();

        let raster = req.state().get_raster(map_name).unwrap();
        let style_gradient = req.state().get_style(map_name).unwrap();

        if !raster.intersects(&tile)? {
            info!(
                "{:?} does not intersect {}. Returning empty {}",
                tile, map_name, ext
            );
            return Ok(Response::builder(StatusCode::Ok)
                .content_type(mime::PNG)
                .body(EMPTY_PNG.clone()));
        }

        info!("Processing {:?} ({:?}) for {:?}", tile, ext, map_name);
        debug!("map: {:?}", req_map);
        debug!("style: {:?}", style_gradient);

        let bands = req_map.get_bands();
        let no_data_value = req_map.get_no_data_values();
        let style_no_data_value = bands
            .iter()
            .map(|v| no_data_value[*v as usize - 1])
            .collect();

        let arr: RawPixels<f64> = raster.read_tile(&tile, Some(bands), None)?;
        let styled = arr.style(style_gradient, style_no_data_value)?;

        let response = Response::builder(StatusCode::Ok)
            .content_type(mime::PNG)
            .body(styled.into_png().expect("Could not create PNG"));
        Ok(response)
}

pub async fn get_params(req: &Request<State>) -> tide::Result<(&str, u32, u32, u32, &str)> {
    let map_name = req.param("map_name")?;
    let z: u32 = req.param("z")?.parse()?;
    let x: u32 = req.param("x")?.parse()?;
    let mut y_ext = req.param("y")?.split('.');
    let y: u32 = y_ext.next().unwrap().parse()?;
    let ext = y_ext.next().unwrap_or("png");
    Ok((map_name, z, x, y, ext))
}

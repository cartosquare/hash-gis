use crate::{
    endpoints::{add_map, get_tile, preview},
    state::State,
};

use http_types::headers::HeaderValue;
use map_engine::vector::Vector;
use tide::security::{CorsMiddleware, Origin};
use tide::{Request, Response, Server, StatusCode};

pub async fn run(
    config: String,
    host: String,
    port: String,
    plugin_dir: String,
    font_dir: String,
) -> tide::Result<()> {
    Vector::mapnik_register(plugin_dir, font_dir);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = create_app(&config).await;
    app.with(cors);
    app.listen(format!("{}:{}", host, port)).await?;

    Ok(())
}

pub async fn create_app(conf_path: &str) -> Server<State> {
    let state = State::from_file(conf_path).unwrap();
    let mut app = tide::with_state(state);

    app.at("/favicon.ico").get(favicon);
    app.at("/:map_name").get(preview);
    app.at("/:map_name/").get(preview);
    app.at("/:map_name/:z/:x/:y").get(get_tile);
    app.at("/map").post(add_map);

    app
}

async fn favicon(_: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(StatusCode::NotFound))
}

#[cfg(test)]
mod test {
    use super::*;
    use tide::http::{mime, StatusCode};
    use tide_testing::TideTestingExt;

    async fn test_get_tile(z: u32, x: u32, y: u32, map_name: String) {
        let state = State::from_file(&format!("./fixtures/{}.json", map_name)).unwrap();
        let mut app = tide::with_state(state);

        app.at("/:map_name/:z/:x/:y").get(get_tile);

        // Ok
        let mut response = app.get(format!("/{}/{}/{}/{}.png", map_name, z, x, y)).await.unwrap();
        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(response.content_type(), Some(mime::PNG));
        let png_data = response.body_bytes().await.unwrap();

        std::fs::write(format!("./fixtures/{}_{}_{}_{}.png", map_name, z, x, y), png_data).unwrap();
    }

    #[async_std::test]
    async fn test_get_tile_8bit_epsg4326_nodata255() {
        test_get_tile(12, 3490, 1450, "8bit_4bands_epsg4326_nodata255".into()).await;
    }

    #[async_std::test]
    async fn test_get_tile_8bit_3bands_epsg3857_nodata255() {
        test_get_tile(15, 26963, 12406, "8bit_3bands_epsg3857_nodata255".into()).await;
    }

    #[async_std::test]
    async fn test_get_tile_8bit_3bands_epsg4490_nodataunset() {
        test_get_tile(12, 3347, 1625, "8bit_3bands_epsg4490_nodataunset".into()).await;
    }

    #[async_std::test]
    async fn test_get_tile_8bit_3bands_cgcs2000_nodata255() {
        test_get_tile(15, 26749, 13023, "8bit_3bands_cgcs2000_nodata255".into()).await;
    }

    #[async_std::test]
    async fn test_get_tile_16bit_3bands_epsg32650_nodata0() {
        test_get_tile(16, 54565, 25807, "16bit_3bands_epsg32650_nodata0".into()).await;
    }
}

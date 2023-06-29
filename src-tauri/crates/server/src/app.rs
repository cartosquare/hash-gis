use crate::{
    endpoints::{add_map, add_map_vector, get_tile, get_tile_vector, preview},
    state::State,
};

use map_engine::vector::Vector;
use tide::{Request, Response, Server, StatusCode};

pub async fn run(
    config: String,
    host: String,
    port: String,
    plugin_dir: String,
    font_dir: String,
) -> tide::Result<()> {
    Vector::mapnik_register(plugin_dir, font_dir);

    let app = create_app(&config).await;
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

    // vector
    app.at("/vector/:map_name/:z/:x/:y").get(get_tile_vector);
    app.at("/vector/map").post(add_map_vector);

    app
}

async fn favicon(_: Request<State>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(StatusCode::NotFound))
}

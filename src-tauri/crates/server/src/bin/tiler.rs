use clap::Parser;
use map_engine::cmap::ColourDefinition;
use map_engine::cmap::Composite;
use map_engine::errors::MapEngineError;
use map_engine::gdal::Dataset;
use map_engine::raster::Raster;
use map_engine::raster::RawPixels;
use map_engine::tiles::Tile;
use map_engine::windows::Window;
use map_engine_server::mapsettings::MapSettings;
use map_engine_server::style::Style;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(long)]
    min_zoom: u32,

    #[clap(long)]
    max_zoom: u32,

    #[clap(short, long)]
    output: String,
}

fn main() -> Result<(), MapEngineError> {
    let args = Args::parse();

    let path = Path::new(&args.input);
    let src = Dataset::open(&path)?;
    let (raster_w, raster_h) = src.raster_size();
    let raster_win = Window::new(0, 0, raster_w, raster_h);
    if src.raster_count() < 3 {
        println!("invalid band count: {}", src.raster_count());
        return Err(MapEngineError::Msg(format!(
            "invalid band count: {}",
            src.raster_count()
        )));
    }
    let no_data_values = (1..=3)
        .map(|i| {
            let band = src.rasterband(i).unwrap();
            band.no_data_value().unwrap_or(0.0)
        })
        .collect();

    println!("no data values: {:?}", no_data_values);
    let raster = Raster::from_src(path.to_path_buf(), &src)?;
    let spatial_ref = raster.spatial_ref()?;
    let epsg = spatial_ref.auth_code()?;
    if epsg != 4326 {
        println!("only support epsg:4326 spatial ref!");
        return Err(MapEngineError::Msg(
            "only support epsg:4326 spatial ref!".into(),
        ));
    }

    // bounds
    let geo = raster.geo();
    let minx = geo.geo[2];
    let maxx = geo.geo[2] + raster_w as f64 * geo.geo[0];
    let maxy = geo.geo[5];
    let miny = geo.geo[5] + raster_h as f64 * geo.geo[4];

    let map = MapSettings {
        extent: Some(raster_win),
        path: args.input.clone(),
        name: "".into(),
        geotransform: Some(geo.clone()),
        no_data_value: Some(no_data_values),
        xml: None,
        driver_name: Some(src.driver().short_name()),
        spatial_info: Some(raster.spatial_info()),
        spatial_units: Some(spatial_ref.linear_units_name()?),
        has_overview: Some(raster.has_overview()),
        // min_lon, min_lat, max_lon, max_lat
        bounds: Some([minx, miny, maxx, maxy]),
        style: Some(Style {
            name: None,
            colours: Some(ColourDefinition::RGB(
                [0.0, 0.0, 0.0],
                [255.0, 255.0, 255.0],
            )),
            bands: Some([1, 2, 3].to_vec()),
            vmax: Some(0.0),
            vmin: Some(255.0),
        }),
        geo_type: "raster".into(),
    };
    println!("Processing {}\n", args.input);

    let output_dir = PathBuf::from(args.output);
    if !Path::exists(&output_dir) {
        fs::create_dir_all(&output_dir)?;
    }
    let style_gradient = map.to_composite();
    for z in args.min_zoom..=args.max_zoom {
        let (tile_minx, tile_miny) = lon_lat_to_tile(minx, maxy, z);
        let (tile_maxx, tile_maxy) = lon_lat_to_tile(maxx, miny, z);
        println!(
            "processing zoom {}, x range: {}-{}; y range: {}-{}",
            z, tile_minx, tile_maxx, tile_miny, tile_maxy,
        );

        for x in tile_minx..=tile_maxx {
            let dir = output_dir.join(z.to_string()).join(x.to_string());
            if !Path::exists(&dir) {
                println!("create dir: {}", dir.display());
                fs::create_dir_all(&dir)?;
            }
            for y in tile_miny..=tile_maxy {
                tile(
                    &map,
                    &raster,
                    &style_gradient,
                    z,
                    x,
                    y,
                    &dir.join(format!("{}.png", y)),
                )?;
            }
        }
    }

    generate_leaflet(
        path.to_path_buf().file_stem().unwrap().to_str().unwrap().into(),
        map.bounds.unwrap().into(),
        args.min_zoom,
        args.max_zoom,
        (args.min_zoom + args.max_zoom) / 2,
        &output_dir.join("map.html"),
    );
    Ok(())
}
/// Return the x,y of a tile which has this lat/lon for this zoom level
pub fn lon_lat_to_tile(lon: f64, lat: f64, zoom: u32) -> (u32, u32) {
    // TODO do this at compile time?
    #[allow(non_snake_case)]
    let MAX_LAT: f64 = std::f64::consts::PI.sinh().atan();

    let lat = lat.to_radians();

    // Clip the latitude to the max & min (~85.0511)
    let lat = if lat > MAX_LAT {
        MAX_LAT
    } else if lat < -MAX_LAT {
        -MAX_LAT
    } else {
        lat
    };

    let n: f64 = 2f64.powi(zoom as i32);
    let xtile: u32 = (n * ((lon + 180.) / 360.)).trunc() as u32;
    let ytile: u32 = (n * (1. - ((lat.tan() + (1. / lat.cos())).ln() / std::f64::consts::PI)) / 2.)
        .trunc() as u32;

    (xtile, ytile)
}

fn tile(
    map: &MapSettings,
    raster: &Raster,
    style_gradient: &Composite,
    z: u32,
    x: u32,
    y: u32,
    output: &PathBuf,
) -> Result<(), MapEngineError> {
    let mut tile = Tile::new(x, y, z);
    tile.set_extension("png").unwrap();

    if !raster.intersects(&tile)? {
        println!("{:?} does not intersect, Returning empty", tile);
        return Ok(());
    }

    let bands = map.get_bands();
    let no_data_value = map.get_no_data_values();
    let style_no_data_value = bands
        .iter()
        .map(|v| no_data_value[*v as usize - 1])
        .collect();

    let arr: RawPixels<f64> = raster.read_tile(&tile, Some(bands), None)?;
    let styled = arr.style(style_gradient.clone(), style_no_data_value)?;

    let png_data = styled.into_png()?;
    let mut file = File::create(&output)?;
    file.write_all(&png_data[..])?;

    Ok(())
}

fn generate_leaflet(
    title: String,
    bounds: Vec<f64>,
    min_zoom: u32,
    max_zoom: u32,
    begin_zoom: u32,
    output_path: &PathBuf,
) {
    let west = bounds[0];
    let south = bounds[1];
    let east = bounds[2];
    let north = bounds[3];
    let center_lon = (west + east) / 2.0;
    let center_lat = (south + north) / 2.0;

    let s = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no' />
            <title>{}</title>

            <!-- Leaflet -->
            <link rel="stylesheet" href="http://cdn.leafletjs.com/leaflet-0.7.5/leaflet.css" />
            <script src="http://cdn.leafletjs.com/leaflet-0.7.5/leaflet.js"></script>

            <style>
                body {{ margin:0; padding:0; }}
                body, table, tr, td, th, div, h1, h2, input {{ font-family: "Calibri", "Trebuchet MS", "Ubuntu", Serif; font-size: 11pt; }}
                #map {{ position:absolute; top:0; bottom:0; width:100%; }} /* full size */
                .ctl {{
                    padding: 2px 10px 2px 10px;
                    background: white;
                    background: rgba(255,255,255,0.9);
                    box-shadow: 0 0 15px rgba(0,0,0,0.2);
                    border-radius: 5px;
                    text-align: right;
                }}
                .title {{
                    font-size: 18pt;
                    font-weight: bold;
                }}
                .src {{
                    font-size: 10pt;
                }}

            </style>

        </head>
        <body>

        <div id="map"></div>

        <script>
        /* **** Leaflet **** */

        // Base layers
        //  .. OpenStreetMap
        var osm = L.tileLayer('http://{{s}}.tile.osm.org/{{z}}/{{x}}/{{y}}.png', {{attribution: '&copy; <a href="http://osm.org/copyright">OpenStreetMap</a> contributors'}});

        //  .. CartoDB Positron
        var cartodb = L.tileLayer('http://{{s}}.basemaps.cartocdn.com/light_all/{{z}}/{{x}}/{{y}}.png', {{attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors, &copy; <a href="http://cartodb.com/attributions">CartoDB</a>'}});

        //  .. OSM Toner
        var toner = L.tileLayer('http://{{s}}.tile.stamen.com/toner/{{z}}/{{x}}/{{y}}.png', {{attribution: 'Map tiles by <a href="http://stamen.com">Stamen Design</a>, under <a href="http://creativecommons.org/licenses/by/3.0">CC BY 3.0</a>. Data by <a href="http://openstreetmap.org">OpenStreetMap</a>, under <a href="http://www.openstreetmap.org/copyright">ODbL</a>.'}});

        //  .. White background
        var white = L.tileLayer("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAAEAAQMAAABmvDolAAAAA1BMVEX///+nxBvIAAAAH0lEQVQYGe3BAQ0AAADCIPunfg43YAAAAAAAAAAA5wIhAAAB9aK9BAAAAABJRU5ErkJggg==");

        // Overlay layers (TMS)
        var lyr = L.tileLayer('./{{z}}/{{x}}/{{y}}.png', {{tms: false, opacity: 0.7, attribution: "SenseTime"}});

        // Map
        var map = L.map('map', {{
            center: [{}, {}],
            zoom: {},
            minZoom: {},
            maxZoom: {},
            layers: [osm]
        }});

        var basemaps = {{"OpenStreetMap": osm, "CartoDB Positron": cartodb, "Stamen Toner": toner, "Without background": white}}
        var overlaymaps = {{"Layer": lyr}}

        // Title
        var title = L.control();
        title.onAdd = function(map) {{
            this._div = L.DomUtil.create('div', 'ctl title');
            this.update();
            return this._div;
        }};
        title.update = function(props) {{
            this._div.innerHTML = "{}";
        }};
        title.addTo(map);

        // Note
        var src = 'Generated by <a href="http://www.klokan.cz/projects/gdal2tiles/">GDAL2Tiles</a>, Copyright &copy; 2008 <a href="http://www.klokan.cz/">Klokan Petr Pridal</a>,  <a href="http://www.gdal.org/">GDAL</a> &amp; <a href="http://www.osgeo.org/">OSGeo</a> <a href="http://code.google.com/soc/">GSoC</a>';
        var title = L.control({{position: 'bottomleft'}});
        title.onAdd = function(map) {{
            this._div = L.DomUtil.create('div', 'ctl src');
            this.update();
            return this._div;
        }};
        title.update = function(props) {{
            this._div.innerHTML = src;
        }};
        title.addTo(map);


        // Add base layers
        L.control.layers(basemaps, overlaymaps, {{collapsed: false}}).addTo(map);

        // Fit to overlay bounds (SW and NE points with (lat, lon))
        map.fitBounds([[{}, {}], [{}, {}]]);

        </script>

        </body>
        </html>"#,
        title, center_lon, center_lat, begin_zoom, min_zoom, max_zoom, title, south, east, north, west
    );
    let mut html = File::create(&output_path).unwrap();
    writeln!(html, "{}", s).unwrap();
}

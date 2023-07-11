//! Types and helpers to work with vectors.
use crate::{
    errors::MapEngineError,
    tiles::{Tile, TILE_SIZE},
};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use rust_mapnik::mapnik::MapnikMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mapnik is used to render map tiles from vector data
/// Following are mapnik stylesheet definiations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineSymbolizer {
    #[serde(rename = "@stroke")]
    pub stroke: String,
    #[serde(rename = "@stroke-opacity")]
    pub stroke_opacity: f64,
    #[serde(rename = "@stroke-width")]
    pub stroke_width: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerSymbolizer {
    #[serde(rename = "@fill")]
    pub fill: String,
    #[serde(rename = "@fill-opacity")]
    pub fill_opacity: f64,
    #[serde(rename = "@stroke")]
    pub stroke: String,
    #[serde(rename = "@stroke-opacity")]
    pub stroke_opacity: f64,
    #[serde(rename = "@stroke-width")]
    pub stroke_width: f64,
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolygonSymbolizer {
    #[serde(rename = "@fill")]
    pub fill: String,
    #[serde(rename = "@fill-opacity")]
    pub fill_opacity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VectorSymbolizer {
    #[serde(rename = "MarkerSymbolizer")]
    Marker(MarkerSymbolizer),
    #[serde(rename = "LineSymbolizer")]
    Line(LineSymbolizer),
    #[serde(rename = "PolygonSymbolizer")]
    Polygon(PolygonSymbolizer),
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub enum CategoryValue {
//     IntegerValue(u32),
//     FloatValue(f64),
//     StringValue(String),
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub enum VectorStyle {
//     SingleSymbol(VectorSymbolizer),
//     Categorized(Vec<(CategoryValue, VectorSymbolizer)>),
//     Graduated(Vec<(f64, VectorSymbolizer)>),
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "$value")]
    pub symbolizer: Vec<VectorSymbolizer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "Rule")]
    pub rule: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleName {
    #[serde(rename = "$text")]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "$text")]
    pub val: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSource {
    #[serde(rename = "Parameter")]
    pub parameter: Vec<Parameter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@srs")]
    pub srs: Option<String>,

    #[serde(rename = "StyleName")]
    pub style_name: StyleName,

    #[serde(rename = "Datasource")]
    pub data_source: DataSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
    #[serde(rename = "@srs")]
    pub srs: String,

    #[serde(rename = "Style")]
    pub style: Vec<Style>,

    #[serde(rename = "Layer")]
    pub layer: Vec<Layer>,
}

/// A Vector data. 
///
/// could be any formats supported by gdal, eg. shapefile, geojson

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    // unique name/id
    pub name: String,

    // mapnik xml stylesheet
    pub style: Map,
}

impl Vector {
    pub fn mapnik_register(plugin_dir: String, font_dir: String) {
        MapnikMap::mapnik_register(plugin_dir, font_dir);
    }

    pub fn from(xml: String) -> Result<Self, MapEngineError> {
        Ok(Self {
            name: Uuid::new_v4().to_string(),
            style: from_str(&xml)?,
        })
    }

    pub fn tile(&self, tile: &Tile) -> Result<Vec<u8>, MapEngineError> {
        let mut m = MapnikMap::from_string(TILE_SIZE, TILE_SIZE, to_string(&self.style)?)?;

        let (minx, maxy, maxx, miny) = tile.bounds_xy();
        let buf = m.read_extent(minx, miny, maxx, maxy)?;
        m.free()?;
        Ok(buf)
    }
}

impl Map {
    pub fn from_xml(xml: &String) -> Result<Self, MapEngineError> {
        Ok(from_str(&xml)?)
    }

    pub fn to_xml(&self) -> Result<String, MapEngineError> {
        Ok(to_string(&self)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tiles::Tile;
    use std::fs;
    use std::fs::File;
    use std::io::Write; // bring trait into scope
    use std::path::PathBuf;

    #[test]
    fn test_map_from_xml() {
        let data = r#"
<Map srs="epsg:3857">
	<Style name="My Style">
		<Rule>
			<PolygonSymbolizer fill="red" fill-opacity="1"/>
			<LineSymbolizer stroke="blue" stroke-opacity="1" stroke-width="0.1"/>
			<MarkerSymbolizer fill="red" fill-opacity="1" stroke="blue" stroke-opacity="1" stroke-width="0.1" width="1" height="1"/>
		</Rule>
	</Style>
	<Layer name="" srs="epsg:4326">
		<StyleName>My Style</StyleName>
		<Datasource>
			<Parameter name="file">D:\ne_10m_admin_0_countries\ne_10m_admin_0_countries.shp</Parameter>
			<Parameter name="layer_by_index">0</Parameter>
			<Parameter name="type">ogr</Parameter>
		</Datasource>
	</Layer>
</Map>
        "#;

        let v: Map = from_str(data).unwrap();
        println!("{:?}", v);
    }

    #[test]
    fn test_map_to_xml() {
        let m = Map {
            srs: "epsg:3857".into(),
            style: vec![Style {
                name: "My Style".into(),
                rule: vec![Rule {
                    symbolizer: vec![
                        VectorSymbolizer::Polygon(PolygonSymbolizer {
                            fill: "red".into(),
                            fill_opacity: 1.0,
                        }),
                        VectorSymbolizer::Line(LineSymbolizer {
                            stroke: "blue".into(),
                            stroke_opacity: 1.0,
                            stroke_width: 0.1,
                        }),
                        VectorSymbolizer::Marker(MarkerSymbolizer {
                            fill: "red".into(),
                            fill_opacity: 1.0,
                            stroke: "blue".into(),
                            stroke_opacity: 1.0,
                            stroke_width: 0.1,
                            width: 1.0,
                            height: 1.0,
                        }),
                    ],
                }],
            }],
            layer: vec![Layer {
                name: None,
                srs: Some("epsg:4326".into()),
                style_name: StyleName {
                    name: "My Style".into(),
                },
                data_source: DataSource {
                    parameter: vec![
                        Parameter {
                            name: "file".into(),
                            val: "D:\\ne_10m_admin_0_countries\\ne_10m_admin_0_countries.shp"
                                .into(),
                        },
                        Parameter {
                            name: "layer_by_index".into(),
                            val: "0".into(),
                        },
                        Parameter {
                            name: "type".into(),
                            val: "ogr".into(),
                        },
                    ],
                },
            }],
        };
        let xml = to_string(&m).unwrap();
        let mut output = File::create("./test.xml").unwrap();
        write!(output, "{}", xml).unwrap();
    }

    #[test]
    fn test_vector_create_instance() {
        MapnikMap::mapnik_register(
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\bin\\mapnik\\input".into(),
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\share\\mapnik\\fonts".into(),
        );

        let data = r#"
<Map srs="epsg:3857">
	<Style name="My Style">
		<Rule>
			<PolygonSymbolizer fill="red" fill-opacity="1"/>
			<LineSymbolizer stroke="blue" stroke-opacity="1" stroke-width="0.1"/>
		</Rule>
	</Style>
	<Layer name="" srs="epsg:4326">
		<StyleName>My Style</StyleName>
		<Datasource>
			<Parameter name="file">D:\ne_10m_admin_0_countries\ne_10m_admin_0_countries.shp</Parameter>
			<Parameter name="layer_by_index">0</Parameter>
			<Parameter name="type">ogr</Parameter>
		</Datasource>
	</Layer>
</Map>
        "#;
        let v = Vector::from(data.to_string()).unwrap();
        println!("{:?}", v);
    }

    #[test]
    fn test_vector_tile() {
        MapnikMap::mapnik_register(
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\bin\\mapnik\\input".into(),
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\share\\mapnik\\fonts".into(),
        );

        let data = r#"
<Map srs="epsg:3857">
	<Style name="My Style">
		<Rule>
			<PolygonSymbolizer fill="red" fill-opacity="1"/>
			<LineSymbolizer stroke="blue" stroke-opacity="1" stroke-width="0.1"/>
		</Rule>
	</Style>
	<Layer srs="epsg:4326">
		<StyleName>My Style</StyleName>
		<Datasource>
			<Parameter name="file">D:\ne_10m_admin_0_countries\ne_10m_admin_0_countries.shp</Parameter>
			<Parameter name="layer_by_index">0</Parameter>
			<Parameter name="type">ogr</Parameter>
		</Datasource>
	</Layer>
</Map>
        "#;
        let v = Vector::from(data.to_string()).unwrap();

        let tile = Tile::new(0, 1, 1);
        let t = v.tile(&tile).unwrap();

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open("../rust-mapnik/0-0-1.png")
            .unwrap();
        file.write_all(&t).unwrap();
    }
}

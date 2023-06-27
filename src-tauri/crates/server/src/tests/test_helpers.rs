use std::io::Write;
use tempfile::Builder;

pub fn create_config(content: Option<&[u8]>, temp: bool) -> String {
    let default_content = br#"
[
  {
    "name": "chile_optimised",
    "path": "../map-engine/src/tests/data/chile_optimised.tif"
  }
]
"#;
    let content = content.unwrap_or(default_content);
    let mut file = if temp {
        Builder::new()
            .prefix("map_engine_server_conf_")
            .suffix(".json")
            .tempfile()
            .unwrap()
    } else {
        Builder::new()
            .prefix("map_engine_server_conf_")
            .suffix(".json")
            .tempfile_in("./")
            .unwrap()
    };
    file.write_all(content).unwrap();
    let (_, path) = file.keep().unwrap();
    path.into_os_string().into_string().unwrap()
}

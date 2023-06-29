use crate::{mapsettings::MapSettings, style::Style};
use map_engine::{
    cmap::{ColourDefinition, Composite},
    errors::MapEngineError,
    gdal::Dataset,
    raster::Raster,
    vector::Vector,
    windows::Window,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// The shared application state.
#[derive(Clone, Debug)]
pub struct State {
    /// Settings for the served maps
    pub maps: Arc<RwLock<HashMap<String, MapSettings>>>,
    pub rasters: Arc<RwLock<HashMap<String, Raster>>>,
    pub styles: Arc<RwLock<HashMap<String, Composite>>>,

    // mapnik maps
    pub vectors: Arc<RwLock<HashMap<String, Vector>>>,
}

impl State {
    /// Create the initial shared state.
    ///
    /// # Arguments
    ///
    /// * `conf_path` - Path to the config file.
    pub fn from_file(conf_path: &str) -> Result<Self, MapEngineError> {
        if conf_path == "" {
            return State::init_state(vec![]);
        }

        let path = Path::new(conf_path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let settings: Vec<MapSettings> = serde_json::from_reader(reader)?;

        State::init_state(settings)
    }

    fn validate_no_data_values(src: &Dataset, map: &mut MapSettings) -> Result<(), MapEngineError> {
        if map.no_data_value.is_none() {
            let no_data_values = (1..=src.raster_count())
                .map(|i| {
                    let band = src.rasterband(i).unwrap();
                    band.no_data_value().unwrap_or(0.0)
                })
                .collect();
            map.no_data_value = Some(no_data_values);
        } else if map.no_data_value.as_ref().unwrap().len() != src.raster_count() as usize {
            return Err(MapEngineError::Msg(format!(
                "The raster has {} bands. Expected the same number of no_data values",
                src.raster_count()
            )));
        };
        Ok(())
    }

    fn validate_bands(map: &MapSettings) -> Result<(), MapEngineError> {
        let map_name: String = map.name.clone();
        match map.style.as_ref() {
            Some(Style {
                colours: Some(col_def),
                bands: Some(bands),
                ..
            }) => match col_def {
                ColourDefinition::RGB(_, _) => {
                    if bands.len() != 3 {
                        return Err(MapEngineError::Msg(format!(
                            "To use a RGB style you need to provide 3 bands for map `{}`",
                            map_name
                        )));
                    }
                }
                _ => {
                    if bands.len() != 1 {
                        return Err(MapEngineError::Msg(format!(
                            "To use a Gradient style you need to provide 1 band for map `{}`",
                            map_name
                        )));
                    }
                }
            },
            Some(Style {
                colours: Some(_),
                bands: None,
                ..
            }) => {
                return Err(MapEngineError::Msg(format!(
                    "You need to provide selected bands for map `{}`",
                    map_name
                )))
            }
            _ => {}
        };
        Ok(())
    }

    fn fill_style(map: &mut MapSettings) -> Result<(), MapEngineError> {
        let default_syle = Style::default();
        let default_bands = default_syle.bands.clone().unwrap();
        let default_vmin = default_syle.vmin.unwrap();
        let default_vmax = default_syle.vmax.unwrap();
        let mut style = map.style.clone().unwrap_or(default_syle);
        let bands = style.bands.clone().unwrap_or(default_bands);
        style.bands = Some(bands);
        let vmin = style.vmin.unwrap_or(default_vmin);
        let vmax = style.vmax.unwrap_or(default_vmax);
        style.vmin = Some(vmin);
        style.vmax = Some(vmax);
        map.style = Some(style);
        Ok(())
    }

    fn init_state(settings: Vec<MapSettings>) -> Result<Self, MapEngineError> {
        let mut maps = HashMap::new();
        let mut rasters = HashMap::new();
        let mut styles = HashMap::new();
        let vectors = HashMap::new();
        for mut map in settings.into_iter() {
            let path = Path::new(&map.path);
            let src = Dataset::open(path)?;
            if map.extent.is_none() {
                let (raster_w, raster_h) = src.raster_size();
                let raster_win = Window::new(0, 0, raster_w, raster_h);
                map.extent = Some(raster_win);
            };
            map.driver_name = Some(src.driver().short_name());

            let raster = Raster::from_src(path.to_path_buf(), &src)?;

            let geo = raster.geo();
            map.geotransform = Some(geo.clone());

            let spatial_ref = raster.spatial_ref()?;
            let spatial_ref_code = spatial_ref.auth_code()?;
            map.spatial_ref_code = Some(spatial_ref_code as i32);
            let spatial_units = spatial_ref.linear_units_name()?;
            map.spatial_units = Some(spatial_units);

            State::validate_no_data_values(&src, &mut map)?;
            State::validate_bands(&map)?;

            State::fill_style(&mut map)?;

            let name = map.name.clone();
            let style_gradient = map.to_composite();
            styles.insert(name.clone(), style_gradient);
            maps.insert(name.clone(), map);
            rasters.insert(name.clone(), raster);
        }

        Ok(State {
            maps: Arc::new(RwLock::new(maps)),
            rasters: Arc::new(RwLock::new(rasters)),
            styles: Arc::new(RwLock::new(styles)),
            vectors: Arc::new(RwLock::new(vectors)),
        })
    }

    pub fn add_map(&self, map_setting: MapSettings) -> Result<MapSettings, MapEngineError> {
        let map: &mut MapSettings = &mut map_setting.clone();
        if map.name == "" {
            map.name = Uuid::new_v4().to_string()
        }

        let path = Path::new(&map.path);
        let src = Dataset::open(path)?;
        if map.extent.is_none() {
            let (raster_w, raster_h) = src.raster_size();
            let raster_win = Window::new(0, 0, raster_w, raster_h);
            map.extent = Some(raster_win);
        };
        map.driver_name = Some(src.driver().short_name());

        let raster = Raster::from_src(path.to_path_buf(), &src)?;

        let geo = raster.geo();
        map.geotransform = Some(geo.clone());

        let spatial_ref = raster.spatial_ref()?;
        let spatial_ref_code = spatial_ref.auth_code()?;
        map.spatial_ref_code = Some(spatial_ref_code as i32);
        let spatial_units = spatial_ref.linear_units_name()?;
        map.spatial_units = Some(spatial_units);

        State::validate_no_data_values(&src, map)?;
        State::validate_bands(&map)?;

        State::fill_style(map)?;

        let name = map.name.clone();
        let style_gradient = map.to_composite();
        self.styles.write().unwrap().insert(name.clone(), style_gradient);
        self.maps.write().unwrap().insert(name.clone(), map.clone());
        self.rasters.write().unwrap().insert(name.clone(), raster);

        Ok(map.clone())
    }

    pub fn add_map_vector(&self, xml: String) -> Result<String, MapEngineError> {
        let v = Vector::from(xml)?;
        let name = v.name.clone();
        self.vectors.write().unwrap().insert(name.clone(), v);

        Ok(name)
    }

    pub fn get_map(&self, map_name: &str) -> Result<MapSettings, MapEngineError> {
        if self.maps.read().unwrap().contains_key(map_name) {
            Ok(self
                .maps
                .read()
                .unwrap()
                .get(map_name)
                .expect("State does not contain the map")
                .clone())
        } else {
            return Err(MapEngineError::Msg(format!(
                "The map {:?} does not exist",
                map_name
            )));
        }
    }

    pub fn get_raster(&self, map_name: &str) -> Result<Raster, MapEngineError> {
        if self.maps.read().unwrap().contains_key(map_name) {
            Ok(self
                .rasters
                .read()
                .unwrap()
                .get(map_name)
                .expect("State does not contain the raster")
                .clone())
        } else {
            return Err(MapEngineError::Msg(format!(
                "The raster {:?} does not exist",
                map_name
            )));
        }
    }

    pub fn get_vector(&self, map_name: &str) -> Result<Vector, MapEngineError> {
        if self.vectors.read().unwrap().contains_key(map_name) {
            Ok(self
                .vectors
                .read()
                .unwrap()
                .get(map_name)
                .expect("State does not contain the raster")
                .clone())
        } else {
            return Err(MapEngineError::Msg(format!(
                "The raster {:?} does not exist",
                map_name
            )));
        }
    }


    pub fn get_style(&self, map_name: &str) -> Result<Composite, MapEngineError> {
        if self.maps.read().unwrap().contains_key(map_name) {
            Ok(self
                .styles
                .read()
                .unwrap()
                .get(map_name)
                .expect("State does not contain the style")
                .clone())
        } else {
            return Err(MapEngineError::Msg(format!(
                "The style {:?} does not exist",
                map_name
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_bands() {
        let style = Style {
            colours: Some(ColourDefinition::RGB([0., 0., 0.], [1., 1., 1.])),
            bands: Some(vec![1]),
            ..Default::default()
        };
        let mapsettings = MapSettings {
            name: "test".to_string(),
            style: Some(style),
            ..Default::default()
        };
        let valid = State::validate_bands(&mapsettings);
        assert!(valid.is_err());
        if let Err(MapEngineError::Msg(msg)) = valid {
            let expected =
                "To use a RGB style you need to provide 3 bands for map `test`".to_string();
            assert_eq!(msg, expected);
        };

        let style = Style {
            colours: Some(ColourDefinition::Colours(vec![
                (0., 0., 0., 1.).into(),
                (1., 1., 1., 1.).into(),
            ])),
            bands: Some(vec![1, 2, 3]),
            ..Default::default()
        };
        let mapsettings = MapSettings {
            name: "test".to_string(),
            style: Some(style),
            ..Default::default()
        };
        let valid = State::validate_bands(&mapsettings);
        assert!(valid.is_err());
        if let Err(MapEngineError::Msg(msg)) = valid {
            let expected =
                "To use a Gradient style you need to provide 1 band for map `test`".to_string();
            assert_eq!(msg, expected);
        };

        let style = Style {
            colours: Some(ColourDefinition::RGB([0., 0., 0.], [1., 1., 1.])),
            bands: None,
            ..Default::default()
        };
        let mapsettings = MapSettings {
            name: "test".to_string(),
            style: Some(style),
            ..Default::default()
        };
        let valid = State::validate_bands(&mapsettings);
        assert!(valid.is_err());
        if let Err(MapEngineError::Msg(msg)) = valid {
            let expected = "You need to provide selected bands for map `test`".to_string();
            assert_eq!(msg, expected);
        };

        let style = Style {
            colours: Some(ColourDefinition::Colours(vec![
                (0., 0., 0., 1.).into(),
                (1., 1., 1., 1.).into(),
            ])),
            bands: Some(vec![1]),
            ..Default::default()
        };
        let mapsettings = MapSettings {
            name: "test".to_string(),
            style: Some(style),
            ..Default::default()
        };
        assert!(State::validate_bands(&mapsettings).is_ok());
    }

    #[test]
    fn test_validate_no_data_values() {
        let path = Path::new("../map-engine/src/tests/data/chile_optimised.tif");
        let src = Dataset::open(path).unwrap();

        let mut mapsettings = MapSettings {
            name: "test".to_string(),
            no_data_value: Some(vec![0.0]),
            ..Default::default()
        };
        let valid = State::validate_no_data_values(&src, &mut mapsettings);
        assert!(valid.is_err());
        if let Err(MapEngineError::Msg(msg)) = valid {
            let expected =
                "The raster has 2 bands. Expected the same number of no_data values".to_string();
            assert_eq!(msg, expected);
        };

        let mut mapsettings = MapSettings {
            name: "test".to_string(),
            ..Default::default()
        };
        let valid = State::validate_no_data_values(&src, &mut mapsettings);
        assert!(valid.is_ok());
        assert_eq!(mapsettings.no_data_value.unwrap(), [0., 0.]);
    }
}

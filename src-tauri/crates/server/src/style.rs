//! Types and helpers to style pixels.
use map_engine::cmap::{inferno, viridis, ColourDefinition, Composite};
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    /// Name of an available colour map (See: [`map_engine::cmap`])
    pub name: Option<String>,
    pub colours: Option<ColourDefinition>,
    /// Minimum pixel value
    pub vmin: Option<f64>,
    /// Maximum pixel value
    pub vmax: Option<f64>,
    /// Band index
    pub bands: Option<Vec<isize>>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            name: None,
            colours: Some(ColourDefinition::Colours(vec![
                (0., 0., 0., 1.).into(),
                (1., 1., 1., 1.).into(),
            ])),
            vmin: Some(0.),
            vmax: Some(1.),
            bands: Some(vec![1]),
        }
    }
}

impl From<&Style> for Composite {
    fn from(style: &Style) -> Composite {
        let vmin = style.vmin.expect("vmin not available in Style");
        let vmax = style.vmax.expect("vmax not available in Style");
        match style {
            Style {
                name: Some(name), ..
            } => match &name[..] {
                "viridis" => Composite::new_gradient(vmin, vmax, &viridis),
                "inferno" => Composite::new_gradient(vmin, vmax, &inferno),
                _ => Composite::new_gradient(vmin, vmax, &viridis),
            },
            Style {
                colours: Some(col_def),
                ..
            } => match col_def {
                ColourDefinition::Colours(col_vec) => Composite::new_custom_gradient(
                    vmin,
                    vmax,
                    col_vec.clone().into_iter().map(Into::into).collect(),
                ),
                ColourDefinition::ColoursAndBreaks(cols_and_breaks) => {
                    Composite::new_gradient_with_breaks(cols_and_breaks.clone())
                }
                ColourDefinition::RGB(vmin, vmax) => {
                    Composite::new_rgb(vmin.to_vec(), vmax.to_vec())
                }
                ColourDefinition::Discrete(col_vec) => {
                    Composite::new_discrete_palette(col_vec.clone())
                }
            },
            _ => Composite::new_gradient(vmin, vmax, &viridis),
        }
    }
}

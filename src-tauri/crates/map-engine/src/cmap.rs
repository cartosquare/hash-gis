//! Types and functions to style a `Raster`.
// cmaps from python
// Eg: `matplotlib.cm.get_cmap('viridis', 7).colors`
// Potentialy use this data: https://github.com/matplotlib/matplotlib/blob/c06e8709dde6504d396349c0c80ef019c88c3927/lib/matplotlib/_cm_listed.py
use crate::colour::{Colour, RgbaComponents};
use ndarray::Array;
use palette::{
    encoding::{Linear, Srgb},
    rgb::Rgb,
    Alpha, Gradient, LinSrgba,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

/// A linear RGBA gradient
pub type GradientLinearRGBA = Gradient<Alpha<Rgb<Linear<Srgb>, f64>, f64>>;

const VIRIDIS7: [Colour; 7] = [
    Colour::Seq((0.267004, 0.004874, 0.329415, 1.)),
    Colour::Seq((0.267968, 0.223549, 0.512008, 1.)),
    Colour::Seq((0.190631, 0.407061, 0.556089, 1.)),
    Colour::Seq((0.127568, 0.566949, 0.550556, 1.)),
    Colour::Seq((0.20803, 0.718701, 0.472873, 1.)),
    Colour::Seq((0.565498, 0.84243, 0.262877, 1.)),
    Colour::Seq((0.993248, 0.906157, 0.143936, 1.)),
];

const INFERNO7: [Colour; 7] = [
    Colour::Seq((0.001462, 0.000466, 0.013866, 1.)),
    Colour::Seq((0.197297, 0.0384, 0.367535, 1.)),
    Colour::Seq((0.472328, 0.110547, 0.428334, 1.)),
    Colour::Seq((0.735683, 0.215906, 0.330245, 1.)),
    Colour::Seq((0.929644, 0.411479, 0.145367, 1.)),
    Colour::Seq((0.986175, 0.713153, 0.103863, 1.)),
    Colour::Seq((0.988362, 0.998364, 0.644924, 1.)),
];

// TODO: Is there any way to generate the documentation (link) dynamically?
macro_rules! gen_cmap_fn {
    ($(#[$attr:meta])* => ($name:ident, $colours:expr)) => {
        $(#[$attr])*
        pub fn $name( vmin: f64, vmax: f64) -> GradientLinearRGBA {
            make_gradient(vmin, vmax, &$colours)
        }
    }
}

gen_cmap_fn! {
/// Create Gradient: ![viridis](https://gitlab.com/spadarian/map_engine/-/raw/master/assets/docs/cmaps/viridis.png)
    => (viridis, VIRIDIS7)
}

gen_cmap_fn! {
/// Create Gradient: ![inferno](https://gitlab.com/spadarian/map_engine/-/raw/master/assets/docs/cmaps/inferno.png)
    => (inferno, INFERNO7)
}

/// Create a colour gradient.
///
/// The colour space is partitioned equally.
///
/// # Arguments
///
/// * `vmin` - Lower limit (pixel value) of the gradient.
/// * `vmax` - Upper limit (pixel value) of the gradient.
/// * `rgba` - Sequence of [`Colour`].
fn make_gradient(vmin: f64, vmax: f64, rgba: &[Colour]) -> GradientLinearRGBA {
    let nums = Array::linspace(vmin, vmax, rgba.len());
    let cols = nums
        .iter()
        .zip(rgba)
        .map(|(v, comps)| {
            (
                *v,
                LinSrgba::from_components(Into::<(f64, f64, f64, f64)>::into(comps.clone())),
            )
        })
        .collect();
    Gradient::with_domain(cols)
}

fn make_gradient_with_breaks(nums: &[(f64, Colour)]) -> GradientLinearRGBA {
    let cols = nums
        .iter()
        .map(|(v, comps)| {
            (
                *v,
                LinSrgba::from_components(Into::<(f64, f64, f64, f64)>::into(comps.clone())),
            )
        })
        .collect();
    Gradient::with_domain(cols)
}

/// Types of palettes supported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColourDefinition {
    /// A discrete palette. See [`Composite::new_discrete_palette`].
    Discrete(Vec<(isize, Colour)>),
    /// An equally-spaced gradient. See [`Composite::new_custom_gradient`].
    Colours(Vec<Colour>),
    /// A gradient with custom breaks. See [`Composite::new_gradient_with_breaks`].
    ColoursAndBreaks(Vec<(f64, Colour)>),
    /// A RGB composite. See [`Composite::new_rgb`].
    RGB([f64; 3], [f64; 3]),
}

/// Object to style `RawPixels`.
#[derive(Debug, Clone)]
pub struct Composite {
    vmin: Option<Vec<f64>>,
    vmax: Option<Vec<f64>>,
    gradient: Option<GradientLinearRGBA>,
    hashmap: Option<HashMap<isize, RgbaComponents>>,
    display: Option<String>,
    colour_definition: ColourDefinition,
    len: usize,
}

impl Default for Composite {
    fn default() -> Self {
        let grad: Gradient<Alpha<Rgb<Linear<Srgb>, f64>, f64>> = viridis(0.0, 1.0);
        Self {
            vmin: Some(vec![0.0]),
            vmax: Some(vec![1.0]),
            gradient: Some(grad),
            hashmap: None,
            display: Some("Gradient".to_string()),
            colour_definition: ColourDefinition::Colours(vec![
                (0.0, 0.0, 0.0, 0.0).into(),
                (1.0, 1.0, 1.0, 1.0).into(),
            ]),
            len: 1,
        }
    }
}

impl std::fmt::Display for Composite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.display.as_ref().unwrap_or(&"".to_string()))
    }
}

impl Composite {
    /// Create a RGB `Composite` that maps 3 pixel values (from 3 different bands) into RGBA
    /// components.
    ///
    /// # Example
    /// ```
    /// use map_engine::cmap::{Composite, HandleGet, viridis};
    /// let comp = Composite::new_rgb(vec![0.0, 0.0, 0.0], vec![100.0, 100.0, 100.0]);
    /// assert_eq!(comp.get(&[0.0, 50.0, 100.0], None), [0, 127, 255, 255]);
    /// ```
    pub fn new_rgb(vmin: Vec<f64>, vmax: Vec<f64>) -> Self {
        Self {
            vmin: Some(vmin.clone()),
            vmax: Some(vmax.clone()),
            display: Some("RGBComposite".to_string()),
            gradient: None,
            colour_definition: ColourDefinition::RGB(
                vmin.try_into().unwrap_or_else(|v: Vec<_>| {
                    panic!("Expected a Vec of length {} but it was {}", 3, v.len())
                }),
                vmax.try_into().unwrap_or_else(|v: Vec<_>| {
                    panic!("Expected a Vec of length {} but it was {}", 3, v.len())
                }),
            ),
            len: 3,
            ..Default::default()
        }
    }

    /// Create a `Composite` that maps 1 pixel value into RGBA using the provided function.
    ///
    /// You can use [one of the functions provided in the cmap module](/.#functions).
    ///
    /// # Example
    /// ```
    /// use map_engine::cmap::{Composite, HandleGet, viridis};
    /// let comp = Composite::new_gradient(0.0, 100.0, &viridis);
    /// assert_eq!(comp.get(&[0.0], None), [68, 1, 84, 255]);
    /// ```
    pub fn new_gradient(
        vmin: f64,
        vmax: f64,
        cmap_f: &'static dyn Fn(f64, f64) -> GradientLinearRGBA,
    ) -> Self {
        let grad = cmap_f(vmin, vmax);
        Self {
            vmin: Some(vec![vmin]),
            vmax: Some(vec![vmax]),
            gradient: Some(grad),
            display: Some("Gradient".to_string()),
            len: 1,
            ..Default::default()
        }
    }

    /// Create an equally-spaced `Composite` that maps 1 pixel value into RGBA using a sequence of [`Colour`].
    ///
    /// # Example
    /// ```
    /// use map_engine::{
    ///     colour::Colour,
    ///     cmap::{Composite, HandleGet},
    /// };
    /// let comp = Composite::new_custom_gradient(0.0, 100.0, vec![
    ///     Colour::from((255, 0, 0, 255)), // red
    ///     Colour::from((0, 0, 255, 255)), // blue
    /// ]);
    /// assert_eq!(comp.get(&[50.0], None), [127, 0, 127, 255]); // purple
    /// ```
    pub fn new_custom_gradient(vmin: f64, vmax: f64, colours: Vec<Colour>) -> Self {
        let grad = make_gradient(vmin, vmax, &colours);
        Self {
            vmin: Some(vec![vmin]),
            vmax: Some(vec![vmax]),
            gradient: Some(grad),
            display: Some("Gradient".to_string()),
            colour_definition: ColourDefinition::Colours(colours),
            len: 1,
            ..Default::default()
        }
    }

    /// Create an `Composite` with custom breaks that maps 1 pixel value into RGBA.
    ///
    /// # Example
    /// ```
    /// use map_engine::{
    ///     colour::Colour,
    ///     cmap::{Composite, HandleGet},
    /// };
    /// let comp = Composite::new_gradient_with_breaks(vec![
    ///     (0.0, Colour::from((255, 0, 0, 255))), // red
    ///     (25.0, Colour::from((127, 0, 127, 255))), // purple shifted to the red
    ///     (100.0, Colour::from((0, 0, 255, 255))), // blue
    /// ]);
    /// assert_eq!(comp.get(&[25.0], None), [127, 0, 127, 255]); // purple
    /// ```
    pub fn new_gradient_with_breaks(cols_and_breaks: Vec<(f64, Colour)>) -> Self {
        let grad = make_gradient_with_breaks(&cols_and_breaks);
        Self {
            gradient: Some(grad),
            display: Some("GradientWithBreaks".to_string()),
            colour_definition: ColourDefinition::ColoursAndBreaks(cols_and_breaks),
            len: 1,
            ..Default::default()
        }
    }

    /// Create an discrete `Composite` that maps 1 pixel value into RGBA.
    ///
    /// # Example
    /// ```
    /// use map_engine::{
    ///     colour::Colour,
    ///     cmap::{Composite, HandleGet},
    /// };
    /// let comp = Composite::new_discrete_palette(vec![
    ///     (0, Colour::from((255, 0, 0, 255))), // red
    ///     (1, Colour::from((0, 255, 0, 255))), // green
    ///     (2, Colour::from((0, 0, 255, 255))), // blue
    /// ]);
    /// assert_eq!(comp.get(&[0.0], None), [255, 0, 0, 255]); // red
    /// assert_eq!(comp.get(&[3.0], None), [0, 0, 0, 0]); // transparent if not defined
    /// ```
    pub fn new_discrete_palette(cols_and_breaks: Vec<(isize, Colour)>) -> Self {
        let hashmap = cols_and_breaks
            .clone()
            .into_iter()
            .map(|(b, c)| (b, c.into()))
            .collect();

        Self {
            display: Some("DiscretePalette".to_string()),
            hashmap: Some(hashmap),
            colour_definition: ColourDefinition::Discrete(cols_and_breaks),
            len: 1,
            ..Default::default()
        }
    }

    pub(crate) fn is_contiguous(&self) -> bool {
        !matches!(self.colour_definition, ColourDefinition::RGB(_, _))
    }

    /// Number of bands supported by the `Composite`.
    ///
    /// ⚠ This will probably be deprecated once we enforce the number of bands using the type
    /// system ⚠
    pub fn n_bands(&self) -> usize {
        self.len
    }
}

fn gradient_handle(comp: &Composite, values: &[f64], no_data_values: Option<&[f64]>) -> [u8; 4] {
    let grad = comp.gradient.as_ref().unwrap();
    let col = grad.get(values[0]);
    let (r, g, b, a) = col.into_components();
    let a = if let Some(ndv) = no_data_values {
        assert!(
            ndv.len() == 1,
            "To use a {} style you need to provide 1 `no_data` value",
            comp
        );
        if (values[0] - ndv[0]).abs() < f64::EPSILON {
            0u8
        } else {
            (a * 255.0) as u8
        }
    } else {
        (a * 255.0) as u8
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a]
}

fn rgb_handle(comp: &Composite, values: &[f64], no_data_values: Option<&[f64]>) -> [u8; 4] {
    let norm: Vec<f64> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if v > &comp.vmax.as_ref().unwrap()[i] {
                1.0
            } else if v < &comp.vmin.as_ref().unwrap()[i] {
                0.0
            } else {
                (v - comp.vmin.as_ref().unwrap()[i])
                    / (comp.vmax.as_ref().unwrap()[i] - comp.vmin.as_ref().unwrap()[i])
            }
        })
        .collect();
    let (r, g, b, a) = (norm[0], norm[1], norm[2], 1.0);
    let a = if let Some(no_data_values) = no_data_values {
        assert!(
            no_data_values.len() == 3,
            "To use a {} style you need to provide 3 `no_data` values",
            comp
        );
        if no_data_values
            .iter()
            .zip(values)
            .all(|(ndv, v)| (v - ndv).abs() < f64::EPSILON)
        {
            0u8
        } else {
            (a * 255.0) as u8
        }
    } else {
        (a * 255.0) as u8
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a]
}

fn hashmap_handle(comp: &Composite, values: &[f64]) -> [u8; 4] {
    let val = values[0];
    let hash = comp.hashmap.as_ref().unwrap();
    let (r, g, b, a) = hash
        .get(&(val.trunc() as isize))
        .unwrap_or(&(0.0, 0.0, 0.0, 0.0));
    [
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    ]
}

/// Get a RGBA colour given a raw pixel value
pub trait HandleGet {
    /// Get a RGBA colour given a raw pixel value.
    ///
    /// The length of `values` might vary, usually 1 or 3, depending on the `ColourDefinition`
    /// contained within [`Composite`].
    ///
    /// ⚠ This will probably change once we enforce the length using the type system ⚠
    fn get(&self, values: &[f64], no_data_values: Option<&[f64]>) -> [u8; 4];
}

impl HandleGet for Composite {
    fn get(&self, values: &[f64], no_data_values: std::option::Option<&[f64]>) -> [u8; 4] {
        match &self.colour_definition {
            ColourDefinition::Discrete(_) => hashmap_handle(self, values),
            ColourDefinition::Colours(_) | ColourDefinition::ColoursAndBreaks(_) => {
                gradient_handle(self, values, no_data_values)
            }
            ColourDefinition::RGB(_, _) => rgb_handle(self, values, no_data_values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_cmap {
        ($test_name:ident, $fn:expr, $colours:expr) => {
            #[test]
            fn $test_name() {
                let len = $colours.len();
                let grad = $fn(0.0, (len - 1) as f64);
                let expected = $colours;
                let from_grad: Vec<Colour> = (0..len)
                    .into_iter()
                    .map(|i| grad.get(i as f64).into())
                    .collect();
                assert_eq!(from_grad, expected)
            }
        };
    }

    test_cmap!(test_viridis, viridis, VIRIDIS7);
    test_cmap!(test_inferno, inferno, INFERNO7);

    #[test]
    fn test_colour_definition_is_deserialized() {
        let expected_col_def = ColourDefinition::Colours(vec![
            (1., 0., 0., 1.).into(),
            (0., 1., 0., 1.).into(),
            (0., 0., 1., 1.).into(),
        ]);

        let s = r#"[
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0]
        ]"#;
        let col_def: ColourDefinition = serde_json::from_str(s).unwrap();
        assert_eq!(col_def, expected_col_def);
        let s = r#"[
        "ff0000ff",
        "00ff00ff",
        "0000ffff"
        ]"#;
        let col_def: ColourDefinition = serde_json::from_str(s).unwrap();
        assert_eq!(col_def, expected_col_def);
    }
}

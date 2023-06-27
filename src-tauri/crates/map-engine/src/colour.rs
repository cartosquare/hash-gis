//! Types and deserializer for colours.
use palette::{
    encoding::{Linear, Srgb},
    rgb::Rgb,
    Alpha,
};
use serde::{
    de::{self, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{convert::TryFrom, fmt, num::ParseIntError};

pub(crate) type RgbaComponents = (f64, f64, f64, f64);
type HexString = String;

/// Colour representations.
///
/// We support different ways of creating colours:
/// ```
/// use map_engine::colour::Colour;
/// use std::convert::TryFrom;
/// # use map_engine::errors::MapEngineError;
///
/// # fn main() -> Result<(), MapEngineError> {
/// // Using components in the range 0.0..=1.0
/// Colour::Seq((1.0, 1.0, 1.0, 1.0));
/// // Using components in the range 0..=255
/// Colour::from((255, 255, 255, 255));
/// // Using a hex string (we support multiple formats)
/// Colour::try_from("FFFFFF")?; // We assume 100% opacity
/// Colour::try_from("FFFFFFFF")?;
/// Colour::try_from("#FFFFFFFF")?;
/// Colour::try_from("#ffffffff")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Colour {
    Seq(RgbaComponents),
    /// Don't use this one directly. Prefer any of the `.from()` methods described above.
    Hex(HexString),
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColourVisitor;
        impl<'de> Visitor<'de> for ColourVisitor {
            type Value = Colour;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "4 values (r, g, b, a) in the range 0.0-1.0 or 0-255, or a hex colour"
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match decode_hex(s) {
                    Ok(c) => Ok(Colour::Seq(c)),
                    Err(_) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
                }
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let r: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let b: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let a: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                if (0f64..=1.0).contains(&r)
                    && (0f64..=1.0).contains(&g)
                    && (0f64..=1.0).contains(&b)
                    && (0f64..=1.0).contains(&a)
                {
                    Ok(Colour::Seq((r, g, b, a)))
                } else if (0f64..=255.0).contains(&r)
                    && (0f64..=255.0).contains(&g)
                    && (0f64..=255.0).contains(&b)
                    && (0f64..=255.0).contains(&a)
                {
                    Ok(Colour::Seq((
                        (r / 255.0),
                        (g / 255.0),
                        (b / 255.0),
                        (a / 255.0),
                    )))
                } else {
                    Err(de::Error::invalid_value(Unexpected::Seq, &self))
                }
            }
        }
        deserializer.deserialize_any(ColourVisitor)
    }
}

fn decode_hex(s: &str) -> Result<RgbaComponents, ParseIntError> {
    let s = s.trim_start_matches('#');
    let mut v: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<_, _>>()?;
    if v.len() == 3 {
        v.extend_from_slice(&[255])
    }
    Ok((
        v[0] as f64 / 255.0,
        v[1] as f64 / 255.0,
        v[2] as f64 / 255.0,
        v[3] as f64 / 255.0,
    ))
}

impl From<RgbaComponents> for Colour {
    fn from(comp: RgbaComponents) -> Self {
        Self::Seq(comp)
    }
}

impl From<(u8, u8, u8, u8)> for Colour {
    fn from(vals: (u8, u8, u8, u8)) -> Self {
        Self::Seq((
            vals.0 as f64 / 255.0,
            vals.1 as f64 / 255.0,
            vals.2 as f64 / 255.0,
            vals.3 as f64 / 255.0,
        ))
    }
}

impl TryFrom<&str> for Colour {
    type Error = ParseIntError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self::Seq(decode_hex(s)?))
    }
}

impl From<Colour> for RgbaComponents {
    fn from(col: Colour) -> Self {
        match col {
            Colour::Seq(comp) => comp,
            // Colour::Hex(s) => decode_hex(&s).expect("Cannot convert hex colour"),
            // Is there any way of check this at compile time?
            Colour::Hex(str) => panic!("Prefer Colour::try_from({:?})", str),
        }
    }
}

impl From<Alpha<Rgb<Linear<Srgb>, f64>, f64>> for Colour {
    fn from(comp: Alpha<Rgb<Linear<Srgb>, f64>, f64>) -> Self {
        Self::Seq(comp.into_components())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_decode_hex() {
        let expected_comp: RgbaComponents = (1., 0., 0., 1.);
        assert_eq!(decode_hex("ff0000ff").unwrap(), expected_comp);
        assert_eq!(decode_hex("FF0000FF").unwrap(), expected_comp);
        assert_eq!(decode_hex("#ff0000ff").unwrap(), expected_comp);
        // Assumes full opacity
        assert_eq!(decode_hex("#ff0000").unwrap(), expected_comp);
    }

    #[test]
    fn test_colour_from() {
        assert_eq!(
            Colour::try_from("ff0000ff").unwrap(),
            "ff0000ff".try_into().unwrap()
        );
        assert_eq!(
            Colour::try_from("ff0000ff").unwrap(),
            (255, 0, 0, 255).into()
        );
        assert!(Colour::try_from("ff0000gg").is_err());
    }

    #[test]
    fn test_colour_is_deserialized() {
        let expected_col = Colour::Seq((1., 0., 0., 1.));
        let expected_comp: RgbaComponents = (1., 0., 0., 1.);

        let s = "\"ff0000ff\"";
        let col: Colour = serde_json::from_str(s).unwrap();
        assert_eq!(col, expected_col);
        let comp: RgbaComponents = col.into();
        assert_eq!(comp, expected_comp);

        let s = r#"
        [255, 0, 0, 255]
        "#;
        let col: Colour = serde_json::from_str(s).unwrap();
        assert_eq!(col, expected_col);
        let comp: RgbaComponents = col.into();
        assert_eq!(comp, expected_comp);

        let s = r#"
        [1.0, 0.0, 0.0, 1.0]
        "#;
        let col: Colour = serde_json::from_str(s).unwrap();
        assert_eq!(col, expected_col);
        let comp: RgbaComponents = col.into();
        assert_eq!(comp, expected_comp);
    }

    #[test]
    #[should_panic]
    fn test_hex_colour_panics() {
        let _: RgbaComponents = Colour::Hex("ff0000ff".to_string()).into();
    }

    #[test]
    fn test_colour_deserialized_fails() {
        let s = "\"ff0000gg\"";
        let col: Result<Colour, _> = serde_json::from_str(s);
        let expected_msg =
            "invalid value: string \"ff0000gg\", expected 4 values (r, g, b, a) in the range 0.0-1.0 or 0-255, or a hex colour at line 1 column 10";
        if let Err(err) = col {
            assert_eq!(format!("{}", err), expected_msg.to_string())
        };
    }
}

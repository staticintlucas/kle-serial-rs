use std::fmt;

use csscolorparser::Color as CssColor;
use serde::{
    de::{Error, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    utils::{Alignment, FontSize},
    Color,
};

fn color_from_str<'de, D>(value: &str) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    csscolorparser::parse(value)
        .map(|c| CssColor::to_rgba8(&c))
        .map(|[r, g, b, a]| Color { r, g, b, a })
        .map_err(|_| D::Error::invalid_value(Unexpected::Str(value), &"a CSS color value"))
}

fn de_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .as_deref()
        .map(color_from_str::<D>)
        .transpose()
}

// Kle color arrays are just \n delimited strings, so we use this function to turn them into Vecs
fn de_nl_delimited_colors<'de, D>(deserializer: D) -> Result<Option<Vec<Option<Color>>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|string| {
            string
                .lines()
                .map(|c| (!c.is_empty()).then(|| color_from_str::<D>(c)).transpose())
                .collect()
        })
        .transpose()
}

#[derive(Deserialize, Default, Debug, Clone)]
pub(crate) struct KleBackground {
    pub name: Option<String>,
    pub style: Option<String>,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct KleMetadata {
    pub author: Option<String>,
    #[serde(deserialize_with = "de_color")]
    pub backcolor: Option<Color>,
    pub background: Option<KleBackground>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub radii: Option<String>,
    pub switch_mount: Option<String>,
    pub switch_brand: Option<String>,
    pub switch_type: Option<String>,
    pub css: Option<String>,
    pub pcb: Option<bool>,
    pub plate: Option<bool>,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub(crate) struct KlePropsObject {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub x2: Option<f64>,
    pub y2: Option<f64>,
    pub w2: Option<f64>,
    pub h2: Option<f64>,
    pub r: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    pub l: Option<bool>,
    pub n: Option<bool>,
    pub d: Option<bool>,
    pub g: Option<bool>,
    pub sm: Option<String>,
    pub sb: Option<String>,
    pub st: Option<String>,
    #[serde(deserialize_with = "de_color")]
    pub c: Option<Color>,
    #[serde(deserialize_with = "de_nl_delimited_colors")]
    pub t: Option<Vec<Option<Color>>>,
    pub a: Option<Alignment>,
    pub p: Option<String>,
    pub f: Option<FontSize>,
    pub f2: Option<FontSize>,
    pub fa: Option<Vec<FontSize>>,
}

// Represents either a key or a JSON object containing properties for the next key(s)
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum KleLegendsOrProps {
    Props(Box<KlePropsObject>),
    Legend(String),
}

#[derive(Debug, Clone)]
pub(crate) struct KleKeyboard {
    pub meta: KleMetadata,
    pub layout: Vec<Vec<KleLegendsOrProps>>,
}

impl<'de> Deserialize<'de> for KleKeyboard {
    fn deserialize<D>(deserializer: D) -> Result<KleKeyboard, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KleFileVisitor;

        impl<'de> Visitor<'de> for KleFileVisitor {
            type Value = KleKeyboard;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // The order in this enum is important. Serde will try to deserialize a vec first,
                // otherwise a struct. This is important since you can deserialise a JSON sequence
                // to a struct but not a JSON object to a Vec.
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum MapOrSeq {
                    Seq(Vec<KleLegendsOrProps>),
                    Map(Box<KleMetadata>),
                }

                // Set a max initial size of 2**12, this is also what serde does internally
                let mut layout = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));

                let meta = match seq.next_element()? {
                    Some(MapOrSeq::Map(meta)) => *meta,
                    Some(MapOrSeq::Seq(row)) => {
                        layout.push(row);
                        KleMetadata::default()
                    }
                    None => KleMetadata::default(),
                };

                while let Some(row) = seq.next_element()? {
                    layout.push(row);
                }

                Ok(Self::Value { meta, layout })
            }
        }

        deserializer.deserialize_seq(KleFileVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;
    use serde_json::{Deserializer, Error};

    #[test]
    fn test_de_color() {
        let colors = [
            ("rebeccapurple", Color::new(102, 51, 153, 255)),
            ("aliceblue", Color::new(240, 248, 255, 255)),
            ("#f09", Color::new(255, 0, 153, 255)),
            ("#ff0099", Color::new(255, 0, 153, 255)),
            ("f09", Color::new(255, 0, 153, 255)),
            ("ff0099", Color::new(255, 0, 153, 255)),
            ("rgb(255 0 153)", Color::new(255, 0, 153, 255)),
            ("rgb(255 0 153 / 80%)", Color::new(255, 0, 153, 204)),
            ("hsl(150 30% 60%)", Color::new(122, 184, 153, 255)),
            ("hsl(150 30% 60% / 0.8)", Color::new(122, 184, 153, 204)),
            ("hwb(12 50% 0%)", Color::new(255, 153, 128, 255)),
            ("hwb(194 0% 0% / 0.5)", Color::new(0, 195, 255, 128)),
        ];

        for (css, res) in colors {
            let color = de_color(&mut Deserializer::from_str(&format!(r#""{css}""#)))
                .unwrap()
                .unwrap();
            assert_eq!(color, res);
        }
    }

    #[test]
    fn test_de_nl_delimited_colors() {
        let colors = de_nl_delimited_colors(&mut Deserializer::from_str(r##""#f00\n\n#ba9""##));
        assert_matches!(colors, Ok(Some(v)) if v.len() == 3 && v[1].is_none());

        let colors = de_nl_delimited_colors(&mut Deserializer::from_str(r##""#abc\\n#bad""##));
        assert_matches!(colors, Err(Error { .. }));

        let colors = de_nl_delimited_colors(&mut Deserializer::from_str("null"));
        assert_matches!(colors, Ok(None));
    }

    #[test]
    fn test_deserialize_kle_keyboard() {
        let result1: KleKeyboard = serde_json::from_str(
            r#"[
                {
                    "name": "test",
                    "unknown": "key"
                },
                [
                    {
                        "a": 4,
                        "unknown2": "key"
                    },
                    "A",
                    "B",
                    "C"
                ],
                [
                    "D"
                ]
            ]"#,
        )
        .unwrap();

        assert_matches!(result1.meta.name, Some(name) if name == "test");
        assert_eq!(result1.layout.len(), 2);
        assert_eq!(result1.layout[0].len(), 4);
        assert_matches!(result1.layout[0][0], KleLegendsOrProps::Props(_));
        assert_matches!(result1.layout[0][1], KleLegendsOrProps::Legend(_));

        let result2: KleKeyboard = serde_json::from_str(r#"[["A"]]"#).unwrap();
        assert!(result2.meta.name.is_none());
        assert_eq!(result2.layout.len(), 1);

        let result3: KleKeyboard = serde_json::from_str(r#"[{"notes": "'tis a test"}]"#).unwrap();
        assert_matches!(result3.meta.notes, Some(notes) if notes == "'tis a test");
        assert_eq!(result3.layout.len(), 0);

        let result4: KleKeyboard = serde_json::from_str(r#"[]"#).unwrap();
        assert!(result4.meta.name.is_none());
        assert_eq!(result4.layout.len(), 0);

        assert_matches!(serde_json::from_str::<KleKeyboard>("null"), Err(_));
    }
}

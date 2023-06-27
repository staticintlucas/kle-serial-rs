use std::fmt;

use csscolorparser::Color as CssColor;
use serde::de::{value::SeqAccessDeserializer, Error, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json as json;

use crate::Color;

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
    pub a: Option<usize>,
    pub p: Option<String>,
    pub f: Option<usize>,
    pub f2: Option<usize>,
    pub fa: Option<Vec<usize>>,
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
    #[allow(dead_code)] // TODO
    pub props: json::Map<String, json::Value>, // TODO global layout properties are unused at the moment
    pub rows: Vec<Vec<KleLegendsOrProps>>,
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
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum MapOrSeq {
                    Props(json::Map<String, json::Value>),
                    Row(Vec<KleLegendsOrProps>),
                }

                let result = match seq.next_element()? {
                    None => {
                        let props = json::Map::new();
                        let rows = Vec::new();
                        Self::Value { props, rows }
                    }
                    Some(MapOrSeq::Props(props)) => {
                        let rows = Vec::deserialize(SeqAccessDeserializer::new(seq))?;
                        Self::Value { props, rows }
                    }
                    Some(MapOrSeq::Row(row)) => {
                        let props = json::Map::new();
                        let mut rows = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
                        rows.push(row);
                        rows.extend(Vec::deserialize(SeqAccessDeserializer::new(seq))?);
                        Self::Value { props, rows }
                    }
                };

                Ok(result)
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
    fn test_de_nl_delimited_colors() {
        let colors = de_nl_delimited_colors(&mut Deserializer::from_str(r##""#f00\n\n#ba9""##));
        assert_matches!(colors, Ok(Some(v)) if v.len() == 3 && v[1].is_none());

        let colors = de_nl_delimited_colors(&mut Deserializer::from_str(r##""#abc\\n#bad""##));
        assert_matches!(colors, Err(Error { .. }));

        let colors = de_nl_delimited_colors(&mut Deserializer::from_str("null"));
        assert_matches!(colors, Ok(None));
    }

    #[test]
    fn test_deserialize_kle_file() {
        let result1: KleKeyboard = serde_json::from_str(
            r#"[
                {
                    "meta": "data"
                },
                [
                    {
                        "a": 4,
                        "unknown": "key"
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

        assert_eq!(result1.props.len(), 1);
        assert_eq!(result1.props["meta"], "data");

        assert_eq!(result1.rows.len(), 2);
        assert_eq!(result1.rows[0].len(), 4);
        assert_matches!(result1.rows[0][0], KleLegendsOrProps::Props(_));
        assert_matches!(result1.rows[0][1], KleLegendsOrProps::Legend(_));

        let result2: KleKeyboard = serde_json::from_str(r#"[["A"]]"#).unwrap();
        assert_eq!(result2.props.len(), 0);
        assert_eq!(result2.rows.len(), 1);

        let result3: KleKeyboard = serde_json::from_str(r#"[{"k": "v"}]"#).unwrap();
        assert_eq!(result3.props.len(), 1);
        assert_eq!(result3.rows.len(), 0);

        let result4: KleKeyboard = serde_json::from_str(r#"[]"#).unwrap();
        assert_eq!(result4.props.len(), 0);
        assert_eq!(result4.rows.len(), 0);

        assert_matches!(serde_json::from_str::<KleKeyboard>("null"), Err(_));
    }
}

use std::fmt::Debug;

use serde::{
    de::{Error, Unexpected},
    Deserialize,
};

use crate::{Legend, NUM_LEGENDS};

#[derive(Debug, Clone, Copy)]
pub(crate) struct BoundsError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoundedUsize<const MAX: usize, const DEF: usize>(usize);

impl<const MAX: usize, const DEF: usize> BoundedUsize<MAX, DEF> {
    pub fn new(value: usize) -> Result<Self, BoundsError> {
        if value <= MAX {
            Ok(Self(value))
        } else {
            Err(BoundsError)
        }
    }
}

impl<const MAX: usize, const DEF: usize> Debug for BoundedUsize<MAX, DEF> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const MAX: usize, const DEF: usize> From<BoundedUsize<MAX, DEF>> for usize {
    fn from(value: BoundedUsize<MAX, DEF>) -> Self {
        value.0
    }
}

impl<const MAX: usize, const DEF: usize> Default for BoundedUsize<MAX, DEF> {
    fn default() -> Self {
        Self(DEF)
    }
}

impl<'de, const MAX: usize, const DEF: usize> Deserialize<'de> for BoundedUsize<MAX, DEF> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let font_size = usize::deserialize(deserializer)?;

        Self::new(font_size).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Unsigned(font_size as u64),
                &format!("0 <= x <= {MAX}").as_str(),
            )
        })
    }
}

// KLE uses default font size of 3 and max of 9
pub(crate) type FontSize = BoundedUsize<9, 3>;

// KLE uses default alignment of 4
const MAX_ALIGNMENT: usize = LEGEND_MAPPING.len() - 1;
pub(crate) type Alignment = BoundedUsize<MAX_ALIGNMENT, 4>;

// This map is the same as that of kle-serial. Note the blanks are also filled
// in, so we're slightly more permissive with not-strictly-valid KLE input.
const LEGEND_MAPPING: [[usize; NUM_LEGENDS]; 8] = [
    [0, 6, 2, 8, 9, 11, 3, 5, 1, 4, 7, 10], // 0 = no centering
    [1, 7, 0, 2, 9, 11, 4, 3, 5, 6, 8, 10], // 1 = center x
    [3, 0, 5, 1, 9, 11, 2, 6, 4, 7, 8, 10], // 2 = center y
    [4, 0, 1, 2, 9, 11, 3, 5, 6, 7, 8, 10], // 3 = center x & y
    [0, 6, 2, 8, 10, 9, 3, 5, 1, 4, 7, 11], // 4 = center front (default)
    [1, 7, 0, 2, 10, 3, 4, 5, 6, 8, 9, 11], // 5 = center front & x
    [3, 0, 5, 1, 10, 2, 6, 7, 4, 8, 9, 11], // 6 = center front & y
    [4, 0, 1, 2, 10, 3, 5, 6, 7, 8, 9, 11], // 7 = center front & x & y
];

pub(crate) fn realign_legends<T>(values: T, alignment: Alignment) -> [Option<Legend>; NUM_LEGENDS]
where
    T: IntoIterator<Item = Option<Legend>>,
{
    // Guaranteed to be in range because of newtype
    let mapping = LEGEND_MAPPING[usize::from(alignment)];

    let mut sorted = mapping.iter().zip(values).collect::<Vec<_>>();
    sorted.sort_by_key(|el| el.0);

    let mut values = sorted.into_iter().map(|el| el.1);
    std::array::from_fn(|_| values.next().unwrap_or(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::de::{
        value::{Error as ValueError, UsizeDeserializer},
        IntoDeserializer,
    };

    #[test]
    fn test_bounded_usize_new() {
        let value = BoundedUsize::<10, 5>::new(7);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().0, 7);

        let value = BoundedUsize::<10, 5>::new(17);
        assert!(value.is_err());
    }

    #[test]
    fn test_bounded_usize_debug() {
        let value = BoundedUsize::<10, 5>::new(7).unwrap();

        assert_eq!(format!("{value:?}"), "7");
    }

    #[test]
    fn test_bounded_usize_into() {
        let value = BoundedUsize::<10, 5>::new(7).unwrap();

        assert_eq!(usize::from(value), 7);
    }

    #[test]
    fn test_bounded_usize_default() {
        let value = BoundedUsize::<10, 5>::default();

        assert_eq!(value.0, 5);
    }

    #[test]
    fn test_bounded_usize_deserialize() {
        let deserializer: UsizeDeserializer<ValueError> = 7_usize.into_deserializer();
        let value = BoundedUsize::<10, 5>::deserialize(deserializer);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().0, 7);

        let deserializer: UsizeDeserializer<ValueError> = 17_usize.into_deserializer();
        let value = BoundedUsize::<10, 5>::deserialize(deserializer);
        assert!(value.is_err());
    }

    #[test]
    fn test_realign_legends() {
        let legends = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L"].map(|text| {
            Some(Legend {
                text: text.into(),
                ..Legend::default()
            })
        });
        let expected = ["A", "I", "C", "G", "J", "H", "B", "K", "D", "F", "E", "L"];

        let result = realign_legends(legends.clone(), Alignment::new(4).unwrap());
        let result_text = result.map(|l| l.unwrap().text);

        assert_eq!(result_text, expected);
    }
}

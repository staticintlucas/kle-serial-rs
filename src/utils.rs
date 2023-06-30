use itertools::Itertools;
use serde::de::{Error, Unexpected};
use serde::Deserialize;

use crate::Legend;
use crate::NUM_LEGENDS;

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Alignment(usize);

impl Alignment {
    pub(crate) const MAX: Self = Self(LEGEND_MAPPING.len() - 1);

    pub(crate) const fn new(alignment: usize) -> Option<Self> {
        if alignment <= Self::MAX.0 {
            Some(Self(alignment))
        } else {
            None
        }
    }

    pub(crate) const fn default() -> Self {
        Self(4) // 4 is the default used by KLE
    }
}

impl From<Alignment> for usize {
    fn from(value: Alignment) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Alignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let alignment = usize::deserialize(deserializer)?;

        Self::new(alignment).ok_or_else(|| {
            D::Error::invalid_value(
                Unexpected::Unsigned(alignment as u64),
                &format!("0 <= x <= {}", Self::MAX.0).as_str(),
            )
        })
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self::default()
    }
}

pub(crate) fn realign_legends<T>(values: T, alignment: Alignment) -> [Option<Legend>; NUM_LEGENDS]
where
    T: IntoIterator<Item = Option<Legend>>,
{
    // Guaranteed to be in range because of newtype
    let mapping = LEGEND_MAPPING[usize::from(alignment)];

    let mut values = mapping
        .iter()
        .zip(values)
        .sorted_by_key(|el| el.0)
        .map(|el| el.1);

    std::array::from_fn(|_| values.next().unwrap_or(None))
}

#[cfg(test)]
mod tests {
    use super::*;

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

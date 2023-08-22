use anyhow::{bail, Error, Result};
use ranges::{GenericRange, Ranges};
use serde::{self, Deserialize};
use std::{ops::RangeInclusive, str::FromStr};

/// A collection of field selection ranges. The overlapping of multiple ranges
/// is tolerated and should be optimized by the underlying data structure.
#[derive(Clone, Deserialize)]
pub struct FieldSelections(#[serde(deserialize_with = "deserialize_ranges")] Ranges<usize>);

/// Describes a range of fields that should be included in the selection.
/// Must always contain a starting field. The format is: "a|a-b|a-".
struct FieldSelection {
    start: usize,
    end: Option<usize>,
}

impl FromStr for FieldSelections {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ranges = s
            .split(",")
            .map(|s| {
                let field: FieldSelection = s.parse()?;
                let range: RangeInclusive<usize> = field.into();
                Ok(GenericRange::from(range))
            })
            .collect::<Result<_>>()?;

        Ok(FieldSelections(ranges))
    }
}

fn deserialize_ranges<'de, D>(deserializer: D) -> Result<Ranges<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FieldSelections::from_str(&s)
        .map_err(serde::de::Error::custom)
        .map(|fs| fs.0)
}

impl FieldSelections {
    /// Check if a field, indicated by its array index, should be selected.
    /// The field selections start counting at 1, while indexes start at 0.
    pub fn contains(&self, index: usize) -> bool {
        self.0.contains(&(index + 1))
    }
}

impl FromStr for FieldSelection {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = match s.split('-').collect::<Vec<_>>().as_slice() {
            &[x] => {
                let x_parsed = x.parse::<usize>()?;
                (x_parsed, Some(x_parsed))
            }
            &[x, ""] => (x.parse::<usize>()?, None),
            &[x, y] => (x.parse::<usize>()?, Some(y.parse::<usize>()?)),
            _ => bail!(
                "Failed to parse \"{}\" as field selection, expected format is a|a-b|a-",
                s
            ),
        };

        if matches!(start, 0) || matches!(end, Some(0)) {
            bail!(
                "Failed to parse \"{}\" as field selection, 0 is not a valid field",
                s
            );
        }

        Ok(FieldSelection { start, end })
    }
}

impl From<FieldSelection> for RangeInclusive<usize> {
    fn from(value: FieldSelection) -> Self {
        match (value.start, value.end) {
            (start, Some(end)) => RangeInclusive::new(start, end),
            (start, None) => RangeInclusive::new(start, usize::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inclusive_range() {
        assert_eq!(
            "1,2-5,8-".parse::<FieldSelections>().unwrap().0,
            Ranges::from(vec![1..=1, 2..=5, 8..=usize::MAX])
        );
    }

    #[test]
    fn test_range_limit() {
        assert_eq!(
            format!("1,{}", usize::MAX)
                .parse::<FieldSelections>()
                .unwrap()
                .0,
            Ranges::from(vec![1..=1, usize::MAX..=usize::MAX])
        );
    }

    #[test]
    fn test_overlapping_ranges() {
        assert_eq!(
            "1-3,1-4,2-5".parse::<FieldSelections>().unwrap().0,
            Ranges::from(vec![1..=5])
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid() {
        let _: FieldSelections = "0".parse().unwrap();
    }
}

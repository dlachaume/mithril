use std::ops::RangeInclusive;

use anyhow::anyhow;

use mithril_common::{entities::ImmutableFileNumber, StdResult};

/// Immutable file range representation
#[derive(Debug, Eq, PartialEq)]
pub enum ImmutableFileRange {
    /// From the first (included) to the last immutable file number (included)
    Full,

    /// From a specific immutable file number (included) to the last immutable file number (included)
    From(ImmutableFileNumber),

    /// From a specific immutable file number (included) to another specific immutable file number (included)
    Range(ImmutableFileNumber, ImmutableFileNumber),

    /// From the first immutable file number (included) up to a specific immutable file number (included)
    UpTo(ImmutableFileNumber),
}

impl ImmutableFileRange {
    /// Returns the range of immutable file numbers
    pub fn to_range_inclusive(
        &self,
        last_immutable_file_number: ImmutableFileNumber,
    ) -> StdResult<RangeInclusive<ImmutableFileNumber>> {
        const FIRST_IMMUTABLE_FILE_NUMBER: ImmutableFileNumber = 0;
        let full_range = FIRST_IMMUTABLE_FILE_NUMBER..=last_immutable_file_number;

        match self {
            ImmutableFileRange::Full => Ok(full_range),
            ImmutableFileRange::From(from) if full_range.contains(from) => {
                Ok(*from..=last_immutable_file_number)
            }
            ImmutableFileRange::Range(from, to)
                if full_range.contains(from)
                    && full_range.contains(to)
                    && !(*from..=*to).is_empty() =>
            {
                Ok(*from..=*to)
            }
            ImmutableFileRange::UpTo(to) if full_range.contains(to) => {
                Ok(FIRST_IMMUTABLE_FILE_NUMBER..=*to)
            }
            _ => Err(anyhow!("Invalid immutable file range: {self:?}")),
        }
    }

    /// Returns the length of the immutable file range
    pub fn length(&self, last_immutable_file_number: ImmutableFileNumber) -> u64 {
        match self {
            ImmutableFileRange::Full => last_immutable_file_number,
            ImmutableFileRange::From(from) => last_immutable_file_number - from + 1,
            ImmutableFileRange::Range(from, to) => to - from + 1,
            ImmutableFileRange::UpTo(to) => *to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_range_inclusive_with_full() {
        let immutable_file_range = ImmutableFileRange::Full;
        let last_immutable_file_number = 10;

        let result = immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .unwrap();
        assert_eq!(0..=10, result);
    }

    #[test]
    fn to_range_inclusive_with_from() {
        let immutable_file_range = ImmutableFileRange::From(5);

        let last_immutable_file_number = 10;
        let result = immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .unwrap();
        assert_eq!(5..=10, result);

        let last_immutable_file_number = 3;
        immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .expect_err("should fail: given last immutable should be greater than range start");
    }

    #[test]
    fn to_range_inclusive_with_range() {
        let immutable_file_range = ImmutableFileRange::Range(5, 8);

        let last_immutable_file_number = 10;
        let result = immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .unwrap();
        assert_eq!(5..=8, result);

        let last_immutable_file_number = 7;
        immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .expect_err(
                "should fail: given last immutable should be greater or equal range max bound",
            );

        let immutable_file_range = ImmutableFileRange::Range(10, 8);
        immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .expect_err("should fail: range start should be lower than range end");
    }

    #[test]
    fn to_range_inclusive_with_up_to() {
        let immutable_file_range = ImmutableFileRange::UpTo(8);

        let last_immutable_file_number = 10;
        let result = immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .unwrap();
        assert_eq!(0..=8, result);

        let last_immutable_file_number = 7;
        immutable_file_range
            .to_range_inclusive(last_immutable_file_number)
            .expect_err(
                "should fail: given last immutable should be greater or equal range max bound",
            );
    }

    #[test]
    fn length() {
        let last_immutable_file_number = 10;

        let immutable_file_range = ImmutableFileRange::Full;
        assert_eq!(
            last_immutable_file_number,
            immutable_file_range.length(last_immutable_file_number)
        );

        let immutable_file_range = ImmutableFileRange::From(5);
        assert_eq!(6, immutable_file_range.length(last_immutable_file_number));

        let immutable_file_range = ImmutableFileRange::Range(5, 8);
        assert_eq!(4, immutable_file_range.length(last_immutable_file_number));

        let immutable_file_range = ImmutableFileRange::UpTo(8);
        assert_eq!(8, immutable_file_range.length(last_immutable_file_number));
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::Epoch;

/// Message structure of a Mithril Stake Distribution list
pub type MithrilStakeDistributionListMessage = Vec<MithrilStakeDistributionListItemMessage>;

/// Message structure of a Mithril Stake Distribution list item
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MithrilStakeDistributionListItemMessage {
    /// Epoch at which the Mithril Stake Distribution is created
    pub epoch: Epoch,

    /// Hash of the Mithril Stake Distribution (different from the AVK).
    pub hash: String,

    /// Hash of the associated certificate
    pub certificate_hash: String,

    /// Date and time at which the Mithril Stake Distribution was created
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn golden_message_current() -> MithrilStakeDistributionListMessage {
        vec![MithrilStakeDistributionListItemMessage {
            epoch: Epoch(1),
            hash: "hash-123".to_string(),
            certificate_hash: "certificate-hash-123".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-01-19T13:43:05.618857482Z")
                .unwrap()
                .with_timezone(&Utc),
        }]
    }

    const CURRENT_JSON: &str = r#"[{
        "epoch": 1,
        "hash": "hash-123",
        "certificate_hash": "certificate-hash-123",
        "created_at": "2023-01-19T13:43:05.618857482Z"
        }]"#;

    #[test]
    fn test_current_json_deserialized_into_current_message() {
        let json = CURRENT_JSON;
        let message: MithrilStakeDistributionListMessage = serde_json::from_str(json).expect(
                    "This JSON is expected to be successfully parsed into a MithrilStakeDistributionListMessage instance.",
                );

        assert_eq!(golden_message_current(), message);
    }
}

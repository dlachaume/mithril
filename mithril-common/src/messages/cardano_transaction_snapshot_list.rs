use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::{BlockNumber, Epoch};

/// Message structure of a Cardano Transactions Snapshots list
pub type CardanoTransactionSnapshotListMessage = Vec<CardanoTransactionSnapshotListItemMessage>;

/// Message structure of a Cardano Transactions Snapshot list item
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardanoTransactionSnapshotListItemMessage {
    /// Merkle root of the Cardano transactions snapshot
    pub merkle_root: String,

    /// Epoch of the Cardano transactions snapshot
    pub epoch: Epoch,

    /// Block number of the Cardano transactions snapshot
    pub block_number: BlockNumber,

    /// Hash of the Cardano Transactions snapshot
    pub hash: String,

    /// Hash of the associated certificate
    pub certificate_hash: String,

    /// DateTime of creation
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn golden_message_current() -> CardanoTransactionSnapshotListMessage {
        vec![CardanoTransactionSnapshotListItemMessage {
            merkle_root: "mkroot-123".to_string(),
            epoch: Epoch(7),
            block_number: BlockNumber(5),
            hash: "hash-123".to_string(),
            certificate_hash: "certificate-hash-123".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-01-19T13:43:05.618857482Z")
                .unwrap()
                .with_timezone(&Utc),
        }]
    }

    const CURRENT_JSON: &str = r#"[{
        "merkle_root": "mkroot-123",
        "epoch": 7,
        "block_number": 5,
        "hash": "hash-123",
        "certificate_hash": "certificate-hash-123",
        "created_at": "2023-01-19T13:43:05.618857482Z"
    }]"#;

    #[test]
    fn test_current_json_deserialized_into_current_message() {
        let json = CURRENT_JSON;
        let message: CardanoTransactionSnapshotListMessage = serde_json::from_str(json).expect(
                    "This JSON is expected to be successfully parsed into a CardanoTransactionSnapshotListMessage instance.",
                );
        assert_eq!(golden_message_current(), message);
    }
}

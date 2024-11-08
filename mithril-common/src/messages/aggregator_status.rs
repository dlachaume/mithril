use serde::{Deserialize, Serialize};

use crate::{entities::Epoch, era::SupportedEra};

/// Message advertised by an Aggregator to inform about its status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregatorStatusMessage {
    /// Current epoch
    pub epoch: Epoch,

    /// Current Cardano era
    pub cardano_era: String,

    /// Current Mithril era
    pub mithril_era: SupportedEra,
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTUAL_JSON: &str = r#"{
        "epoch": 48,
        "cardano_era": "conway",
        "mithril_era": "pythagoras"
        }"#;

    fn golden_actual_message() -> AggregatorStatusMessage {
        AggregatorStatusMessage {
            epoch: Epoch(48),
            cardano_era: "conway".to_string(),
            mithril_era: SupportedEra::Pythagoras,
        }
    }

    // Test the compatibility with current structure.
    #[test]
    fn test_actual_json_deserialized_into_actual_message() {
        let json = ACTUAL_JSON;
        let message: AggregatorStatusMessage = serde_json::from_str(json).expect(
            "This JSON is expected to be successfully parsed into a AggregatorStatusMessage instance.",
        );

        assert_eq!(golden_actual_message(), message);
    }
}

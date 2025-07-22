use serde::{Deserialize, Serialize};

/// Message structure of an immutable files restoration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardanoDatabaseImmutableFilesRestoredMessage {
    /// Number of immutable files restored.
    pub nb_immutable_files: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_JSON: &str = r#"{
        "nb_immutable_files": 62
    }"#;

    fn golden_message_current() -> CardanoDatabaseImmutableFilesRestoredMessage {
        CardanoDatabaseImmutableFilesRestoredMessage {
            nb_immutable_files: 62,
        }
    }

    #[test]
    fn test_current_json_deserialized_into_current_message() {
        let json = CURRENT_JSON;
        let message: CardanoDatabaseImmutableFilesRestoredMessage =
            serde_json::from_str(json).unwrap();

        assert_eq!(golden_message_current(), message);
    }
}

mod api;
mod error;
mod follower;
mod leader;
mod verifier;

pub use api::{
    LeaderAggregatorClient, SignerRecorder, SignerRegisterer, SignerRegistrationRound,
    SignerRegistrationRoundOpener, SignerRegistrationVerifier, SignerSynchronizer,
};
pub use error::SignerRegistrationError;
pub use follower::MithrilSignerRegistrationFollower;
pub use leader::MithrilSignerRegistrationLeader;
pub use verifier::MithrilSignerRegistrationVerifier;

#[cfg(test)]
pub use api::{
    MockLeaderAggregatorClient, MockSignerRecorder, MockSignerRegisterer,
    MockSignerRegistrationVerifier,
};

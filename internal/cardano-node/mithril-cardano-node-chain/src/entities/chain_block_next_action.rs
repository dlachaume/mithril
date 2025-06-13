use crate::entities::RawCardanoPoint;
use crate::entities::ScannedBlock;

/// The action that indicates what to do next when scanning the chain
#[derive(Debug, Clone, PartialEq)]
pub enum ChainBlockNextAction {
    /// RollForward event (we are still on the correct fork)
    RollForward {
        /// The parsed chain block
        parsed_block: ScannedBlock,
    },
    /// RollBackward event (we are on an incorrect fork, we need to get back a point to roll forward again)
    RollBackward {
        /// The rollback point in the chain to read (as a new valid chain point to read from on the main chain, which has already been seen)
        rollback_point: RawCardanoPoint,
    },
}

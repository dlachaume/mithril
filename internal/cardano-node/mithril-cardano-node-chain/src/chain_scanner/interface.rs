use async_trait::async_trait;

use mithril_common::entities::{BlockNumber, SlotNumber};
use mithril_common::StdResult;

use crate::entities::{RawCardanoPoint, ScannedBlock};

/// A scanner that can read cardano transactions in a cardano database
///
/// If you want to mock it using mockall:
/// ```
/// mod test {
///     use std::path::Path;
///
///     use anyhow::anyhow;
///     use async_trait::async_trait;
///     use mockall::mock;
///
///     use mithril_common::entities::{BlockNumber};
///     use mithril_common::StdResult;
///
///     use mithril_cardano_node_chain::chain_scanner::{BlockScanner, BlockStreamer};
///     use mithril_cardano_node_chain::entities::{RawCardanoPoint};
///
///     mock! {
///         pub BlockScannerImpl { }
///
///         #[async_trait]
///         impl BlockScanner for BlockScannerImpl {
///             async fn scan(
///               &self,
///               from: Option<RawCardanoPoint>,
///               until: BlockNumber,
///             ) -> StdResult<Box<dyn BlockStreamer>>;
///         }
///     }
///
///     #[test]
///     fn test_mock() {
///         let mut mock = MockBlockScannerImpl::new();
///         mock.expect_scan().return_once(|_, _| {
///             Err(anyhow!("parse error"))
///         });
///     }
/// }
/// ```
#[async_trait]
pub trait BlockScanner: Sync + Send {
    /// Scan the transactions
    async fn scan(
        &self,
        from: Option<RawCardanoPoint>,
        until: BlockNumber,
    ) -> StdResult<Box<dyn BlockStreamer>>;
}

/// [ChainScannedBlocks] allows to scan new blocks and handle rollbacks
#[derive(Debug, Clone, PartialEq)]
pub enum ChainScannedBlocks {
    /// Roll forward on the chain to the next list of [ScannedBlock]
    RollForwards(Vec<ScannedBlock>),
    /// Roll backward on the chain to the previous [SlotNumber]
    RollBackward(SlotNumber),
}

/// Trait that define how blocks are streamed from a Cardano database
#[async_trait]
pub trait BlockStreamer: Sync + Send {
    /// Stream the next available blocks
    async fn poll_next(&mut self) -> StdResult<Option<ChainScannedBlocks>>;

    /// Get the last polled point of the chain
    fn last_polled_point(&self) -> Option<RawCardanoPoint>;
}

// todo: remove this trait (nearly unused right now)
/// Tests extensions methods for the [BlockStreamer] trait.
#[async_trait]
#[cfg(test)]
pub trait BlockStreamerTestExtensions {
    /// Stream all the available blocks, may be very memory intensive
    async fn poll_all(&mut self) -> StdResult<Vec<ScannedBlock>>;
}

#[async_trait]
#[cfg(test)]
impl<S: BlockStreamer + ?Sized> BlockStreamerTestExtensions for S {
    async fn poll_all(&mut self) -> StdResult<Vec<ScannedBlock>> {
        let mut all_blocks = Vec::new();
        while let Some(next_blocks) = self.poll_next().await? {
            match next_blocks {
                ChainScannedBlocks::RollForwards(mut forward_blocks) => {
                    all_blocks.append(&mut forward_blocks);
                }
                ChainScannedBlocks::RollBackward(_) => {
                    return Err(anyhow::anyhow!("poll_all: RollBackward not supported"));
                }
            };
        }
        Ok(all_blocks)
    }
}

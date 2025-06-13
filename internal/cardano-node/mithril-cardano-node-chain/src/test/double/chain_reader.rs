use std::{collections::VecDeque, sync::Mutex};

use async_trait::async_trait;

use mithril_common::StdResult;

use crate::entities::{ChainBlockNextAction, RawCardanoPoint};

use crate::chain_reader::ChainBlockReader;

/// [FakeChainReader] is a fake implementation of [ChainBlockReader] for testing purposes.
pub struct FakeChainReader {
    chain_block_next_actions: Mutex<VecDeque<ChainBlockNextAction>>,
}

impl FakeChainReader {
    /// Creates a new [FakeChainReader] instance.
    pub fn new(chain_block_next_actions: Vec<ChainBlockNextAction>) -> Self {
        Self {
            chain_block_next_actions: Mutex::new(chain_block_next_actions.into()),
        }
    }

    /// Total remaining next actions
    pub fn get_total_remaining_next_actions(&self) -> usize {
        self.chain_block_next_actions.lock().unwrap().len()
    }
}

#[async_trait]
impl ChainBlockReader for FakeChainReader {
    async fn set_chain_point(&mut self, _point: &RawCardanoPoint) -> StdResult<()> {
        Ok(())
    }

    async fn get_next_chain_block(&mut self) -> StdResult<Option<ChainBlockNextAction>> {
        Ok(self.chain_block_next_actions.lock().unwrap().pop_front())
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::entities::{BlockNumber, SlotNumber};

    use crate::entities::ScannedBlock;

    use super::*;

    #[tokio::test]
    async fn test_get_next_chain_block() {
        let expected_chain_block_next_actions = vec![
            ChainBlockNextAction::RollForward {
                parsed_block: ScannedBlock::new(
                    "hash-1",
                    BlockNumber(1),
                    SlotNumber(10),
                    Vec::<&str>::new(),
                ),
            },
            ChainBlockNextAction::RollForward {
                parsed_block: ScannedBlock::new(
                    "hash-2",
                    BlockNumber(2),
                    SlotNumber(11),
                    Vec::<&str>::new(),
                ),
            },
            ChainBlockNextAction::RollBackward {
                rollback_point: RawCardanoPoint::new(SlotNumber(1), "point-hash-1".as_bytes()),
            },
        ];

        let mut chain_reader = FakeChainReader::new(expected_chain_block_next_actions.clone());

        let mut chain_block_next_actions = vec![];
        while let Some(chain_block_next_action) = chain_reader.get_next_chain_block().await.unwrap()
        {
            chain_block_next_actions.push(chain_block_next_action);
        }

        assert_eq!(expected_chain_block_next_actions, chain_block_next_actions);
    }
}

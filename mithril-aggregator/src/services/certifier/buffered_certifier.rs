use async_trait::async_trait;
use slog::{debug, Logger};
use std::sync::Arc;

use mithril_common::entities::{
    Certificate, Epoch, ProtocolMessage, SignedEntityType, SignedEntityTypeDiscriminants,
    SingleSignatures,
};
use mithril_common::StdResult;

use crate::entities::OpenMessage;
use crate::services::{BufferedSingleSignatureStore, CertifierService, CertifierServiceError};

/// A decorator of [CertifierService] that buffers that can buffer registration of single signatures
/// when the open message is not yet created.
///
/// When an open message is created, buffered single signatures for the open message type are
/// registered.
pub struct BufferedCertifierService {
    certifier_service: Arc<dyn CertifierService>,
    buffered_single_signature_store: Arc<dyn BufferedSingleSignatureStore>,
    logger: Logger,
}

impl BufferedCertifierService {
    /// Create a new instance of `BufferedCertifierService`.
    pub fn new(
        certifier_service: Arc<dyn CertifierService>,
        buffered_single_signature_store: Arc<dyn BufferedSingleSignatureStore>,
        logger: Logger,
    ) -> Self {
        Self {
            certifier_service,
            buffered_single_signature_store,
            logger,
        }
    }
}

#[async_trait]
impl CertifierService for BufferedCertifierService {
    async fn inform_epoch(&self, epoch: Epoch) -> StdResult<()> {
        self.certifier_service.inform_epoch(epoch).await
    }

    async fn register_single_signature(
        &self,
        signed_entity_type: &SignedEntityType,
        signature: &SingleSignatures,
    ) -> StdResult<()> {
        match self
            .certifier_service
            .register_single_signature(signed_entity_type, signature)
            .await
        {
            Ok(res) => Ok(res),
            Err(error) => match error.downcast_ref::<CertifierServiceError>() {
                Some(CertifierServiceError::NotFound(..)) => {
                    debug!(
                        self.logger,
                        "No OpenMessage available for signed entity - Buffering single signature";
                        "signed_entity_type" => ?signed_entity_type,
                        "party_id" => &signature.party_id
                    );

                    self.buffered_single_signature_store
                        .buffer_signature(signed_entity_type.into(), signature)
                        .await?;
                    Ok(())
                }
                _ => Err(error),
            },
        }
    }

    async fn create_open_message(
        &self,
        signed_entity_type: &SignedEntityType,
        protocol_message: &ProtocolMessage,
    ) -> StdResult<OpenMessage> {
        let creation_result = self
            .certifier_service
            .create_open_message(signed_entity_type, protocol_message)
            .await;

        // TODO: does the error in this block should make this method fails or should they just
        // be logged?
        if let Ok(_open_message) = &creation_result {
            let buffered_signatures = self
                .buffered_single_signature_store
                .get_buffered_signatures(signed_entity_type.into())
                .await?;

            for signature in &buffered_signatures {
                self.certifier_service
                    .register_single_signature(signed_entity_type, signature)
                    .await?;
            }

            self.buffered_single_signature_store
                .remove_buffered_signatures(signed_entity_type.into(), buffered_signatures)
                .await?;
        }

        creation_result
    }

    async fn get_open_message(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> StdResult<Option<OpenMessage>> {
        self.certifier_service
            .get_open_message(signed_entity_type)
            .await
    }

    async fn mark_open_message_if_expired(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> StdResult<Option<OpenMessage>> {
        self.certifier_service
            .mark_open_message_if_expired(signed_entity_type)
            .await
    }

    async fn create_certificate(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> StdResult<Option<Certificate>> {
        self.certifier_service
            .create_certificate(signed_entity_type)
            .await
    }

    async fn get_certificate_by_hash(&self, hash: &str) -> StdResult<Option<Certificate>> {
        self.certifier_service.get_certificate_by_hash(hash).await
    }

    async fn get_latest_certificates(&self, last_n: usize) -> StdResult<Vec<Certificate>> {
        self.certifier_service.get_latest_certificates(last_n).await
    }

    async fn verify_certificate_chain(&self, epoch: Epoch) -> StdResult<()> {
        self.certifier_service.verify_certificate_chain(epoch).await
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use std::collections::BTreeMap;

    use mithril_common::test_utils::fake_data;

    use crate::services::{CertifierServiceError, MockCertifierService};
    use crate::test_tools::TestLogger;
    use crate::InMemoryBufferedSingleSignatureStore;

    use super::*;

    fn mock_certifier<F>(certifier_mock_config: F) -> Arc<MockCertifierService>
    where
        F: FnOnce(&mut MockCertifierService),
    {
        let mut certifier = MockCertifierService::new();
        certifier_mock_config(&mut certifier);
        Arc::new(certifier)
    }

    #[tokio::test]
    async fn buffer_signature_if_decorated_certifier_as_no_opened_message() {
        let store = Arc::new(InMemoryBufferedSingleSignatureStore::default());
        let certifier = BufferedCertifierService::new(
            mock_certifier(|mock| {
                mock.expect_register_single_signature().returning(|_, _| {
                    Err(CertifierServiceError::NotFound(
                        SignedEntityType::MithrilStakeDistribution(Epoch(5)),
                    )
                    .into())
                });
            }),
            store.clone(),
            TestLogger::stdout(),
        );

        certifier
            .register_single_signature(
                &SignedEntityType::MithrilStakeDistribution(Epoch(5)),
                &fake_data::single_signatures(vec![1]),
            )
            .await
            .unwrap();

        let buffered_signatures = store
            .get_buffered_signatures(SignedEntityTypeDiscriminants::MithrilStakeDistribution)
            .await
            .unwrap();
        assert_eq!(
            buffered_signatures,
            vec![fake_data::single_signatures(vec![1])]
        );
    }

    #[tokio::test]
    async fn buffered_signatures_are_moved_to_newly_opened_message() {
        let store = Arc::new(InMemoryBufferedSingleSignatureStore::with_data(
            BTreeMap::from([
                (
                    SignedEntityTypeDiscriminants::MithrilStakeDistribution,
                    vec![
                        fake_data::single_signatures(vec![1]),
                        fake_data::single_signatures(vec![2]),
                    ],
                ),
                (
                    SignedEntityTypeDiscriminants::CardanoTransactions,
                    vec![fake_data::single_signatures(vec![10])],
                ),
            ]),
        ));
        let certifier = BufferedCertifierService::new(
            mock_certifier(|mock| {
                mock.expect_create_open_message()
                    .returning(|_, _| Ok(OpenMessage::dummy()));

                // Those configuration Asserts that the buffered signatures are registered
                mock.expect_register_single_signature()
                    .with(
                        eq(SignedEntityType::MithrilStakeDistribution(Epoch(5))),
                        eq(fake_data::single_signatures(vec![1])),
                    )
                    .once()
                    .returning(|_, _| Ok(()));
                mock.expect_register_single_signature()
                    .with(
                        eq(SignedEntityType::MithrilStakeDistribution(Epoch(5))),
                        eq(fake_data::single_signatures(vec![2])),
                    )
                    .once()
                    .returning(|_, _| Ok(()));
            }),
            store.clone(),
            TestLogger::stdout(),
        );

        certifier
            .create_open_message(
                &SignedEntityType::MithrilStakeDistribution(Epoch(5)),
                &ProtocolMessage::new(),
            )
            .await
            .unwrap();

        let remaining_sigs = store
            .get_buffered_signatures(SignedEntityTypeDiscriminants::MithrilStakeDistribution)
            .await
            .unwrap();
        assert!(remaining_sigs.is_empty());
    }
}

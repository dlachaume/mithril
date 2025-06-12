use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::header::{self, HeaderValue};
use reqwest::{self, Client, Proxy, RequestBuilder, Response, StatusCode};
use slog::{debug, error, Logger};
use std::{io, sync::Arc, time::Duration};
use thiserror::Error;

use mithril_common::{
    api_version::APIVersionProvider,
    entities::{
        ClientError, Epoch, ProtocolMessage, ServerError, SignedEntityType, Signer, SingleSignature,
    },
    logging::LoggerExtensions,
    messages::{
        AggregatorFeaturesMessage, EpochSettingsMessage, TryFromMessageAdapter, TryToMessageAdapter,
    },
    StdError, MITHRIL_API_VERSION_HEADER, MITHRIL_SIGNER_VERSION_HEADER,
};

use crate::entities::SignerEpochSettings;
use crate::message_adapters::{
    FromEpochSettingsAdapter, ToRegisterSignatureMessageAdapter, ToRegisterSignerMessageAdapter,
};

const JSON_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/json");

/// Error structure for the Aggregator Client.
#[derive(Error, Debug)]
pub enum AggregatorClientError {
    /// The aggregator host has returned a technical error.
    #[error("remote server technical error")]
    RemoteServerTechnical(#[source] StdError),

    /// The aggregator host responded it cannot fulfill our request.
    #[error("remote server logical error")]
    RemoteServerLogical(#[source] StdError),

    /// Could not reach aggregator.
    #[error("remote server unreachable")]
    RemoteServerUnreachable(#[source] StdError),

    /// Unhandled status code
    #[error("unhandled status code: {0}, response text: {1}")]
    UnhandledStatusCode(StatusCode, String),

    /// Could not parse response.
    #[error("json parsing failed")]
    JsonParseFailed(#[source] StdError),

    /// Mostly network errors.
    #[error("Input/Output error")]
    IOError(#[from] io::Error),

    /// Incompatible API version error
    #[error("HTTP API version mismatch")]
    ApiVersionMismatch(#[source] StdError),

    /// HTTP client creation error
    #[error("HTTP client creation failed")]
    HTTPClientCreation(#[source] StdError),

    /// Proxy creation error
    #[error("proxy creation failed")]
    ProxyCreation(#[source] StdError),

    /// Adapter error
    #[error("adapter failed")]
    Adapter(#[source] StdError),

    /// No signer registration round opened yet
    #[error("a signer registration round is not opened yet, please try again later")]
    RegistrationRoundNotYetOpened(#[source] StdError),
}

#[cfg(test)]
/// convenient methods to error enum
impl AggregatorClientError {
    pub(crate) fn is_api_version_mismatch(&self) -> bool {
        matches!(self, Self::ApiVersionMismatch(_))
    }
}

impl AggregatorClientError {
    /// Create an `AggregatorClientError` from a response.
    ///
    /// This method is meant to be used after handling domain-specific cases leaving only
    /// 4xx or 5xx status codes.
    /// Otherwise, it will return an `UnhandledStatusCode` error.
    pub async fn from_response(response: Response) -> Self {
        let error_code = response.status();

        if error_code.is_client_error() {
            let root_cause = Self::get_root_cause(response).await;
            Self::RemoteServerLogical(anyhow!(root_cause))
        } else if error_code.is_server_error() {
            let root_cause = Self::get_root_cause(response).await;
            match error_code.as_u16() {
                550 => Self::RegistrationRoundNotYetOpened(anyhow!(root_cause)),
                _ => Self::RemoteServerTechnical(anyhow!(root_cause)),
            }
        } else {
            let response_text = response.text().await.unwrap_or_default();
            Self::UnhandledStatusCode(error_code, response_text)
        }
    }

    async fn get_root_cause(response: Response) -> String {
        let error_code = response.status();
        let canonical_reason = error_code
            .canonical_reason()
            .unwrap_or_default()
            .to_lowercase();
        let is_json = response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|ct| JSON_CONTENT_TYPE == ct);

        if is_json {
            let json_value: serde_json::Value = response.json().await.unwrap_or_default();

            if let Ok(client_error) = serde_json::from_value::<ClientError>(json_value.clone()) {
                format!(
                    "{}: {}: {}",
                    canonical_reason, client_error.label, client_error.message
                )
            } else if let Ok(server_error) =
                serde_json::from_value::<ServerError>(json_value.clone())
            {
                format!("{}: {}", canonical_reason, server_error.message)
            } else if json_value.is_null() {
                canonical_reason.to_string()
            } else {
                format!("{canonical_reason}: {json_value}")
            }
        } else {
            let response_text = response.text().await.unwrap_or_default();
            format!("{canonical_reason}: {response_text}")
        }
    }
}

/// Trait for mocking and testing a `AggregatorClient`
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AggregatorClient: Sync + Send {
    /// Retrieves epoch settings from the aggregator
    async fn retrieve_epoch_settings(
        &self,
    ) -> Result<Option<SignerEpochSettings>, AggregatorClientError>;

    /// Registers signer with the aggregator.
    async fn register_signer(
        &self,
        epoch: Epoch,
        signer: &Signer,
    ) -> Result<(), AggregatorClientError>;

    /// Registers single signature with the aggregator.
    async fn register_signature(
        &self,
        signed_entity_type: &SignedEntityType,
        signature: &SingleSignature,
        protocol_message: &ProtocolMessage,
    ) -> Result<(), AggregatorClientError>;

    /// Retrieves aggregator features message from the aggregator
    async fn retrieve_aggregator_features(
        &self,
    ) -> Result<AggregatorFeaturesMessage, AggregatorClientError>;
}

/// AggregatorHTTPClient is a http client for an aggregator
pub struct AggregatorHTTPClient {
    aggregator_endpoint: String,
    relay_endpoint: Option<String>,
    api_version_provider: Arc<APIVersionProvider>,
    timeout_duration: Option<Duration>,
    logger: Logger,
}

impl AggregatorHTTPClient {
    /// AggregatorHTTPClient factory
    pub fn new(
        aggregator_endpoint: String,
        relay_endpoint: Option<String>,
        api_version_provider: Arc<APIVersionProvider>,
        timeout_duration: Option<Duration>,
        logger: Logger,
    ) -> Self {
        let logger = logger.new_with_component_name::<Self>();
        debug!(logger, "New AggregatorHTTPClient created");
        Self {
            aggregator_endpoint,
            relay_endpoint,
            api_version_provider,
            timeout_duration,
            logger,
        }
    }

    fn prepare_http_client(&self) -> Result<Client, AggregatorClientError> {
        let client = match &self.relay_endpoint {
            Some(relay_endpoint) => Client::builder()
                .proxy(
                    Proxy::all(relay_endpoint)
                        .map_err(|e| AggregatorClientError::ProxyCreation(anyhow!(e)))?,
                )
                .build()
                .map_err(|e| AggregatorClientError::HTTPClientCreation(anyhow!(e)))?,
            None => Client::new(),
        };

        Ok(client)
    }

    /// Forge a client request adding protocol version in the headers.
    pub fn prepare_request_builder(&self, request_builder: RequestBuilder) -> RequestBuilder {
        let request_builder = request_builder
            .header(
                MITHRIL_API_VERSION_HEADER,
                self.api_version_provider
                    .compute_current_version()
                    .unwrap()
                    .to_string(),
            )
            .header(MITHRIL_SIGNER_VERSION_HEADER, env!("CARGO_PKG_VERSION"));

        if let Some(duration) = self.timeout_duration {
            request_builder.timeout(duration)
        } else {
            request_builder
        }
    }

    /// API version error handling
    fn handle_api_error(&self, response: &Response) -> AggregatorClientError {
        if let Some(version) = response.headers().get(MITHRIL_API_VERSION_HEADER) {
            AggregatorClientError::ApiVersionMismatch(anyhow!(
                "server version: '{}', signer version: '{}'",
                version.to_str().unwrap(),
                self.api_version_provider.compute_current_version().unwrap()
            ))
        } else {
            AggregatorClientError::ApiVersionMismatch(anyhow!(
                "version precondition failed, sent version '{}'.",
                self.api_version_provider.compute_current_version().unwrap()
            ))
        }
    }
}

#[async_trait]
impl AggregatorClient for AggregatorHTTPClient {
    async fn retrieve_epoch_settings(
        &self,
    ) -> Result<Option<SignerEpochSettings>, AggregatorClientError> {
        debug!(self.logger, "Retrieve epoch settings");
        let url = format!("{}/epoch-settings", self.aggregator_endpoint);
        let response = self
            .prepare_request_builder(self.prepare_http_client()?.get(url.clone()))
            .send()
            .await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::OK => match response.json::<EpochSettingsMessage>().await {
                    Ok(message) => {
                        let epoch_settings = FromEpochSettingsAdapter::try_adapt(message)
                            .map_err(|e| AggregatorClientError::Adapter(anyhow!(e)))?;
                        Ok(Some(epoch_settings))
                    }
                    Err(err) => Err(AggregatorClientError::JsonParseFailed(anyhow!(err))),
                },
                StatusCode::PRECONDITION_FAILED => Err(self.handle_api_error(&response)),
                _ => Err(AggregatorClientError::from_response(response).await),
            },
            Err(err) => Err(AggregatorClientError::RemoteServerUnreachable(anyhow!(err))),
        }
    }

    async fn register_signer(
        &self,
        epoch: Epoch,
        signer: &Signer,
    ) -> Result<(), AggregatorClientError> {
        debug!(self.logger, "Register signer");
        let url = format!("{}/register-signer", self.aggregator_endpoint);
        let register_signer_message =
            ToRegisterSignerMessageAdapter::try_adapt((epoch, signer.to_owned()))
                .map_err(|e| AggregatorClientError::Adapter(anyhow!(e)))?;
        let response = self
            .prepare_request_builder(self.prepare_http_client()?.post(url.clone()))
            .json(&register_signer_message)
            .send()
            .await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => Ok(()),
                StatusCode::PRECONDITION_FAILED => Err(self.handle_api_error(&response)),
                _ => Err(AggregatorClientError::from_response(response).await),
            },
            Err(err) => Err(AggregatorClientError::RemoteServerUnreachable(anyhow!(err))),
        }
    }

    async fn register_signature(
        &self,
        signed_entity_type: &SignedEntityType,
        signature: &SingleSignature,
        protocol_message: &ProtocolMessage,
    ) -> Result<(), AggregatorClientError> {
        debug!(self.logger, "Register signature");
        let url = format!("{}/register-signatures", self.aggregator_endpoint);
        let register_single_signature_message = ToRegisterSignatureMessageAdapter::try_adapt((
            signed_entity_type.to_owned(),
            signature.to_owned(),
            protocol_message,
        ))
        .map_err(|e| AggregatorClientError::Adapter(anyhow!(e)))?;
        let response = self
            .prepare_request_builder(self.prepare_http_client()?.post(url.clone()))
            .json(&register_single_signature_message)
            .send()
            .await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED | StatusCode::ACCEPTED => Ok(()),
                StatusCode::GONE => {
                    let root_cause = AggregatorClientError::get_root_cause(response).await;
                    debug!(self.logger, "Message already certified or expired"; "details" => &root_cause);

                    Ok(())
                }
                StatusCode::PRECONDITION_FAILED => Err(self.handle_api_error(&response)),
                _ => Err(AggregatorClientError::from_response(response).await),
            },
            Err(err) => Err(AggregatorClientError::RemoteServerUnreachable(anyhow!(err))),
        }
    }

    async fn retrieve_aggregator_features(
        &self,
    ) -> Result<AggregatorFeaturesMessage, AggregatorClientError> {
        debug!(self.logger, "Retrieve aggregator features message");
        let url = format!("{}/", self.aggregator_endpoint);
        let response = self
            .prepare_request_builder(self.prepare_http_client()?.get(url.clone()))
            .send()
            .await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(response
                    .json::<AggregatorFeaturesMessage>()
                    .await
                    .map_err(|e| AggregatorClientError::JsonParseFailed(anyhow!(e)))?),
                StatusCode::PRECONDITION_FAILED => Err(self.handle_api_error(&response)),
                _ => Err(AggregatorClientError::from_response(response).await),
            },
            Err(err) => Err(AggregatorClientError::RemoteServerUnreachable(anyhow!(err))),
        }
    }
}

#[cfg(test)]
pub(crate) mod dumb {
    use tokio::sync::RwLock;

    use super::*;

    /// This aggregator client is intended to be used by test services.
    /// It actually does not communicate with an aggregator host but mimics this behavior.
    /// It is driven by a Tester that controls the data it can return, and it can return its internal state for testing.
    pub struct DumbAggregatorClient {
        epoch_settings: RwLock<Option<SignerEpochSettings>>,
        last_registered_signer: RwLock<Option<Signer>>,
        aggregator_features: RwLock<AggregatorFeaturesMessage>,
    }

    impl DumbAggregatorClient {
        /// Return the last signer that called with the `register` method.
        pub async fn get_last_registered_signer(&self) -> Option<Signer> {
            self.last_registered_signer.read().await.clone()
        }

        pub async fn set_aggregator_features(
            &self,
            aggregator_features: AggregatorFeaturesMessage,
        ) {
            let mut aggregator_features_writer = self.aggregator_features.write().await;
            *aggregator_features_writer = aggregator_features;
        }
    }

    impl Default for DumbAggregatorClient {
        fn default() -> Self {
            Self {
                epoch_settings: RwLock::new(Some(SignerEpochSettings::dummy())),
                last_registered_signer: RwLock::new(None),
                aggregator_features: RwLock::new(AggregatorFeaturesMessage::dummy()),
            }
        }
    }

    #[async_trait]
    impl AggregatorClient for DumbAggregatorClient {
        async fn retrieve_epoch_settings(
            &self,
        ) -> Result<Option<SignerEpochSettings>, AggregatorClientError> {
            let epoch_settings = self.epoch_settings.read().await.clone();

            Ok(epoch_settings)
        }

        /// Registers signer with the aggregator
        async fn register_signer(
            &self,
            _epoch: Epoch,
            signer: &Signer,
        ) -> Result<(), AggregatorClientError> {
            let mut last_registered_signer = self.last_registered_signer.write().await;
            let signer = signer.clone();
            *last_registered_signer = Some(signer);

            Ok(())
        }

        /// Registers single signature with the aggregator
        async fn register_signature(
            &self,
            _signed_entity_type: &SignedEntityType,
            _signature: &SingleSignature,
            _protocol_message: &ProtocolMessage,
        ) -> Result<(), AggregatorClientError> {
            Ok(())
        }

        async fn retrieve_aggregator_features(
            &self,
        ) -> Result<AggregatorFeaturesMessage, AggregatorClientError> {
            let aggregator_features = self.aggregator_features.read().await;
            Ok(aggregator_features.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use http::response::Builder as HttpResponseBuilder;
    use httpmock::prelude::*;
    use serde_json::json;

    use mithril_common::api_version::DummyApiVersionDiscriminantSource;
    use mithril_common::entities::Epoch;
    use mithril_common::messages::TryFromMessageAdapter;
    use mithril_common::test_utils::fake_data;

    use crate::test_tools::TestLogger;

    use super::*;

    macro_rules! assert_is_error {
        ($error:expr, $error_type:pat) => {
            assert!(
                matches!($error, $error_type),
                "Expected {} error, got '{:?}'.",
                stringify!($error_type),
                $error
            );
        };
    }

    fn setup_server_and_client() -> (MockServer, AggregatorHTTPClient) {
        let server = MockServer::start();
        let aggregator_endpoint = server.url("");
        let relay_endpoint = None;
        let discriminant_source = DummyApiVersionDiscriminantSource::new("dummy");
        let api_version_provider = APIVersionProvider::new(Arc::new(discriminant_source));

        (
            server,
            AggregatorHTTPClient::new(
                aggregator_endpoint,
                relay_endpoint,
                Arc::new(api_version_provider),
                None,
                TestLogger::stdout(),
            ),
        )
    }

    fn set_returning_412(server: &MockServer) {
        server.mock(|_, then| {
            then.status(412)
                .header(MITHRIL_API_VERSION_HEADER, "0.0.999");
        });
    }

    fn set_returning_500(server: &MockServer) {
        server.mock(|_, then| {
            then.status(500).body("an error occurred");
        });
    }

    fn set_unparsable_json(server: &MockServer) {
        server.mock(|_, then| {
            then.status(200).body("this is not a json");
        });
    }

    fn build_text_response<T: Into<String>>(status_code: StatusCode, body: T) -> Response {
        HttpResponseBuilder::new()
            .status(status_code)
            .body(body.into())
            .unwrap()
            .into()
    }

    fn build_json_response<T: serde::Serialize>(status_code: StatusCode, body: &T) -> Response {
        HttpResponseBuilder::new()
            .status(status_code)
            .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
            .body(serde_json::to_string(&body).unwrap())
            .unwrap()
            .into()
    }

    macro_rules! assert_error_text_contains {
        ($error: expr, $expect_contains: expr) => {
            let error = &$error;
            assert!(
                error.contains($expect_contains),
                "Expected error message to contain '{}'\ngot '{error:?}'",
                $expect_contains,
            );
        };
    }

    #[tokio::test]
    async fn test_aggregator_features_ok_200() {
        let (server, client) = setup_server_and_client();
        let message_expected = AggregatorFeaturesMessage::dummy();
        let _server_mock = server.mock(|when, then| {
            when.path("/");
            then.status(200).body(json!(message_expected).to_string());
        });

        let message = client.retrieve_aggregator_features().await.unwrap();

        assert_eq!(message_expected, message);
    }

    #[tokio::test]
    async fn test_aggregator_features_ko_412() {
        let (server, client) = setup_server_and_client();
        set_returning_412(&server);

        let error = client.retrieve_aggregator_features().await.unwrap_err();

        assert_is_error!(error, AggregatorClientError::ApiVersionMismatch(_));
    }

    #[tokio::test]
    async fn test_aggregator_features_ko_500() {
        let (server, client) = setup_server_and_client();
        set_returning_500(&server);

        let error = client.retrieve_aggregator_features().await.unwrap_err();

        assert_is_error!(error, AggregatorClientError::RemoteServerTechnical(_));
    }

    #[tokio::test]
    async fn test_aggregator_features_ko_json_serialization() {
        let (server, client) = setup_server_and_client();
        set_unparsable_json(&server);

        let error = client.retrieve_aggregator_features().await.unwrap_err();

        assert_is_error!(error, AggregatorClientError::JsonParseFailed(_));
    }

    #[tokio::test]
    async fn test_aggregator_features_timeout() {
        let (server, mut client) = setup_server_and_client();
        client.timeout_duration = Some(Duration::from_millis(10));
        let _server_mock = server.mock(|when, then| {
            when.path("/");
            then.delay(Duration::from_millis(100));
        });

        let error = client.retrieve_aggregator_features().await.unwrap_err();

        assert_is_error!(error, AggregatorClientError::RemoteServerUnreachable(_));
    }

    #[tokio::test]
    async fn test_epoch_settings_ok_200() {
        let (server, client) = setup_server_and_client();
        let epoch_settings_expected = EpochSettingsMessage::dummy();
        let _server_mock = server.mock(|when, then| {
            when.path("/epoch-settings");
            then.status(200)
                .body(json!(epoch_settings_expected).to_string());
        });

        let epoch_settings = client.retrieve_epoch_settings().await;
        epoch_settings.as_ref().expect("unexpected error");
        assert_eq!(
            FromEpochSettingsAdapter::try_adapt(epoch_settings_expected).unwrap(),
            epoch_settings.unwrap().unwrap()
        );
    }

    #[tokio::test]
    async fn test_epoch_settings_ko_412() {
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.path("/epoch-settings");
            then.status(412)
                .header(MITHRIL_API_VERSION_HEADER, "0.0.999");
        });

        let epoch_settings = client.retrieve_epoch_settings().await.unwrap_err();

        assert!(epoch_settings.is_api_version_mismatch());
    }

    #[tokio::test]
    async fn test_epoch_settings_ko_500() {
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.path("/epoch-settings");
            then.status(500).body("an error occurred");
        });

        match client.retrieve_epoch_settings().await.unwrap_err() {
            AggregatorClientError::RemoteServerTechnical(_) => (),
            e => panic!("Expected Aggregator::RemoteServerTechnical error, got '{e:?}'."),
        };
    }

    #[tokio::test]
    async fn test_epoch_settings_timeout() {
        let (server, mut client) = setup_server_and_client();
        client.timeout_duration = Some(Duration::from_millis(10));
        let _server_mock = server.mock(|when, then| {
            when.path("/epoch-settings");
            then.delay(Duration::from_millis(100));
        });

        let error = client
            .retrieve_epoch_settings()
            .await
            .expect_err("retrieve_epoch_settings should fail");

        assert!(
            matches!(error, AggregatorClientError::RemoteServerUnreachable(_)),
            "unexpected error type: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_register_signer_ok_201() {
        let epoch = Epoch(1);
        let single_signers = fake_data::signers(1);
        let single_signer = single_signers.first().unwrap();
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signer");
            then.status(201);
        });

        let register_signer = client.register_signer(epoch, single_signer).await;
        register_signer.expect("unexpected error");
    }

    #[tokio::test]
    async fn test_register_signer_ko_412() {
        let epoch = Epoch(1);
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signer");
            then.status(412)
                .header(MITHRIL_API_VERSION_HEADER, "0.0.999");
        });
        let single_signers = fake_data::signers(1);
        let single_signer = single_signers.first().unwrap();

        let error = client
            .register_signer(epoch, single_signer)
            .await
            .unwrap_err();

        assert!(error.is_api_version_mismatch());
    }

    #[tokio::test]
    async fn test_register_signer_ko_400() {
        let epoch = Epoch(1);
        let single_signers = fake_data::signers(1);
        let single_signer = single_signers.first().unwrap();
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signer");
            then.status(400).body(
                serde_json::to_vec(&ClientError::new(
                    "error".to_string(),
                    "an error".to_string(),
                ))
                .unwrap(),
            );
        });

        match client
            .register_signer(epoch, single_signer)
            .await
            .unwrap_err()
        {
            AggregatorClientError::RemoteServerLogical(_) => (),
            err => {
                panic!(
                    "Expected a AggregatorClientError::RemoteServerLogical error, got '{err:?}'."
                )
            }
        };
    }

    #[tokio::test]
    async fn test_register_signer_ko_500() {
        let epoch = Epoch(1);
        let single_signers = fake_data::signers(1);
        let single_signer = single_signers.first().unwrap();
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signer");
            then.status(500).body("an error occurred");
        });

        match client
            .register_signer(epoch, single_signer)
            .await
            .unwrap_err()
        {
            AggregatorClientError::RemoteServerTechnical(_) => (),
            e => panic!("Expected Aggregator::RemoteServerTechnical error, got '{e:?}'."),
        };
    }

    #[tokio::test]
    async fn test_register_signer_timeout() {
        let epoch = Epoch(1);
        let single_signers = fake_data::signers(1);
        let single_signer = single_signers.first().unwrap();
        let (server, mut client) = setup_server_and_client();
        client.timeout_duration = Some(Duration::from_millis(10));
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signer");
            then.delay(Duration::from_millis(100));
        });

        let error = client
            .register_signer(epoch, single_signer)
            .await
            .expect_err("register_signer should fail");

        assert!(
            matches!(error, AggregatorClientError::RemoteServerUnreachable(_)),
            "unexpected error type: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_register_signature_ok_201() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(201);
        });

        let register_signature = client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await;
        register_signature.expect("unexpected error");
    }

    #[tokio::test]
    async fn test_register_signature_ok_202() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(202);
        });

        let register_signature = client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await;
        register_signature.expect("unexpected error");
    }

    #[tokio::test]
    async fn test_register_signature_ko_412() {
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(412)
                .header(MITHRIL_API_VERSION_HEADER, "0.0.999");
        });
        let single_signature = fake_data::single_signature((1..5).collect());

        let error = client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .unwrap_err();

        assert!(error.is_api_version_mismatch());
    }

    #[tokio::test]
    async fn test_register_signature_ko_400() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(400).body(
                serde_json::to_vec(&ClientError::new(
                    "error".to_string(),
                    "an error".to_string(),
                ))
                .unwrap(),
            );
        });

        match client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .unwrap_err()
        {
            AggregatorClientError::RemoteServerLogical(_) => (),
            e => panic!("Expected Aggregator::RemoteServerLogical error, got '{e:?}'."),
        };
    }

    #[tokio::test]
    async fn test_register_signature_ok_410_log_response_body() {
        let (logger, log_inspector) = TestLogger::memory();

        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, mut client) = setup_server_and_client();
        client.logger = logger;
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(410).body(
                serde_json::to_vec(&ClientError::new(
                    "already_aggregated".to_string(),
                    "too late".to_string(),
                ))
                .unwrap(),
            );
        });

        client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .expect("Should not fail when status is 410 (GONE)");

        assert!(log_inspector.contains_log("already_aggregated"));
        assert!(log_inspector.contains_log("too late"));
    }

    #[tokio::test]
    async fn test_register_signature_ko_409() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(409);
        });

        match client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .unwrap_err()
        {
            AggregatorClientError::RemoteServerLogical(_) => (),
            e => panic!("Expected Aggregator::RemoteServerLogical error, got '{e:?}'."),
        }
    }

    #[tokio::test]
    async fn test_register_signature_ko_500() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, client) = setup_server_and_client();
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.status(500).body("an error occurred");
        });

        match client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .unwrap_err()
        {
            AggregatorClientError::RemoteServerTechnical(_) => (),
            e => panic!("Expected Aggregator::RemoteServerTechnical error, got '{e:?}'."),
        };
    }

    #[tokio::test]
    async fn test_register_signature_timeout() {
        let single_signature = fake_data::single_signature((1..5).collect());
        let (server, mut client) = setup_server_and_client();
        client.timeout_duration = Some(Duration::from_millis(10));
        let _server_mock = server.mock(|when, then| {
            when.method(POST).path("/register-signatures");
            then.delay(Duration::from_millis(100));
        });

        let error = client
            .register_signature(
                &SignedEntityType::dummy(),
                &single_signature,
                &ProtocolMessage::default(),
            )
            .await
            .expect_err("register_signature should fail");

        assert!(
            matches!(error, AggregatorClientError::RemoteServerUnreachable(_)),
            "unexpected error type: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_4xx_errors_are_handled_as_remote_server_logical() {
        let response = build_text_response(StatusCode::BAD_REQUEST, "error text");
        let handled_error = AggregatorClientError::from_response(response).await;

        assert!(
            matches!(
                handled_error,
                AggregatorClientError::RemoteServerLogical(..)
            ),
            "Expected error to be RemoteServerLogical\ngot '{handled_error:?}'",
        );
    }

    #[tokio::test]
    async fn test_5xx_errors_are_handled_as_remote_server_technical() {
        let response = build_text_response(StatusCode::INTERNAL_SERVER_ERROR, "error text");
        let handled_error = AggregatorClientError::from_response(response).await;

        assert!(
            matches!(
                handled_error,
                AggregatorClientError::RemoteServerTechnical(..)
            ),
            "Expected error to be RemoteServerLogical\ngot '{handled_error:?}'",
        );
    }

    #[tokio::test]
    async fn test_550_error_is_handled_as_registration_round_not_yet_opened() {
        let response = build_text_response(StatusCode::from_u16(550).unwrap(), "Not yet available");
        let handled_error = AggregatorClientError::from_response(response).await;

        assert!(
            matches!(
                handled_error,
                AggregatorClientError::RegistrationRoundNotYetOpened(..)
            ),
            "Expected error to be RegistrationRoundNotYetOpened\ngot '{handled_error:?}'",
        );
    }

    #[tokio::test]
    async fn test_non_4xx_or_5xx_errors_are_handled_as_unhandled_status_code_and_contains_response_text(
    ) {
        let response = build_text_response(StatusCode::OK, "ok text");
        let handled_error = AggregatorClientError::from_response(response).await;

        assert!(
            matches!(
                handled_error,
                AggregatorClientError::UnhandledStatusCode(..) if format!("{handled_error:?}").contains("ok text")
            ),
            "Expected error to be UnhandledStatusCode with 'ok text' in error text\ngot '{handled_error:?}'",
        );
    }

    #[tokio::test]
    async fn test_root_cause_of_non_json_response_contains_response_plain_text() {
        let error_text = "An error occurred; please try again later.";
        let response = build_text_response(StatusCode::EXPECTATION_FAILED, error_text);

        assert_error_text_contains!(
            AggregatorClientError::get_root_cause(response).await,
            "expectation failed: An error occurred; please try again later."
        );
    }

    #[tokio::test]
    async fn test_root_cause_of_json_formatted_client_error_response_contains_error_label_and_message(
    ) {
        let client_error = ClientError::new("label", "message");
        let response = build_json_response(StatusCode::BAD_REQUEST, &client_error);

        assert_error_text_contains!(
            AggregatorClientError::get_root_cause(response).await,
            "bad request: label: message"
        );
    }

    #[tokio::test]
    async fn test_root_cause_of_json_formatted_server_error_response_contains_error_label_and_message(
    ) {
        let server_error = ServerError::new("message");
        let response = build_json_response(StatusCode::BAD_REQUEST, &server_error);

        assert_error_text_contains!(
            AggregatorClientError::get_root_cause(response).await,
            "bad request: message"
        );
    }

    #[tokio::test]
    async fn test_root_cause_of_unknown_formatted_json_response_contains_json_key_value_pairs() {
        let response = build_json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &json!({ "second": "unknown", "first": "foreign" }),
        );

        assert_error_text_contains!(
            AggregatorClientError::get_root_cause(response).await,
            r#"internal server error: {"first":"foreign","second":"unknown"}"#
        );
    }

    #[tokio::test]
    async fn test_root_cause_with_invalid_json_response_still_contains_response_status_name() {
        let response = HttpResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, JSON_CONTENT_TYPE)
            .body(r#"{"invalid":"unexpected dot", "key": "value".}"#)
            .unwrap()
            .into();

        let root_cause = AggregatorClientError::get_root_cause(response).await;

        assert_error_text_contains!(root_cause, "bad request");
        assert!(
            !root_cause.contains("bad request: "),
            "Expected error message should not contain additional information \ngot '{root_cause:?}'"
        );
    }

    #[tokio::test]
    async fn test_sends_accept_encoding_header() {
        let (server, client) = setup_server_and_client();
        server.mock(|when, then| {
            when.matches(|req| {
                let headers = req.headers.clone().expect("HTTP headers not found");
                let accept_encoding_header = headers
                    .iter()
                    .find(|(name, _values)| name.to_lowercase() == "accept-encoding")
                    .expect("Accept-Encoding header not found");

                let header_value = accept_encoding_header.clone().1;
                ["gzip", "br", "deflate", "zstd"]
                    .iter()
                    .all(|&value| header_value.contains(value))
            });

            then.status(201);
        });

        client
            .register_signature(
                &SignedEntityType::dummy(),
                &fake_data::single_signature((1..5).collect()),
                &ProtocolMessage::default(),
            )
            .await
            .expect("Should succeed with Accept-Encoding header");
    }
}

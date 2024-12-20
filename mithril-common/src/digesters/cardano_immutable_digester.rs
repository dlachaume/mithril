use crate::{
    digesters::{
        cache::ImmutableFileDigestCacheProvider, ImmutableDigester, ImmutableDigesterError,
        ImmutableFile,
    },
    entities::{CardanoDbBeacon, HexEncodedDigest, ImmutableFileName},
    logging::LoggerExtensions,
};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use slog::{debug, info, warn, Logger};
use std::{collections::BTreeMap, io, path::Path, sync::Arc};

/// Result of a cache computation, contains the digest and the list of new entries to add
/// to the [ImmutableFileDigestCacheProvider].
type CacheComputationResult =
    Result<([u8; 32], Vec<(ImmutableFileName, HexEncodedDigest)>), io::Error>;

/// A digester working directly on a Cardano DB immutables files
pub struct CardanoImmutableDigester {
    cardano_network: String,

    /// A [ImmutableFileDigestCacheProvider] instance
    cache_provider: Option<Arc<dyn ImmutableFileDigestCacheProvider>>,

    /// The logger where the logs should be written
    logger: Logger,
}

impl CardanoImmutableDigester {
    /// ImmutableDigester factory
    pub fn new(
        cardano_network: String,
        cache_provider: Option<Arc<dyn ImmutableFileDigestCacheProvider>>,
        logger: Logger,
    ) -> Self {
        Self {
            cardano_network,
            cache_provider,
            logger: logger.new_with_component_name::<Self>(),
        }
    }
}

#[async_trait]
impl ImmutableDigester for CardanoImmutableDigester {
    async fn compute_digest(
        &self,
        dirpath: &Path,
        beacon: &CardanoDbBeacon,
    ) -> Result<String, ImmutableDigesterError> {
        let up_to_file_number = beacon.immutable_file_number;
        let immutables = ImmutableFile::list_completed_in_dir(dirpath)?
            .into_iter()
            .filter(|f| f.number <= up_to_file_number)
            .collect::<Vec<_>>();
        info!(self.logger, ">> compute_digest"; "beacon" => #?beacon, "nb_of_immutables" => immutables.len());

        match immutables.last() {
            None => Err(ImmutableDigesterError::NotEnoughImmutable {
                expected_number: up_to_file_number,
                found_number: None,
                db_dir: dirpath.to_owned(),
            }),
            Some(last_immutable_file) if last_immutable_file.number < up_to_file_number => {
                Err(ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: up_to_file_number,
                    found_number: Some(last_immutable_file.number),
                    db_dir: dirpath.to_owned(),
                })
            }
            Some(_) => {
                let cached_values = match self.cache_provider.as_ref() {
                    None => BTreeMap::from_iter(immutables.into_iter().map(|i| (i, None))),
                    Some(cache_provider) => match cache_provider.get(immutables.clone()).await {
                        Ok(values) => values,
                        Err(error) => {
                            warn!(
                                self.logger, "Error while getting cached immutable files digests";
                                "error" => ?error
                            );
                            BTreeMap::from_iter(immutables.into_iter().map(|i| (i, None)))
                        }
                    },
                };

                // digest is done in a separate thread because it is blocking the whole task
                let logger = self.logger.clone();
                let thread_cardano_network = self.cardano_network.clone();
                let thread_beacon = beacon.clone();
                let (hash, new_cache_entries) =
                    tokio::task::spawn_blocking(move || -> CacheComputationResult {
                        compute_hash(
                            logger,
                            thread_cardano_network,
                            &thread_beacon,
                            cached_values,
                        )
                    })
                    .await
                    .map_err(|e| ImmutableDigesterError::DigestComputationError(e.into()))??;
                let digest = hex::encode(hash);

                debug!(self.logger, "Computed digest: {digest:?}");

                if let Some(cache_provider) = self.cache_provider.as_ref() {
                    if let Err(error) = cache_provider.store(new_cache_entries).await {
                        warn!(
                            self.logger, "Error while storing new immutable files digests to cache";
                            "error" => ?error
                        );
                    }
                }

                Ok(digest)
            }
        }
    }
}

fn compute_hash(
    logger: Logger,
    cardano_network: String,
    beacon: &CardanoDbBeacon,
    entries: BTreeMap<ImmutableFile, Option<HexEncodedDigest>>,
) -> CacheComputationResult {
    let mut hasher = Sha256::new();
    let mut new_cached_entries = Vec::new();
    let mut progress = Progress {
        index: 0,
        total: entries.len(),
    };

    hasher.update(compute_beacon_hash(&cardano_network, beacon).as_bytes());

    for (ix, (entry, cache)) in entries.iter().enumerate() {
        match cache {
            None => {
                let data = hex::encode(entry.compute_raw_hash::<Sha256>()?);
                hasher.update(&data);
                new_cached_entries.push((entry.filename.clone(), data));
            }
            Some(digest) => {
                hasher.update(digest);
            }
        };

        if progress.report(ix) {
            info!(logger, "Hashing: {progress}");
        }
    }

    Ok((hasher.finalize().into(), new_cached_entries))
}

fn compute_beacon_hash(network: &str, cardano_db_beacon: &CardanoDbBeacon) -> String {
    let mut hasher = Sha256::new();
    hasher.update(network.as_bytes());
    hasher.update(cardano_db_beacon.epoch.to_be_bytes());
    hasher.update(cardano_db_beacon.immutable_file_number.to_be_bytes());
    hex::encode(hasher.finalize())
}

struct Progress {
    index: usize,
    total: usize,
}

impl Progress {
    fn report(&mut self, ix: usize) -> bool {
        self.index = ix;
        (20 * ix) % self.total == 0
    }

    fn percent(&self) -> f64 {
        (self.index as f64 * 100.0 / self.total as f64).ceil()
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}/{} ({}%)", self.index, self.total, self.percent())
    }
}

#[cfg(test)]
mod tests {
    use sha2::Sha256;
    use std::{collections::BTreeMap, io, sync::Arc};
    use tokio::time::Instant;

    use crate::{
        digesters::{
            cache::{
                ImmutableDigesterCacheGetError, ImmutableDigesterCacheProviderError,
                ImmutableDigesterCacheStoreError, MemoryImmutableFileDigestCacheProvider,
                MockImmutableFileDigestCacheProvider,
            },
            DummyImmutablesDbBuilder,
        },
        entities::ImmutableFileNumber,
        test_utils::TestLogger,
    };

    use super::*;

    fn db_builder(dir_name: &str) -> DummyImmutablesDbBuilder {
        DummyImmutablesDbBuilder::new(&format!("cardano_immutable_digester/{dir_name}"))
    }

    #[test]
    fn test_compute_beacon_hash() {
        let hash_expected = "48cbf709b56204d8315aefd3a416b45398094f6fd51785c5b7dcaf7f35aacbfb";
        let (network, epoch, immutable_file_number) = ("testnet", 10, 100);

        assert_eq!(
            hash_expected,
            compute_beacon_hash(network, &CardanoDbBeacon::new(epoch, immutable_file_number))
        );
        assert_ne!(
            hash_expected,
            compute_beacon_hash(
                "mainnet",
                &CardanoDbBeacon::new(epoch, immutable_file_number)
            )
        );
        assert_ne!(
            hash_expected,
            compute_beacon_hash(network, &CardanoDbBeacon::new(20, immutable_file_number))
        );
        assert_ne!(
            hash_expected,
            compute_beacon_hash(network, &CardanoDbBeacon::new(epoch, 200))
        );
    }

    #[test]
    fn reports_progress_every_5_percent() {
        let mut progress = Progress {
            index: 0,
            total: 7000,
        };

        assert!(!progress.report(1));
        assert!(!progress.report(4));
        assert!(progress.report(350));
        assert!(!progress.report(351));
    }

    #[test]
    fn reports_progress_when_total_lower_than_20() {
        let mut progress = Progress {
            index: 0,
            total: 16,
        };

        assert!(progress.report(4));
        assert!(progress.report(12));
        assert!(!progress.report(3));
        assert!(!progress.report(15));
    }

    #[tokio::test]
    async fn fail_if_no_file_in_folder() {
        let immutable_db = db_builder("fail_if_no_file_in_folder").build();
        let digester =
            CardanoImmutableDigester::new("devnet".to_string(), None, TestLogger::stdout());
        let beacon = CardanoDbBeacon::new(1, 1);

        let result = digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect_err("compute_digest should have failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: None,
                    db_dir: immutable_db.dir,
                }
            ),
            format!("{result:?}")
        );
    }

    #[tokio::test]
    async fn fail_if_a_invalid_file_is_in_immutable_folder() {
        let immutable_db = db_builder("fail_if_no_immutable_exist")
            .with_non_immutables(&["not_immutable"])
            .build();
        let digester =
            CardanoImmutableDigester::new("devnet".to_string(), None, TestLogger::stdout());
        let beacon = CardanoDbBeacon::new(1, 1);

        assert!(digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fail_if_theres_only_the_uncompleted_immutable_trio() {
        let immutable_db = db_builder("fail_if_theres_only_the_uncompleted_immutable_trio")
            .append_immutable_trio()
            .build();
        let digester =
            CardanoImmutableDigester::new("devnet".to_string(), None, TestLogger::stdout());
        let beacon = CardanoDbBeacon::new(1, 1);

        let result = digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect_err("compute_digest should've failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: None,
                    db_dir: immutable_db.dir,
                }
            ),
            format!("{result:?}")
        );
    }

    #[tokio::test]
    async fn fail_if_less_immutable_than_what_required_in_beacon() {
        let immutable_db = db_builder("fail_if_less_immutable_than_what_required_in_beacon")
            .with_immutables(&[1, 2, 3, 4, 5])
            .append_immutable_trio()
            .build();
        let digester =
            CardanoImmutableDigester::new("devnet".to_string(), None, TestLogger::stdout());
        let beacon = CardanoDbBeacon::new(1, 10);

        let result = digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect_err("compute_digest should've failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: Some(5),
                    db_dir: immutable_db.dir,
                }
            ),
            format!("{result:?}")
        );
    }

    #[tokio::test]
    async fn can_compute_hash_of_a_hundred_immutable_file_trio() {
        let immutable_db = db_builder("can_compute_hash_of_a_hundred_immutable_file_trio")
            .with_immutables(&(1..=100).collect::<Vec<ImmutableFileNumber>>())
            .append_immutable_trio()
            .build();
        let logger = TestLogger::stdout();
        let digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(Arc::new(MemoryImmutableFileDigestCacheProvider::default())),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 100);

        let result = digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");

        assert_eq!(
            "a27fd67e495c2c77e4b6b0af9925b2b0bc39656c56adfad4aaab9f20fae49122".to_string(),
            result
        )
    }

    #[tokio::test]
    async fn digests_are_stored_into_cache_provider() {
        let immutable_db = db_builder("digests_are_stored_into_cache_provider")
            .with_immutables(&[1, 2])
            .append_immutable_trio()
            .build();
        let immutables = immutable_db.immutables_files;
        let cache = Arc::new(MemoryImmutableFileDigestCacheProvider::default());
        let logger = TestLogger::stdout();
        let digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(cache.clone()),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 2);

        digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");

        let cached_entries = cache
            .get(immutables.clone())
            .await
            .expect("Cache read should not fail");
        let expected: BTreeMap<_, _> = immutables
            .into_iter()
            .map(|i| {
                let digest = hex::encode(i.compute_raw_hash::<Sha256>().unwrap());
                (i, Some(digest))
            })
            .collect();

        assert_eq!(expected, cached_entries);
    }

    #[tokio::test]
    async fn computed_digest_with_cold_or_hot_or_without_any_cache_are_equals() {
        let immutable_db = DummyImmutablesDbBuilder::new(
            "computed_digest_with_cold_or_hot_or_without_any_cache_are_equals",
        )
        .with_immutables(&[1, 2, 3])
        .append_immutable_trio()
        .build();
        let logger = TestLogger::stdout();
        let no_cache_digester =
            CardanoImmutableDigester::new("devnet".to_string(), None, logger.clone());
        let cache_digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(Arc::new(MemoryImmutableFileDigestCacheProvider::default())),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 3);

        let without_cache_digest = no_cache_digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");

        let cold_cache_digest = cache_digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");

        let full_cache_digest = cache_digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");

        assert_eq!(
            without_cache_digest, full_cache_digest,
            "Digests with or without cache should be the same"
        );

        assert_eq!(
            cold_cache_digest, full_cache_digest,
            "Digests with cold or with hot cache should be the same"
        );
    }

    #[tokio::test]
    async fn hash_computation_is_quicker_with_a_full_cache() {
        let immutable_db = db_builder("hash_computation_is_quicker_with_a_full_cache")
            .with_immutables(&(1..=50).collect::<Vec<ImmutableFileNumber>>())
            .append_immutable_trio()
            .set_file_size(65536)
            .build();
        let cache = MemoryImmutableFileDigestCacheProvider::default();
        let logger = TestLogger::stdout();
        let digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(Arc::new(cache)),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 50);

        let now = Instant::now();
        digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");
        let elapsed_without_cache = now.elapsed();

        let now = Instant::now();
        digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail");
        let elapsed_with_cache = now.elapsed();

        // Note real performance doesn't matter here, the purpose is only to check that the computation
        // time is faster with cache.
        // We set the limit to 90% to avoid flakiness and ensure that the cache is useful (Note: Real
        // performance is around ~100 times faster in debug).
        assert!(
            elapsed_with_cache < (elapsed_without_cache * 9 / 10),
            "digest computation with full cache should be faster than without cache,\
            time elapsed: with cache {elapsed_with_cache:?}, without cache {elapsed_without_cache:?}"
        );
    }

    #[tokio::test]
    async fn cache_read_failure_dont_block_computation() {
        let immutable_db = db_builder("cache_read_failure_dont_block_computation")
            .with_immutables(&[1, 2, 3])
            .append_immutable_trio()
            .build();
        let mut cache = MockImmutableFileDigestCacheProvider::new();
        cache.expect_get().returning(|_| Ok(BTreeMap::new()));
        cache.expect_store().returning(|_| {
            Err(ImmutableDigesterCacheProviderError::Store(
                ImmutableDigesterCacheStoreError::Io(io::Error::new(io::ErrorKind::Other, "error")),
            ))
        });
        let logger = TestLogger::stdout();
        let digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(Arc::new(cache)),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 3);

        digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail even with cache write failure");
    }

    #[tokio::test]
    async fn cache_write_failure_dont_block_computation() {
        let immutable_db = db_builder("cache_write_failure_dont_block_computation")
            .with_immutables(&[1, 2, 3])
            .append_immutable_trio()
            .build();
        let mut cache = MockImmutableFileDigestCacheProvider::new();
        cache.expect_get().returning(|_| {
            Err(ImmutableDigesterCacheProviderError::Get(
                ImmutableDigesterCacheGetError::Io(io::Error::new(io::ErrorKind::Other, "error")),
            ))
        });
        cache.expect_store().returning(|_| Ok(()));
        let logger = TestLogger::stdout();
        let digester = CardanoImmutableDigester::new(
            "devnet".to_string(),
            Some(Arc::new(cache)),
            logger.clone(),
        );
        let beacon = CardanoDbBeacon::new(1, 3);

        digester
            .compute_digest(&immutable_db.dir, &beacon)
            .await
            .expect("compute_digest must not fail even with cache read failure");
    }
}

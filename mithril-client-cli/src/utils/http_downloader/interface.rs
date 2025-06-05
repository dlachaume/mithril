use std::path::{Path, PathBuf};

use async_trait::async_trait;
use reqwest::Url;

use mithril_client::MithrilResult;

/// Trait for downloading a file over HTTP from a URL,
/// saving it to a target directory with the given filename.
///
/// Returns the path to the downloaded file.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HttpDownloader {
    async fn download_file(
        &self,
        url: Url,
        download_dir: &Path,
        filename: &str,
    ) -> MithrilResult<PathBuf>;
}

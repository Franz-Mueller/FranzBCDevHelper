use crate::bc::version::{BcVersion, BcVersionError};
use crate::bc_container::{BcArtifact, Manifest};
use crate::utils::file_handling::extract;
use git2::AttrValue::False;
use reqwest::{self, StatusCode};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

// TODO Testing
// TODO implement function that checks if country and deployment type are valid to prevent creation of wrong paths
// TODO Concurrent resolve() calls can corrupt each other

pub struct ArtifactRequest {
    pub deployment_type: String,
    pub version: BcVersion,
    pub country: String,
}

pub struct ArtifactResolver {
    client: reqwest::Client,
    base_url: Url,
    cache_path: PathBuf,
}

impl ArtifactResolver {
    /// # Example
    ///
    /// ```rust
    /// let request = ArtifactRequest {deployment_type: deployment_type, version: version, country: country};
    /// let artifact = state.artifact_resolver.resolve(request).await?;
    /// ```
    ///
    /// `state` = `AppState`
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(), // TODO think about timeouts
            base_url: Url::parse("https://bcartifacts-exdbf9fwegejdqak.b02.azurefd.net").unwrap(), // TODO additional URLs
            cache_path,
        }
    }

    pub async fn resolve(&self, request: ArtifactRequest) -> Result<BcArtifact, ArtifactError> {
        let requested_path = self.artifact_path(&request, &request.version);

        if tokio::fs::try_exists(&requested_path).await? {
            let url = self.artifact_url(&request, &request.version);
            let manifest = Manifest::from_file(&requested_path.join("manifest.json"))
                .await
                .unwrap();
            let platform_path = self.platform_artifact_path(&request, &request.version);

            if !tokio::fs::try_exists(&platform_path).await? {
                self.dowload_platform_artifact(&manifest, &platform_path)
                    .await?;
            }

            return Ok(BcArtifact::new(
                request.deployment_type,
                request.version,
                request.country,
                requested_path,
                url,
                platform_path,
                manifest,
            ));
        }

        let version = self.resolve_version(&request).await?;
        let path = self.artifact_path(&request, &version);
        let url = self.artifact_url(&request, &version);

        if !tokio::fs::try_exists(&path).await? {
            self.download_artifact(&url, &path).await?;
        }

        let platform_path = self.platform_artifact_path(&request, &version);
        let manifest = Manifest::from_file(&path.join("manifest.json"))
            .await
            .unwrap();
        if !tokio::fs::try_exists(&platform_path).await? {
            self.dowload_platform_artifact(&manifest, &platform_path)
                .await?;
        }

        Ok(BcArtifact::new(
            request.deployment_type,
            version,
            request.country,
            path,
            url,
            platform_path,
            manifest,
        ))
    }

    async fn dowload_platform_artifact(
        &self,
        manifest: &Manifest,
        platform_path: &PathBuf,
    ) -> Result<(), ArtifactError> {
        let platform_url = self.platform_url(&manifest);
        self.download_artifact(&platform_url, platform_path).await?;
        Ok(())
    }

    async fn resolve_version(&self, request: &ArtifactRequest) -> Result<BcVersion, ArtifactError> {
        let url = self.artifact_url(request, &request.version);

        if self.artifact_exists(&url).await? {
            return Ok(request.version);
        }

        self.get_next_bc_version(request).await
    }

    async fn artifact_exists(&self, url: &Url) -> Result<bool, ArtifactError> {
        let response = self.client.head(url.clone()).send().await?;

        match response.status() {
            status if status.is_success() => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            status => Err(ArtifactError::UnexpectedStatus {
                status,
                url: url.clone(),
            }),
        }
    }

    async fn get_next_bc_version(
        &self,
        artifact_request: &ArtifactRequest,
    ) -> Result<BcVersion, ArtifactError> {
        let url = self.version_index_url(artifact_request);

        // Data is expected to arrive in this format:
        // [{"Version":"15.4.41023.43755","CreationTime":"2020-06-26T00:13:59Z"},
        // {"Version":"16.0.11240.31204","CreationTime":"2021-10-11T08:49:00Z"}]

        let response = self.client.get(url).send().await?.error_for_status()?;

        let bytes = response.bytes().await?;

        let entries: Vec<VersionEntry> = serde_json::from_slice(&bytes)?;

        let versions = entries
            .into_iter()
            .map(|entry| entry.version.parse())
            .collect::<Result<Vec<BcVersion>, BcVersionError>>()?;

        find_next_higher_version(artifact_request.version, versions)
    }

    fn artifact_path(&self, request: &ArtifactRequest, version: &BcVersion) -> PathBuf {
        self.cache_path
            .join(&request.deployment_type)
            .join(version.to_string())
            .join(&request.country)
    }

    fn platform_artifact_path(&self, request: &ArtifactRequest, version: &BcVersion) -> PathBuf {
        self.cache_path
            .join(&request.deployment_type)
            .join(version.to_string())
            .join("platform".to_string())
    }

    fn artifact_url(&self, request: &ArtifactRequest, version: &BcVersion) -> Url {
        let mut url = self.base_url.clone();

        url.path_segments_mut()
            .expect("HTTPS URL can contain path segments")
            .extend([
                request.deployment_type.as_str(),
                &version.to_string(),
                request.country.as_str(),
            ]);

        url
    }

    fn platform_url(&self, manifest: &Manifest) -> Url {
        let mut url = self.base_url.clone();

        if manifest.platform_url() == "" {
            let deployment_type = match manifest.is_bc_sandbox() {
                true => "sanbox",
                false => "onprem",
            };
            url.path_segments_mut()
                .expect("HTTPS URL can contain path segments")
                .extend([deployment_type, manifest.platform(), "platform"]);
        } else {
            url.path_segments_mut()
                .expect("HTTPS URL can contain path segments")
                .extend([manifest.platform_url()]);
        }

        url
    }

    fn version_index_url(&self, request: &ArtifactRequest) -> Url {
        let mut url = self.base_url.clone();

        let index_file = format!("{}.json", request.country);

        url.path_segments_mut()
            .expect("HTTPS URL can contain path segments")
            .extend([&request.deployment_type, "indexes", &index_file]);

        url
    }

    async fn download_artifact(&self, url: &Url, path: &Path) -> Result<(), ArtifactError> {
        let mut artifact_zip = path.to_path_buf();
        artifact_zip.add_extension("zip");

        if fs::try_exists(&artifact_zip).await? {
            match extract(&artifact_zip, path).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    fs::remove_file(&artifact_zip).await?;
                    return Err(ArtifactError::FileHandling(err));
                }
            } // TODO an error like disk full should not delete the zip
        }

        let temp_zip = artifact_zip.with_extension("zip.part");

        if let Some(parent) = temp_zip.parent() {
            fs::create_dir_all(parent).await?;
        }

        let response = self.client.get(url.clone()).send().await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(ArtifactError::NotFound(url.clone()));
        }

        let mut response = response.error_for_status()?;
        let mut file = fs::File::create(&temp_zip).await?;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        file.sync_all().await?;
        drop(file);

        fs::rename(&temp_zip, &artifact_zip).await?;
        extract(&artifact_zip, path).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct VersionEntry {
    #[serde(rename = "Version")]
    version: String,
}

fn find_next_higher_version(
    searched: BcVersion,
    available: impl IntoIterator<Item = BcVersion>,
) -> Result<BcVersion, ArtifactError> {
    available
        .into_iter()
        .filter(|version| version.major == searched.major && *version > searched)
        .min()
        .ok_or(ArtifactError::NoCompatibleVersion(searched))
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("artifact {0} was not found")]
    NotFound(Url),

    #[error("no suitable artifact version found for {0}")]
    NoCompatibleVersion(BcVersion),

    #[error("invalid BC version: {0}")]
    Version(#[from] BcVersionError),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("invalid artifact version index: {0}")]
    InvalidVersionIndex(#[from] serde_json::Error),

    #[error("background task failed: {0}")]
    Task(#[from] tokio::task::JoinError),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("unexpected HTTP status {status} for {url}")]
    UnexpectedStatus { status: StatusCode, url: Url },

    #[error("file handling error: {0}")]
    FileHandling(#[from] crate::utils::file_handling::FileHandlingError),
}

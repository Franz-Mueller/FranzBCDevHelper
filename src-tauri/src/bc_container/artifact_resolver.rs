use crate::bc::version::BcVersion;
use crate::bc_container::{BcArtifact, Manifest};
use crate::utils::file_handling::extract;
use anyhow::{bail, Context, Result};
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
            base_url: Url::parse("https://bcartifacts-exdbf9fwegejdqak.b02.azurefd.net").unwrap(), // TODO additional URLs, also no unwrap
            cache_path,
        }
    }

    pub async fn resolve(&self, request: ArtifactRequest) -> Result<BcArtifact> {
        let requested_path = self.artifact_path(&request, &request.version);

        if tokio::fs::try_exists(&requested_path)
            .await
            .with_context(|| format!("Failed try_exists on {}", &requested_path.display()))?
        {
            let url = self.artifact_url(&request, &request.version);
            let manifest = Manifest::from_file(&requested_path.join("manifest.json"))
                .await
                .with_context(|| {
                    format!(
                        "Failed to deserialize manifest at {}/manifest.json",
                        &requested_path.display()
                    )
                })?;
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

        let version = self
            .resolve_version(&request)
            .await
            .context("Failed to resolve artifact version")?;
        let path = self.artifact_path(&request, &version);
        let url = self.artifact_url(&request, &version);

        if !tokio::fs::try_exists(&path).await? {
            self.download_artifact(&url, &path).await.with_context(|| {
                format!(
                    "Failed to download artifact from {} to {}",
                    url.clone(),
                    &path.display()
                )
            })?;
        }

        let platform_path = self.platform_artifact_path(&request, &version);
        let manifest = Manifest::from_file(&path.join("manifest.json"))
            .await
            .with_context(|| {
                format!(
                    "Failed to deserialize manifest at {}/manifest.json",
                    &path.display()
                )
            })?;
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
    ) -> Result<()> {
        let platform_url = self.platform_url(&manifest);
        self.download_artifact(&platform_url, platform_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to download platform artifact {} to {}",
                    platform_url.clone(),
                    platform_path.display()
                )
            })?;
        Ok(())
    }

    async fn resolve_version(&self, request: &ArtifactRequest) -> Result<BcVersion> {
        let url = self.artifact_url(request, &request.version);

        if self.artifact_exists(&url).await? {
            return Ok(request.version);
        }

        self.get_next_bc_version(request).await // TODO ask yourself: would it make sense to test if the new artifact exists too? Just in case ms forgets to remove it from the index
    }

    async fn artifact_exists(&self, url: &Url) -> Result<bool> {
        let response = self
            .client
            .head(url.clone())
            .send()
            .await
            .with_context(|| format!("Failed to request artifact from {}", url.clone()))?;

        match response.status() {
            status if status.is_success() => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            status => bail!(format!(
                "Unexpected status {} on artifact url validation for {}",
                status,
                url.clone()
            )),
        }
    }

    async fn get_next_bc_version(&self, artifact_request: &ArtifactRequest) -> Result<BcVersion> {
        let url = self.version_index_url(artifact_request);

        // Data is expected to arrive in this format:
        // [{"Version":"15.4.41023.43755","CreationTime":"2020-06-26T00:13:59Z"},
        // {"Version":"16.0.11240.31204","CreationTime":"2021-10-11T08:49:00Z"}]

        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("Failed to request version index from {url}"))?
            .error_for_status()
            .context("Get request for version index returned an error status")?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to read version index response body")?;

        let entries: Vec<VersionEntry> =
            serde_json::from_slice(&bytes).context("Failed to deserialize version index")?;

        let versions = entries
            .into_iter()
            .map(|entry| entry.version.parse())
            .collect::<Result<Vec<BcVersion>>>()?;

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
                true => "sandbox",
                false => "onprem",
            };
            url.path_segments_mut()
                .expect("HTTPS URL can contain path segments")
                .extend([deployment_type, manifest.version(), "platform"]);
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

    async fn download_artifact(&self, url: &Url, path: &Path) -> Result<()> {
        let mut artifact_zip = path.to_path_buf();
        artifact_zip.add_extension("zip");

        if fs::try_exists(&artifact_zip).await.with_context(|| {
            format!(
                "Failed to check existence of preexisting artifact zip at {}",
                &artifact_zip.display()
            )
        })? {
            match extract(&artifact_zip, path).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    fs::remove_file(&artifact_zip).await?;
                    bail!(format!(
                        "Failed to extract preexisting artifact zip at {} due to: {}",
                        &artifact_zip.display(),
                        err
                    ));
                }
            } // TODO an error like disk full should not delete the zip
        }

        let temp_zip = artifact_zip.with_extension("zip.part");

        if let Some(parent) = temp_zip.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create parent dir for artifact download temp zip at {}",
                    &temp_zip.display()
                )
            })?;
        }

        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .context("Get request for artifact download failed")?;

        let mut response = response
            .error_for_status()
            .context("Artifact get request returned an error status")?;
        let mut file = fs::File::create(&temp_zip).await.with_context(|| {
            format!(
                "Failed to create temporary zip file for artifact download at {}",
                &temp_zip.display()
            )
        })?;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await.with_context(|| {
                format!(
                    "Failed to write chunk from artifact download stream to file {}",
                    &temp_zip.display()
                )
            })?;
        }

        file.flush()
            .await
            .with_context(|| format!("Failed flush on file {}", &temp_zip.display()))?;
        file.sync_all()
            .await
            .with_context(|| format!("Failed sync_all on file {}", &temp_zip.display()))?;
        drop(file);

        fs::rename(&temp_zip, &artifact_zip)
            .await
            .with_context(|| {
                format!(
                    "Failed to rename artifact zip {} to {}",
                    &temp_zip.display(),
                    &artifact_zip.display()
                )
            })?;
        extract(&artifact_zip, path).await.with_context(|| {
            format!(
                "Failed to extract artifact zip {} to {}",
                &artifact_zip.display(),
                path.display()
            )
        })?;
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
) -> Result<BcVersion> {
    match available
        .into_iter()
        .filter(|version| version.major == searched.major && *version > searched)
        .min()
    {
        Some(v) => Ok(v),
        None => bail!(format!(
            "Neither version {searched} nor one above it within the same major could be found."
        )),
    }
}

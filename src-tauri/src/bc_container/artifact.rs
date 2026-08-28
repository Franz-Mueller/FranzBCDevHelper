use crate::bc::version::BcVersion;
use crate::bc_container::Manifest;
use std::path::{Path, PathBuf};
use url::Url;

pub struct BcArtifact {
    deployment_type: String,
    version: BcVersion,
    country: String,
    path: PathBuf,
    url: Url,
    platform_path: PathBuf,
    manifest: Manifest,
}

impl BcArtifact {
    pub fn new(
        deployment_type: String,
        version: BcVersion,
        country: String,
        path: PathBuf,
        url: Url,
        platform_path: PathBuf,
        manifest: Manifest,
    ) -> Self {
        Self {
            deployment_type,
            version,
            country,
            path,
            url,
            platform_path,
            manifest,
        }
    }

    pub fn deployment_type(&self) -> &str {
        &self.deployment_type
    }
    pub fn version(&self) -> BcVersion {
        self.version
    }
    pub fn country(&self) -> &str {
        &self.country
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn platform_path(&self) -> &Path {
        &self.platform_path
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

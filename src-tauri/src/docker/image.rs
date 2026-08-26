use crate::bc::artifact::BcArtifact;
use crate::bc::version::{BcVersion, BcVersionError};
use crate::utils::file_handling::copy_dir_all;
use bollard::{
    body_full,
    query_parameters::{BuildImageOptionsBuilder, ListImagesOptionsBuilder},
    Docker,
};
use bytes::Bytes;
use chrono::Local;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use tar::Builder;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// TODO Testing
// TODO Stream Docker build context instead of loading the complete tar archive into memory
// TODO Move recursive directory copying off the async runtime
// TODO Handle Docker connection errors instead of unwrapping in ImageBuilder::new

static BUILD_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct BcImage {
    id: String,
}

impl BcImage {
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct ImageBuilder {
    docker: Docker,
    build_path: PathBuf,
}

impl ImageBuilder {
    pub fn new(build_path: PathBuf) -> Self {
        let docker = Docker::connect_with_local_defaults().unwrap();
        Self { docker, build_path }
    }

    pub async fn build(&self, artifact: &BcArtifact) -> Result<BcImage, ImageError> {
        let image_name = format!(
            "bc{}{}{}winltsc2025:latest",
            artifact.deployment_type(),
            artifact.version(),
            artifact.country()
        );

        // TODO check if image exists allready

        let build_folder_name = format!(
            "{}-{}-{}",
            image_name.strip_suffix(":latest").unwrap_or(&image_name),
            std::process::id(),
            BUILD_COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        let build_folder = self.build_path.join(build_folder_name);

        tokio::fs::create_dir_all(&build_folder).await?;

        let result = async {
            self.populate_build_folder(&build_folder, artifact).await?;
            let tar_data = self.compress(&build_folder).await?;
            self.execute_docker_build(tar_data, &image_name).await
        }
        .await;

        match result {
            Ok(image_id) => {
                tokio::fs::remove_dir_all(&build_folder).await?;
                Ok(BcImage { id: image_id })
            }
            Err(error) => {
                let _ = tokio::fs::remove_dir_all(&build_folder).await;
                Err(error)
            }
        }
    }

    async fn populate_build_folder(
        &self,
        build_folder: &Path,
        artifact: &BcArtifact,
    ) -> Result<(), ImageError> {
        let manifest = Manifest::from_file(artifact.path().join("manifest.json")).await?;
        let navdvd = build_folder.join("NAVDVD");
        tokio::fs::create_dir(&navdvd).await?;

        tokio::try_join!(
            self.populate_navdvd(&navdvd, artifact, &manifest),
            self.create_dockerfile(build_folder, artifact, &manifest)
        )?;
        Ok(())
    }

    async fn populate_navdvd(
        &self,
        navdvd_folder: &Path,
        artifact: &BcArtifact,
        manifest: &Manifest,
    ) -> Result<(), ImageError> {
        self.copy_demo_db_into_navdvd(navdvd_folder, artifact.path(), manifest)
            .await?;

        let mut entries = tokio::fs::read_dir(artifact.platform_path()).await?;

        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let destination = navdvd_folder.join(&file_name);
            if entry.path().is_dir() {
                copy_dir_all(entry.path(), &destination)?; // TODO async
            } else {
                tokio::fs::copy(entry.path(), &destination).await?;
            }
        }

        for entry in artifact.path().read_dir()? {
            match entry {
                Ok(entry) => {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if [
                        "Installers",
                        "ConfigurationPackages",
                        "TestToolKit",
                        "UpgradeToolKit",
                        "Extensions",
                    ]
                    .contains(&file_name_str.as_ref())
                        || file_name_str.starts_with("Applications")
                    {
                        let destination = navdvd_folder.join(&file_name);
                        if entry.path().is_dir() {
                            copy_dir_all(entry.path(), destination)?; // TODO async
                        } else {
                            tokio::fs::copy(entry.path(), &destination).await?;
                        }
                    }
                }
                Err(e) => return Err(ImageError::Io(e)),
            }
        }
        Ok(())
    }

    async fn copy_demo_db_into_navdvd(
        &self,
        navdvd_folder: &Path,
        artifact_path: &Path,
        manifest: &Manifest,
    ) -> Result<(), ImageError> {
        let db_path = artifact_path.join(manifest.database.replace("\\", "/"));
        let commondata =
            if BcVersion::from_str(&manifest.version)? < BcVersion::from_str("27.0.33344.0")? {
                "CommonAppData"
            } else {
                "CommApp"
            };

        let demo_db_dir = navdvd_folder
            .join("SQLDemoDatabase")
            .join(commondata)
            .join("Microsoft")
            .join("Microsoft Dynamics NAV")
            .join("ver")
            .join("Database");

        tokio::fs::create_dir_all(&demo_db_dir).await?;

        let demo_db_path = demo_db_dir.join("database.bak");

        tokio::fs::copy(&db_path, &demo_db_path).await?;
        Ok(())
    }

    async fn create_dockerfile(
        &self,
        build_folder: &Path,
        artifact: &BcArtifact,
        manifest: &Manifest,
    ) -> Result<(), ImageError> {
        let dockerfile = tokio::fs::File::create(build_folder.join("dockerfile")).await?;

        let base_image = "mcr.microsoft.com/businesscentral:ltsc2025-dev";
        let datetime = Local::now().format("%Y%m%d%H%M").to_string();
        let is_bc_sandbox = if manifest.is_bc_sandbox { "Y" } else { "N" };

        self.populate_dockerfile(
            dockerfile,
            artifact,
            manifest,
            base_image,
            datetime,
            is_bc_sandbox,
        )
        .await?;
        Ok(())
    }

    async fn populate_dockerfile(
        &self,
        mut dockerfile: File,
        artifact: &BcArtifact,
        manifest: &Manifest,
        base_image: &str,
        datetime: String,
        is_bc_sandbox: &str,
    ) -> Result<(), ImageError> {
        dockerfile
            .write_all(format!("FROM {}\n", base_image).as_bytes())
            .await?;
        dockerfile.write_all(format!("ENV DatabaseServer=localhost DatabaseInstance=SQLEXPRESS DatabaseName=CRONUS IsBcSandbox={} artifactUrl={} filesOnly={}\n", is_bc_sandbox, artifact.url(), false).as_bytes()).await?;
        dockerfile.write_all(b"COPY NAVDVD /NAVDVD/\n").await?;
        dockerfile
            .write_all(b"RUN \\Run\\start.ps1 -installOnly\n")
            .await?;
        dockerfile
            .write_all(b"LABEL legal=\"http://go.microsoft.com/fwlink/?LinkId=837447\" \\")
            .await?;
        dockerfile
            .write_all(format!("      created=\"{}\" \\\n", datetime).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      nav=\"{}\" \\\n", manifest.nav).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      cu=\"{}\" \\\n", manifest.cu).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      country=\"{}\" \\\n", manifest.country).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      version=\"{}\" \\\n", manifest.version).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      platform=\"{}\"", manifest.platform).as_bytes())
            .await?;
        dockerfile.flush().await?;
        Ok(())
    }

    async fn compress(&self, build_folder: &Path) -> Result<Vec<u8>, ImageError> {
        let build_folder = build_folder.to_path_buf();

        let tar_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, std::io::Error> {
            let mut archive = Builder::new(Vec::new());
            archive.append_dir_all("", build_folder)?;
            archive.finish()?;

            let tar_data = archive.into_inner()?;
            Ok(tar_data)
        })
        .await??;

        Ok(tar_data)
    }

    async fn execute_docker_build(
        &self,
        tar_data: Vec<u8>,
        image_name: &str,
    ) -> Result<String, ImageError> {
        let options = BuildImageOptionsBuilder::default()
            .dockerfile("dockerfile")
            .t(image_name)
            .rm(true)
            .build();

        let mut stream =
            self.docker
                .build_image(options, None, Some(body_full(Bytes::from(tar_data))));

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    println!("{info:?}");
                }
                Err(err) => return Err(ImageError::ImageBuildFailed(err)),
            }
        }

        let image = self
            .docker
            .inspect_image(image_name)
            .await
            .map_err(ImageError::ImageBuildFailed)?;

        image
            .id
            .ok_or_else(|| ImageError::MissingImageId(image_name.to_string()))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Manifest {
    version: String, // IDEA use version struct from artifact
    #[serde(rename = "platformUrl")]
    platform_url: String,
    #[serde(rename = "licenseFile")]
    license_file: String,
    #[serde(rename = "isBcSandbox")]
    is_bc_sandbox: bool,
    nav: String,
    cu: String,
    country: String,
    platform: String,
    database: String,
}

impl Manifest {
    pub fn from_file<P>(path: P) -> Result<Manifest, ImageError>
    where
        P: AsRef<Path>,
    {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn platform_url(&self) -> &str {
        &self.platform_url
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid BC version: {0}")]
    Version(#[from] BcVersionError),

    #[error("invalid artifact manifest: {0}")]
    Manifest(#[from] serde_json::Error),

    #[error("background task failed: {0}")]
    Task(#[from] tokio::task::JoinError),

    #[error("building docker image failed: {0}")]
    ImageBuildFailed(bollard::errors::Error),

    #[error("docker image {0} does not have an image ID")]
    MissingImageId(String),
}

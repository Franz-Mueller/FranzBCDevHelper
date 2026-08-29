use crate::bc::version::BcVersion;
use crate::bc_container::{BcArtifact, BcImage, Manifest};
use crate::utils::file_handling::{compress, copy_dir_all};
use anyhow::{bail, Context, Result};
use bollard::{body_full, query_parameters::BuildImageOptionsBuilder, Docker};
use bytes::Bytes;
use chrono::Local;
use futures_util::StreamExt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::fs;
use tokio::io::AsyncWriteExt;

// TODO Testing
// TODO Stream Docker build context instead of loading the complete tar archive into memory
// TODO Move recursive directory copying off the async runtime
// TODO Handle Docker connection errors instead of unwrapping in ImageBuilder::new

static BUILD_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct ImageBuilder {
    docker: Docker,
    build_path: PathBuf,
}

impl ImageBuilder {
    pub fn new(build_path: PathBuf) -> Self {
        let docker = Docker::connect_with_local_defaults().unwrap();
        Self { docker, build_path }
    }

    pub async fn build(&self, artifact: &BcArtifact) -> Result<BcImage> {
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

        fs::create_dir_all(&build_folder).await.with_context(|| {
            format!(
                "Failed to create dir for build folder at: {}",
                &build_folder.display()
            )
        })?;

        let result = async {
            self.populate_build_folder(&build_folder, artifact)
                .await
                .context("Failed to populate build folder")?;
            let tar_data = compress(&build_folder)
                .await
                .context("Failed to compress build folder")?;
            self.execute_docker_build(tar_data, &image_name).await
        }
        .await;

        fs::remove_dir_all(&build_folder).await.with_context(|| {
            format!("Failed to remove build folder: {}", &build_folder.display())
        })?;

        match result {
            Ok(image_id) => Ok(BcImage::new(image_id)),
            Err(e) => bail!(format!("Docker image build failed due to: {}", e)),
        }
    }

    async fn populate_build_folder(
        &self,
        build_folder: &Path,
        artifact: &BcArtifact,
    ) -> Result<()> {
        let navdvd = build_folder.join("NAVDVD");
        fs::create_dir(&navdvd)
            .await
            .with_context(|| format!("Failed to create dir:  {}", &navdvd.display()))?;

        tokio::try_join!(
            self.populate_navdvd(&navdvd, artifact),
            self.create_dockerfile(build_folder, artifact)
        )
        .with_context(|| {
            format!(
                "Failed at least one concurrent task while pupulating the build folder: {}",
                build_folder.display()
            )
        })?;
        Ok(())
    }

    async fn populate_navdvd(&self, navdvd_folder: &Path, artifact: &BcArtifact) -> Result<()> {
        tokio::try_join!(
            self.copy_demo_db_into_navdvd(navdvd_folder, artifact.path(), artifact.manifest()),
            self.copy_platform_into_navdvd(navdvd_folder, artifact),
            self.copy_artifact_into_navdvd(navdvd_folder, artifact)
        )
        .context("At least one concurrent task failed while populating navdvd")?;
        Ok(())
    }

    async fn copy_platform_into_navdvd(
        &self,
        navdvd_folder: &Path,
        artifact: &BcArtifact,
    ) -> Result<()> {
        Ok(
            Box::pin(copy_dir_all(artifact.platform_path(), navdvd_folder))
                .await
                .with_context(|| {
                    format!(
                        "Failed to copy platform files from {} into navdvd at {}",
                        artifact.platform_path().display(),
                        navdvd_folder.display()
                    )
                })?,
        )
    }

    async fn copy_artifact_into_navdvd(
        &self,
        navdvd_folder: &Path,
        artifact: &BcArtifact,
    ) -> Result<()> {
        let mut entries = fs::read_dir(artifact.path()).await.with_context(|| {
            format!(
                "Failed to read contents of artifact dir: {}",
                artifact.path().display()
            )
        })?;
        Ok(while let Some(entry) = entries.next_entry().await? {
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
                    Box::pin(copy_dir_all(entry.path(), &destination))
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to copy all entries from dir {} to {}",
                                entry.path().display(),
                                &destination.display()
                            )
                        })?;
                } else {
                    fs::copy(entry.path(), &destination)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to copy file {} to {}",
                                entry.path().display(),
                                &destination.display()
                            )
                        })?;
                }
            }
        })
    }

    async fn copy_demo_db_into_navdvd(
        &self,
        navdvd_folder: &Path,
        artifact_path: &Path,
        manifest: &Manifest,
    ) -> Result<()> {
        let db_path = artifact_path.join(manifest.database().replace("\\", "/"));
        let commondata = if BcVersion::from_str(&manifest.version())
            .context("Failed to parse version from manifest")?
            < BcVersion::from_str("27.0.33344.0")
                .context("Failed to parse hardcoded version from commapp comparison")?
        {
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

        fs::create_dir_all(&demo_db_dir).await.with_context(|| {
            format!(
                "Failed to create demo db dir in image build folder at {}",
                &demo_db_dir.display()
            )
        })?;

        let demo_db_path = demo_db_dir.join("database.bak");

        fs::copy(&db_path, &demo_db_path).await.with_context(|| {
            format!(
                "Failed to copy demo database from {} to {}",
                db_path.display(),
                demo_db_path.display()
            )
        })?;
        Ok(())
    }

    async fn create_dockerfile(&self, build_folder: &Path, artifact: &BcArtifact) -> Result<()> {
        let dockerfile_path = build_folder.join("dockerfile");

        async {
            let dockerfile = fs::File::create(&dockerfile_path).await?;

            let base_image = "mcr.microsoft.com/businesscentral:ltsc2025-dev";
            let datetime = Local::now().format("%Y%m%d%H%M").to_string();
            let is_bc_sandbox = if artifact.manifest().is_bc_sandbox() {
                "Y"
            } else {
                "N"
            };

            self.populate_dockerfile(dockerfile, artifact, base_image, datetime, is_bc_sandbox)
                .await
                .context("Failed to populate dockerfile")?;

            anyhow::Ok(())
        }
        .await
        .with_context(|| format!("Dockerfile: {}", dockerfile_path.display()))
    }

    async fn populate_dockerfile(
        &self,
        mut dockerfile: fs::File,
        artifact: &BcArtifact,
        base_image: &str,
        datetime: String,
        is_bc_sandbox: &str,
    ) -> Result<()> {
        dockerfile
            .write_all(format!("FROM {}\n", base_image).as_bytes())
            .await?;
        dockerfile.write_all(format!("ENV DatabaseServer=localhost DatabaseInstance=SQLEXPRESS DatabaseName=CRONUS IsBcSandbox={} artifactUrl={} filesOnly={}\n", is_bc_sandbox, artifact.url(), false).as_bytes()).await?;
        dockerfile.write_all(b"COPY NAVDVD /NAVDVD/\n").await?;
        dockerfile
            .write_all(b"RUN \\Run\\start.ps1 -installOnly\n")
            .await?;
        dockerfile
            .write_all(b"LABEL legal=\"http://go.microsoft.com/fwlink/?LinkId=837447\" \\\n")
            .await?;
        dockerfile
            .write_all(format!("      created=\"{}\" \\\n", datetime).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      nav=\"{}\" \\\n", artifact.manifest().nav()).as_bytes())
            .await?;
        dockerfile
            .write_all(format!("      cu=\"{}\" \\\n", artifact.manifest().cu()).as_bytes())
            .await?;
        dockerfile
            .write_all(
                format!("      country=\"{}\" \\\n", artifact.manifest().country()).as_bytes(),
            )
            .await?;
        dockerfile
            .write_all(
                format!("      version=\"{}\" \\\n", artifact.manifest().version()).as_bytes(),
            )
            .await?;
        dockerfile
            .write_all(format!("      platform=\"{}\"", artifact.manifest().platform()).as_bytes())
            .await?;
        dockerfile.flush().await?;
        Ok(())
    }

    async fn execute_docker_build(&self, tar_data: Vec<u8>, image_name: &str) -> Result<String> {
        let options = BuildImageOptionsBuilder::default() // TODO add memory parameter when fixed by bollard
            .dockerfile("dockerfile")
            .t(image_name)
            .rm(true)
            .build();

        let mut stream =
            self.docker
                .build_image(options, None, Some(body_full(Bytes::from(tar_data))));

        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => (),
                Err(err) => bail!("Error in image build stream: {err}"),
            }
        }

        let image = self
            .docker
            .inspect_image(image_name)
            .await
            .context("Inspect failed on newly built image")?;

        image.id.context("Image id missing")
    }
}

use crate::bc_container::{ArtifactResolver, ContainerBuilder, DockerActions, ImageBuilder};
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;

// TODO Async
// TODO Error
// TODO Tests
// TODO from pub to getters

// reference for further expansion of Application state: https://rustz2h.com/chapter_15_rust_for_wasm_and_cross_platform_apps/series_03_desktop_apps_with_tauri/tauri_state
pub struct AppState {
    pub application_base_paths: ApplicationBasePaths,
    pub artifact_resolver: ArtifactResolver,
    pub image_builder: ImageBuilder,
    pub container_builder: ContainerBuilder,
    pub docker_actions: DockerActions,
    pub docker: Docker,
    // TODO save docker bollard client
}

impl Default for AppState {
    fn default() -> Self {
        let application_base_paths = ApplicationBasePaths::new();
        let artifact_cache_path = application_base_paths.artifacts_cache.clone();
        let image_build_path = application_base_paths.image_build.clone();

        let docker = Docker::connect_with_defaults().unwrap();

        Self {
            application_base_paths: application_base_paths,
            artifact_resolver: ArtifactResolver::new(artifact_cache_path),
            image_builder: ImageBuilder::new(image_build_path),
            container_builder: ContainerBuilder::new(docker.clone()), // TODO is this a good idea?
            docker_actions: DockerActions::new(docker.clone()),
            docker,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationBasePaths {
    base_path: PathBuf,
    artifacts_cache: PathBuf,
    projects: PathBuf,
    dependencies_cache: PathBuf,
    image_build: PathBuf,
    repos_path: PathBuf,
}

impl ApplicationBasePaths {
    fn new() -> ApplicationBasePaths {
        let base_path: PathBuf = get_base_path();
        if !base_path.try_exists().unwrap() {
            create_dir_all(&base_path).unwrap();
        }
        let artifacts_cache: PathBuf = base_path.join("artifacts_cache");
        if !artifacts_cache.try_exists().unwrap() {
            create_dir_all(&artifacts_cache).unwrap();
        }
        let projects: PathBuf = base_path.join("projects");
        if !projects.try_exists().unwrap() {
            create_dir_all(&projects).unwrap();
        }
        let dependencies_cache: PathBuf = base_path.join("dependencies_cache");
        if !dependencies_cache.try_exists().unwrap() {
            create_dir_all(&dependencies_cache).unwrap();
        }
        let image_build: PathBuf = base_path.join("image_build");
        if !image_build.try_exists().unwrap() {
            create_dir_all(&image_build).unwrap();
        }
        let repos_path: PathBuf = base_path.join("repos");
        if !repos_path.try_exists().unwrap() {
            create_dir_all(&repos_path).unwrap();
        }
        ApplicationBasePaths {
            base_path: base_path,
            artifacts_cache: artifacts_cache,
            projects: projects,
            dependencies_cache: dependencies_cache,
            image_build: image_build,
            repos_path: repos_path,
        }
    }
}

fn get_base_path() -> PathBuf {
    // TODO enable user to configure custom data store location
    if cfg!(target_os = "windows") {
        let local_app_data =
            env::var("LOCALAPPDATA").expect("LOCALAPPDATA environment variable not set");

        PathBuf::from(local_app_data).join("FranzBCDevHelper")
    } else if cfg!(target_os = "linux") {
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home).join("franzbcdevhelper")
        } else {
            let home = env::var("HOME").expect("HOME environment variable not set");

            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("franzbcdevhelper")
        }
    } else {
        panic!("Unsupported operating system");
    }
}

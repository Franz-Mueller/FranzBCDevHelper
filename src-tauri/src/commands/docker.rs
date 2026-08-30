use crate::bc::version::BcVersion;
use crate::bc_container::ArtifactRequest;
use crate::bc_container::BcContainer;
use crate::AppState;
use bollard::Docker;
use tauri::State;

use std::str::FromStr;

#[tauri::command]
pub async fn delete_docker_container(id: String, name: String) {
    let docker = Docker::connect_with_defaults().unwrap();
    let container = BcContainer::new(id, name);
    container.delete(&docker).await.unwrap();
}

struct ContainerFromList {
    name: String,
    id: String,
}

#[tauri::command]
pub async fn get_containers() -> Vec<ContainerFromList> {
    get_containers_inner().await.unwrap()
}

async fn get_containers_inner() -> Result<Vec<ContainerFromList>, String> {
    let docker = Docker::connect_with_defaults().unwrap();
    let container_sum = docker.list_containers(None).await.unwrap();
    let mut containers: Vec<ContainerFromList> = Vec::new();
    for cont in container_sum {
        let name = match cont.names {
            Some(n) => n[0].clone(),
            None => "NA".to_string(),
        };
        let id = match cont.id {
            Some(id) => id,
            None => "NA".to_string(),
        };
        containers.push(ContainerFromList { name, id });
    }
    Ok(containers)
}

#[tauri::command]
pub async fn create_docker_container(
    state: State<'_, AppState>,
    deployment_type: String,
    version: String,
    country: String,
    container_name: String,
) -> Result<(), String> {
    create_docker_container_inner(&state, deployment_type, version, country, container_name).await
}

async fn create_docker_container_inner(
    state: &AppState,
    deployment_type: String,
    version: String,
    country: String,
    container_name: String,
) -> Result<(), String> {
    let version = BcVersion::from_str(&version).map_err(|err| err.to_string())?;

    let artifact = state
        .artifact_resolver
        .resolve(ArtifactRequest {
            deployment_type,
            version,
            country,
        })
        .await
        .map_err(|err| err.to_string())?;

    let image = state
        .image_builder
        .build(&artifact)
        .await
        .map_err(|err| err.to_string())?;

    state
        .container_builder
        .build(&image, &container_name)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn create_docker_container_e2e() {
        let state = AppState::default();

        create_docker_container_inner(
            &state,
            "sandbox".into(),
            "16.0.0.0".into(),
            "de".into(),
            "bc-e2e-test".into(),
        )
        .await
        .unwrap();
    }
}

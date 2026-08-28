use crate::bc::version::BcVersion;
use crate::bc_container::ArtifactRequest;
use crate::AppState;
use tauri::State;

use std::str::FromStr;

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
            "15.0.0.0".into(),
            "de".into(),
            "bc-e2e-test".into(),
        )
        .await
        .unwrap();
    }
}

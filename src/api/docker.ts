import { invoke } from "@tauri-apps/api/core";

export interface CreateContainerRequest {
    deploymentType: string;
    version: string;
    country: string;
    containerName: string;
}

export async function createContainer(
    request: CreateContainerRequest,
): Promise<void> {
    await invoke("create_docker_container", {
        deploymentType: request.deploymentType,
        version: request.version,
        country: request.country,
        containerName: request.containerName,
    });
}
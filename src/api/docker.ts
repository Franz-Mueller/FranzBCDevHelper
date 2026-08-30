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

export async function deleteContainer(id: string, name: string): Promise<void> {
    await invoke("delete_docker_container", {
        id,
        name
    })
}

export interface Container {
    name: string
    id: string
}

export async function getContainers(): Promise<[Container]> {
    return await invoke("get_containers")
}
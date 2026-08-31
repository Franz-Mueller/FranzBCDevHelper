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

export interface Container {
    name: string
    id: string
    version: string,
    status: string,
}

export async function getContainers(): Promise<Container[]> {
    return invoke<Container[]>("get_containers");
}

export async function startContainer(name: string): Promise<void> {
    await invoke("start_docker_container", {
        name
    })
}

export async function stopContainer(name: string): Promise<void> {
    await invoke("stop_docker_container", {
        name
    })
}

export async function deleteContainer(name: string): Promise<void> {
    await invoke("delete_docker_container", {
        name
    })
}

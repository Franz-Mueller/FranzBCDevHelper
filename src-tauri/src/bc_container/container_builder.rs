use crate::bc_container::{BcContainer, BcImage};
use bollard::config::ContainerCreateBody;
use bollard::plugin::ContainerCreateResponse;
use bollard::query_parameters::{CreateContainerOptionsBuilder, ListImagesOptionsBuilder};

pub struct ContainerBuilder {
    docker: bollard::Docker,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        ContainerBuilder {
            docker: bollard::Docker::connect_with_defaults().unwrap(),
        }
    }

    pub async fn build(
        &self,
        image: &BcImage,
        container_name: &str,
    ) -> Result<BcContainer, ContainerError> {
        let options = ListImagesOptionsBuilder::default().all(true).build();
        let images = self.docker.list_images(Some(options)).await?;
        let image_ids: Vec<String> = images.iter().map(|i| i.id.clone()).collect(); // TODO redo
        if !image_ids.contains(&image.id().to_string()) {
            return Err(ContainerError::ImageNotFound(image.id().to_string()));
        }

        let create_response = self.create_container(image, container_name).await?;

        let container = BcContainer::new(create_response.id, container_name.to_string());

        container.start(&self.docker).await?;

        Ok(container)
    }

    pub async fn create_container(
        &self,
        image: &BcImage,
        container_name: &str,
    ) -> Result<ContainerCreateResponse, ContainerError> {
        let options = CreateContainerOptionsBuilder::default()
            .name(container_name)
            .build();
        let config = ContainerCreateBody {
            // TODO detached = true
            image: Some(image.id().to_string()),
            env: Some(Vec::from(["accept_eula=Y".to_string()])),
            ..Default::default()
        };

        let create_response = self.docker.create_container(Some(options), config).await?;

        Ok(create_response)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("docker causes an error: {0}")]
    BollardOperation(#[from] bollard::errors::Error),

    #[error("could not find image {0} in docker")]
    ImageNotFound(String),
}

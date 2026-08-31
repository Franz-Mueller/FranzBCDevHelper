use anyhow::{Context, Result};
use bollard::query_parameters::RemoveContainerOptionsBuilder;
pub struct BcContainer {
    // TODO Container should inherit Connection to docker but should be aware of changes made to it
    // would save some value passing
    id: String,
    name: String,
}

impl BcContainer {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    // pub async fn from_inspect() -> Result<Self, ContainerError> {}

    pub async fn start(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .start_container(&self.name, None)
            .await
            .with_context(|| format!("Failed to start container {}", &self.name))?;
        Ok(())
    }
    pub async fn stop(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .stop_container(&self.name, None)
            .await
            .with_context(|| format!("Failed to stop container {}", &self.name))?;
        Ok(())
    }
    pub async fn delete(&self, docker: &bollard::Docker) -> Result<()> {
        docker.remove_container(&self.name, None).await?;
        Ok(())
    }
}

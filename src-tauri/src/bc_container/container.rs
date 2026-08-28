use crate::bc_container::container_builder::ContainerError;

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

    pub async fn start(&self, docker: &bollard::Docker) -> Result<(), ContainerError> {
        docker.start_container(&self.name, None).await?;
        Ok(())
    }
    pub async fn stop(&self, docker: &bollard::Docker) -> Result<(), ContainerError> {
        docker.stop_container(&self.name, None).await?;
        Ok(())
    }
}

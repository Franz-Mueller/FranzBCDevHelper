use anyhow::{Context, Result};

pub struct DockerActions {
    docker: bollard::Docker,
}

impl DockerActions {
    pub fn new(docker: bollard::Docker) -> Self {
        DockerActions { docker }
    }
    pub async fn start(&self, name: &str) -> Result<()> {
        self.docker
            .start_container(name, None)
            .await
            .with_context(|| format!("Failed to start container {}", name))?;
        Ok(())
    }
    pub async fn stop(&self, name: &str) -> Result<()> {
        self.docker
            .stop_container(name, None)
            .await
            .with_context(|| format!("Failed to stop container {}", name))?;
        Ok(())
    }
    pub async fn delete(&self, name: &str) -> Result<()> {
        self.docker.remove_container(name, None).await?;
        Ok(())
    }
}

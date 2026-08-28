use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

#[derive(Deserialize, Serialize, Debug)]
pub struct Manifest {
    version: String, // IDEA use version struct from artifact
    #[serde(rename = "platformUrl")]
    platform_url: String,
    #[serde(rename = "licenseFile")]
    license_file: String,
    #[serde(rename = "isBcSandbox")]
    is_bc_sandbox: bool,
    nav: String,
    cu: String,
    country: String,
    platform: String,
    database: String,
}

impl Manifest {
    pub async fn from_file<P>(path: P) -> Result<Manifest, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let mut f = fs::File::open(path).await?;
        let mut buffer = String::new();

        f.read_to_string(&mut buffer).await?;

        Ok(serde_json::from_str(&buffer)?)
    }

    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn platform_url(&self) -> &str {
        &self.platform_url
    }
    pub fn license_file(&self) -> &str {
        &self.license_file
    }
    pub fn is_bc_sandbox(&self) -> bool {
        self.is_bc_sandbox
    }
    pub fn nav(&self) -> &str {
        &self.nav
    }
    pub fn cu(&self) -> &str {
        &self.cu
    }
    pub fn country(&self) -> &str {
        &self.country
    }
    pub fn platform(&self) -> &str {
        &self.platform
    }
    pub fn database(&self) -> &str {
        &self.database
    }
}

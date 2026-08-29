use anyhow::{bail, Context, Result};
use encoding_rs::UTF_8;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct Manifest {
    version: Option<String>, // IDEA use version struct from artifact
    #[serde(rename = "platformUrl")]
    platform_url: Option<String>,
    #[serde(rename = "licenseFile")]
    license_file: Option<String>,
    #[serde(rename = "isBcSandbox")]
    is_bc_sandbox: Option<bool>, // TODO maybe do not acces empty fields in manifest if they are mandatory
    nav: Option<String>,
    cu: Option<String>,
    country: Option<String>,
    platform: Option<String>,
    database: Option<String>,
}

impl Manifest {
    /// decodes file to utf-8 since ms sometimes provides utf-16 le
    pub async fn from_file<P>(path: P) -> Result<Manifest>
    where
        P: AsRef<Path>,
    {
        // TODO async
        let path = path.as_ref();

        async {
            let file = fs::read(path)
                .await
                .with_context(|| format!("Failed to read manifest.json"))?;

            let (cow, _, had_errors) = UTF_8.decode(&file);

            if had_errors {
                bail!("Failed to decode manifest.json as UTF-8");
            }

            Ok(serde_json::from_str(&cow[..])
                .with_context(|| format!("Failed to deserialized manifest.json"))?)
        }
        .await
        .with_context(|| format!("Manifest file: {}", path.display()))
    }

    pub fn version(&self) -> &str {
        match &self.version {
            Some(v) => v,
            None => "",
        }
    }
    pub fn platform_url(&self) -> &str {
        match &self.platform_url {
            Some(v) => v,
            None => "",
        }
    }
    pub fn license_file(&self) -> &str {
        match &self.license_file {
            Some(v) => v,
            None => "",
        }
    }
    pub fn is_bc_sandbox(&self) -> bool {
        match self.is_bc_sandbox {
            Some(v) => v,
            None => false,
        }
    }
    pub fn nav(&self) -> &str {
        match &self.nav {
            Some(v) => v,
            None => "",
        }
    }
    pub fn cu(&self) -> &str {
        match &self.cu {
            Some(v) => v,
            None => "",
        }
    }
    pub fn country(&self) -> &str {
        match &self.country {
            Some(v) => v,
            None => "",
        }
    }
    pub fn platform(&self) -> &str {
        match &self.platform {
            Some(v) => v,
            None => "",
        }
    }
    pub fn database(&self) -> &str {
        match &self.database {
            Some(v) => v,
            None => "",
        }
    }
}

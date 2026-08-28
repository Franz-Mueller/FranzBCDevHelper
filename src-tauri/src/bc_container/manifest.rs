use serde::{Deserialize, Serialize};
use std::path::Path;
use encoding_rs::UTF_8;

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
    pub async fn from_file<P>(path: P) -> Result<Manifest, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let file = std::fs::read(path)?;
        let (cow, _, _) = UTF_8.decode(&file);

        Ok(serde_json::from_str(&cow[..])?)
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

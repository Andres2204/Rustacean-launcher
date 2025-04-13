use std::io;
use serde::{Deserialize, Serialize};
use crate::downloader::download_structs::VersionType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    latest: LatestVersion,
    versions: Vec<VersionInfo>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LatestVersion {
    release: String,
    snapshot: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
    pub id: String,
    pub url: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
}

impl Clone for VersionInfo {
    fn clone(&self) -> Self {
        VersionInfo {
            id: self.id.clone(),
            url: self.url.clone(),
            version_type: self.version_type.clone(),
        }
    }
}
impl Manifest {
    pub async fn get_version_manifest(versions_manifest_link: &str) -> io::Result<Manifest> {
        let response = reqwest::get(versions_manifest_link)
            .await;
        match response {
            Ok(res) => {
                Ok(res.json::<Manifest>()
                    .await
                    .expect("failed to parse versions"))
            }
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }

    pub fn get_version_by_id(&self, version: &str) -> io::Result<VersionInfo> {
        for version_info in &self.versions {
            if version_info.id == version {
                return Ok(version_info.clone());
            }
        }

        Err(io::Error::new(io::ErrorKind::NotFound, format!("Version {} not found", version)))
    }
    
    pub fn get_all_version_ref(&self) -> &Vec<VersionInfo> {
        &self.versions
    }
}
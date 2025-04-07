use std::io;
use crate::core::downloader::download_structs::VersionType::*;
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::manifest::Manifest;
use crate::core::versions::version::{StandardVersion, Version, VersionVerifier};

pub struct VersionManager;

impl VersionManager {
    pub async fn fetch_versions() -> io::Result<Vec<Box<dyn Version>>> {
        let manifest = Manifest::get_version_manifest(
            &*LauncherConfig::import_config().version_manifest_link
        ).await.expect("Could not load manifest file");
        
        let versions_info = manifest.get_all_version_ref()
        let versions: Vec<Box<dyn Version>> = versions_info.into_iter()
            .map(
                |version| {
                    match version.version_type {
                        RELEASE | 
                        SNAPSHOT | 
                        OldBeta |
                        OldAlpha => {
                            let mut v: Box<dyn Version> = Box::new(StandardVersion::from(version));
                            VersionVerifier::is_installed(&mut v);
                            v
                        }
                    }
                }).collect();
        Ok(versions)
    }
    
    
    
}
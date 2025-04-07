use crate::core::downloader::download_structs::VersionType;
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::manifest::{Manifest};
use crate::core::versions::version::{StandardVersion, Version, VersionDownloader, VersionVerifier};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct VersionManager {}

impl VersionManager {
    pub async fn fetch_versions() -> io::Result<Vec<Box<dyn Version>>> {
        let manifest =
            Manifest::get_version_manifest(&*LauncherConfig::import_config().version_manifest_link)
                .await?;
        let versions_info = manifest.get_all_version_ref();
        let versions: Vec<Box<dyn Version>> = versions_info
            .iter()
            .map(|v| match v.version_type {
                VersionType::RELEASE
                | VersionType::SNAPSHOT
                | VersionType::OldBeta
                | VersionType::OldAlpha => {
                    let mut sv: Box<dyn Version> = Box::new(StandardVersion::from(v));
                    VersionVerifier::is_installed(&mut sv);
                    sv
                }
            })
            .collect();
        Ok(versions)
    }

    pub fn is_installed(mut version: Box<(dyn Version + 'static)>) -> bool {
        VersionVerifier::is_installed(&mut version)
    }

    pub async fn download_version(mut version: Box<(dyn Version + 'static)>) -> io::Result<()> {
        //if VersionVerifier::is_installed(&mut version) {
            // TODO: verify version.jsn sha256 or download, verify installation
        //    return Ok(());
        //}
        
        let progress = Arc::new(Mutex::new((0,0)));
        let progress_clone = progress.clone();
        VersionDownloader::download_version(version, progress_clone).await.expect("Can't Download the version");
        
        Ok(())
    }

    pub fn verify_version_installation(version: Box<(dyn Version + 'static)>) -> bool {
        VersionVerifier::verify_installation(&version)
    }
}

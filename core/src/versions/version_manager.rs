use crate::downloader::download_structs::VersionType;
use crate::downloader::downloader::DownloaderTracking;
use crate::launcher::launcher_config::{LauncherConfig};
use crate::versions::downloader::VersionDownloader;
use crate::versions::manifest::Manifest;
use crate::versions::verifier::VersionVerifier;
use crate::versions::version::{StandardVersion, Version};
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use log;
use tokio::sync::Mutex;

pub struct VersionManager;

impl VersionManager {
    pub async fn fetch_versions() -> io::Result<Vec<Box<dyn Version>>> {
        let versions = match Self::versions_by_manifest().await {
            Ok(versions) => versions,
            Err(e) => {
                log::error!("{e}");
                Self::versions_local().await?
            }
        };
        Ok(versions)
    }

    async fn versions_by_manifest() -> io::Result<Vec<Box<dyn Version>>> {
        let config = LauncherConfig::import_config();
        let manifest = Manifest::get_version_manifest(&config.version_manifest_link).await?;

        let settings = match config.settings() {
            Some(s) => s,
            None => return Ok(vec![]), // o maneja el error de forma apropiada
        };

        let versions: Vec<Box<dyn Version>> = manifest
            .get_all_version_ref()
            .iter()
            .filter(|v| match v.version_type {
                VersionType::RELEASE => true,
                VersionType::SNAPSHOT => settings.allowSnapshot,
                VersionType::OldBeta => settings.allowBeta,
                VersionType::OldAlpha => settings.allowAlpha,
            })
            .map(|v| { // TODO: manage the versions types
                let mut version: Box<dyn Version> = Box::new(StandardVersion::from(v));
                VersionVerifier::is_installed(&mut version);
                version
            })
            .collect();

        Ok(versions)
    }

    async fn versions_local() -> io::Result<Vec<Box<dyn Version>>> {
        let mut versions: Vec<Box<dyn Version>> = vec![];
        let versions_list = fs::read_dir(
            Path::new(&LauncherConfig::import_config().minecraft_path)
                .join("versions")
                .as_path(),
        )
        .expect("Cant read versions directory");

        for path in versions_list {
            // TODO: multithread
            versions.push(VersionVerifier::from_local(
                path.unwrap().file_name().into_string().unwrap(),
            )?)
        }
        Ok(versions)
    }

    pub fn is_installed(mut version: Box<(dyn Version + 'static)>) -> bool {
        VersionVerifier::is_installed(&mut version)
    }

    pub async fn download_version(
        version: Box<(dyn Version + 'static)>,
        progress: Arc<Mutex<DownloaderTracking>>,
    ) -> io::Result<()> 
    {
        //if VersionVerifier::is_installed(&mut version) {
        // TODO: verify version.jsn sha256 or download, verify installation
        //    return Ok(())
        //}
        let progress_clone = progress.clone();
        VersionDownloader::download_version(version, progress_clone)
            .await
            .expect("Can't Download the version");
        Ok(())
    }

    pub fn verify_version_installation(mut version: Box<(dyn Version + 'static)>) -> bool {
        VersionVerifier::verify_installation(&mut version)
    }

    pub fn delete_version(mut version: Box<(dyn Version + 'static)>) -> Option<Box<dyn Version>> {
        todo!("delete_version() not implemented yet")
    }
}

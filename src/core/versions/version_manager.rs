use crate::core::downloader::download_structs::VersionType;
use crate::core::downloader::downloader::DownloaderTracking;
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::downloader::VersionDownloader;
use crate::core::versions::manifest::Manifest;
use crate::core::versions::verifier::VersionVerifier;
use crate::core::versions::version::{StandardVersion, Version};
use std::{io, fs};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct VersionManager {}

impl VersionManager {
    pub async fn fetch_versions() -> io::Result<Vec<Box<dyn Version>>> {
        let versions = match Self::versions_by_manifest().await {
            Ok(versions) => versions,
            Err(e) => {
                eprintln!("{e}");
                Self::versions_local().await?
            }
        };
        Ok(versions)
    }

    async fn versions_by_manifest() -> io::Result<Vec<Box<dyn Version>>> {
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

    async fn versions_local() -> io::Result<Vec<Box<dyn Version>>> {
        let mut versions: Vec<Box<dyn Version>> = vec![];
        let versions_list = fs::read_dir(
            Path::new(&LauncherConfig::import_config().minecraft_path)
                .join("versions")
                .as_path(),
        )
        .expect("Cant read versions directory");

        for path in versions_list { // TODO: multithread
            versions.push(VersionVerifier::from_local(
                path.unwrap()
                    .file_name()
                    .into_string()
                    .unwrap())?)
        }
        Ok(versions)
    }

    pub fn is_installed(mut version: Box<(dyn Version + 'static)>) -> bool {
        VersionVerifier::is_installed(&mut version)
    }

    pub async fn download_version(
        mut version: Box<(dyn Version + 'static)>,
        progress: Arc<Mutex<DownloaderTracking>>,
    ) -> io::Result<()> {
        //if VersionVerifier::is_installed(&mut version) {
        // TODO: verify version.jsn sha256 or download, verify installation
        //    return Ok(());
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
}

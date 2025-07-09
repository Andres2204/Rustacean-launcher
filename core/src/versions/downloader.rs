use std::{io};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::fs;
use tokio::sync::Mutex;
use crate::downloader::download_structs::{VersionJson, VersionType};
use crate::downloader::downloader::{download_file, DownloaderTracking, FileData};
use crate::launcher::launcher_config::LauncherConfig;
use crate::versions::assets::AssetDownloader;
use crate::versions::libraries::LibraryDownloader;
use crate::versions::version::{Version, VersionState};

pub struct VersionDownloader;
impl VersionDownloader {
    pub async fn download_version(
        version: Box<dyn Version + 'static>,
        progress: Arc<Mutex<DownloaderTracking>>,
    ) -> io::Result<()> {
        log::info!("Matching version type: {:?}", version.version_type());
        match version.version_type() {
            VersionType::RELEASE
            | VersionType::SNAPSHOT
            | VersionType::OldBeta
            | VersionType::OldAlpha => Self::download_standard(version, progress).await,
        }
    }

    async fn download_standard(mut version: Box<dyn Version + 'static>, progress: Arc<Mutex<DownloaderTracking>>) -> io::Result<()> {
        
        log::info!("Download_standard version: {:?}", version);
        
        // Initialize variables
        version.set_state(VersionState::DOWNLOADING);
        let config = LauncherConfig::import_config();

        // ensure intial files are downloaded
        let vc = version.clone();
        Self::download_initial_files(&vc).await.expect("Unable to asecure initial files");
        
        // version json local
        let minecraft_path = config.minecraft_path.clone();
        let version_json = VersionJson::get_from_local(&minecraft_path, &version.name()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        //  Calculate total of files to download and set value to progress
        let assets_json = version_json.get_assets_json();
        let total_assets = assets_json.clone().objects.len();
        let total_libraries = version_json.get_libraries().len();
        
        // calculate total of files
        let total_objects: usize = total_libraries + total_assets;
        log::info!("assets: {total_assets}, lib: {total_libraries}, t: {total_objects}");
        
        //  Spawn threads
        
        progress.lock().await.set_progress((0usize, total_objects));
        log::info!("Starting progress: {:?}", progress.lock().await);
               
        log::info!("Downloading libraries ............");
        let mut mine_path_clone = minecraft_path.clone();
        let libraries_progress = progress.clone();
        tokio::spawn(async move {
            LibraryDownloader::download_libraries(
                version_json.get_libraries().clone(),
                Path::new(&mine_path_clone),
                Some(libraries_progress),
            )
                .await
                .expect("can't download libraries");
            
        });
        
        
        log::info!("Downloading Assets ---------");
        mine_path_clone = minecraft_path.clone();
        let asset_progress = progress.clone();
        tokio::spawn(async move {
            AssetDownloader::download_assets(
                assets_json,
                Path::new(&mine_path_clone),
                Some(asset_progress),
            )
                .await
                .expect("cant download assets");
        });

        loop {
            let p = progress.lock().await;
            log::info!("ACTUAL PROGRESS: {:?}  UNITS: {:?}", p.progress(), p.units().len());
            if p.finished() {
                break;
            }
        }
        // Wait until finish

        version.set_state(VersionState::INSTALLED(true));
        Ok(())
    }

    async fn download_initial_files(version: &Box<dyn Version + 'static>) -> io::Result<()> {
        let LauncherConfig { minecraft_path, .. } = LauncherConfig::import_config();
        match fs::create_dir_all(minecraft_path.clone()) {
            Ok(e) => log::info!("Directory created {:?}", e),
            _ => {}
        }

        // download version.json
        let version_name = version.name();
        download_file(
            FileData::new(
                format!("{}/versions/{}/{}.json", &minecraft_path, &version_name, &version_name).to_string(),
                version.json_url(),
                None
            ),
            None,
            None
        ).await.expect("Download method failed.");

        // download asset index json
        let version_json = VersionJson::get_from_local(&minecraft_path, &version_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let assets_index = version_json.get_asset_index();
        download_file(
            FileData::new(
                Path::new(&minecraft_path)
                    .join("assets")
                    .join("indexes")
                    .join(format!("{}.json", assets_index.id).as_str())
                    .to_str().unwrap().to_string(),
                assets_index.url,
                None
                
            ),
            None,
            None
        ).await.expect("Cant download assets indexes json");

        // download client
        download_file(
            FileData::new(
                Path::new(&minecraft_path.clone())
                    .join("versions")
                    .join(version_name.as_str())
                    .join(format!("{}.jar", version_name).as_str())
                    .to_str().unwrap().to_string(),
                version_json.get_client_url(),
                None
            ),
            None,
            None
        ).await.expect("Cant download client.jar ");

        Ok(())
    }
}
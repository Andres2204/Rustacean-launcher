use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::fs;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::core::downloader::download_structs::{AssetsJson, VersionJson, VersionType};
use crate::core::downloader::downloader::{download_file, DownloaderTracking};
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::assets::AssetDownloader;
use crate::core::versions::libraries::LibraryDownloader;
use crate::core::versions::version::{Version, VersionState};

pub struct VersionDownloader;
impl VersionDownloader {
    pub async fn download_version(
        mut version: Box<dyn Version + 'static>,
        progress: Arc<Mutex<DownloaderTracking>>,
    ) -> io::Result<()> {
        match version.version_type() {
            VersionType::RELEASE
            | VersionType::SNAPSHOT
            | VersionType::OldBeta
            | VersionType::OldAlpha => Self::download_standard(version, progress).await,
        }
    }

    async fn download_standard(mut version: Box<dyn Version + 'static>, mut progress: Arc<Mutex<DownloaderTracking>>) -> io::Result<()> {
        version.set_state(VersionState::DOWNLOADING);
        let config = LauncherConfig::import_config();
        Self::download_initial_files(&version).await?;

        // version json local
        let minecraft_path = config.minecraft_path.clone();
        let version_json = VersionJson::get_from_local(minecraft_path.clone(), version.name());

        let asset_index = version_json.get_asset();
        let assets = AssetsJson::from_local(
            Path::new(&config.minecraft_path)
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", asset_index.id).as_str())
                .as_path(),
        );
        let total_assets = assets.objects.len();
        let total_libraries = version_json.get_libraries().len();
        let total_objects: usize = total_libraries + total_assets;
        println!("assets: {total_assets}, lib: {total_libraries}, t: {total_objects}");
        sleep(Duration::from_secs(1)).await;

        progress.lock().await.set_progress((0usize, total_objects));
        let library_progress = Arc::new(Mutex::new(DownloaderTracking::new((0, total_libraries))));
        let lib_progress_clone = Arc::clone(&library_progress);

        let mut mine_path_clone = minecraft_path.clone();
        tokio::spawn(async move {
            LibraryDownloader::download_libraries(
                version_json.get_libraries().clone(),
                Path::new(&mine_path_clone),
                Some(lib_progress_clone),
            )
                .await
                .expect("can't download libraries");
            
        });
        
        let asset_progress = Arc::new(Mutex::new(DownloaderTracking::new((0, total_assets))));
        let asset_progress_clone = Arc::clone(&asset_progress);
        mine_path_clone = minecraft_path.clone();
        tokio::spawn(async move {
            AssetDownloader::download_assets(
                assets,
                Path::new(&mine_path_clone),
                Some(asset_progress_clone),
            )
                .await
                .expect("cant download assets");
        });
        
        println!("asset progress: {:?}", asset_progress.lock().await);
        sleep(Duration::from_secs(1)).await;
        
        
        let mut cached_progress = 0usize;
        loop {
            let mut main_progress = progress.lock().await;
            let library_progress_guard = library_progress.lock().await;
            let asset_progress_guard = asset_progress.lock().await;
            
            let temp_progress = library_progress_guard.actual_progress() + asset_progress_guard.actual_progress();
            main_progress.set_actual_progress(temp_progress);// TODO: .add_to_progress(usize) -> progress.0 += 1
            if main_progress.actual_progress() != cached_progress {
                println!(" <> --- {main_progress:?} --- <>");
                cached_progress = main_progress.actual_progress(); // TODO: actual_progress -> return progress.0
            }
            if main_progress.finished() {
                break;
            }
        }

        version.set_state(VersionState::INSTALLED(true));
        Ok(())
    }

    async fn download_initial_files(version: &Box<dyn Version + 'static>) -> io::Result<()> {
        let LauncherConfig { minecraft_path, .. } = LauncherConfig::import_config();
        match fs::create_dir_all(minecraft_path.clone()) {
            Ok(e) => println!("Directory created {:?}", e),
            _ => {}
        }

        // download version.json
        let version_name = version.name();
        download_file(
            &version.json_url(),
            Path::new(&format!("{}/versions/{}/{}.json", &minecraft_path, &version_name, &version_name)),
            None,
            None
        ).await.expect("Download method failed.");

        // download asset index json
        let version_json = VersionJson::get_from_local(minecraft_path.clone(), version_name.clone());
        let assets_index = version_json.get_asset();
        download_file(
            assets_index.url.as_str(),
            Path::new(&minecraft_path)
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", assets_index.id).as_str())
                .as_path(),
            None,
            None
        ).await.expect("Cant download assets indexes json");

        // download client
        download_file(
            version_json.get_client_url().as_str(),
            Path::new(&minecraft_path.clone())
                .join("versions")
                .join(version_name.as_str())
                .join(format!("{}.jar", version_name).as_str())
                .as_path(),
            None,
            None
        ).await.expect("Cant download client.jar ");

        Ok(())
    }
}
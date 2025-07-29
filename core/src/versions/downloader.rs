use std::{io};
use std::path::Path;
use std::sync::Arc;
use std::fs;
use tokio::sync::Mutex;
use crate::versions::version_json::{AssetsJson, Library, VersionJson, VersionType};
use crate::downloader::downloader::{Downloader, DownloaderTracking, DownloadState, FileData};
use crate::launcher::launcher_config::LauncherConfig;
use crate::versions::version::Version;

pub struct VersionDownloader;
impl VersionDownloader {
    
    pub async fn download_version(
        version: Box<dyn Version + 'static>,
        progress: Arc<Mutex<DownloaderTracking>>,
    ) -> io::Result<()> 
    {
        log::info!("Matching version type: {:?}", version.version_type());
        match version.version_type() {
            VersionType::RELEASE
            | VersionType::SNAPSHOT
            | VersionType::OldBeta
            | VersionType::OldAlpha => Self::download_standard(version, progress).await,
        }
    }

    async fn download_standard(version: Box<dyn Version + 'static>, progress: Arc<Mutex<DownloaderTracking>>) -> io::Result<()> {
        // Initialize variables
        let config = LauncherConfig::import_config();
        let downloader = Downloader::builder()
            .concurret_downloads(64)
            .retries(5)
            .progress(progress.clone())
            .build()?;
        log::info!("Download_standard version: {:?}", version);

        // ensure intial files are downloaded
        let vc = version.clone();
        progress.lock().await.set_state(DownloadState::DownloadingInitials);
        Self::download_initial_files(&vc, downloader).await.expect("Unable to asecure initial files");
        
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
        let mut total_files = Self::libraries_files(
            version_json.get_libraries(),
            Path::new(&minecraft_path)
        ).await?;
        let mut assets_files = Self::assets_files(
            assets_json,
            Path::new(&minecraft_path)
        ).await?;
        total_files.append(&mut assets_files);

        let downloader_concurrent = Downloader::builder()
            .concurret_downloads(64)
            .retries(5)
            .progress(progress.clone())
            .build()?;

        // Spawn
        tokio::spawn(async move {
            downloader_concurrent.download_files_concurrently(total_files).await
        });
        

        // Wait until finish
        loop {
            let p = progress.lock().await;
            log::info!("ACTUAL PROGRESS: {:?}  UNITS: {:?}", p.progress(), p.units().len());
            match (p.finished(), p.state()) { 
                (true, _) | (_, DownloadState::Finished) => {break}
                _ => {}
            }
        }
        Ok(())
    }

    async fn download_initial_files(version: &Box<dyn Version + 'static>, mut downloader: Downloader) -> io::Result<()> {
        let LauncherConfig { minecraft_path, .. } = LauncherConfig::import_config();
        match fs::create_dir_all(minecraft_path.clone()) {
            Ok(e) => log::info!("Directory created {:?}", e),
            _ => {}
        }

        let version_name = version.name();
        let version_json = VersionJson::get_from_local(&minecraft_path, &version_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let assets_index = version_json.get_asset_index();
        let files = vec![
            FileData::new( // Version json
                format!("{}/versions/{}/{}.json", &minecraft_path, &version_name, &version_name).to_string(),
                version.json_url(),
                None
            ),
            FileData::new( // Asset json
                Path::new(&minecraft_path)
                    .join("assets")
                    .join("indexes")
                    .join(format!("{}.json", assets_index.id).as_str())
                    .to_str().unwrap().to_string(),
                assets_index.url,
                None
            ),
            FileData::new( // client
                Path::new(&minecraft_path.clone())
                    .join("versions")
                    .join(version_name.as_str())
                    .join(format!("{}.jar", version_name).as_str())
                    .to_str().unwrap().to_string(),
                version_json.get_client_url(),
                None
            ),
            FileData::new(
                Path::new(&minecraft_path.clone())
                    .join("versions")
                    .join(version_name.as_str())
                    .join(format!("{}.txt", version_name).as_str())
                    .to_str().unwrap().to_string(),
                version_json.get_client_mappings_url(),
                None
            )
        ];

        downloader.download_files_concurrently(files).await.expect("Failed to download initial files");
        downloader.clean_progress().await;
        Ok(())
    }

    async fn libraries_files(libraries: Vec<Library>, minecraft_path: &Path) -> io::Result<Vec<FileData>> {
        let mut files: Vec<FileData> = Vec::new();
        libraries.into_iter()
            //.filter(|lib| {
            //     ! (lib.is_native() && lib.filter_native_by_os())
            //})
            .for_each(
                |lib| {
                    let url = lib.get_download_url();
                    let path = Path::new(minecraft_path)
                        .join("libraries")
                        .join(lib.get_path())
                        .as_path()
                        .display()
                        .to_string();
                    let sha1 = lib.get_sha1();
                    files.push(FileData::new(path, url, Some(sha1)));
                }
            );
        Ok(files)
    }

    async fn assets_files(assets: AssetsJson, minecraft_path: &Path)  -> io::Result<Vec<FileData>> {
        let mut files: Vec<FileData> = Vec::new();
        let assets_dir = minecraft_path.join("assets").join("objects");
        assets.objects.into_iter().for_each(|object| {
            let hash = object.1.hash;
            let dir = format!("{}/{}", &hash[..2], hash);
            let url = format!("https://resources.download.minecraft.net/{}", dir);
            let file_path = assets_dir.join(dir.clone());

            files.push(FileData::new(file_path.to_str().unwrap().to_string(), url, None));
        });
        Ok(files)
    }
}
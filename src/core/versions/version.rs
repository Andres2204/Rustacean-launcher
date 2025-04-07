use crate::core::downloader::download_structs::{AssetsJson, VersionJson, VersionType};
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::assets::AssetDownloader;
use crate::core::versions::libraries::LibraryDownloader;
use crate::core::versions::manifest::VersionInfo;
use reqwest;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use std::fmt::{Display, Formatter};
use tokio::sync::Mutex;
use crate::core::downloader::downloader::download_file;
// TODO: trait -> normalversion, modpackversion, forgeversion (mod client)

pub trait Version {
    fn name(&self) -> String;
    fn set_name(&mut self, name: String);

    fn state(&self) -> VersionState;
    fn set_state(&mut self, state: VersionState);
    fn version_type(&self) -> VersionType;
    fn set_version_type(&mut self, version_type: VersionType);
    fn json_url(&self) -> String;
}

impl Display for dyn Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, State: {:?}, Type: {:?}, URL: {}",
            self.name(),
            self.state(),
            self.version_type(),
            self.json_url()
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VersionState {
    INSTALLED(bool),
    DOWNLOADING,
    VERIFYING,
}

#[derive(Debug)]
pub struct StandardVersion {
    name: String,
    version_type: VersionType,
    url: String,
    state: VersionState,
}
impl StandardVersion {
    pub fn new(name: &str, version_type: VersionType, url: &str, state: VersionState) -> Self {
        Self {
            name: name.to_string(),
            version_type,
            url: url.to_string(),
            state,
        }
    }
}
impl From<&VersionInfo> for StandardVersion {
    fn from(value: &VersionInfo) -> Self {
        Self {
            // TODO: avoid clone
            name: value.id.clone(),
            version_type: value.version_type.clone(),
            url: value.url.clone(),
            state: VersionState::INSTALLED(false),
        }
    }
}
impl Version for StandardVersion {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn set_name(&mut self, name: String) {
        self.name = name
    }
    fn state(&self) -> VersionState {
        self.state.clone()
    }
    fn set_state(&mut self, state: VersionState) {
        self.state = state;
    }
    fn version_type(&self) -> VersionType {
        self.version_type.clone()
    }
    fn set_version_type(&mut self, version_type: VersionType) {
        self.version_type = version_type;
    }
    fn json_url(&self) -> String {
        self.url.clone()
    }
}

// UTILITY FOR VERSIONS TODO: move to own files
pub struct VersionVerifier;
impl VersionVerifier {
    pub fn is_installed(mut version: &mut Box<(dyn Version + 'static)>) -> bool {
        if Path::new(&LauncherConfig::import_config().minecraft_path)
            .join("versions")
            .join(version.name())
            .join(format!("{}.json", version.name()))
            .as_path()
            .exists()
        {
            version.set_state(VersionState::INSTALLED(true));
            return true;
        }
        version.set_state(VersionState::INSTALLED(false));
        false
    }

    pub fn verify_installation(version: &Box<(dyn Version + 'static)>) -> bool {
        todo!()
    }
}

pub struct VersionDownloader;
impl VersionDownloader {
    pub async fn download_version(
        mut version: Box<dyn Version + 'static>,
        progress: Arc<Mutex<(usize, usize)>>,
    ) -> io::Result<()> {
        match version.version_type() {
            VersionType::RELEASE
            | VersionType::SNAPSHOT
            | VersionType::OldBeta
            | VersionType::OldAlpha => Self::download_standard(version, progress).await,
        }
    }

    async fn download_standard(
        mut version: Box<dyn Version + 'static>,
        mut progress: Arc<Mutex<(usize, usize)>>,
    ) -> io::Result<()> {
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

        *progress.lock().await = (0usize, total_objects);

        let library_progress = Arc::new(Mutex::new((0, total_libraries)));
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

        let asset_progress = Arc::new(Mutex::new((0, total_assets)));
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
        
        let mut cached_progress = 0usize;
        loop {
            let mut main_progress = progress.lock().await;
            let library_progress_guard = library_progress.lock().await;
            let asset_progress_guard = asset_progress.lock().await;

            main_progress.0 = library_progress_guard.0 + asset_progress_guard.0;
            if main_progress.0 != cached_progress {
                println!(" <> --- {main_progress:?} --- <>");
                cached_progress = main_progress.0;
            }
            if main_progress.0 == main_progress.1 {
                break;
            }
        }

        version.set_state(VersionState::INSTALLED(true));
        Ok(())
    }
    
    async fn download_initial_files(version: &Box<dyn Version + 'static>) -> io::Result<()> {

        let LauncherConfig { minecraft_path, version_manifest_link, .. } = LauncherConfig::import_config();
        // Paso 0.1: crear carpeta si no existe
        match fs::create_dir_all(minecraft_path.clone()) {
            Ok(e) => println!("Directory created {:?}", e),
            _ => {}
        }
        
        // download version.json
        let version_name = version.name();
        download_file(
            &version.json_url(),
            Path::new(&format!("{}/versions/{}/{}.json", &minecraft_path, &version_name, &version_name)),
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
            None
        ).await.expect("Cant download assets indexes json");
        
        // download client
        download_file(
            version_json.get_client_url().as_str(),
            Path::new(&minecraft_path.clone())
                .join("versions")
                .join(version_name.as_str())
                .join(format!("{}.json", version_name).as_str())
                .as_path(),
            None
        ).await.expect("Cant download client.jar ");
        
        Ok(())
    }
}

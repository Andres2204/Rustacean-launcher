use std::io;
use crate::core::downloader::download_structs::{AssetsJson, VersionJson, VersionType};
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::versions::version::{StandardVersion, Version, VersionState};
use std::path::{Path, PathBuf};

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
    
    // TODO: mejorar verificador con sha256
    pub fn verify_installation(version: &mut Box<(dyn Version + 'static)>) -> bool {
        // TODO: version file an client sha256
        let LauncherConfig { minecraft_path, .. } = LauncherConfig::import_config();
        let version_json = VersionJson::get_from_local(minecraft_path.clone(), version.name());
        let assets_paths: Vec<Box<PathBuf>> = {
            AssetsJson::from_local(
                Path::new(&minecraft_path)
                    .join("assets")
                    .join("indexes")
                    .join(format!("{}.json", &version_json.get_asset().id).as_str())
                    .as_path()
            )
            .get_assets_directories()
            .into_iter()
            .map(|p| Box::new(Path::new(&minecraft_path)
                .join("assets")
                .join("objects")
                .join(p)))
            .collect()
        };

        let libraries_paths: Vec<Box<PathBuf>> = {
            version_json
                .get_libraries()
                .into_iter()
                .map(|l| Box::new(Path::new(&minecraft_path)
                    .join("libraries")
                    .join(l.get_path())))
                .collect()
        };

        let assets_number = assets_paths.len();
        let libraries_number = libraries_paths.len();
        let mut verified = 0usize;
        println!("Verifying Minecraft version.
            assets: {assets_number}, libraries: {libraries_number}");

        // verify assets
        for path in &assets_paths {
            if !path.as_path().exists() {
                println!("Assets path does not exist: {:?}", path);
                return false;
            }
            verified += 1;
        }

        // verify libraries
        for path in &libraries_paths {
            if !path.as_path().exists() {
                println!("Library path does not exist: {:?}", path);
                return false;
            }
            verified += 1;
        }
        println!("Verified {verified} files");
        true
    }
    pub fn from_local(name: String) -> io::Result<Box<(dyn Version + 'static)>> {
        //TODO: adapt for forge, etc...
        
        let version_json = VersionJson::get_from_local(
            LauncherConfig::import_config().minecraft_path,
            name,
        );
        match version_json.get_type() {
            VersionType::RELEASE
            | VersionType::SNAPSHOT
            | VersionType::OldBeta
            | VersionType::OldAlpha => {
                Ok(StandardVersion::from_local(version_json))
            }
        }
    }
}

// TODO: verify with sha256
fn verify_file(file: PathBuf) -> bool {
    !file.as_path().exists()
}

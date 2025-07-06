use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::launcher::launcher_config::LauncherConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VersionType {
    #[serde(rename = "release")]
    RELEASE,

    #[serde(rename = "snapshot")]
    SNAPSHOT,

    #[serde(rename = "old_beta")]
    OldBeta,

    #[serde(rename = "old_alpha")]
    OldAlpha,
}

// version.json
#[derive(Debug, Deserialize)]
pub struct VersionJson {
    id: String,
    pub arguments: Arguments,
    downloads: Downloads,
    libraries: Vec<Library>,

    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    
    #[serde(rename = "type")]
    version_type: VersionType 
}

impl VersionJson {
    pub fn get_from_local(minecraft_path: String, version: String) -> Self {
        let mut file = File::open(format!(
            "{}/versions/{}/{}.json",
            &minecraft_path, &version, &version
        ))
        .expect("Failed to open version.json");

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read launcher_profiles.json");
        let json: VersionJson =
            serde_json::from_str(&content).expect("Failed to parse launcher_profiles.json");
        json
    }
}

impl VersionJson {
    pub fn get_client_url(&self) -> String {
        self.downloads.client.url.clone()
    }

    pub fn get_libraries(&self) -> Vec<Library> {
        self.libraries.clone()
    }

    pub fn get_asset_index(&self) -> AssetIndex {
        self.asset_index.clone()
    }
    
    pub fn get_assets_json(&self) -> AssetsJson {
        let assets = AssetsJson::from_local(
            Path::new(&LauncherConfig::import_config().minecraft_path)
                .join("assets")
                .join("indexes")
                .join(format!("{}.json", self.asset_index.id).as_str())
                .as_path(),
        );
        assets
    }
    
    pub fn get_type(&self) -> VersionType {
        self.version_type.clone()
    }
    
    pub fn id(&self) -> String {
        self.id.clone()
    }
}

// Arguments field
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Arguments {
    pub game: Vec<ArgumentRule>,
    pub jvm: Vec<ArgumentRule>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentRule {
    Simple(String), // Maneja argumentos como cadenas
    Complex {
        // Maneja argumentos como objetos con `rules` y `value`
        rules: Option<Vec<Rule>>,
        value: serde_json::Value, // `serde_json::Value` para manejar strings y arrays
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    action: String,
    features: Option<Features>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Features {
    is_demo_user: Option<bool>,
    has_custom_resolution: Option<bool>,
    has_quick_plays_support: Option<bool>,
    is_quick_play_singleplayer: Option<bool>,
    is_quick_play_multiplayer: Option<bool>,
    is_quick_play_realms: Option<bool>,
}

// Downloads field
#[derive(Debug, Deserialize)]
pub struct Downloads {
    client: Download,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Download {
    path: Option<String>,
    url: String,
}

// TODO: Separar Natives y diferenciar por arquitectura
// Libraries field
#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    downloads: LibraryDownload,
    rules: Option<Vec<LibraryRule>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryRule {
    action: String,
    os: Option<Os>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Os {
    name: String,
}
impl Library {
    pub fn get_download_url(&self) -> String {
        self.downloads.artifact.url.clone()
    }

    pub fn get_path(&self) -> String {
        self.downloads.artifact.path.clone().unwrap()
    }

    pub fn is_native(&self) -> bool {
        if let Some(rule) = &self.rules {
            if !rule.is_empty() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownload {
    artifact: Download,
}

// assets_index field
#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetsJson {
    pub objects: std::collections::HashMap<String, Asset>,
}
impl AssetsJson {
    pub fn from_local(assets_path: &Path) -> Self {
        let mut file = File::open(assets_path).expect("Failed to open assets.json");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read launcher_profiles.json");
        let json: AssetsJson =
            serde_json::from_str(&content).expect("Failed to parse launcher_profiles.json");
        json
    }
}
impl AssetsJson {
    pub fn get_assets_directories(&self) -> Vec<String> {
        let directories: Vec<String> = self.objects.clone()
            .into_iter()
            .map(|(_, h)| {
                let hash = h.hash;
                let dir = format!("{}/{}", &hash[..2], hash);
                dir
            })
            .collect();
        directories
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub hash: String,
}

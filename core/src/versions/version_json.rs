use crate::launcher::launcher_config::LauncherConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Define the type of the version (RELEASE, SNAPSHOT, OldBeta, OldAlpha)
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

// +============================+
//          VersionJson          
// +============================+

/// Abstracts the json of a version and implements getters for the different fields.
#[derive(Debug, Deserialize)]
pub struct VersionJson {
    id: String,
    arguments: Arguments,
    downloads: Downloads,
    libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    #[serde(rename = "type")]
    version_type: VersionType,
}

impl VersionJson {

    pub fn get_from_local(minecraft_path: &str, version: &str) -> Result<Self, String> {
        let path = Path::new(minecraft_path)
            .join("versions")
            .join(version)
            .join(format!("{}.json", version));

        let mut file = File::open(path.as_path()).expect(&format!(
            "Failed to open version.json on {}",
            path.display()
        ));
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read launcher_profiles.json");
        let json: VersionJson =
            serde_json::from_str(&content).expect("Failed to parse launcher_profiles.json");
        Ok(json)
    }
    
    pub fn get_client_url(&self) -> String {
        self.downloads.client.url.clone()
    }

    pub fn get_client_mappings_url(&self) -> String {
        self.downloads.client_mappings.url.clone()
    }

    pub fn get_arguments(&self) -> Arguments {
        self.arguments.clone()
    }

    pub fn get_libraries(&self) -> Vec<Library> {
        self.libraries.clone()
    }

    pub fn get_libraries_path(&self, minecraft_path: &str) -> Vec<String> {
        let libraries = self
            .libraries
            .iter()
            .map(|library| -> String {
                Path::new(minecraft_path)
                    .join("libraries")
                    .join(library.get_path())
                    .as_path()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        libraries
    }

    // pub fn filter_libraries(&self);

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
    
    pub fn get_main_class(&self) -> String {
        self.main_class.clone()
    }
    
    pub fn id(&self) -> String {
        self.id.clone()
    }
}

// +============================+
//           Arguments           
// +============================+

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Arguments {
    game: Vec<ArgumentRule>,
    jvm: Vec<ArgumentRule>,
}

impl Arguments {
    pub fn get_game(&self) -> &Vec<ArgumentRule> {
        &self.game
    }
    
    pub fn get_jvm(&self) -> &Vec<ArgumentRule> {
        &self.jvm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentRule {
    Simple(String), // Maneja argumentos como cadenas
    Complex {
        // Maneja argumentos como objetos con `rules` y `value`
        rules: Option<Vec<Rule>>,
        value: serde_json::Value, // `serde_json::Value` para manejar strings y arrays
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    action: String,
    features: Option<Features>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Features {
    is_demo_user: Option<bool>,
    has_custom_resolution: Option<bool>,
    has_quick_plays_support: Option<bool>,
    is_quick_play_singleplayer: Option<bool>,
    is_quick_play_multiplayer: Option<bool>,
    is_quick_play_realms: Option<bool>,
}

// +============================+
//           Downloads           
// +============================+

#[derive(Debug, Deserialize)]
pub struct Downloads {
    client: Download,
    client_mappings: Download
    // TODO: server & server mapping
}

#[derive(Debug, Clone, Deserialize)]
pub struct Download { // TODO: File data
    path: Option<String>,
    url: String,
    sha1: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    downloads: LibraryDownload,
    name: String,
    rules: Option<Vec<LibraryRule>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryRule {
    os: Option<Os>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Os {
    name: String,
}
impl Library {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_download_url(&self) -> String {
        self.downloads.artifact.url.clone()
    }

    pub fn get_path(&self) -> String {
        self.downloads.artifact.path.clone().unwrap()
    }

    pub fn get_sha1(&self) -> String {
        self.downloads.artifact.sha1.clone()
    }

    pub fn is_native(&self) -> bool {
        self.name.contains(":natives")
    }

    pub fn filter_native_by_os(&self) -> bool {
        let mut current_os: String = { env::consts::OS.to_string() };
        let mut current_arch: String = { env::consts::ARCH.to_string() }; // TODO: use arch
        if let Some(rules) = &self.rules {
            if current_os == "macos".to_owned() {
                current_os = "osx".to_owned();
            }
            for r in rules {
                if let Some(os) = r.os.as_ref() {
                    if os.name == current_os {
                        return true;
                    }
                }
            }
        } else {
            if self.name.contains(&format!(":natives-{}", current_os)) {
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

// +============================+
//             Assets            
// +============================+

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
    pub sha1: String,
}

// +============================+
//           AssetsJson          
// +============================+

/// Deserialize assets.json and implements usefull getters
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
        let directories: Vec<String> = self
            .objects
            .clone()
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

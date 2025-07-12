use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherConfig {
    pub minecraft_path: String,
    pub version_manifest_link: String, 
    pub ui: Ui,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Ui {
    TUI,
    GUI
}

// initializations & auxiliar
impl LauncherConfig {
    pub fn import_config() -> LauncherConfig {
        let path = Path::new("launcher_profiles.json");

        if !path.exists() {
            let default_config = Self::default();

            let serialized = serde_json::to_string_pretty(&default_config)
                .expect("Failed to serialize default launcher config");

            let mut file = File::create(path)
                .expect("Failed to create launcher_profiles.json");
            file.write_all(serialized.as_bytes())
                .expect("Failed to write default launcher config");

            return default_config;
        }

        let mut file = File::open(path)
            .expect("Failed to open launcher_profiles.json");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read launcher_profiles.json");

        serde_json::from_str(&content)
            .expect("Failed to parse launcher_profiles.json")
    }
    
    fn default() -> Self {
        Self {
            minecraft_path: "Minecraft".to_string(),
            version_manifest_link: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".to_string(),
            ui: Ui::TUI,
        }
    }

    pub fn save_config(&self) {
        todo!("save_config() not implemented yet")
    }
}

// Path getters
impl LauncherConfig {
    pub fn minecraft_path(&self) -> PathBuf {
        PathBuf::from(&self.minecraft_path)
    }
    pub fn libraries_path(&self) -> PathBuf {
        self.minecraft_path().join("libraries")
    }
    pub fn assets_path(&self) -> PathBuf {
        self.minecraft_path().join("assets")
    }
    pub fn versions_path(&self) -> PathBuf {
        self.minecraft_path().join("versions")
    }
}

// Launcher Profiles
#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherProfiles {
    profiles: HashMap<String, Profile>,
    #[serde(rename = "selectedUser")]
    selected_user: SelectedUser,
    #[serde(rename = "authenticationDatabase")]
    authentication_database: HashMap<String, AuthData>,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "launcherVersion", skip_serializing_if = "Option::is_none")]
    launcher_version: Option<LauncherVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<LauncherSettings>,
}

impl LauncherProfiles {
    pub fn import_profiles() -> Option<LauncherProfiles> {
        let path = Path::new("launcher_profiles.json");

        /*
        if !path.exists() {
            let default_config = Self::default();

            let serialized = serde_json::to_string_pretty(&default_config)
                .expect("Failed to serialize default launcher config");

            let mut file = File::create(path)
                .expect("Failed to create launcher_profiles.json");
            file.write_all(serialized.as_bytes())
                .expect("Failed to write default launcher config");

            return default_config;
        }
        */

        let mut file = File::open(path)
            .expect("Failed to open launcher_profiles.json");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read launcher_profiles.json");

        match serde_json::from_str(&content) {
            Ok(profile) => Some(profile),
            Err(_) => None,
        }
    }
    
    fn default() -> Self {
        Self {
            profiles: Default::default(),
            selected_user: SelectedUser { account: "".to_string() },
            authentication_database: Default::default(),
            client_token: "".to_string(),
            launcher_version: None,
            settings: None,
        }
    }
    
    pub fn settings(&self) -> Option<LauncherSettings> {
        self.settings.clone()
    }
    
    pub fn selected_user_account(&self) -> String {
        self.selected_user.account.clone()
    }
    
    pub fn authentication_database(&self) -> HashMap<String, AuthData> {
        self.authentication_database.clone()
    }
    
    pub fn client_token(&self) -> String {
        self.client_token.clone()
    }
    
    pub fn profiles(&self) -> HashMap<String, Profile> {
        self.profiles.clone()
    }
    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<String>, // "type" es palabra reservada
    pub created: Option<String>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<String>,
    #[serde(rename = "lastVersionId")]
    pub last_version_id: String,
    #[serde(rename = "gameDir", skip_serializing_if = "Option::is_none")]
    pub game_dir: Option<String>,
    #[serde(rename = "javaDir", skip_serializing_if = "Option::is_none")]
    pub java_dir: Option<String>,
    #[serde(rename = "javaArgs", skip_serializing_if = "Option::is_none")]
    pub java_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "useLatestVersion", skip_serializing_if = "Option::is_none")]
    pub use_latest_version: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectedUser {
    pub account: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthData {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userid: Option<String>,
    pub uuid: String,
    pub username: String,
    #[serde(rename = "userType", skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xuid: Option<String>, // Microsoft accounts
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherVersion {
    pub name: String,
    pub format: u8,
    #[serde(rename = "profilesFormat")]
    pub profiles_format: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LauncherSettings {
    #[serde(default)]
    pub crashAssistance: bool,
    #[serde(default)]
    pub enableAdvanced: bool,
    #[serde(default)]
    pub keepLauncherOpen: bool,
    #[serde(default)]
    pub showGameLog: bool,
    #[serde(default)]
    pub allowSnapshot: bool,
    #[serde(default)]
    pub allowBeta: bool,
    #[serde(default)]
    pub allowAlpha: bool,
    #[serde(default)]
    pub useNativeLauncher: bool,
    #[serde(default)]
    pub profileSorting: Option<String>,
}

impl LauncherSettings {
    pub fn default() -> Self {
        Self {
            crashAssistance: false,
            enableAdvanced: false,
            keepLauncherOpen: false,
            showGameLog: false,
            allowSnapshot: true,
            allowBeta: false,
            allowAlpha: false,
            useNativeLauncher: false,
            profileSorting: Some("LastUsed".to_owned()),
        }
    }
}



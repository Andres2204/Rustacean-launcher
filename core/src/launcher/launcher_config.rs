use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: singleton
#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherConfig {
    pub minecraft_path: String,
    pub version_manifest_link: String, 
    pub ui: Ui,
    pub profiles: HashMap<String, Profile>,
    #[serde(rename = "selectedUser")]
    pub selected_user: SelectedUser,
    #[serde(rename = "authenticationDatabase")]
    pub authentication_database: HashMap<String, AuthData>,
    #[serde(rename = "clientToken")]
    pub client_token: String,
    #[serde(rename = "launcherVersion", skip_serializing_if = "Option::is_none")]
    pub launcher_version: Option<LauncherVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<LauncherSettings>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectedUser {
    pub account: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Ui {
    TUI,
    GUI
}

// initializations & auxiliar
impl LauncherConfig {
    pub fn import_config() -> LauncherConfig {
        let mut file = File::open("launcher_profiles.json").expect("Failed to open launcher_profiles.json");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read launcher_profiles.json");
        let config: LauncherConfig = serde_json::from_str(&content).expect("Failed to parse launcher_profiles.json");
        config
    }
}

// Atribute getters
impl LauncherConfig {
    pub fn profiles(&self) -> &HashMap<String, Profile> {
        &self.profiles
    }
    
    pub fn selected_user(&self) -> &SelectedUser {
        &self.selected_user
    }

    pub fn authentication_database(&self) -> &HashMap<String, AuthData> {
        &self.authentication_database
    }
    
    pub fn client_token(&self) -> &str {
        &self.client_token
    }
    
    pub fn launcher_version(&self) -> Option<&LauncherVersion> {
        self.launcher_version.as_ref()
    }
    
    pub fn settings(&self) -> Option<&LauncherSettings> {
        self.settings.as_ref()
    }
}

// Atribute setters (write on json)
impl LauncherConfig {
    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.insert(profile.name.clone(), profile);
    }
    
    pub fn remove_profile(&mut self, profile: &str) {
        self.profiles.remove(profile);
    }
    
    pub fn edit_profile(&mut self, profile_id: &str, profile: Profile) {
        todo!("edit_profile() not implemented yet")
    }
    
    pub fn save_config() {
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
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

// TODO: singleton LauncherConfig
#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub minecraft_path: String,
    pub version_manifest_link: String,
    pub ui: Ui
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Ui {
    TUI,
    GUI
}

impl LauncherConfig {
    pub fn import_config() -> LauncherConfig {
        let mut file = File::open("launcher_config.json").expect("Failed to open launcher_config.json");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read launcher_config.json");
        let config: LauncherConfig = serde_json::from_str(&content).expect("Failed to parse launcher_config.json");
        config
    }
}



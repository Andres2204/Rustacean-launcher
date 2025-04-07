use crate::command::command::Command;
use crate::core::downloader::download_structs::VersionType;
use crate::core::launcher::launcher::MinecraftLauncher;
use crate::core::launcher::launcher_config::LauncherConfig;
use crate::core::user::offline_user::OfflineUser;
use crate::core::versions::version::{StandardVersion, VersionState};

pub struct LaunchCommand;
impl Command for LaunchCommand {
    fn execute(&mut self) {
        let launcher_config = LauncherConfig::import_config();
        let launcher: MinecraftLauncher = MinecraftLauncher {
            version: Box::new(StandardVersion::new(
                "1.19.3",
                VersionType::RELEASE,
                "https://piston-meta.mojang.com/v1/packages/526571ff4d3513ff70d59c72ad525f5cc3c0db4d/1.19.3.json",
                VersionState::INSTALLED(true)
            )),
            launcher_config,
            user: OfflineUser {
                name: "AndresGamer4444".to_string(),
            },
        };

        println!("Launching minecraft by LaunchCommand: ");
        launcher.launch_minecraft().expect("Failed to launch minecraft ");
    }

    fn undo(&mut self) {}
}
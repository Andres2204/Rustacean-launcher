use std::error::Error;
mod core;
mod tui;
pub(crate) mod command;
use command::command::Command;
use command::commands::launch::LaunchCommand;
use crate::core::downloader::download_structs::VersionType;
use crate::core::versions::version::{StandardVersion, Version, VersionDownloader, VersionState};
use crate::core::versions::version_manager::VersionManager;
// TODO: descargar los jdks necesarios y guardarlos en una carpeta

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //download_version("1.21.3", VersionType::RELEASE)
    //    .await
    //    .expect("[MAIN] Download failed ");
    
    //if let Ui::TUI = LauncherConfig::import_config().ui {
    //    println!("Starting tui...");
    //    tui::app::Tui::new().run_tui().expect("[MAIN/RATATUI] Failed to run UI");
    //}

    let v: Box<dyn Version> = Box::new(StandardVersion::new(
        "1.19.3",
        VersionType::RELEASE,
        "https://piston-meta.mojang.com/v1/packages/526571ff4d3513ff70d59c72ad525f5cc3c0db4d/1.19.3.json",
        VersionState::INSTALLED(false)
    ));
    VersionManager::download_version(v).await?;
    let versions = VersionManager::fetch_versions().await?;
    // versions.into_iter().for_each(|v| {println!("{v}")});
    LaunchCommand.execute();
    Ok(())
}

// TODO: REFINE THE VersionJson Structs
// TODO: quitar exceso de clonaciones para libraries, modulaizar
// TODO: multithreading download



/*
minecraft/
├── assets/
│   ├── indexes/
│   │   └── <version>.json                    # Archivo JSON del índice de assets para la versión (por ejemplo, "1.20.1.json")
│   └── objects/
│       ├── 4e/                               # Subcarpeta con los primeros dos caracteres del hash del asset
│       │   └── 4e0c9a57bb83358f5c36f5d32cf7635b2ec66532  # Archivo de asset (por ejemplo, sonidos, texturas)
│       ├── a5/
│       │   └── a5d830475ec0958d9fdba1559efa99aef211e6ff  # Otro archivo de asset
│       └── ...                               # Otras subcarpetas con los assets organizados por hash
├── libraries/
│   ├── com/
│   │   └── mojang/
│   │       ├── authlib/
│   │       │   └── 1.5.25/
│   │       │       └── authlib-1.5.25.jar    # Biblioteca específica de Mojang
│   │       └── ...                           # Otras bibliotecas necesarias para la ejecución de Minecraft
│   ├── org/
│   │   └── lwjgl/
│   │       ├── lwjgl/
│   │       │   └── 3.3.1/
│   │       │       └── lwjgl-3.3.1.jar       # Biblioteca LWJGL
│   │       └── ...                           # Otras bibliotecas de LWJGL
│   └── ...                                   # Otras bibliotecas usadas por el cliente
└── versions/
    └── <version>/                            # Directorio específico de la versión (por ejemplo, "1.20.1")
        ├── <version>.json                    # Archivo JSON de la versión descargado (por ejemplo, "1.20.1.json")
        └── client.jar                        # Archivo `client.jar` de la versión específica


*/

use std::env;
mod tui;
use core;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use core::launcher::{
    launcher_config::{
        LauncherConfig,
        Ui
    },
    launcher::MinecraftBuilder
};
use core::versions::{
    version::{
        VersionBuilder,
        VersionState
    },
    version_manager::VersionManager
};
use core::downloader::downloader::DownloaderTracking;
use core::users::UserBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Ui::TUI = LauncherConfig::import_config().ui {
       log::info!("Starting tui...");
       tui::app::Tui::new().run_tui().expect("[MAIN/RATATUI] Failed to run UI");
    } else {
        unsafe { env::set_var("RUST_LOG", "info"); }
        env_logger::init();

        
        VersionManager::download_version(VersionBuilder::default()
            .name("1.21.3")
            .state(VersionState::INSTALLED(false))
            .url("https://piston-meta.mojang.com/v1/packages/b64c551553e59c369f4a3529b15c570ac6b9b73e/1.21.3.json")
            .build().unwrap(),
            Arc::new(Mutex::new(DownloaderTracking::default()))
        ).await.expect("Failed to download version");
        
        /*
        let ml = MinecraftBuilder::new()
            .version(VersionBuilder::default()
                .name("1.21.7")
                .state(VersionState::INSTALLED(true))
                .build().unwrap()
            )
            .user(UserBuilder::default_boxed())
            .build();
        ml?.launch().expect("this shit failed ¯\\_(ツ)_/¯");
        */
         
    }
    Ok(())
}

/*
minecraft/
├── assets/
│   ├── indexes/
│   │   └── <index_id>.json                    # Archivo JSON del índice de assets para la versión (por ejemplo, "1.20.1.json")
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

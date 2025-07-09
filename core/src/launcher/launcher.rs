use std::path::Path;
use std::process::Stdio;
use crate::downloader::download_structs::{VersionJson, Arguments};
use crate::launcher::launcher_config::{LauncherConfig, LauncherProfiles};
use crate::versions::Version;
use crate::users::User;

pub struct MinecraftLauncher {
    version: Box<dyn Version>,
    user: Box<dyn User>,
    launcher_config: LauncherConfig,
    version_json: VersionJson,
    profiles: Option<LauncherProfiles>
}

impl MinecraftLauncher {
    pub fn launch(&self) -> std::io::Result<()> {
        // TODO: java -Xmx8G -Djava.library.path=C:\Minecraft\natives -cp "C:\Minecraft\libraries\lib1.jar;C:\Minecraft\libraries\lib2.jar;C:\Minecraft\versions\1.21.3&self, &self, \client.jar" net.minecraft.client.main.Main --username "MiUsuarioOffline" --version "1.21.3" --gameDir "C:\Minecraft" --assetsDir "C:\Minecraft\assets" --assetIndex "1.21.3" --uuid "OfflineUUID" --accessToken "OfflineAccessToken" --userType "legacy"

        /*
        let java_path = JavaPath::default();
        let jvm_args = self.build_jvm_args();
        let classpath = self.build_classpath();
        let main_class = self.version_json.get_main_class()
        let game_args = self.build_game_args();
        let auth_args = self.build_auth_args();
        */

        let minecraft_path = &self.launcher_config.minecraft_path;
        let client_jar = Path::new(minecraft_path)
            .join("versions")
            .join(self.version.name())
            .join(format!("{}.jar", self.version.name()).as_str());
        let classpath = self.build_classpath(client_jar.as_path());

        log::debug!("Classpath: {}", classpath);

        let java_path = JavaPath::default(); // TODO: configure a nice java path selector JavaPath::get(major_version).unwrap_or(default system jdk)
        let username = self.user.username().clone();
        let binding = minecraft_path.clone();
        let game_dir = Path::new(&binding);
        let assets_dir = Path::new(&minecraft_path)
            .join("assets");

        log::info!(
            "Launching minecraft with:
            java: {java_path}
            users: {username}
            game_dir: {game_dir:?}
            assets_dir: {:?}
            assets_index: {:?}
            client_jar. {:?}
            version: {:?}
            ", &assets_dir.as_path(),
            &self.version_json.get_asset_index().id,
            client_jar.as_path(),
            self.version.name().clone()
        );
        
        // TODO: https://minecraft-launcher-lib.readthedocs.io/en/latest/modules/command.html
        // TODO: https://minecraft.fandom.com/wiki/Client.json
        // Construye el comando para ejecutar Minecraft
        let mut command = std::process::Command::new(java_path);
        log::info!("Command created {:?}", command);
        command
            // jvm args
            //.arg(format!("-Djava.library.path={}", Path::new(&minecraft_path.clone()).join("libraries").as_path().to_str().unwrap())) // Ruta de las bibliotecas nativas
            //.arg("-Xmx8G") // Ajusta la memoria según sea necesario

            .arg("-cp").arg(classpath) // Classpath con `client.jar` y bibliotecas
            .arg("net.minecraft.client.main.Main") // Clase principal del cliente

            // game arguments
            .arg("--username").arg(username) // Nombre de usuario
            .arg("--version").arg(&self.version.name()) // Versión
            .arg("--gameDir").arg(game_dir) // Directorio del juego
            .arg("--assetsDir").arg(assets_dir) // Directorio de assets
            .arg("--assetIndex").arg(self.version_json.get_asset_index().id) // Índice de assets para la versión
            .arg("--accessToken").arg("notokenxd")
            .arg("--userType").arg("legacy")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Ejecuta el comando y maneja posibles errores
        let status = command.spawn()?.wait()?;

        log::info!("Minecraft finished with status {:?}", status);
        if status.success() {
            log::info!("Minecraft ha finalizado correctamente.");
        } else {
            log::error!("Error al iniciar Minecraft.");
        }
        Ok(())
    }

    fn build_jvm_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        // TODO: Adapt command to arguments in version.json
        /* if ! profile args {
            let Arguments { game , jvm } = &version_json.arguments; <- get defaults?
        } else { profile args }

        */

        args
    }

    fn build_game_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args
    }

    fn build_auth_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args
    }

    fn build_classpath(&self, client_jar_path: &Path) -> String {
        // Delimitador para classpath: `:` en Unix y `;` en Windows
        let delimiter = if cfg!(target_os = "windows") { ";" } else { ":" };

        let libraries = self.version_json.get_libraries_path(&self.launcher_config.minecraft_path);
        let mut classpath = libraries.join(delimiter);
        classpath.push_str(delimiter);
        classpath.push_str(client_jar_path.to_str().unwrap());
        classpath
    }
}

pub struct MinecraftBuilder {
    version: Option<Box<dyn Version>>,
    user: Option<Box<dyn User>>
}

impl MinecraftBuilder {
    pub fn default() -> MinecraftLauncher {
        todo!();
        /*
        MinecraftLauncher {
            version: Lastest dowload version or cached,
            user: Cached user or offline defualt,
            launcher_config: idk,
        }
        */
    }
    
    pub fn new() -> Self {
        MinecraftBuilder {
            version: None,
            user: None,
        }
    }
}

impl MinecraftBuilder {
    pub fn version(mut self, version: Box<dyn Version>) -> Self {
        self.version = Some(version);
        self
    }
    
    pub fn user(mut self, user: Box<dyn User>) -> Self {
        self.user = Some(user);
        self
    }
    
    pub fn build(self) -> Result<MinecraftLauncher, String> {
        let version = self.version.ok_or_else(|| "Version is required".to_string())?;
        let user = self.user.ok_or_else(|| "User is required".to_string())?;
        
        let launcher_config = LauncherConfig::import_config();
        let version_name = version.name();

        let version_json = VersionJson::get_from_local(&launcher_config.minecraft_path, &version_name)
            .map_err(|e| format!("Failed to load version JSON: {e}"))?;

        let profiles = LauncherProfiles::import_profiles();

        Ok(MinecraftLauncher {
            version,
            user,
            launcher_config,
            version_json,
            profiles,
        })
    }
}

struct JavaPath;

impl JavaPath {
    fn default() -> String {
        "/home/andres/.jdks/openjdk-23.0.1/bin/java".to_owned()
    }
}
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

        let minecraft_path = &self.launcher_config.minecraft_path;
        let client_jar = Path::new(minecraft_path)
            .join("versions")
            .join(self.version.name())
            .join(format!("{}.jar", self.version.name()).as_str());
        let java_path = JavaPath::default();
        let jvm_args = self.build_jvm_args();
        let classpath = self.build_classpath(client_jar.as_path());
        let main_class = self.version_json.get_main_class();
        let game_args = self.build_game_args();
        let auth_args = self.build_auth_args();
        

        log::info!(
            "Launching minecraft with:
            java: {java_path}
            users: {}
            game_dir: {minecraft_path}
            client_jar. {:?}
            version: {:?}",
            self.user.username(),
            client_jar.as_path(),
            self.version.name()
        );
        
        // TODO: https://minecraft-launcher-lib.readthedocs.io/en/latest/modules/command.html
        // TODO: https://minecraft.fandom.com/wiki/Client.json
        // Construye el comando para ejecutar Minecraft
        let mut command = std::process::Command::new(java_path);
        log::info!("Command created {:?}", command);
        command
            .args(jvm_args)
            .arg("-cp").arg(classpath) // Classpath con `client.jar` y bibliotecas
            .arg(main_class) // Clase principal del cliente
            .args(game_args)
            .args(auth_args)
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

    fn build_jvm_args(&self) -> Vec<&str> {
        let mut args = Vec::new();
        // TODO: Adapt command to arguments in version.json
        /* if ! profile args {
            let Arguments { game , jvm } = &version_json.arguments; <- get defaults?
        } else { profile args }
        */

        if let Some(profiles) = self.profiles.as_ref() {
            let p = profiles.profiles();
            
        }
        args.push("-Xmx8G");
        //args.push("-Djava.library.path=${natives_directory}".to_owned());
        args
    }

    fn build_game_args(&self) -> Vec<String> {
        let version_name = &self.version.name();
        let game_dir = &self.launcher_config.minecraft_path;
        let assets_dir = &self.launcher_config.assets_path(); 
        let index_clone = self.version_json.get_asset_index().clone();
        let assets_index = &format!("{}", index_clone.id);
        let version_type = "release"; // TODO: VersonType to_str()
        let args: Vec<&str> = vec![
            "--version", version_name,
            "--gameDir", game_dir,
            "--assetsDir", assets_dir.as_path().to_str().unwrap(),
            "--assetIndex", assets_index,
            "--versionType", version_type
        ];
        // TODO: Parse or use a macro

        args.iter().map(|a| a.to_string()).collect()
    }

    fn build_auth_args(&self) -> Vec<String> {
        let username = &self.user.username();
        let auth_uuid = "what?";
        let auth_access_token = "i haven't that";
        let client_id= "where client_id is obtained?";
        let auth_xuid= "sad";
        let user_type= "legacy";
        let args = vec![
            "--username", username,
            "--uuid", auth_uuid,
            "--accessToken", auth_access_token,
            "--clientId", client_id,
            "--xuid", auth_xuid,
            "--userType", user_type,
        ];
        args.iter().map(|a| a.to_string()).collect()
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
            profiles
        })
    }
}

struct JavaPath;

impl JavaPath {
    fn default() -> String {
        "/home/andres/.jdks/openjdk-23.0.1/bin/java".to_owned()
    }
}
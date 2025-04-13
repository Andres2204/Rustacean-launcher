use std::path::Path;
use std::process::Stdio;
use crate::downloader::download_structs::{VersionJson, Arguments};
use crate::launcher::launcher_config::LauncherConfig;
use crate::versions::version::Version;
use crate::user::offline_user::OfflineUser;

// TODO: is public!!
pub struct MinecraftLauncher {
    pub version: Box<dyn Version>,
    pub user: OfflineUser,
    pub launcher_config: LauncherConfig
}

impl MinecraftLauncher {
    pub fn launch_minecraft(
        &self,
    ) -> std::io::Result<()> {
        // TODO: java -Xmx8G -Djava.library.path=C:\Minecraft\natives -cp "C:\Minecraft\libraries\lib1.jar;C:\Minecraft\libraries\lib2.jar;C:\Minecraft\versions\1.21.3&self, &self, \client.jar" net.minecraft.client.main.Main --username "MiUsuarioOffline" --version "1.21.3" --gameDir "C:\Minecraft" --assetsDir "C:\Minecraft\assets" --assetIndex "1.21.3" --uuid "OfflineUUID" --accessToken "OfflineAccessToken" --userType "legacy"
        let LauncherConfig {minecraft_path, ..} = &self.launcher_config;
        
        // obtener librerias
        let version_json = VersionJson::get_from_local(minecraft_path.clone(), self.version.name());
        
        // TODO: Adapt command to arguments in version.json
        //let Arguments { game , jvm } = &version_json.arguments;
        
        let mut count = 0;
        let libraries = version_json.get_libraries().iter().map(
            |library| -> String {
                if library.is_native() { count+=1; }
                Path::new(&minecraft_path.clone())
                    .join("libraries")
                    .join(library.get_path()).as_path().to_str().unwrap().to_string()
            }
        ).collect();
        println!("Natives found: {count}");
        
        let client_jar = Path::new(&minecraft_path)
            .join("versions")
            .join(self.version.name())
            .join(format!("{}.jar", self.version.name()).as_str());
        
        let classpath = build_classpath(client_jar.as_path(), libraries);
        // println!("Classpath: {}", classpath);

        let java_path = "/home/andres/.jdks/openjdk-23.0.1/bin/java"; // TODO
        let username = self.user.name.clone();
        let binding = minecraft_path.clone();
        let game_dir = Path::new(&binding);
        let assets_dir = Path::new(&minecraft_path)
            .join("assets");

        println!(
            "Launching minecraft with:
            java: {java_path}
            user: {username}
            game_dir: {game_dir:?}
            assets_dir: {:?}
            assets_index: {:?}
            client_jar. {:?}
            version: {:?}
            ", &assets_dir.as_path(),
            &version_json.get_asset().id,
            client_jar.as_path(),
            self.version.name().clone()
        );
        
        // TODO: https://minecraft-launcher-lib.readthedocs.io/en/latest/modules/command.html
        // TODO: https://minecraft.fandom.com/wiki/Client.json
        // Construye el comando para ejecutar Minecraft
        let mut command = std::process::Command::new(java_path);
        println!("Command created {:?}", command);
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
            .arg("--assetIndex").arg(version_json.get_asset().id) // Índice de assets para la versión
            .arg("--accessToken").arg("notokenxd")
            .arg("--userType").arg("legacy")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Ejecuta el comando y maneja posibles errores
        let status = command.spawn()?.wait()?;
        
        println!("Minecraft finished with status {:?}", status);
        if status.success() {
            println!("Minecraft ha finalizado correctamente.");
        } else {
            eprintln!("Error al iniciar Minecraft.");
        }
        Ok(())
    }
}

pub fn build_classpath(client_jar_path: &Path, libraries: Vec<String>) -> String {
    // Delimitador para classpath: `:` en Unix y `;` en Windows
    let delimiter = if cfg!(target_os = "windows") { ";" } else { ":" };
    // println!("Class path delimiter: {}", delimiter);

    // Construye el classpath con client.jar y todas las bibliotecas
    let mut classpath = libraries.join(delimiter);
    classpath.push_str(delimiter);
    classpath.push_str(client_jar_path.to_str().unwrap());
    classpath
}

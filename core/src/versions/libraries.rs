use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::downloader::download_structs::Library;
use crate::downloader::downloader::{download_files_concurrently, DownloaderTracking};

pub struct LibraryDownloader;

impl LibraryDownloader {
    pub async fn download_libraries(libraries: Vec<Library>, minecraft_path: &Path, progress: Option<Arc<Mutex<DownloaderTracking>>>) -> io::Result<()> {
        let mut map: HashMap<String, String> = HashMap::new();
        libraries.into_iter().for_each(
            |lib| {
                let url = lib.get_download_url();
                let path = Path::new(minecraft_path)
                    .join("libraries")
                    .join(lib.get_path())
                    .as_path()
                    .display()
                    .to_string();
                map.insert(url, path);
            }
        );
        
        println!("Downloading libraries... {}", map.len());
        //map.into_iter().for_each(|v| println!("{v:?}"));
        download_files_concurrently(map, None, progress).await.expect("Failed to download the files");
        Ok(())
    }
}
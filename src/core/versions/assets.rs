use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::downloader::download_structs::AssetsJson;
use crate::core::downloader::downloader::download_files_concurrently;

pub struct AssetDownloader;
impl AssetDownloader {
    pub async fn download_assets(assets: AssetsJson, minecraft_path: &Path, progress: Option<Arc<Mutex<(usize, usize)>>>)  -> io::Result<()> {
        let mut map: HashMap<String, String> = HashMap::new();
        let assets_dir = minecraft_path.join("assets").join("objects");
        
        assets.objects.into_iter().for_each(|object| {
            let hash = object.1.hash;
            let dir = format!("{}/{}", &hash[..2], hash);
            let url = format!("https://resources.download.minecraft.net/{}", dir);
            let file_path = assets_dir.join(dir.clone());
            
            map.insert(url, file_path.to_str().unwrap().to_string());
        });
        
        download_files_concurrently(map, None, progress).await?;
        Ok(())
    }
}
use futures_util::StreamExt;
use futures_util::future::join_all;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Semaphore};
use tokio::{fs::File as AsyncFile, task};
use crate::versions::verifier::VersionVerifier;
#[derive(Debug)]
pub struct DownloaderTracking {
    progress: (usize, usize),
    units: Vec<Arc<RwLock<FileProgress>>>,
}

impl DownloaderTracking {
    pub fn new(progress: (usize, usize)) -> Self {
        Self {
            progress,
            units: Vec::new(),
        }
    }

    pub fn default() -> Self {
        Self::new((0, 0))
    }
}

impl DownloaderTracking {
    // TODO: enhance initialization methods
    pub fn progress(&self) -> (usize, usize) {
        self.progress
    }
    pub fn set_progress(&mut self, progress: (usize, usize)) {
        self.progress = progress
    }

    pub fn actual_progress(&self) -> usize {
        self.progress.0
    }

    pub fn set_actual_progress(&mut self, actual: usize) {
        self.progress.0 = actual
    }

    pub fn total_progress(&self) -> usize {
        self.progress.1
    }

    pub fn finished(&self) -> bool {
        self.progress.0 >= self.progress.1
    }

    pub fn units(&self) -> Vec<Arc<RwLock<FileProgress>>> {
        self.units.clone()
    }

    pub fn add_unit(&mut self, unit: Arc<RwLock<FileProgress>>) {
        self.units.push(unit)
    }

    pub async fn remove_unit(&mut self, unit: String) {
        // Filtra la colección de manera asíncrona
        let filtered_units: Vec<_> = join_all(self.units.iter().map(|u| {
            let unit_name = unit.to_string();
            let u_clone = Arc::clone(u);
            async move {
                let guard = u_clone.read().unwrap();
                if !guard.name.eq(&unit_name) {
                    Some(u_clone.clone())
                } else {
                    None
                }
            }
        }))
        .await
        .into_iter()
        .flatten()
        .collect();

        // Reemplaza la colección original
        self.units = filtered_units;
    }
}

#[derive(Debug, Clone)]
pub struct FileProgress {
    name: String,
    progress: (usize, usize),
}

impl FileProgress {
    pub fn new(name: String) -> Self {
        Self {
            name,
            progress: (0, 0),
        }
    }
}

impl FileProgress {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn progress(&self) -> (usize, usize) {
        self.progress
    }
    pub fn set_progress(&mut self, progress: (usize, usize)) {
        self.progress = progress
    }

    pub fn actual_progress(&self) -> usize {
        self.progress.0
    }

    pub fn set_actual_progress(&mut self, actual: usize) {
        self.progress.0 = actual
    }

    pub fn total_progress(&self) -> usize {
        self.progress.1
    }

    pub fn finished(&self) -> bool {
        self.progress.0 >= self.progress.1
    }
}

#[derive(Debug, Clone)]
pub struct FileData {
    path: String,
    url: String,
    sha1: Option<String>,
}

impl FileData {
    pub fn new(path: String, url: String, sha1: Option<String>) -> Self {
        Self {path, url, sha1}
    }
}

pub async fn download_files_concurrently(
    files: Vec<FileData>,
    reqwest_client: Option<&Client>,
    progress: Option<Arc<Mutex<DownloaderTracking>>>,
) -> io::Result<()> 
{
    log::info!("Downloading files");
    let client = match reqwest_client {
        Some(c) => c,
        None => &Client::new(),
    };

    let max_concurrent_tasks = 64usize;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
    
    let tasks: Vec<_> = files
        .into_iter()
        .map(|v| {
            let client = client.clone();
            let permit = semaphore.clone().acquire_owned();
            let value = v.clone();
            let progress = progress.clone();
            task::spawn(async move {
                let _permit = permit.await;

                // add to vector
                let url = value.url.clone();
                let mut file_progress: Option<Arc<RwLock<FileProgress>>> = None;
                if let Some(p) = progress.clone() {
                    let fp = Arc::new(RwLock::new(FileProgress::new(url.clone())));
                    p.lock().await.add_unit(fp.clone());
                    file_progress = Some(fp);
                }
                
                let res = download_file(
                    value,
                    Some(&client),
                    file_progress.clone(),
                )
                .await;
                
                // add 1 to actual progress and file download from remove to vector
                if let Some(p) = progress {
                    let mut pro = p.lock().await;
                    let actual_progress = pro.actual_progress();
                    pro.set_actual_progress(actual_progress + 1);
                    pro.remove_unit(url).await;
                }
                drop(_permit);
                res
            })
        })
        .collect();

    let results = join_all(tasks).await;
    for result in results {
        match result {
            Ok(Ok(())) => {} // La tarea se completó correctamente
            Ok(Err(e)) => log::error!("Error descargando el archivo: {:?}", e),
            Err(e) => log::error!("Error en la tarea: {:?}", e),
        }
    }
    Ok(())
}

pub async fn download_files_secuentialy(
    _files: HashMap<String, String>,
    _reqwest_client: Option<&Client>,
    _progress: Option<Arc<Mutex<(usize, usize)>>>,
) {
    unimplemented!("[File dowloader secuentialy] Not implemented ")
}

// TODO: Builter pattern
pub async fn download_file(
    file: FileData,
    client: Option<&Client>,
    progress: Option<Arc<RwLock<FileProgress>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> 
{
    let url = file.url;
    let dest= Path::new(&file.path);
    
    if dest.exists() {
        match (VersionVerifier::get_sha1(dest), &file.sha1) {
            (Ok(local_sha1), Some(expected_sha1)) => {
                if &local_sha1 == expected_sha1 {
                    log::info!(
                    "File already exists and passed checksum: {}",
                    dest.display()
                );
                    return Ok(());
                } else {
                    log::warn!(
                    "Checksum mismatch for {}. Expected: {}, Found: {}. Redownloading.",
                    dest.display(),
                    expected_sha1,
                    local_sha1
                );
                }
            }
            (Err(e), Some(_)) => {
                log::warn!(
                "Failed to compute checksum for {}: {}. Redownloading.",
                dest.display(),
                e
            );
            }
            (_, None) => {
                log::info!(
                "File exists but no checksum to verify: {}. Proceeding to redownload.",
                dest.display()
            );
            }
        }
    }

    if let Some(parent) = dest.parent() {
        log::debug!("Create parents directories: {:?}", parent);
        fs::create_dir_all(parent)?;
    }

    // Download file
    let client = match client {
        Some(c) => c,
        _ => &Client::new(),
    };
    
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            // remove to vector
            return Err(Box::new(e));
        }
    };
    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut file = AsyncFile::create(dest).await?;
    let mut total_writen = 0;

    if let Some(progress) = progress.as_ref() {
        progress
            .write()
            .unwrap()
            .set_progress((0, total_size as usize));
    }

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        total_writen += chunk.len();
        if let Some(progress) = progress.as_ref() {
            progress.write().unwrap().set_actual_progress(total_writen);
        }
        let progress_percent = (total_writen as f64 / total_size as f64) * 100.0;
        log::debug!("{:?} - {:.2}%", dest.to_str(), progress_percent);
    }
    file.flush().await?;
    
    /*
    TODO:
    Después de descargar el archivo, verifica también el checksum, y si falla:
        reintenta la descarga (máx. 2 o 3 veces), si no, abortar
    */
    Ok(())
}

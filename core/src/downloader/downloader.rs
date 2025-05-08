use std::fs;
use std::{io};
use std::collections::HashMap;
use std::path::{Path};
use std::sync::Arc;
use futures_util::future::join_all;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::sync::{Mutex, Semaphore};
use tokio::{fs::File as AsyncFile, task};
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct DownloaderTracking {
    progress: (usize, usize),
    units: Vec<Arc<Mutex<FileProgress>>>
}

#[derive(Debug, Clone)]
pub struct FileProgress {
    name: String,
    progress: (usize, usize),
}

impl FileProgress {
    pub fn new(name: String) -> Self {
        Self { name, progress: (0, 0) }
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

impl DownloaderTracking {
    pub fn new(progress: (usize, usize)) -> Self {
        Self {
            progress,
            units: Vec::new()
        }
    }
}

impl DownloaderTracking { // TODO: enhance initialization methods
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
    
    pub fn units(&self) -> Vec<Arc<Mutex<FileProgress>>> {
        self.units.clone()
    }
    
    pub fn add_unit(&mut self, unit: Arc<Mutex<FileProgress>>) {
        self.units.push(unit)
    }
    
    pub async fn remove_unit(&mut self, unit: String) {
        // Filtra la colección de manera asíncrona
        let filtered_units: Vec<_> = join_all(
            self.units.iter().map(|u| {
                let unit_name = unit.to_string();
                let u_clone = Arc::clone(u);
                async move {
                    let guard = u_clone.lock().await;
                    if !guard.name.eq(&unit_name) {
                        Some(u_clone.clone())
                    } else {
                        None
                    }
                }
            })
        )
            .await
            .into_iter()
            .flatten()
            .collect();

        // Reemplaza la colección original
        self.units = filtered_units;
    }
}

pub async fn download_files_concurrently(
    files: HashMap<String, String>, 
    reqwest_client: Option<&Client>,
    progress: Option<Arc<Mutex<DownloaderTracking>>>
) -> io::Result<()> {
    let client = match reqwest_client {
        Some(c) => {c}
        None => {&Client::new()}
    };

    let max_concurrent_tasks = 96usize;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    let tasks: Vec<_> = files.into_iter().map(|v| {
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned();
        let value = v.clone();
        let progress = progress.clone();
        task::spawn({
            async move {
                let _permit = permit.await;
                let url = value.0;
                // add to vector
                let mut file_progress: Option<Arc<Mutex<FileProgress>>> = None;
                if let Some(p) = progress.clone() {
                    let fp = Arc::new(Mutex::new(FileProgress::new(
                        url.clone())));
                    p.lock().await.add_unit(
                        fp.clone()
                    );
                    file_progress = Some(fp);
                }
                
                let res = download_file(&url, Path::new::<>(&value.1), Some(&client), file_progress.clone()).await;
                
                // add 1 to actual progress and file download from remove to vector
                if let Some(p) = progress {
                    let mut pro = p.lock().await;
                    let actual_progress = pro.actual_progress();
                    pro.set_actual_progress(actual_progress + 1);
                    pro.remove_unit(url).await;
                }
                drop(_permit);
                res
            }
        })
    } ).collect();
    
    let results = join_all(tasks).await;
    for result in results {
        match result {
            Ok(Ok(())) => {} // La tarea se completó correctamente
            Ok(Err(e)) => eprintln!("Error descargando el archivo: {:?}", e),
            Err(e) => eprintln!("Error en la tarea: {:?}", e),
        }
    }
    Ok(())
}

// TODO: TRACK ALL THE FILES DOWNLOADS
pub async fn download_files_secuentialy(
    _files: HashMap<String, String>,
    _reqwest_client: Option<&Client>,
    _progress: Option<Arc<Mutex<(usize, usize)>>>
) {
    unimplemented!("[File dowloader secuentialy] Not implemented ")
}

// TODO: Builter pattern
// TODO: track file download progress
pub async fn download_file(url: &str, dest: &Path, client: Option<&Client>, progress: Option<Arc<Mutex<FileProgress>>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if dest.exists() {
        println!("El archivo ya existe: {:?}", dest);
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        println!("Create parents directories: {:?}", parent);
        fs::create_dir_all(parent)?;
    }
    
    // Donwload file
    let client = match client {Some(c) => {c}, _ => {&Client::new()}};
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            // remove to vector
            return Err(Box::new(e))
        }
    };
    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();

    let mut file = AsyncFile::create(dest).await?;
    let mut total_writen = 0;
    
    if let Some(progress) = progress.as_ref() {
        progress.lock().await.set_progress((0, total_size as usize));
    }
    
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        total_writen += chunk.len();
        
        if let Some(progress) = progress.as_ref() {
            progress.lock().await.set_actual_progress(total_writen);
        }
        
        let progress_percent = (total_writen as f64 / total_size as f64) * 100.0;
        println!("{} - {:.2}%", url, progress_percent);
    }
    file.flush().await?;
    Ok(())
}

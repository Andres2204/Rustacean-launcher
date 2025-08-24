use crate::tasks::tasks::{ConcurrentTask, Task};
use crate::versions::verifier::VersionVerifier;
use futures_util::StreamExt;
use futures_util::future::join_all;
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Semaphore};
use tokio::{fs::File as AsyncFile, task};

// +============================+
//       DonwloaderTracking
// +============================+

#[derive(Debug)]
pub struct DownloaderTracking {
    state: DownloadState,
    progress: (usize, usize),
    units: Vec<Arc<RwLock<FileProgress>>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum DownloadState {
    #[default]
    NotDownloading,
    DownloadingInitials,
    Downloading,
    Finished,
}

impl DownloaderTracking {
    pub fn new(progress: (usize, usize)) -> Self {
        Self {
            state: Default::default(),
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

    pub fn state(&self) -> DownloadState {
        self.state
    }

    pub fn set_state(&mut self, state: DownloadState) {
        self.state = state;
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

    pub fn clean(&mut self) {
        self.state = DownloadState::NotDownloading;
        self.progress = (0, 0);
        self.units.clear();
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
        Self { path, url, sha1 }
    }
}

// +============================+
//           Downloader
// +============================+

pub struct Downloader {
    client: Client,
    concurrent_downloads: usize,
    retries: u16,
    progress: Option<Arc<Mutex<DownloaderTracking>>>,
}

unsafe impl Send for Downloader {}

impl Downloader {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub async fn download_files_concurrently(&self, files: Vec<FileData>) -> io::Result<()> {
        let tasks = files
            .iter()
            .map(|f| DownloadTask {
                client: self.client.clone(),
                file: f.clone(),
                file_progress: None,
                global_progess: self.progress.clone(),
            })
            .collect();
        ConcurrentTask::new(tasks, self.concurrent_downloads)
            .execute()
            .await
            .expect("Error downloading");

        /*
        log::info!("Downloading files");
        let semaphore = Arc::new(Semaphore::new(self.concurrent_downloads));

        let tasks: Vec<_> = files
            .into_iter()
            .map(|v| {
                let client = self.client.clone();
                let permit = semaphore.clone().acquire_owned();
                let value = v.clone();
                let progress = self.progress.clone();
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

                    let res = Self::download_file(value, client, file_progress.clone()).await;

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
        if let Some(p) = self.progress.clone() {
            let mut p = p.lock().await;
            p.state = DownloadState::Finished;
        }
        */
        Ok(())
    }

    pub async fn download_file(
        file: &FileData,
        client: Client,
        progress: Option<Arc<RwLock<FileProgress>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> 
    {
        if Self::verify_file(&file) {
            if let Some(p) = progress {
                p.write().unwrap().set_progress((1,1));
            }
            return Ok(());
        }
        
        log::debug!("Starting download of {}", &file.path);
        let url = &file.url;
        let dest = Path::new(&file.path);
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

    fn verify_file(file: &FileData) -> bool {
        let dest = Path::new(&file.path);
        if dest.exists() {
            return match (VersionVerifier::get_sha1(dest), &file.sha1) {
                (Ok(local_sha1), Some(expected_sha1)) => {
                    if &local_sha1 == expected_sha1 {
                        log::info!(
                            "File already exists and passed checksum: {}",
                            dest.display()
                        );
                        true
                    } else {
                        log::warn!(
                            "Checksum mismatch for {}. Expected: {}, Found: {}. Redownloading.",
                            dest.display(),
                            expected_sha1,
                            local_sha1
                        );
                        false
                    }
                }
                (Err(e), Some(_)) => {
                    log::warn!(
                        "Failed to compute checksum for {}: {}. Redownloading.",
                        dest.display(),
                        e
                    );
                    false
                }
                (_, None) => {
                    log::info!(
                        "File exists but no checksum to verify: {}. Proceeding to redownload.",
                        dest.display()
                    );
                    false
                }
            };
        }

        if let Some(parent) = dest.parent() {
            log::debug!("Create parents directories: {:?}", parent);
            fs::create_dir_all(parent).unwrap();
        }
        false
    }

    pub async fn clean_progress(&mut self) {
        if let Some(progress) = &self.progress {
            let mut lock = progress.lock().await;
            lock.clean();
        }
    }
}

// +============================+
//     Builder for Downloader
// +============================+
pub struct Builder {
    concurret_downloads: usize,
    connect_timeout: Duration,
    timeout: Duration,
    retries: u16,
    progress: Option<Arc<Mutex<DownloaderTracking>>>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            concurret_downloads: 32,
            connect_timeout: Duration::from_secs(60),
            timeout: Duration::from_secs(180),
            retries: 5,
            progress: None,
        }
    }
}

impl Builder {
    pub fn connect_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn concurret_downloads(&mut self, c: usize) -> &mut Self {
        self.concurret_downloads = c;
        self
    }

    pub fn retries(&mut self, r: u16) -> &mut Self {
        self.retries = r;
        self
    }

    pub fn progress(&mut self, p: Arc<Mutex<DownloaderTracking>>) -> &mut Self {
        self.progress = Some(p);
        self
    }

    fn build_client(&self) -> io::Result<Client> {
        Ok(Client::builder()
            .connect_timeout(self.connect_timeout)
            .timeout(self.timeout)
            .build()
            .expect("Failed to build reqwest client"))
    }

    pub fn build_with_client(&self, client: Client) -> io::Result<Downloader> {
        Ok(Downloader {
            client,
            concurrent_downloads: self.concurret_downloads,
            retries: self.retries,
            progress: self.progress.clone(),
        })
    }
    pub fn build(&self) -> io::Result<Downloader> {
        let client = self.build_client()?;
        self.build_with_client(client)
    }
}

// +============================+
//          DownloadTask         
// +============================+

#[derive(Debug)]
struct DownloadTask {
    client: Client,
    file: FileData,
    file_progress: Option<Arc<RwLock<FileProgress>>>,
    global_progess: Option<Arc<Mutex<DownloaderTracking>>>,
}

impl Task for DownloadTask {
    async fn execute(&mut self) -> Result<(), String> {
        if let Some(progress) = self.global_progess.as_ref() {
            if self.file_progress.is_none() {
                self.file_progress = Some(Arc::new(RwLock::new(FileProgress::new(self.file.url.clone()))))
            }
            let fp = self.file_progress.clone().unwrap();
            progress.lock().await.add_unit(fp);
        }
        match Downloader::download_file(
            &self.file,
            self.client.clone(),
            self.file_progress.clone(),
        ).await {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Error while downloading file: {}", e).to_owned()),
        }
    }
}

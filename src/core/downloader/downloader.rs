use std::fs::File;
use std::{fs, io, io::copy};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path};
use std::sync::Arc;
use futures_util::future::join_all;
use reqwest::Client;
use tokio::sync::{Mutex, Semaphore};
use tokio::task;

pub async fn download_files_concurrently(
    files: HashMap<String, String>, 
    reqwest_client: Option<&Client>,
    progress: Option<Arc<Mutex<(usize, usize)>>>
) -> io::Result<()> {
    let client = match reqwest_client {
        Some(c) => {c}
        None => {&Client::new()}
    };

    let max_concurrent_tasks = 64usize;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    let tasks: Vec<_> = files.into_iter().map(|v| {
        let client = client.clone();
        let permit = semaphore.clone().acquire_owned();
        let value = v.clone();
        let progress = progress.clone();
        task::spawn({
            async move {
                let _permit = permit.await;
                let res = download_file(&*value.0, Path::new::<>(&value.1), Some(&client)).await;
                if let Some(p) = progress {
                    p.lock().await.0 += 1;
                }
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

pub async fn download_file(url: &str, dest: &Path, client: Option<&Client>) -> io::Result<()> {
    //sleep(Duration::from_millis(200));
    if dest.exists() {
        println!("El archivo ya existe: {:?}", dest);
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        println!("Create parents directories: {:?}", parent);
        fs::create_dir_all(parent)?;
    }

    let response = match client.as_ref() {
        Some(client) => {
            client
                .get(url)
                .send()
                .await
                .expect(format!("[with client] Cant download the file {url}").as_str())
        }
        None => {
            Client::new()
                .get(url)
                .send()
                .await
                .expect(format!("[No client] Cant download the file {url}").as_str())
        }
    };

    let bytes = response.bytes()
        .await
        .expect("Cant convert to bytes");   // Obtiene los datos como `Bytes`
    let mut content = Cursor::new(bytes);  // Usa `Cursor` para implementar `Read`

    let mut file = File::create(dest)?;
    copy(&mut content, &mut file)?;  // Ahora `content` implementa `Read`

    Ok(())
}

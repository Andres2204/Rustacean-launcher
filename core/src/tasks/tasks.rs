use std::pin::Pin;
use std::sync::Arc;
use futures_util::StreamExt;
use tokio::sync::Semaphore;

pub trait Task: Sync + Send {
    fn execute(&mut self) -> impl std::future::Future<Output = Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>> + std::marker::Send;
    fn is_abortable(&self) -> bool { false } 
    fn abort(&mut self) -> bool { false }
}

pub struct SequentialTask<T: Task> {
    tasks:  Vec<T>,
}
impl<T: Task> SequentialTask<T> {
    pub fn new(tasks: Vec<T>) -> Self {
        Self { tasks }
    }

    pub fn default() -> Self {
        Self::new(Vec::new())
    }

    pub fn add_task(&mut self, task: T) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub async fn execute(&mut self) -> Result<(), String> {
        for task in &mut self.tasks {
            task.execute().await;
        }
        Ok(())
    }
}

pub struct ConcurrentTask<T: Task> {
    tasks: Vec<T>,
    max_concurrent_tasks: usize,
}

impl<T: Task + 'static + std::fmt::Debug > ConcurrentTask<T> {
    pub fn new(tasks: Vec<T>, max_concurrent_tasks: usize) -> Self {
        Self { tasks, max_concurrent_tasks }
    }

    pub fn default() -> Self {
        Self::new(Vec::new(), 32)
    }

    pub fn add_task(&mut self, task: T) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub async fn execute(&mut self) -> Result<(), String> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));
        let mut set = tokio::task::JoinSet::new();

        log::info!("Tareas recibidas {:?}", self.tasks.len());
        for mut task in self.tasks.drain(..) {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            set.spawn(async move {
                let _permit = permit;
                log::info!("Executing {:?}", task);
                task.execute().await;
            });
        }

        while let Some(result) = set.join_next().await {
            match result {
                Ok(n) => log::debug!("task result: {:?}", n),
                Err(err) => log::error!("task result: {}", err),
            }
        }
        log::debug!("tasks finished");
        Ok(())
    }
}


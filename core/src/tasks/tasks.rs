use std::sync::Arc;
use tokio::sync::Semaphore;

pub trait Task<T>: Sync + Send {
    fn execute(&mut self) -> impl Future<Output = TaskResult<T>> + Send;
    fn is_abortable(&self) -> bool { false } 
    fn abort(&mut self) -> bool { false }
}

pub enum TaskResult<R> {
    SUCCESS(R),
    FAILURE(String),
    ABORTED,
}

impl<R> From<TaskResult<R>> for Result<R, String> {
    fn from(value: TaskResult<R>) -> Self {
        match value {
            TaskResult::SUCCESS(v) => {Ok(v)}
            TaskResult::FAILURE(e) => {Err(e)}
            TaskResult::ABORTED => {Err("Aborted".into())}
        }
    }
}

impl<T, E: ToString> From<Result<T, E>> for TaskResult<T> {
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(v) => TaskResult::SUCCESS(v),
            Err(e) => TaskResult::FAILURE(e.to_string()),
        }
    }
}

pub struct SequentialTask<S: Task<()>> {
    tasks:  Vec<S>,
}
impl<S: Task<()>> SequentialTask<S> {
    pub fn new(tasks: Vec<S>) -> Self {
        Self { tasks }
    }

    pub fn default() -> Self {
        Self::new(Vec::new())
    }

    pub fn add_task(&mut self, task: S) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub async fn run(&mut self) -> TaskResult<()> {
        for task in &mut self.tasks {
            task.execute().await;
        }
        TaskResult::SUCCESS(())
    }
}

pub struct ConcurrentTask<C: Task<()>> {
    tasks: Vec<C>,
    max_concurrent_tasks: usize,
}

impl<C: Task<()> + 'static> ConcurrentTask<C> {
    pub fn new(tasks: Vec<C>, max_concurrent_tasks: usize) -> Self {
        Self { tasks, max_concurrent_tasks }
    }

    pub fn default() -> Self {
        Self::new(Vec::new(), 32)
    }

    pub fn add_task(&mut self, task: C) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub async fn run(&mut self) -> TaskResult<()> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));
        let mut set = tokio::task::JoinSet::new();

        log::debug!("Tareas recibidas {:?}", self.tasks.len());
        for mut task in self.tasks.drain(..) {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            set.spawn(async move {
                let _permit = permit;
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
        TaskResult::SUCCESS(())
    }
}


use std::sync::Arc;

use anyhow::{Result, bail};
use tokio::sync::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Debug, Clone)]
pub struct TaskManager {
    cancellation_token: CancellationToken,
    failure: Arc<Mutex<bool>>,
    tasks: TaskTracker,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            failure: Arc::new(Mutex::new(false)),
            tasks: TaskTracker::new(),
        }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let failure = self.failure.clone();
        self.tasks.spawn(async move {
            if let Err(error) = task.await {
                let mut flag = failure.lock().await;
                *flag = true;
                eprintln!("Task failed!");
                for error in error.chain() {
                    eprintln!("Caused by:");
                    eprintln!("  - {error}")
                }
            }
        });
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub async fn wait_all(self) -> Result<()> {
        self.tasks.close();
        self.tasks.wait().await;
        if *self.failure.lock().await {
            bail!("At least one task failed, please consult the logs")
        }
        Ok(())
    }
}

use std::path::PathBuf;

use tokio::{
    fs,
    io::{self, AsyncWriteExt},
    select,
};

use crate::taks::TaskManager;

use super::{StorageCall, StorageInbox};

pub fn start(
    task_manager: &TaskManager,
    mut inbox: StorageInbox,
    dir_path: PathBuf,
) {
    let cancellation_token = task_manager.cancellation_token();

    task_manager.spawn(async move {
        let dir_path = &dir_path;
        fs::create_dir_all(dir_path).await?;

        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };

            match message {
                StorageCall::Get(call) => {
                    call.handle(|input| async move {
                        let path = dir_path.join(format!("{}.json", input.key));
                        let value = match fs::read_to_string(&path).await {
                            Ok(contents) => {
                                Some(serde_json::from_str(&contents)?)
                            },
                            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                                None
                            },
                            Err(e) => Err(e)?,
                        };
                        Ok(value)
                    })
                    .await;
                },

                StorageCall::Put(call) => {
                    call.handle(|input| async move {
                        let path = dir_path.join(format!("{}.json", input.key));
                        let contents = serde_json::to_vec(&input.value)?;
                        let (mut file, new) = loop {
                            match fs::OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(&path)
                                .await
                            {
                                Ok(file) => break (file, true),
                                Err(e)
                                    if e.kind()
                                        != io::ErrorKind::AlreadyExists =>
                                {
                                    Err(e)?
                                },
                                _ => (),
                            }
                            match fs::OpenOptions::new()
                                .write(true)
                                .create(false)
                                .open(&path)
                                .await
                            {
                                Ok(file) => break (file, true),
                                Err(e)
                                    if e.kind() != io::ErrorKind::NotFound =>
                                {
                                    Err(e)?
                                },
                                _ => (),
                            }
                        };
                        file.write(&contents).await?;
                        Ok(new)
                    })
                    .await;
                },
            }
        }
    });
}

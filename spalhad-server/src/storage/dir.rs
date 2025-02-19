use std::path::PathBuf;

use tokio::{
    fs,
    io::{self, AsyncWriteExt},
    select,
    sync::mpsc,
};

use crate::taks::TaskManager;

use super::StorageMessage;

pub fn start(
    task_manager: &TaskManager,
    mut receiver: mpsc::Receiver<StorageMessage>,
    dir_path: PathBuf,
) {
    let cancellation_token = task_manager.cancellation_token();

    task_manager.spawn(async move {
        fs::create_dir_all(&dir_path).await?;
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = receiver.recv() => message,
            };
            let Some(message) = result else { break Ok(()) };
            match message {
                StorageMessage::Get(key, callback) => {
                    let path = dir_path.join(format!("{key}.json"));
                    let value = match fs::read_to_string(&path).await {
                        Ok(contents) => Some(serde_json::from_str(&contents)?),
                        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
                        Err(e) => Err(e)?,
                    };
                    if callback.send(value).is_err() {
                        break Ok(());
                    }
                },
                StorageMessage::Put(key, value, callback) => {
                    let path = dir_path.join(format!("{key}.json"));
                    let contents = serde_json::to_vec(&value)?;
                    let (mut file, new) = loop {
                        match fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&path)
                            .await
                        {
                            Ok(file) => break (file, true),
                            Err(e)
                                if e.kind() != io::ErrorKind::AlreadyExists =>
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
                            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                                Err(e)?
                            },
                            _ => (),
                        }
                    };
                    file.write(&contents).await?;
                    if callback.send(new).is_err() {
                        break Ok(());
                    }
                },
            }
        }
    });
}

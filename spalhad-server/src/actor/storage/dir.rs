use std::path::PathBuf;

use anyhow::Result;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

use crate::actor::core::ReactiveActor;

use super::StorageCall;

#[derive(Debug, Clone)]
pub struct DirStorage {
    dir_path: PathBuf,
}

impl DirStorage {
    pub fn open(dir_path: impl Into<PathBuf>) -> Self {
        Self { dir_path: dir_path.into() }
    }
}

impl ReactiveActor for DirStorage {
    type ReactiveCall = StorageCall;

    async fn on_call(&mut self, call: Self::ReactiveCall) -> Result<()> {
        let dir_path = &self.dir_path;

        match call {
            StorageCall::Get(call) => {
                call.handle(|input| async move {
                    let path = dir_path.join(format!("{}.json", input.key));
                    let value = match fs::read_to_string(&path).await {
                        Ok(contents) => Some(serde_json::from_str(&contents)?),
                        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
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
                            Ok(file) => break (file, false),
                            Err(e) if e.kind() != io::ErrorKind::NotFound => {
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

        Ok(())
    }
}

use anyhow::Result;
use spalhad_spec::cluster::RunId;

use crate::actor::storage::StorageHandle;

#[derive(Debug, Clone)]
pub struct App {
    storage: StorageHandle,
    run_id: RunId,
}

impl App {
    pub fn new(storage: StorageHandle) -> Result<Self> {
        Ok(Self { storage, run_id: RunId::generate()? })
    }

    pub fn storage(&self) -> &StorageHandle {
        &self.storage
    }

    pub fn self_run_id(&self) -> RunId {
        self.run_id
    }
}

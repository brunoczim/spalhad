use anyhow::Result;
use spalhad_actor::ActorOptions;
use spalhad_spec::cluster::RunId;

use crate::actor::{
    bouncer::{Bouncer, BouncerHandle},
    coordinator::CoordinatorHandle,
    storage::StorageHandle,
};

#[derive(Debug, Clone)]
pub struct App {
    bouncer: BouncerHandle,
    run_id: RunId,
}

impl App {
    pub fn new(
        storage_options: &ActorOptions<'_>,
        storage: StorageHandle,
        coordinator: CoordinatorHandle,
    ) -> Result<Self> {
        let run_id = RunId::generate()?;
        let bouncer_actor = Bouncer::open(run_id, storage, coordinator);
        let bouncer = storage_options.spawn(bouncer_actor);
        Ok(Self { bouncer, run_id })
    }

    pub fn bouncer(&self) -> &BouncerHandle {
        &self.bouncer
    }

    pub fn self_run_id(&self) -> RunId {
        self.run_id
    }
}

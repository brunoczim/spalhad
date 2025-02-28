use anyhow::Result;
use spalhad_actor::{Actor, ActorOptions};
use spalhad_spec::cluster::RunId;

use crate::actor::{
    bouncer::{Bouncer, BouncerHandle},
    storage::StorageCall,
};

#[derive(Debug, Clone)]
pub struct App {
    bouncer: BouncerHandle,
    run_id: RunId,
}

impl App {
    pub fn new<A>(
        storage_options: &ActorOptions<'_>,
        storage_actor: A,
    ) -> Result<Self>
    where
        A: Actor<Call = StorageCall> + 'static,
    {
        let run_id = RunId::generate()?;
        let bouncer_actor =
            Bouncer::open(run_id, storage_options, storage_actor);
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

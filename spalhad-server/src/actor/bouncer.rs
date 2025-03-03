use anyhow::Result;
use spalhad_actor::{ActorCall, ActorHandle, CallSuperset, TrivialLoopActor};
use spalhad_spec::cluster::RunId;
use thiserror::Error;

use super::{
    coordinator::{self, CoordinatorCall, CoordinatorHandle},
    storage::{self, StorageCall, StorageHandle},
};

#[derive(Debug)]
pub struct Bouncer {
    active: bool,
    run_id: RunId,
    storage: StorageHandle,
    coordinator: CoordinatorHandle,
}

impl Bouncer {
    pub fn open(
        run_id: RunId,
        storage: StorageHandle,
        coordinator: CoordinatorHandle,
    ) -> Self {
        Self { active: false, run_id, storage, coordinator }
    }
}

impl TrivialLoopActor for Bouncer {
    type Call = BouncerCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            BouncerCall::Activate(call) if self.active => {
                call.back.reply_error(Error::AlreadyActive);
            },
            BouncerCall::Activate(call) if self.run_id == call.input.run_id => {
                self.active = true;
                call.back.reply_ok(Activated);
            },
            BouncerCall::Activate(call) => {
                call.back.reply_error(Error::BadRunId);
            },
            BouncerCall::IsActive(call) => {
                call.back.reply_ok(self.active);
            },
            BouncerCall::Storage(call) if self.active => {
                self.storage.forward(call).await?;
            },
            BouncerCall::Coordinator(call) => {
                self.coordinator.forward(call).await?;
            },
            BouncerCall::Storage(call) => {
                call.reply_error(Error::NotActive);
            },
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("bouncer is already active")]
    AlreadyActive,
    #[error("attempted to activate bouncer with incorrect run id")]
    BadRunId,
    #[error("bouncer is not active yet")]
    NotActive,
}

pub type BouncerHandle = ActorHandle<BouncerCall>;

#[derive(Debug, CallSuperset)]
pub enum BouncerCall {
    Activate(ActivateCall),
    IsActive(IsActiveCall),
    #[spalhad(flatten { storage::GetCall, storage::PutCall })]
    Storage(StorageCall),
    #[spalhad(flatten { coordinator::GetCall, coordinator::PutCall })]
    Coordinator(CoordinatorCall),
}

impl From<StorageCall> for BouncerCall {
    fn from(call: StorageCall) -> Self {
        Self::Storage(call)
    }
}

impl From<CoordinatorCall> for BouncerCall {
    fn from(call: CoordinatorCall) -> Self {
        Self::Coordinator(call)
    }
}

#[derive(Debug, Clone)]
pub struct Activate {
    pub run_id: RunId,
}

#[derive(Debug)]
pub struct Activated;

pub type ActivateCall = ActorCall<Activate, Activated>;

#[derive(Debug, Clone)]
pub struct IsActive;

pub type IsActiveOutput = bool;

pub type IsActiveCall = ActorCall<IsActive, IsActiveOutput>;

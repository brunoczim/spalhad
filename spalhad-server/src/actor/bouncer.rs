use anyhow::Result;
use spalhad_actor::{
    Actor,
    ActorCall,
    ActorHandle,
    ActorOptions,
    CallSuperSet,
    TrivialLoopActor,
};
use spalhad_spec::cluster::RunId;
use thiserror::Error;

use super::storage::{StorageCall, StorageHandle};

#[derive(Debug)]
pub struct Bouncer {
    active: bool,
    run_id: RunId,
    storage: StorageHandle,
}

impl Bouncer {
    pub fn open<A>(
        run_id: RunId,
        storage_options: &ActorOptions<'_>,
        storage_actor: A,
    ) -> Self
    where
        A: Actor<Call = StorageCall> + 'static,
    {
        let storage = storage_options.spawn(storage_actor);
        Self { active: false, run_id, storage }
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

#[derive(Debug, CallSuperSet)]
pub enum BouncerCall {
    Activate(ActivateCall),
    IsActive(IsActiveCall),
    Storage(StorageCall),
}

impl From<ActivateCall> for BouncerCall {
    fn from(call: ActivateCall) -> Self {
        Self::Activate(call)
    }
}

impl From<IsActiveCall> for BouncerCall {
    fn from(call: IsActiveCall) -> Self {
        Self::IsActive(call)
    }
}

impl<C> From<C> for BouncerCall
where
    C: Into<StorageCall>,
{
    fn from(call: C) -> Self {
        Self::Storage(call.into())
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

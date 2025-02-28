use anyhow::Result;
use spalhad_spec::cluster::RunId;
use thiserror::Error;

use super::{
    core::{ActorCall, ActorHandle, CallSuperSet, TrivialLoopActor},
    storage::{StorageCall, StorageHandle},
};

#[derive(Debug)]
pub struct Bouncer {
    active: bool,
    run_id: RunId,
    storage: StorageHandle,
}

impl Bouncer {
    pub fn open(run_id: RunId, storage: StorageHandle) -> Self {
        Self { active: false, run_id, storage }
    }
}

impl TrivialLoopActor for Bouncer {
    type Call = BouncerCall;

    async fn on_call(&mut self, call: Self::Call) -> Result<()> {
        match call {
            BouncerCall::Activate(call) if self.active => {
                call.back.reply_error(AlreadyActive);
            },
            BouncerCall::Activate(call) if self.run_id == call.input.run_id => {
                self.active = true;
                call.back.reply_ok(ActivateOutput);
            },
            BouncerCall::Activate(call) => {
                call.back.reply_error(BadRunId);
            },
            BouncerCall::IsActive(call) => {
                call.back.reply_ok(self.active);
            },
            BouncerCall::Storage(call) if self.active => {
                self.storage.forward(call).await?;
            },
            BouncerCall::Storage(call) => {
                call.reply_error(NotActive);
            },
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("bouncer is already active")]
pub struct AlreadyActive;

#[derive(Debug, Error)]
#[error("attempted to activate bouncer with incorrect run id")]
pub struct BadRunId;

#[derive(Debug, Error)]
#[error("bouncer is not active yet")]
pub struct NotActive;

#[derive(Debug, Clone)]
pub struct Activate {
    pub run_id: RunId,
}

#[derive(Debug)]
pub struct ActivateOutput;

pub type ActivateCall = ActorCall<Activate, ActivateOutput>;

#[derive(Debug, Clone)]
pub struct IsActive;

pub type IsActiveOutput = bool;

pub type IsActiveCall = ActorCall<IsActive, IsActiveOutput>;

#[derive(Debug)]
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

impl CallSuperSet for BouncerCall {
    fn reply_error<E>(self, error: E) -> bool
    where
        E: Into<anyhow::Error>,
    {
        match self {
            Self::Activate(call) => call.back.reply_error(error),
            Self::IsActive(call) => call.back.reply_error(error),
            Self::Storage(call) => call.reply_error(error),
        }
    }
}

pub type BouncerHandle = ActorHandle<BouncerCall>;

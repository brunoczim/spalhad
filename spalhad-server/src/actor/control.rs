use spalhad_spec::cluster::RunId;

use crate::{actor, taks::TaskManager};

use super::{
    core::{ActorCall, ActorHandle},
    storage::{StorageCall, StorageHandle},
};

#[derive(Debug, Clone)]
pub struct Activate {
    pub run_id: RunId,
}

pub type ActivateOutput = ();

pub type ActivateCall = ActorCall<Activate, ActivateOutput>;

#[derive(Debug, Clone)]
pub struct IsActive;

pub type IsActiveOutput = bool;

pub type IsActiveCall = ActorCall<IsActive, IsActiveOutput>;

#[derive(Debug)]
pub enum ControlMessage {
    Activate(ActivateCall),
    IsActive(IsActiveCall),
    Storage(StorageCall),
}

pub type ControlHandle = ActorHandle<ControlMessage>;

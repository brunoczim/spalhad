use anyhow::Result;
use spalhad_spec::cluster::RunId;

use crate::mux::Multiplexer;

#[derive(Debug, Clone)]
pub struct App {
    mux: Multiplexer,
    run_id: RunId,
}

impl App {
    pub fn new(mux: Multiplexer) -> Result<Self> {
        Ok(Self { mux, run_id: RunId::generate()? })
    }

    pub fn mux(&self) -> &Multiplexer {
        &self.mux
    }

    pub fn self_run_id(&self) -> RunId {
        self.run_id
    }
}

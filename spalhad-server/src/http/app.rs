use crate::mux::Multiplexer;

#[derive(Debug, Clone)]
pub struct App {
    mux: Multiplexer,
}

impl App {
    pub fn new(mux: Multiplexer) -> Self {
        Self { mux }
    }

    pub fn mux(&self) -> &Multiplexer {
        &self.mux
    }
}

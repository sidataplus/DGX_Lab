//! Simulation bridge used by the Leptos UI.
//!
//! Runs the pure [`sim_session::SimSession`] inside the UI WASM module so the
//! protocol and snapshot path are exercised end-to-end.

use dgxlab_contracts::{SimRequest, SimResponse, TerminalLine, UiWorldView};
use sim_session::SimSession;

#[derive(Clone, Debug)]
pub struct SimBridge {
    session: SimSession,
}

impl SimBridge {
    pub fn from_session(session: SimSession) -> Self {
        Self { session }
    }

    pub fn handle(&mut self, request: SimRequest) -> SimResponse {
        self.session.handle(request)
    }

    #[must_use]
    pub fn view(&self) -> UiWorldView {
        self.session.view()
    }

    pub fn export_json(&self) -> Result<String, String> {
        self.session.export_json().map_err(|error| error.to_string())
    }

    #[must_use]
    pub fn critical_practical_passed(&self) -> bool {
        self.session.critical_practical_passed()
    }
}

#[derive(Clone, Debug, Default)]
pub struct TerminalBuffer {
    pub lines: Vec<TerminalLine>,
}

#![forbid(unsafe_code)]

//! WASM adapter around the pure Rust simulation session.
//! The adapter exposes typed simulator operations only; it has no browser network,
//! process, filesystem, SSH, or real scheduler capability.

use dgxlab_contracts::SimRequest;
use serde::Serialize;
use sim_session::SimSession;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SimWorkerRuntime {
    session: SimSession,
}

#[wasm_bindgen]
impl SimWorkerRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new(scenario_id: &str, seed: u64) -> Result<SimWorkerRuntime, JsValue> {
        console_error_panic_hook::set_once();
        let session = SimSession::new(scenario_id, seed)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { session })
    }

    /// Handle a JSON-encoded [`SimRequest`] and return a JSON-encoded [`SimResponse`].
    pub fn handle_json(&mut self, request_json: &str) -> Result<String, JsValue> {
        let request: SimRequest = serde_json::from_str(request_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let response = self.session.handle(request);
        serde_json::to_string(&response).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn execute(&mut self, command: &str) -> Result<JsValue, JsValue> {
        let response = self.session.handle(SimRequest::ExecuteCommand {
            command: command.into(),
        });
        to_js(&response)
    }

    pub fn advance_by(&mut self, delta_ms: u64) -> Result<JsValue, JsValue> {
        let response = self
            .session
            .handle(SimRequest::AdvanceClock { delta_ms });
        to_js(&response)
    }

    pub fn reset(&mut self, scenario_id: &str, seed: u64) -> Result<JsValue, JsValue> {
        let response = self.session.handle(SimRequest::Reset {
            scenario_id: scenario_id.into(),
            seed,
        });
        to_js(&response)
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        to_js(&WorkerSnapshotView {
            seq: self.session.seq(),
            state: self.session.view(),
        })
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&WorkerSnapshotView {
            seq: self.session.seq(),
            state: self.session.view(),
        })
        .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn state_digest(&self) -> String {
        self.session.state_digest()
    }
}

#[derive(Serialize)]
struct WorkerSnapshotView {
    seq: u64,
    state: dgxlab_contracts::UiWorldView,
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}

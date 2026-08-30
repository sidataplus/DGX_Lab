//! Browser-local session persistence.
//!
//! M1 stores a JSON snapshot of [`sim_session::SimSession`] plus the terminal buffer.
//! The snapshot is small and uses `localStorage` for synchronous save and restore.

use serde::{Deserialize, Serialize};
use web_sys::window;

const STORAGE_KEY: &str = "dgxlab.session.v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedUiState {
    pub session_json: String,
    pub terminal_lines: Vec<dgxlab_contracts::TerminalLine>,
    pub saved_at_ms: u64,
}

pub fn save_local(state: &PersistedUiState) {
    if let Ok(json) = serde_json::to_string(state)
        && let Some(storage) = local_storage()
    {
        let _ = storage.set_item(STORAGE_KEY, &json);
    }
}

pub fn load_local() -> Option<PersistedUiState> {
    let storage = local_storage()?;
    let json = storage.get_item(STORAGE_KEY).ok().flatten()?;
    serde_json::from_str(&json).ok()
}

pub fn clear_local() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(STORAGE_KEY);
    }
}

fn local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok().flatten()
}

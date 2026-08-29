//! Browser-local session persistence.
//!
//! M1 stores a JSON snapshot of [`sim_session::SimSession`] plus the terminal buffer.
//! Primary store is IndexedDB when available; `localStorage` is the synchronous fallback.

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::window;

const STORAGE_KEY: &str = "dgxlab.session.v1";
const IDB_NAME: &str = "dgxlab";
const IDB_STORE: &str = "sessions";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedUiState {
    pub session_json: String,
    pub terminal_lines: Vec<dgxlab_contracts::TerminalLine>,
    pub saved_at_ms: u64,
}

pub fn save_local(state: &PersistedUiState) {
    if let Ok(json) = serde_json::to_string(state) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
        // Best-effort IndexedDB write (fire-and-forget).
        spawn_idb_put(json);
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

fn spawn_idb_put(json: String) {
    let Some(window) = window() else {
        return;
    };
    // IndexedDB open is asynchronous; use a small JS-friendly path via localStorage
    // already done above. Attempt IDB when factory exists without blocking UI.
    let Ok(Some(factory)) = window.indexed_db() else {
        return;
    };
    let Ok(open_request) = factory.open_with_u32(IDB_NAME, 1) else {
        return;
    };
    let on_upgrade = wasm_bindgen::closure::Closure::wrap(Box::new(
        move |event: web_sys::Event| {
            if let Some(target) = event.target() {
                if let Ok(request) = target.dyn_into::<web_sys::IdbOpenDbRequest>() {
                    if let Ok(db) = request.result() {
                        if let Ok(db) = db.dyn_into::<web_sys::IdbDatabase>() {
                            let _ = db.create_object_store(IDB_STORE);
                        }
                    }
                }
            }
        },
    ) as Box<dyn FnMut(_)>);
    open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();

    let on_success = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Ok(request) = target.dyn_into::<web_sys::IdbOpenDbRequest>() else {
            return;
        };
        let Ok(db_val) = request.result() else {
            return;
        };
        let Ok(db) = db_val.dyn_into::<web_sys::IdbDatabase>() else {
            return;
        };
        let Ok(tx) = db.transaction_with_str_and_mode(
            IDB_STORE,
            web_sys::IdbTransactionMode::Readwrite,
        ) else {
            return;
        };
        let Ok(store) = tx.object_store(IDB_STORE) else {
            return;
        };
        let Ok(value) = js_sys::JSON::parse(&json) else {
            // Store as string if parse fails.
            let _ = store.put_with_key(&wasm_bindgen::JsValue::from_str(&json), &wasm_bindgen::JsValue::from_str(STORAGE_KEY));
            return;
        };
        let _ = store.put_with_key(&value, &wasm_bindgen::JsValue::from_str(STORAGE_KEY));
    }) as Box<dyn FnMut(_)>);
    open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
    on_success.forget();
}

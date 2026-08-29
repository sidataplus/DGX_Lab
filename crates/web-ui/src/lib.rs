#![forbid(unsafe_code)]

//! Leptos CSR shell. Authoritative simulation state comes only from [`sim_session::SimSession`]
//! (the same pure runtime used by the WASM worker adapter). The UI never invents job state.

mod app;
mod bridge;
mod persist;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    leptos::mount::mount_to_body(app::App);
}

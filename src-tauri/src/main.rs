#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        // Intentionally no shell, process, HTTP, SSH, scheduler, or unrestricted
        // filesystem commands/plugins. Browser-side simulation is the product.
        .run(tauri::generate_context!())
        .expect("failed to run DGX Lab desktop shell");
}

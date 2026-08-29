# Tauri shell

This crate packages the Leptos/WASM application. It intentionally exposes no custom native commands and installs no shell, process, HTTP, SSH, or filesystem plugins. Session import/export is implemented through browser file APIs in the first milestone; a future native dialog must receive a separate ADR and capability review.

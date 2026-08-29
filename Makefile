.PHONY: validate check test web tauri prototype clean

# Prefer real toolchain binaries if ~/.cargo/bin rustup proxies are broken:
#   export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

PYTHON ?= python3

validate:
	$(PYTHON) scripts/validate_all.py
	$(PYTHON) scripts/check_forbidden_apis.py

check: validate
	cargo fmt --all -- --check
	cargo clippy --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop --all-targets --all-features -- -D warnings
	cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop
	cargo build -p sim-worker-wasm --target wasm32-unknown-unknown

test:
	cargo test --workspace --exclude web-ui --exclude sim-worker-wasm --exclude dgx-lab-desktop

web:
	cd crates/web-ui && trunk serve --port 1420 --address 127.0.0.1

# Desktop app: Tauri shell + Leptos CSR UI (Trunk on port 1420).
tauri:
	cargo tauri dev

prototype:
	cd prototype && $(PYTHON) serve.py --port 1420

clean:
	rm -rf target dist .trunk crates/web-ui/dist

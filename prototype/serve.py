#!/usr/bin/env python3
"""Serve the self-contained prototype on loopback only."""
from __future__ import annotations

import argparse
import http.server
import os
import socketserver
import webbrowser
from pathlib import Path


class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "no-referrer")
        super().end_headers()


def main() -> None:
    parser = argparse.ArgumentParser(description="Serve the DGX Lab prototype on loopback.")
    parser.add_argument(
        "--port",
        type=int,
        default=int(os.environ.get("DGX_LAB_PROTOTYPE_PORT", "0")),
        help="Port to bind (0 = ephemeral). Env: DGX_LAB_PROTOTYPE_PORT.",
    )
    parser.add_argument(
        "--no-open",
        action="store_true",
        help="Do not open a browser tab (useful when embedded in Tauri).",
    )
    args = parser.parse_args()

    os.chdir(Path(__file__).resolve().parent)
    # Allow quick restarts (e.g. cargo tauri dev) after a previous bind.
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("127.0.0.1", args.port), Handler) as server:
        port = server.server_address[1]
        url = f"http://127.0.0.1:{port}/"
        print(f"DGX Lab prototype: {url}", flush=True)
        if not args.no_open:
            try:
                webbrowser.open(url)
            except Exception:
                pass
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            print("\nStopped.")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Fail when product source gains a path to real infrastructure."""
from __future__ import annotations
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOTS = [ROOT / "crates", ROOT / "src-tauri", ROOT / "prototype"]
TEXT_SUFFIXES = {".rs", ".toml", ".json", ".js", ".html", ".css", ".py"}
PATTERNS = {
    "Rust process spawning": re.compile(r"std::process::(?:Command|Child)|tokio::process"),
    "Tauri shell/process plugin": re.compile(r"tauri[_-]plugin[_-](?:shell|process)"),
    "SSH dependency/API": re.compile(r"(?:^|[^a-z])(ssh2|russh|openssh|paramiko)(?:[^a-z]|$)", re.I),
    "Slurm client invocation": re.compile(r"(?:Command::new|spawn\s*\()[^\n]*(?:sbatch|srun|squeue|scontrol|sacct)"),
    "HTTP/network dependency": re.compile(r"(?:reqwest|hyper::Client|ureq|tauri[_-]plugin[_-]http|TcpStream|UdpSocket|WebSocket)"),
    "Node child process": re.compile(r"child_process|Deno\.Command|Bun\.spawn"),
    "Browser network call": re.compile(r"\bfetch\s*\(|XMLHttpRequest|new\s+WebSocket|EventSource\s*\("),
}
ALLOWED_FALSE_POSITIVES = {
    # The Python prototype server is a static loopback-only convenience, not part of Tauri runtime.
    ("prototype/serve.py", "HTTP/network dependency"),
}

SKIP_DIR_NAMES = {
    "dist",
    "target",
    ".trunk",
    "node_modules",
    "pkg",
}


def iter_files():
    for source_root in SOURCE_ROOTS:
        for path in source_root.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
                continue
            if any(part in SKIP_DIR_NAMES for part in path.parts):
                continue
            yield path

def main() -> int:
    failures=[]
    for path in iter_files():
        rel=str(path.relative_to(ROOT))
        text=path.read_text(encoding="utf-8", errors="replace")
        for label, pattern in PATTERNS.items():
            if (rel,label) in ALLOWED_FALSE_POSITIVES:
                continue
            for match in pattern.finditer(text):
                line=text.count("\n",0,match.start())+1
                # Documentation comments may state forbidden words; executable-pattern regexes avoid most noise.
                failures.append(f"{rel}:{line}: {label}: {match.group(0)!r}")
    # Manifest-level dependency names are the most important simple gate.
    manifests="\n".join(p.read_text(encoding="utf-8") for p in ROOT.rglob("Cargo.toml"))
    for dependency in ("ssh2", "russh", "openssh", "reqwest", "tauri-plugin-shell", "tauri-plugin-http"):
        if re.search(rf"(?m)^\s*{re.escape(dependency)}\s*=", manifests):
            failures.append(f"Cargo manifests include forbidden dependency {dependency}")
    if failures:
        print("FORBIDDEN CAPABILITY CHECK FAILED", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    print("forbidden capability check passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

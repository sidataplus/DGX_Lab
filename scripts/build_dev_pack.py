#!/usr/bin/env python3
"""Build a deterministic DGX Lab source-and-documentation development ZIP."""
from __future__ import annotations

import argparse
import hashlib
import os
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_NAME = "DGX_Lab_Dev_Pack_v0.1.0.zip"
TOP_LEVEL = "DGX_Lab_Dev_Pack_v0.1.0"
EPOCH = (2026, 8, 5, 0, 0, 0)
EXCLUDED_NAMES = {".DS_Store"}
EXCLUDED_PARTS = {"__pycache__", "target", "node_modules", ".git"}


def included_files() -> list[Path]:
    result: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if path.name in EXCLUDED_NAMES or any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        if path.suffix == ".pyc":
            continue
        result.append(path)
    return sorted(result, key=lambda item: item.relative_to(ROOT).as_posix())


def write_zip(output: Path) -> str:
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(output.suffix + ".tmp")
    temporary.unlink(missing_ok=True)
    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in included_files():
            relative = path.relative_to(ROOT).as_posix()
            info = zipfile.ZipInfo(f"{TOP_LEVEL}/{relative}", EPOCH)
            mode = 0o755 if os.access(path, os.X_OK) else 0o644
            info.external_attr = mode << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, path.read_bytes())
    temporary.replace(output)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    sidecar = output.with_suffix(output.suffix + ".sha256")
    sidecar.write_text(f"{digest}  {output.name}\n", encoding="utf-8")
    print(f"wrote {output} ({output.stat().st_size} bytes)")
    print(f"sha256 {digest}")
    return digest


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", nargs="?", type=Path, default=ROOT.parent / DEFAULT_NAME)
    args = parser.parse_args()
    write_zip(args.output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

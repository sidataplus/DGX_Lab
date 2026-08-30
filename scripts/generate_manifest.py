#!/usr/bin/env python3
from __future__ import annotations
import hashlib
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXCLUDE = {"MANIFEST.sha256"}
COURSE_PACK = "dist/DGX_Lab_SLURM_Fundamentals_v1.0.0.dgxlabpack"


def release_paths() -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    paths = {path for path in result.stdout.split("\0") if path}
    if (ROOT / COURSE_PACK).is_file():
        paths.add(COURSE_PACK)
    return sorted(paths)


def main() -> None:
    lines = []
    for rel in release_paths():
        path = ROOT / rel
        if rel in EXCLUDE or not path.is_file():
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {rel}")
    (ROOT / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(lines)} checksums")


if __name__ == "__main__":
    main()

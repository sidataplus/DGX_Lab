#!/usr/bin/env python3
"""Validate the generated static distribution before GitHub Pages publication."""
from __future__ import annotations

import argparse
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

MAX_ARTIFACT_BYTES = 1_000_000_000
REQUIRED_SUFFIXES = {".css", ".js", ".wasm"}
DISCLAIMER = (
    "DGX Lab is not affiliated with, sponsored by, or endorsed by "
    "NVIDIA Corporation or SchedMD LLC."
)
LOCAL_PROGRESS_NOTICE = "Web-edition progress is stored only in this browser."
QUOTED_ROOT_URL = re.compile(r"""(?P<quote>["'])(?P<url>/[^"'<>\\\s]+)(?P=quote)""")


class AssetReferenceParser(HTMLParser):
    """Collect browser-loaded URLs and reject a base tag."""

    def __init__(self) -> None:
        super().__init__()
        self.urls: set[str] = set()
        self.has_base_tag = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag == "base":
            self.has_base_tag = True
        for name, value in attrs:
            if name in {"href", "src"} and value:
                self.urls.add(value)


def normalize_base(raw: str) -> str:
    base = raw.strip()
    if not base.startswith("/") or not base.endswith("/"):
        raise ValueError("expected base path must start and end with '/'")
    if "//" in base:
        raise ValueError("expected base path must not contain an empty path segment")
    if any(part in {".", ".."} for part in Path(base).parts):
        raise ValueError("expected base path must not contain '.' or '..'")
    return base


def local_target(url: str, base: str, root: Path) -> Path | None:
    if url.startswith(("data:", "mailto:", "tel:", "#")):
        return None

    parsed = urlsplit(url)
    if parsed.scheme or parsed.netloc:
        raise ValueError(f"external URL is not allowed in the static entry point: {url}")
    if not parsed.path.startswith("/"):
        raise ValueError(f"local asset URL is not absolute: {url}")
    if not parsed.path.startswith(base):
        raise ValueError(f"local asset URL escapes expected base {base!r}: {url}")

    relative_text = unquote(parsed.path[len(base) :])
    if not relative_text or relative_text.endswith("/"):
        return None

    relative = Path(relative_text)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError(f"unsafe local asset path: {url}")

    target = (root / relative).resolve()
    if not target.is_relative_to(root.resolve()):
        raise ValueError(f"local asset path escapes distribution root: {url}")
    return target


def validate_distribution(dist: Path, base: str) -> list[str]:
    failures: list[str] = []
    if not dist.is_dir():
        return [f"distribution directory does not exist: {dist}"]

    paths = sorted(path for path in dist.rglob("*") if path.is_file() or path.is_symlink())
    symlinks = [path.relative_to(dist).as_posix() for path in paths if path.is_symlink()]
    if symlinks:
        failures.append(f"distribution contains symlinks: {', '.join(symlinks)}")

    files = [path for path in paths if path.is_file() and not path.is_symlink()]
    total_bytes = sum(path.stat().st_size for path in files)
    if total_bytes > MAX_ARTIFACT_BYTES:
        failures.append(
            f"distribution is {total_bytes} bytes; maximum is {MAX_ARTIFACT_BYTES} bytes"
        )

    suffixes = {path.suffix.lower() for path in files}
    missing_suffixes = sorted(REQUIRED_SUFFIXES - suffixes)
    if missing_suffixes:
        failures.append(f"distribution lacks required assets: {', '.join(missing_suffixes)}")

    index = dist / "index.html"
    if not index.is_file():
        failures.append("distribution lacks index.html")
        return failures

    html = index.read_text(encoding="utf-8")
    if DISCLAIMER not in html:
        failures.append("index.html lacks the required independent-product disclaimer")
    if LOCAL_PROGRESS_NOTICE not in html:
        failures.append("index.html lacks the browser-local progress notice")

    parser = AssetReferenceParser()
    parser.feed(html)
    if parser.has_base_tag:
        failures.append("index.html must not override the build-time public URL with <base>")

    root_urls = {match.group("url") for match in QUOTED_ROOT_URL.finditer(html)}
    for url in sorted(parser.urls | root_urls):
        try:
            target = local_target(url, base, dist)
        except ValueError as exc:
            failures.append(str(exc))
            continue
        if target is not None and not target.is_file():
            failures.append(
                f"referenced asset does not exist: {url} -> {target.relative_to(dist)}"
            )

    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("dist", type=Path)
    parser.add_argument("expected_base")
    args = parser.parse_args()

    try:
        base = normalize_base(args.expected_base)
    except ValueError as exc:
        print(f"PAGES DIST VALIDATION FAILED: {exc}", file=sys.stderr)
        return 2

    dist = args.dist.resolve()
    failures = validate_distribution(dist, base)
    if failures:
        print("PAGES DIST VALIDATION FAILED", file=sys.stderr)
        print("\n".join(f"  - {failure}" for failure in failures), file=sys.stderr)
        return 1

    files = [path for path in dist.rglob("*") if path.is_file() and not path.is_symlink()]
    total_bytes = sum(path.stat().st_size for path in files)
    print(
        f"validated GitHub Pages artifact: {len(files)} files, "
        f"{total_bytes} bytes, base path {base}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

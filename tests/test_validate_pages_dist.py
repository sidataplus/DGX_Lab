from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.validate_pages_dist import normalize_base, validate_distribution


DISCLAIMER = (
    "DGX Lab is not affiliated with, sponsored by, or endorsed by "
    "NVIDIA Corporation or SchedMD LLC."
)
LOCAL_PROGRESS_NOTICE = "Web-edition progress is stored only in this browser."
ROOT = Path(__file__).resolve().parents[1]


class PagesDistributionValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        (ROOT / "temp").mkdir(exist_ok=True)
        self.temporary_directory = tempfile.TemporaryDirectory(
            prefix="pages-dist-test-", dir=ROOT / "temp"
        )
        self.dist = Path(self.temporary_directory.name)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def write_valid_distribution(self, asset_url: str = "/DGX_Lab/app.js") -> None:
        (self.dist / "app.css").write_text("body {}\n", encoding="utf-8")
        (self.dist / "app.js").write_text("export {};\n", encoding="utf-8")
        (self.dist / "app.wasm").write_bytes(b"\x00asm")
        (self.dist / "index.html").write_text(
            "<!doctype html>\n"
            "<html><head>\n"
            '  <link rel="stylesheet" href="/DGX_Lab/app.css">\n'
            f'  <script type="module" src="{asset_url}"></script>\n'
            "</head><body>\n"
            f"  <p>{DISCLAIMER}</p>\n"
            f"  <p>{LOCAL_PROGRESS_NOTICE}</p>\n"
            '  <a href="#lesson">Start lesson</a>\n'
            "</body></html>\n",
            encoding="utf-8",
        )

    def test_accepts_complete_project_path_distribution(self) -> None:
        self.write_valid_distribution()

        self.assertEqual(validate_distribution(self.dist, "/DGX_Lab/"), [])

    def test_rejects_root_hosted_asset(self) -> None:
        self.write_valid_distribution(asset_url="/app.js")

        failures = validate_distribution(self.dist, "/DGX_Lab/")

        self.assertTrue(
            any("escapes expected base" in failure for failure in failures), failures
        )

    def test_rejects_missing_referenced_asset(self) -> None:
        self.write_valid_distribution(asset_url="/DGX_Lab/missing.js")

        failures = validate_distribution(self.dist, "/DGX_Lab/")

        self.assertTrue(
            any("referenced asset does not exist" in failure for failure in failures),
            failures,
        )

    def test_rejects_external_entry_point_asset(self) -> None:
        self.write_valid_distribution(asset_url="https://example.invalid/app.js")

        failures = validate_distribution(self.dist, "/DGX_Lab/")

        self.assertTrue(
            any("external URL is not allowed" in failure for failure in failures), failures
        )

    def test_rejects_missing_public_notices(self) -> None:
        self.write_valid_distribution()
        index = self.dist / "index.html"
        index.write_text(
            index.read_text(encoding="utf-8")
            .replace(DISCLAIMER, "")
            .replace(LOCAL_PROGRESS_NOTICE, ""),
            encoding="utf-8",
        )

        failures = validate_distribution(self.dist, "/DGX_Lab/")

        self.assertIn(
            "index.html lacks the required independent-product disclaimer", failures
        )
        self.assertIn("index.html lacks the browser-local progress notice", failures)


class PagesBasePathValidationTests(unittest.TestCase):
    def test_requires_leading_and_trailing_slashes(self) -> None:
        for value in ("DGX_Lab/", "/DGX_Lab"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                normalize_base(value)

    def test_rejects_parent_segments_and_empty_segments(self) -> None:
        for value in ("/DGX_Lab/../other/", "/DGX_Lab//assets/"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                normalize_base(value)


if __name__ == "__main__":
    unittest.main()

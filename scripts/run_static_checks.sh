#!/usr/bin/env bash
set -euo pipefail
python3 scripts/validate_all.py
python3 scripts/check_forbidden_apis.py
python3 scripts/static_rust_sanity.py
node --check prototype/app.js
python3 -m py_compile prototype/serve.py scripts/*.py
python3 scripts/build_course_pack.py
python3 scripts/generate_manifest.py
python3 scripts/verify_manifest.py

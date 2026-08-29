#!/usr/bin/env python3
"""Validate all declarative DGX Lab content and cross-references."""
from __future__ import annotations
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any
import yaml
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "schemas"

class ValidationFailure(Exception):
    pass

def load_yaml(path: Path) -> Any:
    try:
        return yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValidationFailure(f"{path}: YAML parse failed: {exc}") from exc

def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValidationFailure(f"{path}: JSON parse failed: {exc}") from exc

def validate_document(path: Path, schema_name: str) -> Any:
    document = load_yaml(path) if path.suffix in {".yaml", ".yml"} else load_json(path)
    schema = load_json(SCHEMA_DIR / schema_name)
    errors = sorted(Draft202012Validator(schema).iter_errors(document), key=lambda e: list(e.absolute_path))
    if errors:
        details = "\n".join(f"  - {list(error.absolute_path)}: {error.message}" for error in errors[:25])
        raise ValidationFailure(f"{path}: schema validation failed\n{details}")
    return document

def unique(items: list[dict[str, Any]], field: str, context: str) -> None:
    seen: set[str] = set()
    for item in items:
        value = str(item[field])
        if value in seen:
            raise ValidationFailure(f"{context}: duplicate {field} {value!r}")
        seen.add(value)

def validate_scenarios() -> set[str]:
    ids: set[str] = set()
    for path in sorted((ROOT / "scenario-src").glob("*.yaml")):
        doc = validate_document(path, "scenario.schema.json")
        if doc["id"] in ids:
            raise ValidationFailure(f"duplicate scenario id: {doc['id']}")
        ids.add(doc["id"])
        unique(doc.get("objectives", []), "id", str(path))
        unique(doc.get("checks", []), "id", str(path))
        for file in doc.get("initial_files", []):
            parts = Path(file["path"]).parts
            if ".." in parts:
                raise ValidationFailure(f"{path}: unsafe path {file['path']}")
    if not ids:
        raise ValidationFailure("no scenarios found")
    return ids

def validate_course(scenario_ids: set[str]) -> None:
    for course_path in sorted((ROOT / "course-src").glob("*/course.yaml")):
        course = validate_document(course_path, "course.schema.json")
        unique(course["modules"], "id", str(course_path))
        course_root = course_path.parent
        for module in course["modules"]:
            lab_path = course_root / module["lab_path"]
            if not lab_path.is_file():
                raise ValidationFailure(f"{course_path}: missing lab {module['lab_path']}")
            lab = validate_document(lab_path, "lab.schema.json")
            if lab["id"] != module["id"]:
                raise ValidationFailure(f"{lab_path}: id differs from course module")
            if lab["scenario"] not in scenario_ids:
                raise ValidationFailure(f"{lab_path}: unknown scenario {lab['scenario']}")
            unique(lab["objectives"], "id", str(lab_path))
            unique(lab["steps"], "id", str(lab_path))
            guide = lab_path.with_name("guide.md")
            if not guide.is_file() or guide.stat().st_size < 100:
                raise ValidationFailure(f"{lab_path}: missing/substantial guide.md")

def validate_questions() -> set[str]:
    ids: set[str] = set()
    option_ids_by_question: dict[str, set[str]] = {}
    for path in sorted((ROOT / "question-src").glob("*core.yaml")):
        bank = validate_document(path, "question-bank.schema.json")
        for question in bank["questions"]:
            qid = question["id"]
            if qid in ids:
                raise ValidationFailure(f"duplicate question id: {qid}")
            ids.add(qid)
            if question["type"] in {"single_choice", "multi_select"}:
                options = {option["id"] for option in question["options"]}
                if len(options) != len(question["options"]):
                    raise ValidationFailure(f"{qid}: duplicate option id")
                option_ids_by_question[qid] = options
                correct = {question["correct"]} if isinstance(question["correct"], str) else set(question["correct"])
                if not correct <= options:
                    raise ValidationFailure(f"{qid}: correct answer references unknown option")
            if question["type"] == "fill_blank":
                unique(question["blanks"], "id", qid)
    if not ids:
        raise ValidationFailure("no questions found")
    return ids

def validate_blueprint() -> None:
    path = ROOT / "question-src" / "certification-blueprint.yaml"
    doc = validate_document(path, "certification-blueprint.schema.json")
    weights = doc["weights"]
    if sum(weights.values()) != 100:
        raise ValidationFailure(f"{path}: certification weights must sum to 100")
    if weights != {"practical": 60, "multiple_choice": 25, "fill_blank": 15}:
        raise ValidationFailure(f"{path}: approved weights changed: {weights}")
    policy = doc["pass_policy"]
    if policy["overall_percent"] != 80 or policy["knowledge_percent"] != 70:
        raise ValidationFailure(f"{path}: approved pass policy changed")

def validate_tauri() -> None:
    path = ROOT / "src-tauri" / "tauri.conf.json"
    doc = load_json(path)
    csp = doc["app"]["security"]["csp"]
    if "connect-src 'self'" not in csp:
        raise ValidationFailure("Tauri CSP must restrict fetches to the application origin")
    if any(token in csp for token in ("connect-src *", "http:", "https:")):
        raise ValidationFailure("Tauri CSP permits an external connection origin")
    if "worker-src 'self' blob:" not in csp:
        raise ValidationFailure("Tauri CSP must permit only bundled/blob simulation workers")
    capability = load_json(ROOT / "src-tauri" / "capabilities" / "main.json")
    unexpected = set(capability["permissions"]) - {"core:default"}
    if unexpected:
        raise ValidationFailure(f"unexpected Tauri permissions: {sorted(unexpected)}")

def validate_prototype() -> None:
    for name in ("index.html", "styles.css", "app.js", "serve.py"):
        path = ROOT / "prototype" / name
        if not path.is_file() or path.stat().st_size == 0:
            raise ValidationFailure(f"prototype missing {name}")
    html = (ROOT / "prototype" / "index.html").read_text(encoding="utf-8")
    if "connect-src 'none'" not in html:
        raise ValidationFailure("prototype CSP does not deny connections")
    if "http://" in html or "https://" in html:
        raise ValidationFailure("prototype HTML contains external URL")

def validate_workspace() -> None:
    root_manifest = ROOT / "Cargo.toml"
    try:
        workspace = tomllib.loads(root_manifest.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValidationFailure(f"{root_manifest}: TOML parse failed: {exc}") from exc
    members = workspace.get("workspace", {}).get("members", [])
    if not members:
        raise ValidationFailure("Cargo workspace contains no members")
    for member in members:
        manifest = ROOT / member / "Cargo.toml"
        if not manifest.is_file():
            raise ValidationFailure(f"workspace member is missing Cargo.toml: {member}")
        try:
            tomllib.loads(manifest.read_text(encoding="utf-8"))
        except Exception as exc:
            raise ValidationFailure(f"{manifest}: TOML parse failed: {exc}") from exc
    for manifest in ROOT.rglob("*.toml"):
        try:
            tomllib.loads(manifest.read_text(encoding="utf-8"))
        except Exception as exc:
            raise ValidationFailure(f"{manifest}: TOML parse failed: {exc}") from exc


def requirement_ids(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    return set(re.findall(r"(?m)^\|\s*`?([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-\d{3})`?\s*\|", text))


def validate_traceability() -> None:
    prd = ROOT / "docs" / "DGX_Lab_PRD_v1.0.md"
    trace = ROOT / "docs" / "handoff" / "REQUIREMENTS_TRACEABILITY.md"
    prd_ids = requirement_ids(prd)
    trace_ids = requirement_ids(trace)
    if len(prd_ids) != 241:
        raise ValidationFailure(f"expected 241 PRD requirement IDs, found {len(prd_ids)}")
    missing = sorted(prd_ids - trace_ids)
    extra = sorted(trace_ids - prd_ids)
    if missing or extra:
        raise ValidationFailure(
            f"requirements traceability mismatch; missing={missing[:10]}, extra={extra[:10]}"
        )


def main() -> int:
    try:
        scenario_ids = validate_scenarios()
        validate_course(scenario_ids)
        validate_questions()
        validate_blueprint()
        validate_tauri()
        validate_prototype()
        validate_workspace()
        validate_traceability()
    except ValidationFailure as exc:
        print(f"VALIDATION FAILED: {exc}", file=sys.stderr)
        return 1
    print(
        f"validated {len(scenario_ids)} scenarios, course content, questions, "
        "Tauri boundary, Cargo manifests, 241 requirement links, and prototype"
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

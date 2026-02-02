#!/usr/bin/env python3
import csv
import datetime as dt
import json
import os
import subprocess
import sys
from typing import List, Dict, Optional


def run_gh(args: List[str], input_text: Optional[str] = None) -> str:
    proc = subprocess.run(
        ["gh"] + args,
        input=input_text,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or "gh command failed")
    return proc.stdout


def load_labels() -> set:
    out = run_gh(["label", "list", "--json", "name"])
    data = json.loads(out)
    return {item["name"] for item in data}


def ensure_label(name: str, description: str, existing: set) -> None:
    if name in existing:
        return
    run_gh(["label", "create", name, "--description", description])
    existing.add(name)


def classify_label(title: str, body: str) -> str:
    text = f"{title}\n{body}".lower()
    rules = [
        ("spec", ["spec", "syntax", "grammar", "semantics", "language design", "proposal"]),
        ("compiler", ["compiler", "parser", "frontend", "backend", "codegen", "typecheck", "type checker", "optimizer"]),
        ("runtime", ["runtime", "vm", "jit", "gc", "garbage collector", "scheduler", "interpreter"]),
        ("stdlib", ["stdlib", "standard library", "library", "collections", "io", "fs", "net"]),
        ("tooling", ["tooling", "cli", "lsp", "formatter", "linter", "debugger", "ide", "build tool"]),
        ("docs", ["docs", "documentation", "guide", "tutorial", "reference", "readme", "examples"]),
        ("infra", ["infra", "ci", "cd", "pipeline", "release", "packaging", "docker", "k8s", "deployment"]),
    ]
    for label, keywords in rules:
        if any(k in text for k in keywords):
            return label
    return "tooling"


def generate_subtasks(title: str, body: str, label: str) -> List[str]:
    base = [
        f"Review existing context and constraints for: {title}",
        "Define functional and non-functional requirements",
        "Design the approach and document key decisions",
        "Implement the core changes in a minimal, testable slice",
        "Add/update tests to cover success and failure cases",
        "Update docs or examples to match behavior",
        "Run relevant checks locally and fix regressions",
    ]
    if label in {"compiler", "runtime"}:
        base.insert(4, "Add targeted benchmarks or perf checks for hot paths")
    if label == "infra":
        base.insert(4, "Validate changes in CI-like environment")
    return base[:12]


def generate_acceptance(title: str, label: str) -> List[str]:
    checks = [
        f"Behavior matches the goal described for: {title}",
        "All new/updated tests pass locally",
        "No regressions in existing functionality are observed",
        "Docs/examples accurately describe the new behavior",
        "Edge cases are handled or explicitly documented",
        "Code changes are reviewed and ready to merge",
    ]
    if label in {"compiler", "runtime"}:
        checks.insert(3, "Performance impact is measured and acceptable")
    if label == "infra":
        checks.insert(3, "CI/release pipeline runs successfully with changes")
    return checks[:10]


def generate_notes(body: str) -> Optional[List[str]]:
    text = body.lower()
    notes = []
    if any(k in text for k in ["breaking", "migration", "deprecate"]):
        notes.append("Potential breaking change; confirm migration path")
    if any(k in text for k in ["security", "auth", "secret"]):
        notes.append("Review security implications and threat model")
    if any(k in text for k in ["perf", "performance", "latency"]):
        notes.append("Track performance impact before and after")
    return notes or None


def expand_body(csv_body: str, title: str, label: str) -> str:
    base = (csv_body or "").strip()
    subtasks = generate_subtasks(title, base, label)
    acceptance = generate_acceptance(title, label)
    notes = generate_notes(base)

    parts = []
    if base:
        parts.append(base)
        parts.append("**Goal**\n" + base)
    else:
        parts.append("**Goal**\nTBD")
    parts.append("**Subtasks**\n" + "\n".join(f"- {s}" for s in subtasks))
    parts.append("**Acceptance criteria**\n" + "\n".join(f"- {c}" for c in acceptance))
    if notes:
        parts.append("**Notes**\n" + "\n".join(f"- {n}" for n in notes))
    return "\n\n".join(parts).strip() + "\n"


def find_existing_issue(title: str) -> Optional[Dict[str, str]]:
    search = f'"{title}" in:title'
    out = run_gh(["issue", "list", "--state", "all", "--search", search, "--json", "number,title,url"])
    data = json.loads(out)
    for item in data:
        if item.get("title") == title:
            return item
    return None


def read_csv(path: str) -> List[Dict[str, str]]:
    rows = []
    with open(path, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            title = (row.get("Title") or "").strip()
            body = (row.get("Body") or "").strip()
            if not title:
                continue
            rows.append({"title": title, "body": body})
    return rows


def main(argv: List[str]) -> int:
    if len(argv) != 2:
        print("Usage: create_issues_from_csv.py issues.csv", file=sys.stderr)
        return 2

    csv_path = argv[1]
    if not os.path.exists(csv_path):
        print(f"CSV not found: {csv_path}", file=sys.stderr)
        return 2

    label_descriptions = {
        "mvp": "MVP scope",
        "spec": "Language specification and design",
        "compiler": "Compiler implementation",
        "runtime": "Runtime and VM",
        "stdlib": "Standard library",
        "tooling": "Developer tooling",
        "docs": "Documentation",
        "infra": "Infrastructure and CI",
    }

    results = []
    created = 0
    updated = 0
    failed = 0

    try:
        existing_labels = load_labels()
        for name, desc in label_descriptions.items():
            ensure_label(name, desc, existing_labels)
    except Exception as exc:
        print(f"Failed to load/create labels: {exc}", file=sys.stderr)
        return 1

    rows = read_csv(csv_path)
    today = dt.date.today().isoformat()

    for row in rows:
        title = row["title"]
        body = row["body"]
        action = "failed"
        issue_number = None
        url = None
        error = None

        try:
            label = classify_label(title, body)
            expanded = expand_body(body, title, label)
            labels = f"mvp,{label}"

            existing = find_existing_issue(title)
            if existing:
                issue_number = existing.get("number")
                url = existing.get("url")
                comment_body = f"Re-imported from issues.csv on {today}\n\n{expanded}"
                run_gh(["issue", "comment", str(issue_number), "--body", comment_body])
                action = "updated"
                updated += 1
            else:
                out = run_gh(["issue", "create", "--title", title, "--body", expanded, "--label", labels])
                url = out.strip()
                action = "created"
                created += 1
        except Exception as exc:
            failed += 1
            error = str(exc)

        results.append(
            {
                "title": title,
                "action": action,
                "issue_number": issue_number,
                "url": url,
                "error": error,
            }
        )

    os.makedirs("artifacts", exist_ok=True)
    with open("artifacts/issues_import_log.json", "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)

    print(f"Created: {created}")
    print(f"Updated: {updated}")
    if failed:
        print(f"Failed: {failed}")
        for item in results:
            if item["action"] == "failed":
                print(f"- {item['title']}: {item['error']}")
    else:
        print("Failed: 0")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

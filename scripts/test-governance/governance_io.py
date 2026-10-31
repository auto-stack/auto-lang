"""Shared governance I/O and identity helpers.

PLAN-898: split out of the retired module_catalog.py (tm engine); only the
generic helpers that fixed-group runners, meters and case tooling consume
remain. Catalog/module/selection machinery lives in archived PLAN-888.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from process_capture import checked_output

ROOT = Path(__file__).resolve().parents[2]


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"),
                                     ensure_ascii=False).encode()).hexdigest()


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")


def source_identity(root):
    """Bind tracked and new code/config/input bytes, including path dependencies.

    Ignore only plan bookkeeping/generated reports and build evidence.
    Reference documents and other tracked fixtures participate in identity.
    A deleted file changes identity.
    A new test, feature or source changes identity even before git add.
    """
    rows = []
    for repo in (root, root.parent / "auto-ui", root.parent / "auto-down", root.parent / "auto-man"):
        if not repo.is_dir():
            continue
        head = checked_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
        # HEAD records provenance separately: a docs-only commit does not stale code.
        raw = checked_output(["git", "-C", str(repo), "ls-files", "-z",
                                       "--cached", "--others", "--exclude-standard"])
        files = set(raw.decode("utf-8").split("\0")) - {""}
        inputs = []
        for rel in sorted(files):
            if rel.startswith(("docs/plans/", "docs/reports/", ".autoos/")):
                continue
            path = repo / rel
            if path.is_file():
                inputs.append((rel, hashlib.sha256(path.read_bytes()).hexdigest()))
            else:
                inputs.append((rel, "DELETED"))
        rows.append({"repo": repo.name, "head": head, "inputs": digest(inputs)})
    # Hash effective execution settings without persisting credentials or paths
    # from their values. An env-only change must not reuse a measurement freeze.
    from native_meter import workload_environment
    effective = workload_environment(root=root)
    # cargo run injects launcher package labels, not execution settings. Cargo
    # home and configured env defaults must compare by their effective values.
    for key in list(effective):
        if key.startswith('CARGO_PKG_') or key in ('CARGO_MANIFEST_DIR', 'CARGO_MANIFEST_PATH'):
            effective.pop(key)
    effective.setdefault('CARGO_HOME', str(Path.home() / '.cargo'))
    environment=digest(sorted((key,value) for key,value in effective.items()
        if key.startswith(("AC_","AUTO_","CARGO_","RUST_","NEXTEST_","AV798_","MN818_"))
        or key in ("RUSTFLAGS","T3_MILESTONE","PATH","PATHEXT","PYTHONPATH","PYTHONHASHSEED","TEMP","TMP","TMPDIR","GENERATE_EXPECTED","UPDATE_EXPECTED","UPDATE_GOLDENS")))
    return {"code": digest({"inputs":[(r["repo"], r["inputs"]) for r in rows],"environment":environment}),
            "repositories": rows,"environment_sha256":environment}

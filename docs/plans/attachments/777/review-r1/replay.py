"""PLAN-777 independent review controls. No writes to implementation checkout.

Run: python replay.py --implementation D:/autostack/.wt/lang-777/auto-lang
All mutations happen in owned temporary copies of the committed fixture.
"""
import argparse
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result.stdout.strip()


def commit(repo, message):
    git(repo, "add", "-A")
    git(repo, "-c", "user.name=777-review", "-c", "user.email=review@invalid",
        "commit", "-qm", message)


def cli(script, repo, *args):
    result = subprocess.run([sys.executable, str(script), "--root", str(repo), *args],
                            capture_output=True, text=True, encoding="utf-8", timeout=40)
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError:
        report = None
    return {"exit_code": result.returncode, "stderr": result.stderr, "report": report}


def summarize(run):
    r = run["report"] or {}
    return {"exit_code": run["exit_code"], "stderr": run["stderr"],
            "strict": r.get("strict"), "declared_edges": r.get("declared_edges"),
            "revision": r.get("revision")}


def cargo_oracle(repo, package_name="fixture-lang"):
    # Independent declaration oracle in an owned disposable fixture only.
    # Missing fixture locks may be created; production inputs are untouched.
    q = subprocess.run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                       cwd=repo, capture_output=True, text=True, encoding="utf-8", timeout=40)
    edges = []
    if q.returncode == 0:
        data = json.loads(q.stdout)
        for p in data["packages"]:
            if p["name"] != package_name:
                continue
            for d in p["dependencies"]:
                if d.get("path"):
                    edges.append({"from": p["name"], "to": d["name"], "kind": d.get("kind") or "normal",
                                  "target": d.get("target"), "optional": d.get("optional"),
                                  "path_tail": Path(d["path"]).name})
    return {"command": "cargo metadata --offline --no-deps --format-version 1 (temporary fixture)",
            "exit_code": q.returncode, "edges": edges, "stderr": q.stderr[:500]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--implementation", required=True)
    args = parser.parse_args()
    impl = Path(args.implementation).resolve()
    script = impl / "scripts/modularity-audit.py"
    evidence = Path(__file__).resolve().parent
    results = []
    container = Path(tempfile.mkdtemp(prefix="plan777-review-")).resolve()
    # Every test repo is a direct descendant of this fresh, explicitly owned path.
    def fresh(name):
        repo = container / name
        assert repo.parent.resolve() == container
        shutil.copytree(impl / "test/fixtures/modularity/audit", repo)
        git(repo, "init", "-q")
        commit(repo, "fixture init")
        return repo

    def freeze(repo):
        q = cli(script, repo, "--refresh-baseline")
        assert q["exit_code"] == 0, q

    def source_before(repo):
        p = repo / "crates/fixture-lang/Cargo.toml"
        return p, p.read_text(encoding="utf-8")

    def append_manifest(repo, extra):
        p, before = source_before(repo)
        after = before + extra
        p.write_text(after, encoding="utf-8")
        assert p.read_text(encoding="utf-8") == after and before != after
        return {"path": p.relative_to(repo).as_posix(), "before": before, "after": after}

    try:
        repo = fresh("single-target-control")
        freeze(repo)
        mutation = append_manifest(repo, '\n[target.\'cfg(windows)\'.dependencies]\n'
                                   'fixture-ui-kit = { path = "../fixture-ui-kit" }\n')
        run = cli(script, repo, "--mode", "strict")
        results.append({"id": "C01-control", "expected": "exit 2 naming fixture-ui-kit under cfg(windows)",
                        "mutation": mutation, "actual": summarize(run),
                        "contract_pass": run["exit_code"] == 2})

        repo = fresh("multiple-target-tables")
        freeze(repo)
        mutation = append_manifest(repo, '\n[target.\'cfg(windows)\'.dependencies]\n'
                                   'fixture-ui-kit = { path = "../fixture-ui-kit" }\n'
                                   '\n[target.\'cfg(unix)\'.dependencies]\n'
                                   'fixture-lang-sdk = { path = "../fixture-lang-sdk" }\n')
        run = cli(script, repo, "--mode", "strict")
        results.append({"id": "C01-multiple-target", "expected": "exit 2 naming the actual Windows UI edge",
                        "mutation": mutation, "actual": summarize(run), "cargo_oracle": cargo_oracle(repo),
                        "contract_pass": run["exit_code"] == 2 and any(
                            x.get("edge", {}).get("to") == "fixture-ui-kit"
                            for x in (run["report"] or {}).get("strict", {}).get("violations", []))})

        repo = fresh("workspace-inherited-path")
        freeze(repo)
        root_manifest = repo / "Cargo.toml"
        root_before = root_manifest.read_text(encoding="utf-8")
        root_after = root_before + '\n[workspace.dependencies]\nfixture-ui-kit = { path = "crates/fixture-ui-kit" }\n'
        root_manifest.write_text(root_after, encoding="utf-8")
        mutation = append_manifest(repo, '\n[dependencies]\nfixture-ui-kit = { workspace = true }\n')
        run = cli(script, repo, "--mode", "strict")
        results.append({"id": "C02-workspace-path", "expected": "resolve inherited path relative to workspace root; exit 2",
                        "root_mutation": {"before": root_before, "after": root_after},
                        "mutation": mutation, "actual": summarize(run), "cargo_oracle": cargo_oracle(repo),
                        "contract_pass": run["exit_code"] == 2})

        repo = fresh("readonly-output-alias")
        path = repo / ".autoos/module-boundaries.toml"
        before_bytes = path.read_bytes()
        before_hash = digest(path)
        run = cli(script, repo, "--mode", "observe", "--output", str(path))
        after_hash = digest(path)
        results.append({"id": "C03-output-input-collision", "expected": "reject output/input alias before modifying config",
                        "input": before_bytes.decode(), "before_sha256": before_hash,
                        "after_sha256": after_hash, "actual": summarize(run),
                        "after_prefix": path.read_text(encoding="utf-8")[:180],
                        "contract_pass": run["exit_code"] != 0 and before_hash == after_hash})

        repo = fresh("optional-activation")
        mutation0 = append_manifest(repo, '\n[dependencies]\nfixture-ui-kit = { path = "../fixture-ui-kit", optional = true }\n')
        config = repo / ".autoos/module-boundaries.toml"
        config.write_text(config.read_text(encoding="utf-8") + '\n[[baseline.edge_notes]]\n'
                          'from = "fixture-lang"\nto = "fixture-ui-kit"\nkind = "normal"\n'
                          'owner = "ui-owner"\nexit_phase = "RX"\nstatus = "active"\n', encoding="utf-8")
        commit(repo, "freeze optional declaration and ownership note")
        freeze(repo)
        accepted = cli(script, repo, "--mode", "strict")
        path, before = source_before(repo)
        after = before.replace("optional = true", "optional = false")
        path.write_text(after, encoding="utf-8")
        assert before != after and "optional = false" in path.read_text(encoding="utf-8")
        run = cli(script, repo, "--mode", "strict")
        results.append({"id": "C04-newly-forbidden-activation", "expected": "optional allowed control, mandatory forbidden change rejected",
                        "accepted_control": summarize(accepted),
                        "mutation": {"before": before, "after": after}, "actual": summarize(run),
                        "contract_pass": accepted["exit_code"] == 0 and run["exit_code"] != 0})

        # Source inspection complement: every path named in the research's full
        # responsibility table must have an owner or explicitly documented group.
        config_text = (impl / ".autoos/module-boundaries.toml").read_text(encoding="utf-8")
        import tomllib
        cfg = tomllib.loads(config_text)
        registered = {x["path"] for x in cfg["ownership"]}
        candidates = ["crates/auto-lang/src/parser.rs", "crates/auto-lang/src/ast.rs",
                      "crates/auto-lang/src/dialect.rs", "crates/auto-lang/src/dialect/ui.rs",
                      "crates/auto-man/src/builder", "crates/auto/src/main.rs"]
        coverage = {p: {"exists": (impl / p).exists(), "registered": p in registered} for p in candidates}
        results.append({"id": "C05-ownership-completeness", "expected": "all research weld/responsibility categories have explicit ownership",
                        "actual": coverage, "contract_pass": all(v["registered"] for v in coverage.values())})

        repo = fresh("legal-cross-repository-sdk")
        external = container / "external-sdk"
        shutil.copytree(impl / "test/fixtures/modularity/audit/crates/fixture-lang-sdk", external)
        package_manifest = external / "Cargo.toml"
        package_manifest.write_text(package_manifest.read_text(encoding="utf-8").replace(
            'name = "fixture-lang-sdk"', 'name = "external-lang-sdk"'), encoding="utf-8")
        config = repo / ".autoos/module-boundaries.toml"
        text = config.read_text(encoding="utf-8").replace(
            '[domains.members]', '[domains.members]\nexternal-lang-sdk = "lang-core"')
        production_rule = next(x for x in cfg["rules"]["forbidden_declared"]
                               if x["id"] == "out-of-root-edges-must-be-optional")
        text = text.replace('[[text_scan]]', '[[rules.forbidden_declared]]\n'
                            'id = "out-of-root-edges-must-be-optional"\n'
                            'to_scopes = ["external"]\nkinds = ["normal", "build"]\n\n'
                            '[[text_scan]]', 1)
        config.write_text(text, encoding="utf-8")
        commit(repo, "production cross-root rule and explicit SDK domain")
        freeze(repo)
        ui_manifest = repo / "crates/fixture-ui/Cargo.toml"
        before = ui_manifest.read_text(encoding="utf-8")
        after = before + 'external-lang-sdk = { path = "../../../external-sdk" }\n'
        ui_manifest.write_text(after, encoding="utf-8")
        assert before != after and external.is_dir()
        run = cli(script, repo, "--mode", "strict")
        results.append({"id": "C06-allowed-cross-repository-sdk", "expected": "Design37 allows UI to depend on a separate language SDK using explicit development path",
                        "production_rule": production_rule,
                        "mutation": {"before": before, "after": after}, "actual": summarize(run),
                        "cargo_oracle": cargo_oracle(repo, "fixture-ui"),
                        "contract_pass": run["exit_code"] == 0})
    finally:
        # Resolved removal target is the original freshly created temp container,
        # under the system temp root; no repository path or links were used.
        target = container.resolve()
        assert target == container and target.parent == Path(tempfile.gettempdir()).resolve()
        def clear_readonly(function, path, exc):
            owned = Path(path).resolve()
            assert owned.is_relative_to(target)
            os.chmod(owned, owned.stat().st_mode | stat.S_IWRITE)
            function(path)
        shutil.rmtree(target, onexc=clear_readonly)

    envelope = {"implementation_head": git(impl, "rev-parse", "HEAD"),
                "script_sha256": digest(script), "cases": results}
    (evidence / "controls.jsonl").write_text(json.dumps(envelope, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    for result in results:
        actual = result["actual"]
        print(json.dumps({"id": result["id"], "contract_pass": result["contract_pass"],
                          "exit_code": actual.get("exit_code"),
                          "strict": actual.get("strict"), "coverage": actual if result["id"].startswith("C05") else None}, ensure_ascii=False))
    return 0 if all(row["contract_pass"] for row in results) else 1


if __name__ == "__main__":
    sys.exit(main())

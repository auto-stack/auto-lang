#!/usr/bin/env python3
"""PLAN-741 Phase 8 (T-42/T-43) controlled fixture matrix - identity-aware.

v3 (post P741P6-R1/R2, P741P7-R1/R2/R3 and P741-R7-QA-01/02 reviews):
- plan source auto-detected: docs/plans/ (partial executing) OR the
  archived state replayed from git history; both matrices run when both
  states are available;
- totals are derived from the EXECUTION record only - the module navigation
  controls (all-correct / forgotten-update) are REAL executed cases riding
  dedicated sandbox repos (R7-QA-01: built-but-never-run controls were
  falsely counted via a fixed +2);
- identity controls (R7-QA-02): a missing prototype-README 741 reference,
  or any 741 reference (README / module nav / ledger review item)
  resolving to a NON-canonical target, must fail with the file/line/entry
  named;
- all status transforms use wildcard set_fm; every mutation passes
  assert_modified; flip_one_conflict prefers multi-row groups and compares
  sets; TemporaryDirectory reclaimed via try/finally.

Exit 0 = all cases pass; 1 = any failure.
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).resolve()
REPO = SCRIPT.parents[3]
ASSERT = SCRIPT.parent / "final_assertions.py"
PLAN_NAME = "741-ac-hir-native-core.md"
ACTIVE_REPORT = "../reports/"
ARCHIVE_REPORT = "../../reports/"
ACTIVE_PLAN = REPO / "docs" / "plans" / PLAN_NAME
ARCHIVE_PLAN = REPO / "docs" / "plans" / "archive" / PLAN_NAME
ACTIVE_NAV_ROW = ("| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                  "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")
CANONICAL_NAV_ROW = ("| 741 | ac-hir-native-core | ✅（delivered，r6） | "
                     "[741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | x |")
STALE_NAV_ROW = ("| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                 "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")


def git_archived_text() -> str | None:
    """Extract the most recently COMMITTED archived plan text from git
    history, skipping deletion commits."""
    log = subprocess.run(
        ["git", "-C", str(REPO), "log", "--format=%H", "--",
         f"docs/plans/archive/{PLAN_NAME}"],
        capture_output=True, text=True)
    for commit in log.stdout.split():
        show = subprocess.run(
            ["git", "-C", str(REPO), "show", f"{commit}:docs/plans/archive/{PLAN_NAME}"],
            capture_output=True, text=True)
        if show.returncode == 0 and show.stdout.strip():
            return show.stdout
    return None


def set_fm(text: str, field: str, value: str) -> str:
    new = re.sub(rf"^{field}: \S+", f"{field}: {value}", text, count=1, flags=re.M)
    if new == text and f"{field}: {value}" not in text:
        raise RuntimeError(f"frontmatter transform for {field} produced no modification")
    return new


def drop_fm(text: str, field: str) -> str:
    new = re.sub(rf"^{field}: \S+\n", "", text, count=1, flags=re.M)
    if new == text:
        raise RuntimeError(f"frontmatter drop for {field} produced no modification")
    return new


def convert_to_archive(text: str) -> str:
    return text.replace("../reports/", "../../reports/")


def convert_to_active(text: str) -> str:
    return text.replace("../../reports/", "../reports/")


def flip_one_conflict(text: str) -> str:
    """Flip exactly one row of one same-ID group (preferring MULTI-row
    groups so the ROWS disagree, not just the completed count) derived from
    the REAL row state; assert the text changed."""
    groups: dict[str, list[str]] = {}
    for line in text.splitlines():
        m = re.match(r"^- \[([ x])\] \*\*(T-\d+)\*\*", line)
        if m:
            groups.setdefault(m.group(2), []).append(m.group(1))
    ordered = sorted(groups.items(), key=lambda kv: -len(kv[1]))
    for tid, sts in ordered:
        if len(set(sts)) > 1:
            continue
        target = "[ ]" if set(sts) == {" "} else "[x]"
        new_state = "[x]" if target == "[ ]" else "[ ]"
        pat = re.compile(
            r"^- \[" + re.escape(target[1]) + r"\] (\*\*" + re.escape(tid) + r"\*\*)", re.M)
        new = pat.sub(lambda m: "- " + new_state + " " + m.group(1), text, count=1)
        if new != text:
            return new
    raise RuntimeError("no same-ID group available to flip into conflict")


def assert_modified(text: str, new: str) -> str:
    if new == text:
        raise RuntimeError("transform produced no modification")
    return new


def run_assert(plan_path: Path, location: str, repo_root: Path) -> tuple[int, str]:
    r = subprocess.run(
        [sys.executable, str(ASSERT), "--plan", str(plan_path),
         "--location", location, "--repo-root", str(repo_root)],
        capture_output=True, text=True,
    )
    return r.returncode, r.stdout + r.stderr


def materialize_reports(root: Path, archived_text: str) -> None:
    """Copy the report targets the archived plan links to (both link forms
    resolve into <repo>/docs/reports/<rest>) from the real repository."""
    for form in (ARCHIVE_REPORT, ACTIVE_REPORT):
        link_re = re.compile(r"\]\((" + re.escape(form) + r"[^)#\s]+)\)")
        body = re.sub(r"```.*?```", "", archived_text, flags=re.S)
        for m in link_re.finditer(body):
            rest = m.group(1).split("reports/", 1)[1]
            src = (REPO / "docs" / "reports" / rest).resolve()
            if src.is_file():
                dst = (root / "docs" / "reports" / rest).resolve()
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(src, dst)


def build_nav_sandbox(sandbox_root: Path, archived_text: str,
                      forget_auto_ac_update: bool) -> Path:
    """Sandbox repo with the archived plan, README link, ledger, and both
    module nav rows (auto-hir always correct; auto-ac FORGETS the archive
    update when forget_auto_ac_update - the R6-QA-02 negative control)."""
    root = sandbox_root
    for d in ("docs/plans/archive", "docs/specs/auto-hir", "docs/specs/auto-ac",
              "experimental/ac-core", ".autoos", "docs/reports"):
        (root / d).mkdir(parents=True, exist_ok=True)
    (root / "docs/plans/archive" / PLAN_NAME).write_text(archived_text, encoding="utf8")
    materialize_reports(root, archived_text)
    (root / "experimental/ac-core/README.md").write_text(
        "[plan](../../docs/plans/archive/741-ac-hir-native-core.md)", encoding="utf8")
    ledger = {
        "sections": [
            {"id": "reviews", "items": [
                {"id": "P741-8", "title": "r6 review", "status": "published",
                 "file": "docs/plans/archive/741-ac-hir-native-core.md"}
            ]},
        ]
    }
    (root / ".autoos/specs.json").write_text(
        json.dumps(ledger, ensure_ascii=False, indent=2), encoding="utf8")
    auto_ac = (STALE_NAV_ROW if forget_auto_ac_update else CANONICAL_NAV_ROW)
    (root / "docs/specs/auto-hir/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + CANONICAL_NAV_ROW + "\n", encoding="utf8")
    (root / "docs/specs/auto-ac/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + auto_ac + "\n", encoding="utf8")
    return root


def build_identity_repo(sandbox_root: Path, name: str, archived_text: str,
                        auto_ac_row: str, readme_text: str,
                        ledger_file: str) -> Path:
    """Sandbox repo with correct auto-hir nav and CUSTOM auto-ac nav row /
    README text / ledger review-file target (identity negatives, R7-QA-02)."""
    root = sandbox_root / name
    for d in ("docs/plans/archive", "docs/specs/auto-hir", "docs/specs/auto-ac",
              "experimental/ac-core", ".autoos", "docs/reports"):
        (root / d).mkdir(parents=True, exist_ok=True)
    (root / "docs/plans/archive" / PLAN_NAME).write_text(archived_text, encoding="utf8")
    materialize_reports(root, archived_text)
    (root / "experimental/ac-core/README.md").write_text(readme_text, encoding="utf8")
    ledger = {
        "sections": [
            {"id": "reviews", "items": [
                {"id": "P741-8", "title": "r6 review", "status": "published",
                 "file": ledger_file}
            ]},
        ]
    }
    (root / ".autoos/specs.json").write_text(
        json.dumps(ledger, ensure_ascii=False, indent=2), encoding="utf8")
    (root / "docs/specs/auto-hir/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + CANONICAL_NAV_ROW + "\n", encoding="utf8")
    (root / "docs/specs/auto-ac/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + auto_ac_row + "\n", encoding="utf8")
    return root


def build_active_control_repo(sandbox_root: Path, name: str, active_text: str) -> Path:
    """Sandbox repo in the ACTIVE lifecycle state: plan at docs/plans/ with
    active-form report links, executing+active-form nav rows, README link
    to the active plan, transitional archive-pointing ledger."""
    root = sandbox_root / name
    for d in ("docs/plans", "docs/specs/auto-hir", "docs/specs/auto-ac",
              "experimental/ac-core", ".autoos", "docs/reports"):
        (root / d).mkdir(parents=True, exist_ok=True)
    (root / "docs/plans" / PLAN_NAME).write_text(active_text, encoding="utf8")
    materialize_reports(root, active_text)
    (root / "experimental/ac-core/README.md").write_text(
        "[plan](../../docs/plans/741-ac-hir-native-core.md)", encoding="utf8")
    ledger = {
        "sections": [
            {"id": "reviews", "items": [
                {"id": "P741-8", "title": "r6 review", "status": "published",
                 "file": "docs/plans/archive/741-ac-hir-native-core.md"}
            ]},
        ]
    }
    (root / ".autoos/specs.json").write_text(
        json.dumps(ledger, ensure_ascii=False, indent=2), encoding="utf8")
    nav_row = ("| 741 | ac-hir-native-core | executing（r6 Phase6） | "
               "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")
    (root / "docs/specs/auto-hir/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + nav_row + "\n", encoding="utf8")
    (root / "docs/specs/auto-ac/plans.md").write_text(
        "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
        + nav_row + "\n", encoding="utf8")
    return root


def run_matrix(source_kind: str, base_text: str, sandbox_root: Path) -> list[str]:
    results: list[str] = []
    cases: list[tuple[str, Path, str, int, str | None, Path]] = []

    def add_valid(name: str, text: str, location: str, repo: Path) -> None:
        """Baseline control: valid by construction; the assertion run is the
        verdict."""
        dst = sandbox_root / name / ("plans" if location == "active" else "plans/archive")
        dst.mkdir(parents=True, exist_ok=True)
        p = dst / PLAN_NAME
        p.write_text(text, encoding="utf8")
        cases.append((name, p, location, 0, None, repo))

    def add(name: str, text: str, location: str, expect_exit: int,
            expect_diag: str | None, repo: Path = REPO) -> None:
        new = assert_modified(base_text, text)
        dst = sandbox_root / name / ("plans" if location == "active" else "plans/archive")
        dst.mkdir(parents=True, exist_ok=True)
        p = dst / PLAN_NAME
        p.write_text(new, encoding="utf8")
        cases.append((name, p, location, expect_exit, expect_diag, repo))

    def add_external(name: str, plan_path: Path, location: str, expect_exit: int,
                     expect_diag: str | None, repo_root: Path) -> None:
        cases.append((name, plan_path, location, expect_exit, expect_diag, repo_root))

    if source_kind.startswith("active"):
        # ---- partial-executing source matrix ----
        complete_text = re.sub(r"^- \[ \] (\*\*T-\d+\*\*)", r"- [x] \1", base_text, flags=re.M)
        complete_text = set_fm(complete_text, "status", "archived")
        total = re.search(r"^total_steps: (\d+)", complete_text, re.M).group(1)
        complete_text = set_fm(complete_text, "current_step", total)
        complete_text = convert_to_archive(complete_text)
        syn_repo = build_nav_sandbox(sandbox_root / "syn-complete", complete_text,
                                     forget_auto_ac_update=False)
        add_valid("valid-active-partial", base_text, "active", REPO)
        add_valid("valid-synthesized-fully-complete", complete_text, "archive", syn_repo)
        add("negative-current-step-zero", set_fm(base_text, "current_step", "0"),
            "active", 1, "current_step=0")
        add("negative-total-steps-mismatch", set_fm(base_text, "total_steps", "999"),
            "active", 1, "total_steps")
        add("negative-missing-status", drop_fm(base_text, "status"),
            "active", 1, "frontmatter missing required field: status")
        add("negative-nonnumeric-revision", set_fm(base_text, "plan_revision", "r7"),
            "active", 1, "is not numeric")
        add("negative-archived-status-while-partial", set_fm(base_text, "status", "archived"),
            "active", 1, "active mode requires status")
        add("negative-conflicting-duplicate", flip_one_conflict(base_text),
            "active", 1, "conflicting checkbox state")
        add("negative-active-with-archive-links", convert_to_archive(base_text),
            "active", 1, "broken from the REAL active parent")
        add("negative-broken-link-injection",
            base_text + "\n[broken](../reports/nonexistent-probe.md)",
            "active", 1, "broken from the REAL active parent")
        # Nav controls (R7-QA-01): REAL executed cases.
        nav_ok = build_nav_sandbox(sandbox_root / "nav-ok", complete_text,
                                   forget_auto_ac_update=False)
        nav_stale = build_nav_sandbox(sandbox_root / "nav-stale", complete_text,
                                      forget_auto_ac_update=True)
        add_external("nav-controls-all-correct",
                     nav_ok / "docs/plans/archive" / PLAN_NAME, "archive", 0, None, nav_ok)
        add_external("nav-controls-forgotten-nav-update",
                     nav_stale / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "auto-ac/plans.md:", nav_stale)
    else:
        # ---- fully-complete archived source matrix ----
        # All plan-body cases ride the sandbox repo built from the archived
        # text (the REAL repo is transitional-active during the phase, so
        # its ledger cannot resolve archive pointers); nav/identity controls
        # ride dedicated sandbox repos.
        repo_ok = build_nav_sandbox(sandbox_root / "arch-src", base_text,
                                    forget_auto_ac_update=False)
        active_control = set_fm(base_text, "status", "executing")
        active_control = convert_to_active(active_control)
        act_repo_ok = build_active_control_repo(
            sandbox_root / "act-src", "active-control", active_control)
        add_valid("valid-archive-complete", base_text, "archive", repo_ok)
        add_valid("valid-synthesized-active-from-archive", active_control, "active",
                  act_repo_ok)
        add("negative-conflicting-duplicate", flip_one_conflict(base_text),
            "archive", 1, "conflicting checkbox state", repo_ok)
        add("negative-current-step-off-by-one",
            re.sub(r"^current_step: (\d+)",
                   lambda m: f"current_step: {int(m.group(1)) - 1}",
                   base_text, count=1, flags=re.M),
            "archive", 1, "!= completed unique tasks", repo_ok)
        add("negative-executing-status-in-archive-location",
            set_fm(base_text, "status", "executing"),
            "archive", 1, "archive mode requires status", repo_ok)
        add("negative-stale-active-form-link-in-archive",
            convert_to_active(base_text),
            "archive", 1, "stale active-form link", repo_ok)
        add("negative-broken-link-injection",
            base_text + "\n[broken](../../reports/nonexistent-probe.md)",
            "archive", 1, "from the REAL archive parent", repo_ok)
        # Nav controls (R7-QA-01): REAL executed cases.
        nav_ok = build_nav_sandbox(sandbox_root / "nav-ok", base_text,
                                   forget_auto_ac_update=False)
        nav_stale = build_nav_sandbox(sandbox_root / "nav-stale", base_text,
                                      forget_auto_ac_update=True)
        add_external("nav-controls-all-correct",
                     nav_ok / "docs/plans/archive" / PLAN_NAME, "archive", 0, None, nav_ok)
        add_external("nav-controls-forgotten-nav-update",
                     nav_stale / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "auto-ac/plans.md:", nav_stale)
        # Identity controls (R7-QA-02): REAL executed cases.
        wrong_href_ac = "../../plans/archive/743-placeholder.md"
        wrong_href_readme = "../../docs/plans/archive/743-placeholder.md"
        wrong_ledger = "docs/plans/archive/743-placeholder.md"
        id_wrong_nav = build_identity_repo(
            sandbox_root / "id-wrong-nav", "wrong-nav", base_text,
            auto_ac_row=("| 741 | ac-hir-native-core | ✅（delivered，r6） | "
                         "[741-ac-hir-native-core.md](" + wrong_href_ac + ") | x |"),
            readme_text="[plan](../../docs/plans/archive/741-ac-hir-native-core.md)",
            ledger_file="docs/plans/archive/" + PLAN_NAME)
        add_external("identity-nav-wrong-target",
                     id_wrong_nav / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "does not match the canonical plan", id_wrong_nav)
        id_missing_readme = build_identity_repo(
            sandbox_root / "id-missing-readme", "missing-readme", base_text,
            auto_ac_row=CANONICAL_NAV_ROW,
            readme_text="# prototype (no plan link)",
            ledger_file="docs/plans/archive/" + PLAN_NAME)
        add_external("identity-readme-missing-reference",
                     id_missing_readme / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "missing required 741 plan reference", id_missing_readme)
        id_wrong_readme = build_identity_repo(
            sandbox_root / "id-wrong-readme", "wrong-readme", base_text,
            auto_ac_row=CANONICAL_NAV_ROW,
            # Contains the 741 filename (so the reference IS detected) but
            # resolves to a non-canonical directory (R7-QA-02 negative).
            readme_text="[plan](../../docs/plans/archive/wrong/741-ac-hir-native-core.md)",
            ledger_file="docs/plans/archive/" + PLAN_NAME)
        add_external("identity-readme-wrong-target",
                     id_wrong_readme / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "does not match the canonical plan target", id_wrong_readme)
        id_wrong_ledger = build_identity_repo(
            sandbox_root / "id-wrong-ledger", "wrong-ledger", base_text,
            auto_ac_row=CANONICAL_NAV_ROW,
            readme_text="[plan](../../docs/plans/archive/741-ac-hir-native-core.md)",
            ledger_file=wrong_ledger)
        add_external("identity-ledger-wrong-target",
                     id_wrong_ledger / "docs/plans/archive" / PLAN_NAME, "archive", 1,
                     "P741-8 does not match the canonical plan target", id_wrong_ledger)

    for name, plan_path, location, expect_exit, expect_diag, repo_root in cases:
        code, out = run_assert(plan_path, location, repo_root)
        ok = code == expect_exit
        if ok and expect_diag:
            ok = expect_diag in out
        line = f"{'PASS' if ok else 'FAIL'}  {name}: exit {code} (expected {expect_exit})"
        if expect_diag:
            line += f", diagnostic {'matched' if ok else 'MISMATCH'}: {expect_diag!r}"
        results.append(line)
        if not ok:
            results.append("      output tail: " + out.strip().replace("\n", "\n      ")[-400:])

    return results


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", choices=["auto", "active", "archive"], default="auto",
                    help="which plan state drives the matrix (auto = both available states)")
    args = ap.parse_args()

    matrices: list[tuple[str, str]] = []
    if args.source in ("auto", "active") and ACTIVE_PLAN.is_file():
        matrices.append(("active (partial executing)", ACTIVE_PLAN.read_text(encoding="utf8")))
    if args.source in ("auto", "archive"):
        if ARCHIVE_PLAN.is_file():
            matrices.append(("archive (real archived file)", ARCHIVE_PLAN.read_text(encoding="utf8")))
        else:
            extracted = git_archived_text()
            if extracted:
                matrices.append(("archive (replayed from git history)", extracted))
            elif args.source == "archive":
                print(f"FAIL: archive source unavailable "
                      f"(active={ACTIVE_PLAN.exists()}, archive={ARCHIVE_PLAN.exists()}, "
                      f"git history empty)")
                sys.exit(1)
    if not matrices:
        print(f"FAIL: no plan source available (active={ACTIVE_PLAN.exists()}, "
              f"archive={ARCHIVE_PLAN.exists()})")
        sys.exit(1)

    sandbox_root = Path(tempfile.mkdtemp(prefix="p741p8-fixture-"))
    all_results: list[str] = []
    total_executed = 0
    try:
        for source_kind, base_text in matrices:
            print(f"=== source: {source_kind} ===")
            results = run_matrix(source_kind, base_text, sandbox_root)
            all_results.extend(results)
            total_executed += len(results)  # R7-QA-01: totals from execution records
    finally:
        shutil.rmtree(sandbox_root, ignore_errors=True)

    for line in all_results:
        print(line)
    bad = [r for r in all_results if r.startswith("FAIL")]
    if bad:
        print(f"FIXTURES-FAILED ({len(bad)} case(s))")
        sys.exit(1)
    print(f"FIXTURES-ALL-PASS ({total_executed} case(s) executed)")


if __name__ == "__main__":
    main()

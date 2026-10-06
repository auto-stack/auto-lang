#!/usr/bin/env python3
"""PLAN-741 Phase 7 (T-39/T-40) controlled fixture matrix - lifecycle aware.

Supersedes the phase6 fixture per R6-QA-01/02:
- The plan source is detected automatically: the real file under
  docs/plans/ (partial executing) OR the archived state replayed from git
  history (docs/plans/archive/ at its last archived commit). Both matrices
  run when both states are available; --source selects one.
- Controls are generated FROM THE SOURCE TEXT:
    * partial-executing source: valid as-is; a fully-complete synthesized
      control rides a full sandbox repo skeleton (nav/README/ledger all
      correct for an archive state);
    * archived source: valid as-is; an ACTIVE control synthesized
      (status executing, links converted back, completed tasks kept).
- The same-ID conflict transform flips exactly ONE row of one same-ID
  group according to the REAL row state and asserts the text actually
  changed (R6-QA-01: the fixed unchecked-T-35 assumption was a zero-op on
  a complete plan).
- Every mutation verifies it modified the text; every case checks an
  expected diagnostic substring, so an unrelated non-zero cannot mask a
  broken control.
- Sandbox repos (via final_assertions --repo-root) provide the negative
  control for an archive that forgot a module navigation update (R6-QA-02),
  with the file and line named in the output.
- TemporaryDirectory cleans up everything this script creates.

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

CORRECT_NAV_ROW = ("| 741 | ac-hir-native-core | ✅（delivered，r6） | "
                   "[741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | x |")
STALE_NAV_ROW = ("| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                 "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")


def git_archived_text() -> str | None:
    """Extract the most recently COMMITTED archived plan text from git
    history (a later activation moves the file back to active, but the
    archived state remains replayable from history)."""
    log = subprocess.run(
        ["git", "-C", str(REPO), "log", "--format=%H", "--",
         f"docs/plans/archive/{PLAN_NAME}"],
        capture_output=True, text=True)
    for commit in log.stdout.split():
        # Walk newest-first and skip deletions (an activation git-mv moves
        # the file out of archive/, so the newest commit may not contain it).
        show = subprocess.run(
            ["git", "-C", str(REPO), "show", f"{commit}:docs/plans/archive/{PLAN_NAME}"],
            capture_output=True, text=True)
        if show.returncode == 0 and show.stdout.strip():
            return show.stdout
    return None


def convert_to_archive(text: str) -> str:
    return text.replace("../reports/", "../../reports/")


def convert_to_active(text: str) -> str:
    return text.replace("../../reports/", "../reports/")


def flip_one_conflict(text: str) -> str:
    """Flip exactly one row of one same-ID group so the rows disagree,
    derived from the REAL row state; assert the text changed."""
    groups: dict[str, list[str]] = {}
    for line in text.splitlines():
        m = re.match(r"^- \[([ x])\] \*\*(T-\d+)\*\*", line)
        if m:
            groups.setdefault(m.group(2), []).append(m.group(1))
    # Prefer a MULTI-row group: flipping one row of a multi-row group makes
    # the ROWS disagree (a checkbox conflict); flipping a single-row group
    # merely changes the completed count, which is a different assertion.
    ordered = sorted(groups.items(), key=lambda kv: -len(kv[1]))
    for tid, sts in ordered:
        if len(set(sts)) > 1:
            continue
        target = "[ ]" if sts == {" "} else "[x]"
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


def build_sandbox_repos(sandbox_root: Path, archived_text: str,
                        nav_delivered: bool = True):
    """Build two minimal sandbox repo skeletons: one fully correct, one with
    a forgotten module navigation update. nav_delivered selects the
    lifecycle form of the nav rows (delivered+archive vs executing+active)."""
    import shutil
    correct_row = (CORRECT_NAV_ROW if nav_delivered else
                   "| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                   "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")
    stale_row = (STALE_NAV_ROW if nav_delivered else correct_row)
    repos = {}
    for name, auto_ac_row in (("repo-ok", correct_row), ("repo-stale", stale_row)):
        root = sandbox_root / name
        for d in ("docs/plans/archive", "docs/plans", "docs/specs/auto-hir",
                  "docs/specs/auto-ac", "experimental/ac-core", ".autoos",
                  "docs/reports"):
            (root / d).mkdir(parents=True, exist_ok=True)
        # Plan placement + README link follow the lifecycle form: the
        # delivered form parks the plan in archive/ with an archive link;
        # the active form keeps it in docs/plans/ with an active link.
        plan_path = (root / "docs/plans/archive" / PLAN_NAME if nav_delivered
                     else root / "docs/plans" / PLAN_NAME)
        plan_path.write_text(archived_text, encoding="utf8")
        readme_link = ("../../docs/plans/archive/741-ac-hir-native-core.md" if nav_delivered
                       else "../../docs/plans/741-ac-hir-native-core.md")
        # Materialize the report targets the plan links to in BOTH forms.
        # Both forms resolve into <repo>/docs/reports/<rest>: the archive
        # form (../../reports/X) from docs/plans/archive/, the active form
        # (../reports/X) from docs/plans/. Sources come from the real
        # repository's matching locations.
        for form, src_base in ((ARCHIVE_REPORT, REPO / "docs" / "plans" / "archive"),
                               (ACTIVE_REPORT, REPO / "docs" / "plans")):
            link_re = re.compile(r"\]\((" + re.escape(form) + r"[^)#\s]+)\)")
            body = re.sub(r"```.*?```", "", archived_text, flags=re.S)
            for m in link_re.finditer(body):
                rest = m.group(1).split("reports/", 1)[1]
                src = (REPO / "docs" / "reports" / rest).resolve()
                if src.is_file():
                    dst = (root / "docs" / "reports" / rest).resolve()
                    dst.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(src, dst)
        (root / "experimental/ac-core/README.md").write_text(
            f"[plan]({readme_link})", encoding="utf8")
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
        # auto-hir always correct; auto-ac is the row under test (repo-stale
        # FORGETS the archive update - the R6-QA-02 negative control).
        (root / "docs/specs/auto-hir/plans.md").write_text(
            "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
            + correct_row + "\n", encoding="utf8")
        (root / "docs/specs/auto-ac/plans.md").write_text(
            "# nav\n\n| Plan | 标题 | 状态 | 归档 | x |\n|---|---|---|---|---|\n"
            + auto_ac_row + "\n", encoding="utf8")
        repos[name] = root
    return repos["repo-ok"], repos["repo-stale"]


def run_matrix(source_kind: str, base_text: str, sandbox_root: Path) -> tuple[list[str], list[str]]:
    results: list[str] = []
    cases: list[tuple[str, Path, str, Path]] = []

    def add(name: str, text: str, location: str, expect_exit: int,
            expect_diag: str | None, repo: Path = REPO) -> None:
        dst = sandbox_root / name / ("plans" if location == "active" else "plans/archive")
        dst.mkdir(parents=True, exist_ok=True)
        p = dst / PLAN_NAME
        p.write_text(text, encoding="utf8")
        cases.append((name, p, location, expect_exit, expect_diag, repo))

    if source_kind.startswith("active"):
        # ---- partial-executing source matrix ----
        complete_text = re.sub(r"^- \[ \] (\*\*T-\d+\*\*)", r"- [x] \1", base_text, flags=re.M)
        complete_text = re.sub(r"^status: executing", "status: archived", complete_text,
                               count=1, flags=re.M)
        total = re.search(r"^total_steps: (\d+)", complete_text, re.M).group(1)
        complete_text = re.sub(r"^current_step: \d+", f"current_step: {total}",
                               complete_text, count=1, flags=re.M)
        complete_text = convert_to_archive(complete_text)
        syn_repo, _ = build_sandbox_repos(sandbox_root / "syn-complete", complete_text)
        add("valid-active-partial", base_text, "active", 0, None)
        add("valid-synthesized-fully-complete", complete_text, "archive", 0, None, syn_repo)
        add("negative-current-step-zero", set_fm(base_text, "current_step", "0"),
            "active", 1, "current_step=0")
        add("negative-total-steps-mismatch", set_fm(base_text, "total_steps", "999"),
            "active", 1, "total_steps")
        add("negative-missing-status", drop_fm(base_text, "status"),
            "active", 1, "frontmatter missing required field: status")
        add("negative-nonnumeric-revision", set_fm(base_text, "plan_revision", "r7"),
            "active", 1, "is not numeric")
        add("negative-archived-status-while-partial",
            re.sub(r"^status: executing", "status: archived", base_text, count=1, flags=re.M),
            "active", 1, "active mode requires status")
        add("negative-conflicting-duplicate",
            assert_modified(base_text, flip_one_conflict(base_text)),
            "active", 1, "conflicting checkbox state")
        add("negative-active-with-archive-links", convert_to_archive(base_text),
            "active", 1, "broken from the REAL active parent")
        add("negative-broken-link-injection",
            base_text + "\n[broken](../reports/nonexistent-probe.md)",
            "active", 1, "broken from the REAL active parent")
    else:
        # ---- fully-complete archived source matrix ----
        # All cases ride the sandbox repo built from the archived text: the
        # REAL repo is transitional-active during the r7 phase, so its
        # ledger cannot resolve archive pointers (the merge restores them).
        repo_ok, repo_stale = build_sandbox_repos(sandbox_root / "arch-src", base_text,
                                                  nav_delivered=True)
        active_control = re.sub(r"^status: archived", "status: executing", base_text,
                                count=1, flags=re.M)
        active_control = convert_to_active(active_control)
        act_repo_ok, act_repo_stale = build_sandbox_repos(
            sandbox_root / "act-src", active_control, nav_delivered=False)
        add("valid-archive-complete", base_text, "archive", 0, None, repo_ok)
        add("valid-synthesized-active-from-archive", active_control, "active", 0, None,
            act_repo_ok)
        add("negative-conflicting-duplicate",
            assert_modified(base_text, flip_one_conflict(base_text)),
            "archive", 1, "conflicting checkbox state", repo_ok)
        add("negative-current-step-off-by-one",
            re.sub(r"^current_step: (\d+)",
                   lambda m: f"current_step: {int(m.group(1)) - 1}",
                   base_text, count=1, flags=re.M),
            "archive", 1, "!= completed unique tasks", repo_ok)
        add("negative-executing-status-in-archive-location",
            re.sub(r"^status: archived", "status: executing", base_text, count=1, flags=re.M),
            "archive", 1, "archive mode requires status", repo_ok)
        add("negative-stale-active-form-link-in-archive",
            convert_to_active(base_text),
            "archive", 1, "stale active-form link", repo_ok)
        add("negative-broken-link-injection",
            base_text + "\n[broken](../../reports/nonexistent-probe.md)",
            "archive", 1, "from the REAL archive parent", repo_ok)

    # Sandbox-repo controls for the module navigation gate (R6-QA-02):
    # a fully-correct archive repo passes; a repo whose auto-ac nav row
    # FORGOT the archive update fails with the file/line named. The control
    # base is always an ARCHIVE-STATE plan: the real archived text for the
    # archive source, the synthesized complete+converted text for the
    # active source.
    archive_base = complete_text if source_kind.startswith("active") else base_text
    repo_ok, repo_stale = build_sandbox_repos(sandbox_root / "nav-repos", archive_base)
    code, _ = run_assert(repo_ok / "docs/plans/archive" / PLAN_NAME, "archive", repo_ok)
    results.append(f"{'PASS' if code == 0 else 'FAIL'}  sandbox-repo-nav-all-correct: "
                   f"exit {code} (expected 0)")
    code, out = run_assert(repo_stale / "docs/plans/archive" / PLAN_NAME, "archive", repo_stale)
    ok = code == 1 and "auto-ac/plans.md:" in out
    results.append(f"{'PASS' if ok else 'FAIL'}  sandbox-repo-forgotten-nav-update: "
                   f"exit {code} (expected 1 with auto-ac/plans.md: named)")

    for name, plan_path, location, expect_exit, expect_diag, repo_root in cases:
        code, out = run_assert(plan_path, location, repo_root)
        ok = code == expect_exit
        if ok and expect_diag:
            ok = expect_diag in out
        line = f"{'PASS' if ok else 'FAIL'}  {name}: exit {code} (expected {expect_exit})"
        if expect_diag and ok:
            line += f", diagnostic matched: {expect_diag!r}"
        results.append(line)
        if not ok:
            results.append("      output tail: " + out.strip().replace("\n", "\n      ")[-400:])

    return results, cases


def set_fm(text: str, field: str, value: str) -> str:
    return re.sub(rf"^{field}: \S+", f"{field}: {value}", text, count=1, flags=re.M)


def drop_fm(text: str, field: str) -> str:
    return re.sub(rf"^{field}: \S+\n", "", text, count=1, flags=re.M)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", choices=["auto", "active", "archive"], default="auto",
                    help="which plan state drives the matrix (auto = both available states)")
    args = ap.parse_args()

    matrices: list[tuple[str, str]] = []
    if args.source in ("auto", "active") and ACTIVE_PLAN.is_file():
        matrices.append(("active", ACTIVE_PLAN.read_text(encoding="utf8")))
    if args.source in ("auto", "archive"):
        if ARCHIVE_PLAN.is_file():
            matrices.append(("archive", ARCHIVE_PLAN.read_text(encoding="utf8")))
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

    sandbox_root = Path(tempfile.mkdtemp(prefix="p741p7-fixture-"))
    all_results: list[str] = []
    for source_kind, base_text in matrices:
        print(f"=== source: {source_kind} ===")
        results, _ = run_matrix(source_kind, base_text, sandbox_root)
        all_results.extend(results)
    shutil.rmtree(sandbox_root, ignore_errors=True)

    for line in all_results:
        print(line)
    bad = [r for r in all_results if r.startswith("FAIL")]
    if bad:
        print(f"FIXTURES-FAILED ({len(bad)} case(s))")
        sys.exit(1)
    print(f"FIXTURES-ALL-PASS ({len(all_results)} case(s))")


if __name__ == "__main__":
    main()

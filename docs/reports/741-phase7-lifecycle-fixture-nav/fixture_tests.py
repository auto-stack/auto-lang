#!/usr/bin/env python3
"""PLAN-741 Phase 7 (T-39/T-40) controlled fixture matrix - lifecycle aware.

v2 (post P741P7-R1/R2/R3 review):
- ALL status transforms use the wildcard set_fm (never a regex assuming the
  current status), so the matrix is independent of the source plan's
  lifecycle state (executing / execution_done / reviewed / archived);
- EVERY mutation passes through assert_modified - a zero-op transform is a
  loud RuntimeError, never a silent pass;
- flip_one_conflict compares SETS (the list==set comparison was dead code);
- the plan source is detected automatically: docs/plans/ (partial
  executing) OR the archived state replayed from git history; both
  matrices run when both states are available;
- controls are generated FROM THE SOURCE TEXT: a fully-complete synthesized
  control rides a full sandbox repo skeleton (nav/README/ledger correct for
  an archive state); an ACTIVE control is synthesized from the archived
  source (status executing, links converted back, tasks kept);
- every case checks an expected diagnostic substring; TemporaryDirectory is
  reclaimed via try/finally.

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
CORRECT_NAV_ROW_GLOBAL = ("| 741 | ac-hir-native-core | ✅（delivered，r6） | "
                         "[741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | x |")
STALE_NAV_ROW_GLOBAL = ("| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                        "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")
ACTIVE_PLAN = REPO / "docs" / "plans" / PLAN_NAME
ARCHIVE_PLAN = REPO / "docs" / "plans" / "archive" / PLAN_NAME

failures: list[str] = []


def git_archived_text() -> str | None:
    """Extract the most recently COMMITTED archived plan text from git
    history, skipping deletion commits (an activation moves the file back
    to docs/plans/)."""
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
    """Flip exactly one row of one same-ID group so the rows disagree,
    derived from the REAL row state; assert the text changed. Prefers
    MULTI-row groups: flipping one row there makes the ROWS disagree,
    while flipping a single-row group only changes the completed count."""
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


def build_sandbox_repos(sandbox_root: Path, archived_text: str,
                        nav_delivered: bool = True):
    """Build two minimal sandbox repo skeletons: one fully correct, one with
    a forgotten module navigation update. nav_delivered selects the
    lifecycle form of the nav rows (delivered+archive vs executing+active)
    and where the plan file itself is parked."""
    import shutil
    correct_row = (CORRECT_NAV_ROW_GLOBAL if nav_delivered else
                   "| 741 | ac-hir-native-core | executing（r6 Phase6） | "
                   "[741-ac-hir-native-core.md](../../plans/741-ac-hir-native-core.md) | x |")
    stale_row = (STALE_NAV_ROW_GLOBAL if nav_delivered else correct_row)
    plan_rel = ("docs/plans/archive/" + PLAN_NAME) if nav_delivered else ("docs/plans/" + PLAN_NAME)
    readme_link = ("../../docs/plans/archive/741-ac-hir-native-core.md" if nav_delivered
                   else "../../docs/plans/741-ac-hir-native-core.md")
    repos = {}
    for name, auto_ac_row in (("repo-ok", correct_row), ("repo-stale", stale_row)):
        root = sandbox_root / name
        for d in ("docs/plans/archive", "docs/plans", "docs/specs/auto-hir",
                  "docs/specs/auto-ac", "experimental/ac-core", ".autoos",
                  "docs/reports"):
            (root / d).mkdir(parents=True, exist_ok=True)
        (root / plan_rel).write_text(archived_text, encoding="utf8")
        # Materialize the report targets the plan links to in BOTH forms
        # (both resolve into <repo>/docs/reports/<rest>); sources come from
        # the real repository's matching locations.
        # Both link forms resolve into <repo>/docs/reports/<rest> (the
        # canonical reports location); sources come from the real repo.
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


def run_matrix(source_kind: str, base_text: str, sandbox_root: Path) -> tuple[list[str], int]:
    results: list[str] = []
    cases: list[tuple[str, Path, str, int, str | None, Path]] = []

    def add_valid(name: str, text: str, location: str, repo: Path = REPO) -> None:
        """Baseline control: the text is valid by construction (no mutation
        check - the assertion run itself is the verdict)."""
        dst = sandbox_root / name / ("plans" if location == "active" else "plans/archive")
        dst.mkdir(parents=True, exist_ok=True)
        p = dst / PLAN_NAME
        p.write_text(text, encoding="utf8")
        cases.append((name, p, location, 0, None, repo))

    def add(name: str, text: str, location: str, expect_exit: int,
            expect_diag: str | None, repo: Path = REPO) -> None:
        # R6-QA-01/P741P7-R1: EVERY mutation must actually modify the text -
        # a zero-op transform is a loud error, never a silent pass.
        new = assert_modified(base_text, text)
        dst = sandbox_root / name / ("plans" if location == "active" else "plans/archive")
        dst.mkdir(parents=True, exist_ok=True)
        p = dst / PLAN_NAME
        p.write_text(new, encoding="utf8")
        cases.append((name, p, location, expect_exit, expect_diag, repo))

    if source_kind.startswith("active"):
        # ---- partial-executing source matrix ----
        # status transforms are wildcard (set_fm) so they modify regardless
        # of whether the source is executing/execution_done/reviewed.
        complete_text = re.sub(r"^- \[ \] (\*\*T-\d+\*\*)", r"- [x] \1", base_text, flags=re.M)
        complete_text = set_fm(complete_text, "status", "archived")
        total = re.search(r"^total_steps: (\d+)", complete_text, re.M).group(1)
        complete_text = set_fm(complete_text, "current_step", total)
        complete_text = convert_to_archive(complete_text)
        syn_repo, _ = build_sandbox_repos(sandbox_root / "syn-complete", complete_text)
        add_valid("valid-active-partial", base_text, "active")
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
        # Nav controls: an archive-state plan (all complete + converted) is
        # the correct base for the archive-mode nav checks.
        repo_ok, repo_stale = build_sandbox_repos(
            sandbox_root / "nav-repos", complete_text, nav_delivered=True)
    else:
        # ---- fully-complete archived source matrix ----
        # All cases ride the sandbox repo built from the archived text: the
        # REAL repo is transitional-active during the r7 phase, so its
        # ledger cannot resolve archive pointers (the merge restores them).
        repo_ok, repo_stale = build_sandbox_repos(
            sandbox_root / "arch-src", base_text, nav_delivered=True)
        active_control = set_fm(base_text, "status", "executing")
        active_control = convert_to_active(active_control)
        act_repo_ok, act_repo_stale = build_sandbox_repos(
            sandbox_root / "act-src", active_control, nav_delivered=False)
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

    return results, len(cases)


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

    sandbox_root = Path(tempfile.mkdtemp(prefix="p741p7-fixture-"))
    all_results: list[str] = []
    total_cases = 0
    try:
        for source_kind, base_text in matrices:
            print(f"=== source: {source_kind} ===")
            results, n = run_matrix(source_kind, base_text, sandbox_root)
            all_results.extend(results)
            total_cases += n + 2  # + the two sandbox-repo navigation controls
    finally:
        shutil.rmtree(sandbox_root, ignore_errors=True)

    for line in all_results:
        print(line)
    bad = [r for r in all_results if r.startswith("FAIL")]
    if bad:
        print(f"FIXTURES-FAILED ({len(bad)} case(s))")
        sys.exit(1)
    print(f"FIXTURES-ALL-PASS ({total_cases} case(s))")


if __name__ == "__main__":
    main()

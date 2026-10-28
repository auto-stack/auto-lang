#!/usr/bin/env python3
"""PLAN-867 T-01: freeze the pre-split test-ID universe (the closure denominator).

Enumerates every test face that existed in the pre-split language workspace at
commit 464e2316b (= abd34a355^, the parent of PLAN-831 T-06 切源) using real
compiled listings (nextest list --message-format json / libtest -- --list).
The selector table below is transcribed verbatim from THAT commit's
.cargo/config.toml and .config/nextest*.toml; post-split tier shapes
(auto-ui t/tu/ti/tn, auto-man t/tc/ti) are applied to the same tree so both
sides of the split compare like for like. man-tcore is recorded not_applicable:
its core/ workspace did not exist before the split.

Outputs (deterministic; reruns on the same tree are byte-identical):
  <out>/<face>-ids.txt        sorted unique "suite\ttest" lines (all cases)
  <out>/<face>-selected.txt   sorted subset the face actually selects
  <out>/universe-ids.txt      sorted union of every face's ids
  <out>/manifest.json         commit, toolchain, exact commands, hashes, counts

Raw nextest/libtest output stays under --raw (local evidence, not committed).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

SCHEMA = "pre-split-universe-manifest/1"
PINNED_COMMIT = "464e2316b"  # = abd34a355^; manifest records the full sha
HERE = Path(__file__).resolve().parent.parent

# Verbatim selectors from 464e2316b .cargo/config.toml [alias] unless noted.
LANG_PACKAGES = ["-p", "auto-lang", "-p", "auto-lang-core",
                 "-p", "auto-ui-compiler", "-p", "auto-ui-dialect", "-p", "auto-ui-runtime"]
UI_FIVE = ["-p", "auto-ui-compiler", "-p", "auto-ui-dialect", "-p", "auto-ui-runtime",
           "-p", "auto-ui-tools", "-p", "shell-pack"]
MAN_NINE = ["-p", "auto", "-p", "auto-man", "-p", "auto-man-runtime", "-p", "auto-man-core",
            "-p", "auto-package-lock", "-p", "auto-package-resolver", "-p", "auto-toolchain-core",
            "-p", "auto-man-assets", "-p", "auto-tool-client"]
MAN_SEVEN = ["-p", "auto-man-runtime", "-p", "auto-man-core", "-p", "auto-package-lock",
             "-p", "auto-package-resolver", "-p", "auto-toolchain-core", "-p", "auto-man-assets",
             "-p", "auto-tool-client"]

NEXTEST_FACES: dict[str, list[str]] = {
    # old `t` alias verbatim (joint daily face, old default-filter applies)
    "t": [*LANG_PACKAGES, "--lib", "--test", "schema_drift", "--test", "docs_gen",
          "--test", "component_registry_test", "--features", "auto-lang/ui-iced,auto-lang/test-cases"],
    # old `tf` alias verbatim
    "tf": [*LANG_PACKAGES, "--lib", "--test", "schema_drift", "--test", "docs_gen",
           "--test", "component_registry_test", "--config-file", ".config/nextest-full.toml"],
    "tv": ["-p", "auto-lang", "--lib", "--profile", "tv", "--features", "test-cases"],
    "tt": [*LANG_PACKAGES, "--lib", "--features", "auto-lang/test-trans,auto-lang/test-cases"],
    "tb": [*LANG_PACKAGES, "--lib", "--features", "auto-lang/test-book"],
    "taa": [*LANG_PACKAGES, "--lib", "--features", "auto-lang/test-aavm",
            "--config-file", ".config/nextest-full.toml"],
    "ta": [*LANG_PACKAGES, "--lib", "--features",
           "auto-lang/test-aavm,auto-lang/test-trans,auto-lang/test-book",
           "--config-file", ".config/nextest-full.toml"],
    "t3": [*LANG_PACKAGES, "--lib", "--features",
           "auto-lang/test-aavm,auto-lang/test-trans,auto-lang/test-book",
           "--config-file", ".config/nextest-t3.toml"],
    "th": ["-p", "auto-lang", "--lib", "--features", "test-http-e2e", "http_e2e"],
    # post-split auto-ui `t` shape on the pre-split five UI packages
    "ui-t": [*UI_FIVE, "--lib"],
    # post-split auto-ui `ti` shape (targets verified present at 464e2316b)
    "ui-ti": ["-p", "auto-ui-tools", "-p", "auto-ui-runtime", "--test", "legacy_build_semantics",
              "--test", "lifecycle", "--test", "resources_manifest", "--test", "tool_process",
              "--test", "osconfig_integration"],
    # post-split auto-man shapes on the pre-split nine man packages
    "man-t": [*MAN_NINE, "--lib", "--bin", "auto"],
    "man-tc": [*MAN_SEVEN, "--lib"],
    "man-ti": ["-p", "auto", "--test", "plan816_cli_driver", "--test", "cmd_ui",
               "--test", "autodesk_client_smoke", "-p", "auto-man-runtime",
               "--test", "plan816_t03_non_ui_service", "--test", "plan816_t03_ui_default_fallback",
               "--test", "plan816_t03_ui_inject_explicit_first", "-p", "auto-man",
               "--test", "plan803_asset_facade_tests", "--test", "plan805_toolchain_parity",
               "--test", "plan805_type_identity", "--test", "plan809_management_core",
               "--test", "plan811_media_consumers", "--test", "plan815_t04_compat",
               "--test", "plan816_t03_create_jet_default", "--test", "plan816_t03_legacy_util_paths",
               "--test", "plan816_t03_sidecar_request", "--test", "plan816_t03_ui_inject_compat"],
}

# libtest faces: (label, package-attribution, cargo-test argv). The pre-split
# alias is verbatim; ui-tu/ui-tn are the post-split shapes on the same tree.
LIBTEST_FACES: dict[str, tuple[str, list[str]]] = {
    # old `tu` alias verbatim (auto-lang contributes no ui_gen cases by then)
    "tu": ("auto-ui-compiler",
           ["-p", "auto-ui-compiler", "-p", "auto-lang", "--lib",
            "--features", "auto-lang/ui-iced", "ui_gen::", "--", "--list"]),
    "ui-tu": ("auto-ui-compiler",
              ["-p", "auto-ui-compiler", "--lib", "ui_gen::", "--", "--list"]),
    "ui-tn": ("auto-ui-runtime",
              ["-p", "auto-ui-runtime", "--features", "native-dnd,test-native-dock",
               "--test", "native_dnd_e2e", "--", "--list"]),
}

NOT_APPLICABLE = {
    "man-tcore": "core/ workspace did not exist at 464e2316b (created by the split, "
                 "PLAN-832); its cases are carried by man-t (--bin auto) and man-ti faces.",
    "ui faces vs old aliases": "ui t/tu/ti/tn aliases postdate the split; their selector "
                               "shapes are applied to the pre-split packages instead.",
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd: Path) -> tuple[int, bytes, bytes]:
    proc = subprocess.run(cmd, cwd=str(cwd), capture_output=True)
    return proc.returncode, proc.stdout, proc.stderr


def parse_nextest(payload: bytes) -> tuple[set[str], set[str], int]:
    data = json.loads(payload)
    ids, selected, ignored = set(), set(), 0
    for suite, value in data["rust-suites"].items():
        for name, case in value["testcases"].items():
            identity = suite + "\t" + name
            ids.add(identity)
            if case["ignored"]:
                ignored += 1
            if case["filter-match"]["status"] == "matches" and not case["ignored"]:
                selected.add(identity)
    return ids, selected, ignored


def parse_libtest(payload: bytes, package: str) -> tuple[set[str], int]:
    ids, ignored = set(), 0
    for line in payload.decode("utf-8", "replace").splitlines():
        line = line.rstrip()
        if line.endswith(": test"):
            ids.add(package + "\t" + line[: -len(": test")].strip())
        elif line.endswith(": ignore"):
            name = line[: -len(": ignore")].strip()
            ids.add(package + "\t" + name)
            ignored += 1
    return ids, ignored


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, required=True,
                        help="read-only pre-split worktree (default pin 464e2316b)")
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("pre-split-universe"))
    parser.add_argument("--raw", type=Path, required=True,
                        help="local directory for raw listing output (never committed)")
    parser.add_argument("--faces", nargs="*", help="subset of faces (default: all)")
    parser.add_argument("--pin", default=PINNED_COMMIT,
                        help="required commit prefix of --root (default 464e2316b = abd34a355^)")
    parser.add_argument("--witness", action="store_true",
                        help="mark this run as a last-compiling-witness enumeration: the faces "
                             "selected are compile-broken at the pinned commit (PLAN-818 T-02 left "
                             "probe_adapter_tests ungated until Plan 847 fixed it post-split) and are "
                             "enumerated at the last pre-split commit where they compiled")
    args = parser.parse_args(argv)

    root = args.root.resolve()
    head = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
    if not head.startswith(args.pin[:9]):
        raise SystemExit(f"refusing: {root} is {head[:9]}, expected {args.pin}")
    dirty = subprocess.run(["git", "-C", str(root), "status", "--porcelain"], capture_output=True, text=True)
    if dirty.stdout.strip():
        raise SystemExit(f"refusing: pre-split tree is dirty:\n{dirty.stdout}")

    selected_faces = args.faces or [*NEXTEST_FACES, *LIBTEST_FACES]
    unknown = [f for f in selected_faces if f not in NEXTEST_FACES and f not in LIBTEST_FACES]
    if unknown:
        raise SystemExit(f"unknown faces: {unknown}")
    args.output.mkdir(parents=True, exist_ok=True)
    args.raw.mkdir(parents=True, exist_ok=True)

    manifest = {"schema": SCHEMA, "commit": head, "root": str(root),
                "toolchain": {}, "faces": {}, "not_applicable": NOT_APPLICABLE,
                "witness_runs": []}
    if args.witness:
        manifest_path = args.output / "manifest.json"
        if not manifest_path.is_file():
            raise SystemExit("--witness requires an existing primary manifest to merge into")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest.setdefault("witness_runs", []).append(
            {"commit": head, "root": str(root), "faces": list(selected_faces)})
    for tool, cmd in (("rustc", ["rustc", "-V"]), ("cargo", ["cargo", "-V"]),
                      ("nextest", ["cargo", "nextest", "--version"]), ("git", ["git", "-C", str(root), "--version"])):
        code, out, _ = run(cmd, root)
        manifest["toolchain"][tool] = out.decode(errors="replace").strip() if code == 0 else f"error {code}"

    broken: list[str] = []
    for face in selected_faces:
        started = time.time()
        if face in NEXTEST_FACES:
            kind, cmd = "nextest", ["cargo", "nextest", "list", *NEXTEST_FACES[face], "--message-format", "json"]
            code, out, err = run(cmd, root)
            raw_name = args.raw / f"{face}.json"
            raw_name.write_bytes(out)
            parsed = parse_nextest(out) if code == 0 else None
        else:
            package, argv = LIBTEST_FACES[face]
            kind, cmd = "libtest", ["cargo", "test", *argv]
            code, out, err = run(cmd, root)
            raw_name = args.raw / f"{face}.list"
            raw_name.write_bytes(out)
            parsed = None
            if code == 0:
                ids, ignored = parse_libtest(out, package)
                parsed = (ids, set(ids), ignored)  # libtest --list prints only the filtered face
        if code:
            (args.raw / f"{face}.stderr").write_bytes(err)
            if args.witness:
                raise SystemExit(f"{face}: compile failed at the WITNESS commit too ({code}); "
                                 f"the witness analysis is wrong — do not weaken the denominator: "
                                 f"{(args.raw / f'{face}.stderr').resolve()}")
            manifest["faces"][face] = {"kind": kind, "command": cmd, "exit": code,
                                       "compile_broken": True,
                                       "stderr_sha256": sha256(err),
                                       "stderr_evidence": str((args.raw / f"{face}.stderr").resolve())}
            broken.append(face)
            print(f"{face}: COMPILE BROKEN ({code}); evidence={raw_name.with_suffix('.stderr')}", flush=True)
            continue
        ids, selected, ignored = parsed
        ids_path = args.output / f"{face}-ids.txt"
        selected_path = args.output / f"{face}-selected.txt"
        ids_path.write_text("".join(line + "\n" for line in sorted(ids)), encoding="utf-8")
        selected_path.write_text("".join(line + "\n" for line in sorted(selected)), encoding="utf-8")
        entry = {
            "kind": kind, "command": cmd, "exit": code,
            "ids": len(ids), "selected": len(selected), "ignored": ignored,
            "raw_sha256": sha256(raw_name.read_bytes()),
            "ids_sha256": sha256(ids_path.read_bytes()),
            "selected_sha256": sha256(selected_path.read_bytes()),
        }
        previous = manifest["faces"].get(face, {})
        if args.witness:
            entry["witness_commit"] = head
            if previous.get("compile_broken"):
                entry["broken_at_pin"] = {k: previous[k] for k in
                                          ("exit", "stderr_sha256", "stderr_evidence") if k in previous}
        manifest["faces"][face] = entry
        print(f"{face}: ids={len(ids)} selected={len(selected)} ignored={ignored} "
              f"({time.time() - started:.1f}s)", flush=True)

    universe: set[str] = set()
    for ids_file in sorted(args.output.glob("*-ids.txt")):
        if ids_file.name == "universe-ids.txt":
            continue
        universe |= set(ids_file.read_text(encoding="utf-8").splitlines())
    universe_path = args.output / "universe-ids.txt"
    universe_path.write_text("".join(line + "\n" for line in sorted(universe)), encoding="utf-8")
    manifest["universe_ids"] = len(universe)
    manifest["universe_ids_sha256"] = sha256(universe_path.read_bytes())
    still_broken = sorted(face for face, entry in manifest["faces"].items() if entry.get("compile_broken"))
    if not args.witness:
        still_broken += [face for face in broken if face not in still_broken]
        still_broken = sorted(set(still_broken))
    manifest["compile_broken_faces"] = still_broken
    manifest_path = args.output / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"universe: {len(universe)} ids → {manifest_path}")
    if still_broken:
        print(f"compile-broken faces (unenumerated at their pin): {still_broken}")
    return 1 if (broken and not args.witness) or still_broken else 0


if __name__ == "__main__":
    sys.exit(main())

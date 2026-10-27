#!/usr/bin/env python3
"""Check retired public paths against a real external Cargo consumer."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
UI = ROOT.parent / "auto-ui"


def cargo_path(path):
    return json.dumps(str(path.resolve()).replace("\\", "/"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = f'''[package]
name = "plan842-retirement-probe"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
auto-lang = {{ path = {cargo_path(ROOT / "crates/auto-lang")}, default-features = false }}
auto-lang-core = {{ path = {cargo_path(ROOT / "crates/auto-lang-core")} }}
auto-ast = {{ path = {cargo_path(ROOT / "crates/auto-ast")} }}
auto-diagnostics = {{ path = {cargo_path(ROOT / "crates/auto-diagnostics")} }}
auto-parser-api = {{ path = {cargo_path(ROOT / "crates/auto-parser-api")} }}
auto-ui-dialect = {{ path = {cargo_path(UI / "crates/auto-ui-dialect")} }}
auto-ui-runtime = {{ path = {cargo_path(UI / "crates/auto-ui-runtime")} }}
'''
    cases = {
        "ast": ('use auto_lang::ast::Stmt;\nfn main() {}\n', 'E0603', 'ast'),
        "parser": ('use auto_lang::parser::Parser;\nfn main() {}\n', 'E0603', 'parser'),
        "session": ('use auto_lang::session::CompilerSession;\nfn main() {}\n', 'E0603', 'session'),
        "error": ('use auto_lang::error::AutoError;\nfn main() {}\n', 'E0603', 'error'),
        "ui-alias-root": ('use auto_ui_dialect::UiParseContext;\nfn main() {}\n', 'E0432', 'UiParseContext'),
        "ui-alias-grammar": ('use auto_ui_dialect::grammar::UiParseContext;\nfn main() {}\n', 'E0432', 'UiParseContext'),
        "builder": ('''use auto_ui_runtime::ui::aura_view_builder::AuraViewBuilder;
fn main() { let _ = AuraViewBuilder::with_registry_and_imports; }
''', 'E0599', 'with_registry_and_imports'),
        "canonical": ('''use auto_ast::ast::Stmt;
use auto_diagnostics::error::AutoError;
use auto_lang_core::parser::Parser;
use auto_parser_api::{CompilerSession, DomainParseContext};
use auto_ui_runtime::ui::{aura_view_builder::AuraViewBuilder, vm_bridge::VmBridge, widget_registry::WidgetRegistry};
fn builder<'a>(bridge: &'a VmBridge, registry: &'a WidgetRegistry) -> AuraViewBuilder<'a> {
    AuraViewBuilder::with_registry(bridge, "App", registry)
}
fn context(_: Option<&dyn DomainParseContext>, _: Option<Stmt>, _: Option<Parser>, _: Option<CompilerSession>, _: Option<AutoError>) {}
fn main() { let _ = builder; let _ = context; }
''', None, None),
    }
    with tempfile.TemporaryDirectory(prefix="plan842-consumer-") as directory:
        project = Path(directory)
        (project / "src").mkdir()
        (project / "Cargo.toml").write_text(manifest, encoding="utf-8")
        for name, (source, expected_code, expected_item) in cases.items():
            (project / "src/main.rs").write_text(source, encoding="utf-8")
            command = ["cargo", "check", "--offline", "--target-dir", str(ROOT / "target")]
            run = subprocess.run(command, cwd=project, capture_output=True, text=True,
                                 encoding="utf-8", errors="replace", env=os.environ.copy())
            output = run.stdout + "\n" + run.stderr
            (args.output / (name + ".log")).write_text(output, encoding="utf-8")
            if expected_code is None:
                if run.returncode:
                    raise RuntimeError(f"{name}: canonical consumer failed: {args.output / (name + '.log')}")
            elif run.returncode == 0 or expected_code not in output or expected_item not in output:
                raise RuntimeError(f"{name}: expected {expected_code} naming {expected_item}: {args.output / (name + '.log')}")
            print(f"{name}: {'canonical passes' if expected_code is None else 'retired path rejected'}", flush=True)
        # Each old exact Cargo version must reject the new source rather than
        # resolving a breaking API under the old identity.
        paths = {
            "auto-lang": ROOT / "crates/auto-lang",
            "auto-ui-dialect": UI / "crates/auto-ui-dialect",
            "auto-ui-runtime": UI / "crates/auto-ui-runtime",
        }
        pinned = manifest
        for name, path in paths.items():
            pinned = pinned.replace(f'{name} = {{ path = {cargo_path(path)}',
                                    f'{name} = {{ version = "=0.2.0", path = {cargo_path(path)}')
        (project / "src/main.rs").write_text(cases["canonical"][0], encoding="utf-8")
        for name in paths:
            previous = pinned.replace(f'{name} = {{ version = "=0.2.0"',
                                      f'{name} = {{ version = "=0.1.0"')
            (project / "Cargo.toml").write_text(previous, encoding="utf-8")
            run = subprocess.run(["cargo", "check", "--offline", "--target-dir", str(ROOT / "target")],
                                 cwd=project, capture_output=True, text=True, encoding="utf-8", errors="replace")
            output = run.stdout + "\n" + run.stderr
            (args.output / (name + "-old-version.log")).write_text(output, encoding="utf-8")
            if run.returncode == 0 or name not in output or "0.2.0" not in output:
                raise RuntimeError(f"{name}: old exact version did not reject new source")
            print(f"{name}: old exact version rejected", flush=True)
        (project / "Cargo.toml").write_text(pinned, encoding="utf-8")
        run = subprocess.run(["cargo", "check", "--offline", "--target-dir", str(ROOT / "target")],
                             cwd=project, capture_output=True, text=True, encoding="utf-8", errors="replace")
        (args.output / "new-versions.log").write_text(run.stdout + "\n" + run.stderr, encoding="utf-8")
        if run.returncode:
            raise RuntimeError("new exact versions failed to resolve")
        print("all 0.2.0 exact versions: canonical consumer passes", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


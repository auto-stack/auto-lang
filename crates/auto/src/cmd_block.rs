//! `auto block` command (Plan 343, Design 17) — Skill-tier block catalog.
//!
//! Agent-driven architecture: the `auto` binary never calls an LLM. It only
//! *supplies* the spec (show) and *validates* output (check + `auto build`).
//! AI generation happens in the agent, which reads `auto block show`, writes a
//! `.at`, and loops on `auto build` + `auto block check`.
//!
//! Usage:
//!   auto block list
//!   auto block show form/login
//!   auto block add form/login --reference minimal --out src/front/blocks
//!   auto block check src/front/blocks/login.at --spec form/login

use std::fs;

use auto_lang::ui_gen::block::{BlockPackage, BlockRegistry};
use auto_lang::ui_gen::WidgetRegistry;
use miette::{miette, Result};

use crate::BlockAction;

/// Entry point dispatched from main.
pub fn run(action: BlockAction) -> Result<()> {
    match action {
        BlockAction::List => list(),
        BlockAction::Show { key } => show(&key),
        BlockAction::Add { key, reference, out } => add(&key, reference.as_deref(), &out),
        BlockAction::Check { file, spec } => check(&file, spec.as_deref()),
    }
}

fn registry() -> BlockRegistry {
    BlockRegistry::with_defaults()
}

/// Split a `kind/name` key.
fn split_key(key: &str) -> Result<(&str, &str)> {
    let (kind, name) = key
        .split_once('/')
        .ok_or_else(|| miette!("expected `kind/name`, got {key:?}"))?;
    if kind.is_empty() || name.is_empty() {
        return Err(miette!("empty kind or name in {key:?}"));
    }
    Ok((kind, name))
}

fn list() -> Result<()> {
    let reg = registry();
    if reg.packages().is_empty() {
        println!("(no blocks found under blocks/)");
        return Ok(());
    }
    // Group by kind (packages are sorted by key, so kind blocks are contiguous).
    let mut current_kind = String::new();
    for pkg in reg.iter() {
        if pkg.spec.kind != current_kind {
            current_kind = pkg.spec.kind.clone();
            println!("\n# {current_kind}");
        }
        println!("  {}/{}", pkg.spec.kind, pkg.spec.name);
    }
    Ok(())
}

fn show(key: &str) -> Result<()> {
    let (kind, name) = split_key(key)?;
    let reg = registry();
    let pkg = reg
        .get(kind, name)
        .ok_or_else(|| miette!("unknown block `{key}`; try `auto block list`"))?;

    let spec_md = fs::read_to_string(pkg.dir.join("spec.md"))
        .map_err(|e| miette!("read spec.md: {e}"))?;
    print!("{spec_md}");

    println!("\n────────  variants  ────────");
    let mut variants: Vec<&String> = pkg.references.keys().collect();
    variants.sort();
    for v in variants {
        println!("  - {v}");
    }

    if let Some(gotchas_path) = &pkg.gotchas {
        println!("\n────────  gotchas  ────────");
        let gotchas = fs::read_to_string(gotchas_path)
            .map_err(|e| miette!("read gotchas.md: {e}"))?;
        print!("{gotchas}");
    }
    Ok(())
}

fn add(key: &str, reference: Option<&str>, out: &str) -> Result<()> {
    let (kind, name) = split_key(key)?;
    let reg = registry();
    let pkg = reg
        .get(kind, name)
        .ok_or_else(|| miette!("unknown block `{key}`; try `auto block list`"))?;

    // Resolve the variant to copy.
    let variant = resolve_variant(pkg, reference)?;
    let src = pkg
        .references
        .get(variant.as_str())
        .expect("variant path present");
    let content = fs::read(src).map_err(|e| miette!("read reference: {e}"))?;

    fs::create_dir_all(out).map_err(|e| miette!("create {out:?}: {e}"))?;
    let dst = std::path::Path::new(out).join(format!("{name}.at"));
    fs::write(&dst, content).map_err(|e| miette!("write {}: {e}", dst.display()))?;

    println!("copied {key} [{variant}] -> {}", dst.display());
    report(pkg, &variant);
    Ok(())
}

fn resolve_variant(pkg: &BlockPackage, reference: Option<&str>) -> Result<String> {
    let mut variants: Vec<&String> = pkg.references.keys().collect();
    variants.sort();
    match reference {
        Some(v) => {
            if pkg.references.contains_key(v) {
                Ok(v.to_string())
            } else {
                Err(miette!(
                    "block {}/{} has no variant {v:?}; variants: {}",
                    pkg.spec.kind,
                    pkg.spec.name,
                    variants.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ))
            }
        }
        None => variants
            .first()
            .map(|s| s.to_string())
            .ok_or_else(|| miette!("block {}/{} has no references", pkg.spec.kind, pkg.spec.name)),
    }
}

/// Print the adopt-and-edit guidance: palette deps, dataSource wiring, gotcha titles.
fn report(pkg: &BlockPackage, _variant: &str) {
    println!("\n# palette (widgets this block composes)");
    for w in &pkg.spec.palette {
        println!("  - {w}");
    }
    if !pkg.spec.data_source.is_empty() {
        println!("\n# dataSource wiring (bind your #[api] fns to these slots)");
        for (slot, sig) in &pkg.spec.data_source {
            println!("  - {slot}: {sig}");
        }
    }
    if let Some(gotchas_path) = &pkg.gotchas {
        if let Ok(gotchas) = fs::read_to_string(gotchas_path) {
            let titles: Vec<&str> = gotchas
                .lines()
                .filter_map(|l| l.trim_start().strip_prefix("### ").map(str::trim))
                .filter(|t| !t.is_empty())
                .collect();
            if !titles.is_empty() {
                println!("\n# gotchas");
                for t in titles {
                    println!("  - {t}");
                }
            }
        }
    }
}

/// Static acceptance check on a generated/copied `.at`. Returns non-zero on any
/// failed *hard* gate so the agent repair loop can detect it.
///
/// Hard gates (count toward exit code): loading + error contract slots, and
/// used-widget-within-palette. Extension-point EDIT markers are reported as
/// **info** only — a specific variant legitimately omits points it doesn't
/// implement (e.g. `minimal` has no `third_party`), so absence isn't a failure.
fn check(file: &str, spec_key: Option<&str>) -> Result<()> {
    let src = fs::read_to_string(file).map_err(|e| miette!("read {file:?}: {e}"))?;

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut hard = |ok: bool, what: &str| {
        if ok {
            passed += 1;
            println!("  \u{2713} {what}");
        } else {
            failed += 1;
            println!("  \u{2717} {what}");
        }
    };

    // 0. Extension-point EDIT markers — info only (variants may omit points).
    if let Some(key) = spec_key {
        let (kind, name) = split_key(key)?;
        let reg = registry();
        if let Some(pkg) = reg.get(kind, name) {
            let mut marked = 0usize;
            for ep in &pkg.spec.extension_points {
                if src.contains(&format!("EDIT: {ep}")) {
                    marked += 1;
                }
            }
            println!(
                "  \u{00b7} EDIT markers: {}/{} extension_points marked (info — variants may omit)",
                marked,
                pkg.spec.extension_points.len()
            );
        } else {
            hard(false, &format!("spec `{key}` known"));
        }
    }

    // 1. Behavior-contract slots: loading + error states must appear.
    hard(
        src.to_lowercase().contains("loading"),
        "loading state present (behavior contract)",
    );
    hard(
        src.to_lowercase().contains("error"),
        "error state present (behavior contract)",
    );

    // 2. Palette widgets: any used tag outside the palette is flagged.
    if let Some(key) = spec_key {
        let (kind, name) = split_key(key)?;
        let reg = registry();
        if let Some(pkg) = reg.get(kind, name) {
            let widgets = WidgetRegistry::with_defaults();
            let used: Vec<&str> = widgets
                .all_widgets()
                .keys()
                .map(|s| s.as_str())
                .filter(|tag| {
                    // crude: tag appears as a line-leading view element `tag {`
                    src.contains(&format!("\n{tag} "))
                        || src.contains(&format!("\n  {tag} "))
                        || src.contains(&format!("\n    {tag} "))
                })
                .collect();
            let palette: std::collections::HashSet<&str> =
                pkg.spec.palette.iter().map(|s| s.as_str()).collect();
            let outside: Vec<&&str> = used.iter().filter(|w| !palette.contains(**w)).collect();
            hard(
                outside.is_empty(),
                &format!(
                    "used widgets within palette ({})",
                    if outside.is_empty() {
                        "ok".to_string()
                    } else {
                        outside.iter().map(|s| **s).collect::<Vec<_>>().join(", ")
                    }
                ),
            );
        }
    }

    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        return Err(miette!("{failed} acceptance check(s) failed"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_kind_name() {
        assert_eq!(split_key("form/login").unwrap(), ("form", "login"));
        assert!(split_key("bogus").is_err());
        assert!(split_key("/x").is_err());
        assert!(split_key("x/").is_err());
    }

    #[test]
    fn check_passes_on_known_good_login_reference() {
        // Resolve the committed minimal reference and run check against its spec.
        let reg = BlockRegistry::with_defaults();
        let pkg = reg.get("form", "login").expect("form/login present");
        let path = pkg.references.get("minimal").expect("minimal variant");
        let file = path.to_string_lossy().to_string();
        // Should pass: loading + error both appear in minimal.at.
        assert!(check(&file, Some("form/login")).is_ok());
    }
}

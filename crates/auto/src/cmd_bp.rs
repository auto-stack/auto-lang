//! `auto bp` command (Plan 343, Design 17; PLAN-639 rename) — Skill-tier blueprint catalog.
//!
//! Agent-driven architecture: the `auto` binary never calls an LLM. It only
//! *supplies* the spec (show) and *validates* output (check + `auto build`).
//! AI generation happens in the agent, which reads `auto bp show`, writes a
//! `.at`, and loops on `auto build` + `auto bp check`.
//!
//! Usage:
//!   auto bp list
//!   auto bp show form/login
//!   auto bp add form/login --reference minimal --out src/front/bps
//!   auto bp check src/front/bps/login.at --spec form/login

use std::fs;

use auto_lang::ui_gen::bp::{BlueprintPackage, BlueprintRegistry};
use auto_lang::ui_gen::WidgetRegistry;
use miette::{miette, Result};

use crate::BpAction;

/// Entry point dispatched from main.
pub fn run(action: BpAction) -> Result<()> {
    match action {
        BpAction::List => list(),
        BpAction::Show { key } => show(&key),
        BpAction::Add { key, reference, out, bind: bind_flag, dep } => {
            if bind_flag {
                bind(&key, reference.as_deref(), &out, &dep)
            } else {
                add(&key, reference.as_deref(), &out)
            }
        }
        BpAction::Bind { key, reference, out, dep } => {
            bind(&key, reference.as_deref(), &out, &dep)
        }
        BpAction::Check { file, spec } => check(&file, spec.as_deref()),
    }
}

fn registry() -> BlueprintRegistry {
    BlueprintRegistry::with_defaults()
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
        println!("(no blueprints found under blueprints/)");
        return Ok(());
    }
    // Group by kind (packages are sorted by key, so same-kind packages are contiguous).
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
        .ok_or_else(|| miette!("unknown blueprint `{key}`; try `auto bp list`"))?;

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
        .ok_or_else(|| miette!("unknown blueprint `{key}`; try `auto bp list`"))?;

    // Resolve the variant to copy.
    let variant = resolve_variant(pkg, reference)?;
    let src = pkg
        .references
        .get(variant.as_str())
        .expect("variant path present");
    let content = fs::read_to_string(src).map_err(|e| miette!("read reference: {e}"))?;

    fs::create_dir_all(out).map_err(|e| miette!("create {out:?}: {e}"))?;
    let dst = std::path::Path::new(out).join(format!("{name}.at"));
    // PLAN-639 SD-02 artifact discipline: L2 copies carry a provenance header
    // (source bp + variant + date) so future L1 migration has an account.
    let today = time_header_date();
    let provenance = format!(
        "// source: blueprints/{key} @ {variant} ({today}, `auto bp add --reference`)
"
    );
    let mut owned = String::with_capacity(provenance.len() + content.len());
    owned.push_str(&provenance);
    owned.push_str(&content);
    fs::write(&dst, owned).map_err(|e| miette!("write {}: {e}", dst.display()))?;

    println!("copied {key} [{variant}] -> {}", dst.display());
    report(pkg, &variant);
    Ok(())
}

fn resolve_variant(pkg: &BlueprintPackage, reference: Option<&str>) -> Result<String> {
    let mut variants: Vec<&String> = pkg.references.keys().collect();
    variants.sort();
    match reference {
        Some(v) => {
            if pkg.references.contains_key(v) {
                Ok(v.to_string())
            } else {
                Err(miette!(
                    "blueprint {}/{} has no variant {v:?}; variants: {}",
                    pkg.spec.kind,
                    pkg.spec.name,
                    variants.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ))
            }
        }
        None => variants
            .first()
            .map(|s| s.to_string())
            .ok_or_else(|| miette!("blueprint {}/{} has no references", pkg.spec.kind, pkg.spec.name)),
    }
}

/// Print the adopt-and-edit guidance: palette deps, dataSource wiring, gotcha titles.
fn report(pkg: &BlueprintPackage, _variant: &str) {
    println!("\n# palette (widgets this blueprint composes)");
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

/// PLAN-639 L1 import/bind (T-06): emit a declarative bind artifact that
/// imports the blueprint widget from the app's declared package — zero copy.
/// Consistency checks: bp/variant existence (hard), pac.at dep declaration
/// (warning + fix hint), spec-declared action/prop obligations (report).
fn bind(key: &str, reference: Option<&str>, out: &str, dep: &str) -> Result<()> {
    let (kind, name) = split_key(key)?;
    let reg = registry();
    let pkg = reg
        .get(kind, name)
        .ok_or_else(|| miette!("unknown blueprint `{key}`; try `auto bp list`"))?;

    let variant = resolve_variant(pkg, reference)?;
    let src = pkg
        .references
        .get(variant.as_str())
        .expect("variant path present");

    // The import item must be the widget symbol declared in the reference.
    let content = fs::read_to_string(src).map_err(|e| miette!("read reference: {e}"))?;
    let widget_sym = extract_widget_symbol(&content)
        .ok_or_else(|| miette!("reference {} declares no widget", src.display()))?;

    // pac.at dep declaration check (declaration gating, PLAN-635 style).
    let pac_declared = fs::read_to_string("pac.at").ok().is_some_and(|pac| {
        for form in [format!("dep \"{dep}\""), format!("dep {dep}")] {
            if let Some(pos) = pac.find(&form) {
                let after = pac[pos + form.len()..].chars().next();
                if after.is_some_and(|c| !(c.is_alphanumeric() || c == '_' || c == '.')) {
                    return true;
                }
            }
        }
        false
    });
    if !pac_declared {
        println!(
            "warning: pac.at does not declare `dep \"{dep}\" {{ path: ... }}` — the bind's use import will not resolve until it is declared"
        );
    }

    // Spec-declared obligations (contract Q1/Q2): report what the consumer
    // must satisfy; declared-but-unmet is a report, not an error, in v0.
    let spec_doc = fs::read_to_string(pkg.dir.join("spec.md")).unwrap_or_default();
    let (spec, _) = auto_lang::ui_gen::bp::BlueprintSpec::parse_document(&spec_doc)
        .unwrap_or_default();
    if !spec.actions.is_empty() {
        println!("# actions the consumer must register (contract Q2):");
        for a in &spec.actions {
            println!("  - {a}");
        }
    }
    if !spec.props.is_empty() {
        println!("# data props expected by the blueprint (contract Q1):");
        for p in &spec.props {
            println!("  - {p}");
        }
    }

    fs::create_dir_all(out).map_err(|e| miette!("create {out:?}: {e}"))?;
    let stem = format!("{name}_bind");
    let dst = std::path::Path::new(out).join(format!("{stem}.at"));
    let bind_widget = pascal_case(name) + "Bind";
    let today = time_header_date();
    let artifact = format!(
        "// GENERATED by `auto bp add --bind` — PLAN-639 L1 import/bind channel.\n\
         // Source: blueprints/{key} @ {variant} ({today}). Regenerate freely;\n\
         // hand edits belong in the consuming layout, not here. Zero-copy: the\n\
         // blueprint widget is imported from the declared `{dep}` package.\n\
         use {dep}.{kind}.{name}.reference.{variant}: {widget_sym}\n\
         \n\
         // L1 bind: declarative layout assembly over the imported blueprint.\n\
         widget {bind_widget} {{\n\
         \x20   view {{\n\
         \x20       col {{\n\
         \x20           {widget_sym} {{}}\n\
         \x20           style: \"w-full max-w-md mx-auto p-6\"\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );
    fs::write(&dst, artifact).map_err(|e| miette!("write {}: {e}", dst.display()))?;

    println!("bound {key} [{variant}] -> {}", dst.display());
    println!("  import: {dep}.{kind}.{name}.reference.{variant}: {widget_sym}");
    println!("  zero-copy: no blueprint source was vendored (L1 artifact discipline)");
    report(pkg, &variant);
    Ok(())
}

/// First `widget <Ident>` symbol in the reference source.
fn extract_widget_symbol(src: &str) -> Option<String> {
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("widget ") {
            let sym: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !sym.is_empty() {
                return Some(sym);
            }
        }
    }
    None
}

fn pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Local date (YYYY-MM-DD) for provenance/GENERATED headers.
fn time_header_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Days since epoch -> civil date (Howard Hinnant's algorithm).
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
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
            // PLAN-643: 候选 tag 集含 schema package_origin 词汇面(chart 四
            // tag 首批)——它们不在 WidgetRegistry,不扩集则 bp check 对其
            // 视而不见;palette 合法集语义与 BlueprintRegistry::palette_drift
            // 对齐(registry ∪ package_origin)。
            let mut candidates: Vec<String> = widgets
                .all_widgets()
                .keys()
                .map(|s| s.to_string())
                .collect();
            if let Some(schema) = auto_lang::aura::default_schema_cached() {
                for (tag, meta) in schema.meta.iter() {
                    if meta.tier == auto_lang::aura::schema::ElementTier::PackageOrigin {
                        candidates.push(tag.to_string());
                    }
                }
            }
            let used: Vec<&str> = candidates
                .iter()
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
        let reg = BlueprintRegistry::with_defaults();
        let pkg = reg.get("form", "login").expect("form/login present");
        let path = pkg.references.get("minimal").expect("minimal variant");
        let file = path.to_string_lossy().to_string();
        // Should pass: loading + error both appear in minimal.at.
        assert!(check(&file, Some("form/login")).is_ok());
    }
}

//! PLAN-960 packages module lane: the existing resolver, typecheck,
//! emitter and HIR verifier over the package graph (the same text-record
//! pipeline the single-module lane uses; no parallel semantic analysis).
//! Names/visibility/initialization order are bound by
//! `ace_core::source::packages`; this layer never widens them.
//!
//! The PLAN-934 per-function fact pages only cover the finite scalar
//! portable type set (`PortableTy`), so a package graph carrying text or
//! record types cannot be served by those pages — the lane runs the whole
//! resolver and relies on the binder's reuse ledger (`binding_walks`) for
//! incremental evidence. Fact-page coverage for text/record types is a
//! named query-layer follow-up, not a silent fallback.
use ace_core::atom_text::Diagnostic;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    bind_packages, BoundPackages, PackageDiagnostic, PackageModuleInput,
};
use ace_core::source::{
    emit_resolved_packages_parts, resolve_with, typecheck_with, CompiledSource, SourceProfile,
};

pub struct PackageProduct {
    pub compiled: CompiledSource,
    pub bound: BoundPackages,
}

fn locate_all(bound: &BoundPackages, ds: Vec<Diagnostic>) -> Vec<PackageDiagnostic> {
    ds.into_iter().map(|d| bound.locate(d)).collect()
}

pub fn compile_package_product(
    inputs: &[PackageModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let bound = bind_packages(inputs, entry, caps, budget).map_err(|e| vec![e])?;
    compile_bound_packages(bound, caps)
}

/// Cold/retained shared compile over an already-bound package graph. The
/// caller retains the graph's identity/visibility admission.
pub fn compile_bound_packages(
    bound: BoundPackages,
    caps: &[String],
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let resolved = resolve_with(&bound.program, caps, SourceProfile::TextRecord)
        .map_err(|ds| locate_all(&bound, ds))?;
    typecheck_with(&resolved, SourceProfile::TextRecord).map_err(|ds| locate_all(&bound, ds))?;
    let parts = emit_resolved_packages_parts(&resolved, &bound, "packages")
        .map_err(|ds| locate_all(&bound, ds))?;
    let checked =
        ace_core::verify::verify_text_record(parts.bundle).map_err(|ds| locate_all(&bound, ds))?;
    Ok(PackageProduct {
        compiled: CompiledSource {
            profile: parts.profile,
            input_hash: parts.input_hash,
            atom_text: parts.atom_text,
            checked,
        },
        bound,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::packages::PACKAGES_CAPABILITY;

    fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: package.into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
        }
    }

    #[test]
    fn package_graph_reaches_checked_hir_in_the_text_record_domain() {
        let inputs = [
            input(
                "lib",
                "std",
                "pub const BASE int = 40\npub fn greet() str { return \"m2\" }\npub type P { x int\n fn sum() int { return .x } }",
            ),
            input(
                "app",
                "main",
                "use lib.std.greet\nuse lib.std.P\nuse lib.std.BASE\nvar count int = 0\npub fn bump() int { count = count + 1\n return count }\nfn main() int { let p = P { x: BASE }\n let n = bump()\n return p.sum() + n }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert!(product.bound.module_init_fns.contains_key("app::main"));
        let checked = &product.compiled.checked;
        assert!(checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")));
        let syms: Vec<&String> = checked
            .bundle()
            .modules
            .iter()
            .flat_map(|m| {
                m.defs.iter().filter_map(|d| match &d.kind {
                    ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol),
                    _ => None,
                })
            })
            .collect();
        assert!(
            syms.iter()
                .any(|s| s.as_str() == "module.global.get:g3_libm3_std_BASE"),
            "{syms:?}"
        );
        assert!(
            syms.iter()
                .any(|s| s.as_str() == "module.global.set:g3_appm4_main_count"),
            "{syms:?}"
        );
    }

    #[test]
    fn package_type_and_visibility_failures_keep_source_location() {
        let inputs = [
            input("lib", "std", "pub fn give(x bool) int { return 1 }"),
            input(
                "app",
                "main",
                "use lib.std.give\nfn main() int { return give(1) }",
            ),
        ];
        let ds = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .err()
        .unwrap();
        assert_eq!(ds[0].module, "app::main", "{ds:?}");
        let src = &inputs[1].source;
        assert!(
            ds[0].diagnostic.span.start >= src.find("give(1)").unwrap()
                && ds[0].diagnostic.span.end <= src.len(),
            "{ds:?}"
        );
    }
}

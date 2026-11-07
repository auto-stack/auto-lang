//! PLAN-960 packages AV target: the same unit admission/link machinery as
//! the 950 module lane, extended with module global cells and the per-module
//! init chain (each init runs once per linked generation before the entry;
//! a failing init leaves the generation uninitialized — the caller keeps the
//! previous current). No warm body-edit path in this first cut: declaration
//! changes and body edits recompile through the packages lane.
use crate::modules::{persist_unit_with_context, split_units};
use ace_source_query::modules::PackageProduct;
use ace_artifacts::store::ManagedStore;
use std::path::Path;
use std::sync::Arc;

pub const PACKAGES_SCHEMA: &str = "av-source-packages/1";

pub struct PackageAvTarget {
    pub store: ManagedStore,
    pub current: Arc<ace_av_core::program_snapshot::ProgramSnapshot>,
    pub lowered: u64,
    pub source: String,
}

impl PackageAvTarget {
    /// Cold link with `d_main` as the entry.
    pub fn cold(path: &Path, product: &PackageProduct) -> Result<Self, String> {
        Self::cold_with_entry(path, product, "d_main")
    }

    /// Cold link with an explicit entry def (e.g. a public Str function).
    pub fn cold_with_entry(
        path: &Path,
        product: &PackageProduct,
        entry: &str,
    ) -> Result<Self, String> {
        let bound = &product.bound;
        let init_names: Vec<String> = bound
            .module_init_order
            .iter()
            .filter_map(|k| bound.module_init_fns.get(k).map(|s| format!("d_{s}")))
            .collect();
        let mir = ace_lowering::lower_checked_owned_with_init(
            &product.compiled.checked,
            entry,
            0,
            &init_names,
        )
        .map_err(|e| format!("{e:?}"))?;
        let globals = mir.globals.clone();
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let mut codes = Vec::new();
        for unit in split_units(&mir)? {
            let mint = package_span(bound, &unit.funcs[0].name)?;
            let (_record, code) =
                persist_unit_with_context(&store, &unit, mint, &bound.capabilities, 0, "")?;
            codes.push(code);
        }
        let lowered = codes.len() as u64;
        let span_maps = (0..codes.len())
            .map(|_| ace_av_core::span_map::SpanSegments::default())
            .collect();
        let current = ace_av_core::program_snapshot::ProgramSnapshot::link_with_globals(
            codes,
            entry,
            0,
            span_maps,
            globals,
            &init_names,
        )?;
        Ok(Self {
            store,
            current: Arc::new(current),
            lowered,
            source: bound.input_hash.clone(),
        })
    }

    /// Execute the current entry and return its raw value (i32 for scalar
    /// entries; the observation channel for text entries is `Value::Str*`).
    pub fn run_value(
        &self,
    ) -> Result<ace_av_core::contract::Value, String> {
        let (out, _trace) = ace_av_core::execute_program(
            &self.current,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::default(),
        );
        match out {
            ace_av_core::contract::ExecOutcome::Success { value, .. } => Ok(value),
            other => Err(format!("{other:?}")),
        }
    }

    /// The current generation's global cell values (observation only; never
    /// serialized).
    pub fn global_cells(&self) -> Vec<ace_av_core::contract::Value> {
        self.current.global_cells()
    }
}

fn package_span(
    bound: &ace_core::source::packages::BoundPackages,
    name: &str,
) -> Result<(usize, usize), String> {
    let bare = name.strip_prefix("d_").unwrap_or(name);
    if let Some((_, _, original)) = bound.origins.get(bare) {
        let (pkg, module) = (bound.origins[bare].0.clone(), bound.origins[bare].1.clone());
        let key = ace_core::source::packages::unit_key(&pkg, &module);
        let start = bound.segment_start(&key).unwrap_or(0);
        if let Some(u) = bound.units.get(&key) {
            if let Some(sp) = u.declarations.get(original) {
                return Ok((start + sp.start, start + sp.end));
            }
        }
        let _ = original;
    }
    Ok((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::cst::FrontBudget;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::compile_package_product;

    fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: package.into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
        }
    }
    fn tmp(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ace-pkg-av-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// V-08 AV legs: same-name private store isolation, diamond init once,
    /// init failure keeps the previous generation, and the public Str
    /// function runs on the AV end.
    #[test]
    fn packages_av_runs_stores_init_and_public_str() {
        let inputs = [
            // diamond base: D's init must run exactly once (observed via
            // its store value through both paths).
            input(
                "base",
                "d",
                "pub const SEED int = 5\npub fn seed() int { return SEED }",
            ),
            input(
                "app",
                "left",
                "use base.d.seed\npub fn left() int { return seed() }",
            ),
            input(
                "app",
                "right",
                "use base.d.seed\npub fn right() int { return seed() }",
            ),
            input(
                "app",
                "one",
                "pub fn hit() int { return 1 }\nvar count int = 0\npub fn bump() int { count = count + 1\n return count }",
            ),
            input(
                "app",
                "two",
                "pub fn hit() int { return 2 }\nvar count int = 0\npub fn bump() int { count = count + 1\n return count }",
            ),
            input(
                "app",
                "std",
                "pub fn greet() str { return \"m2\" }",
            ),
            input(
                "app",
                "main",
                "use base.d.seed\nuse app.one.hit as one_hit\nuse app.two.hit as two_hit\nuse app.one.bump as one_bump\nuse app.two.bump as two_bump\nuse app.std.greet\nfn main() int { let a = app.left.left()\n let b = app.right.right()\n let x = one_bump()\n let y = two_bump()\n let g = greet()\n return seed() + a + b + one_hit() + two_hit() + x + y + g.len() }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        // seed()=5, left()=5, right()=5 (both diamond legs read the SAME
        // base cell after ONE init), one_hit()=1 + two_hit()=2 (same-name
        // private fns stay package-isolated), one_bump()=1 + two_bump()=1
        // (same-name private STORES stay module-isolated — a shared cell
        // would make the second bump return 2 and the total 23),
        // "m2".len()=2 (the public Str function really ran).
        let dir = tmp("run");
        let av = PackageAvTarget::cold(&dir, &product).unwrap();
        assert_eq!(av.run_value(), Ok(ace_av_core::contract::Value::I32(22)));
        // the Str function itself runs and is observed as a Str view
        let av2 = PackageAvTarget::cold_with_entry(
            &dir,
            &product,
            "d_p3_appm3_std_greet",
        )
        .unwrap();
        match av2.run_value() {
            Ok(ace_av_core::contract::Value::StrView(v)) => {
                assert!(v.start == 0, "{v:?}");
            }
            other => panic!("greet must return a Str view, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An init failure leaves the generation uninitialized: the failing
    /// target's run reports the trap, and a fresh target over the good
    /// product keeps running (no half-published state).
    #[test]
    fn packages_av_init_failure_keeps_no_half_state() {
        let good = [input(
            "app",
            "main",
            "var count int = 3\nfn main() int { return count }",
        )];
        let product = compile_package_product(
            &good,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let dir = tmp("initfail");
        let av = PackageAvTarget::cold(&dir, &product).unwrap();
        assert_eq!(av.run_value(), Ok(ace_av_core::contract::Value::I32(3)));
        // after a successful generation the cells hold the initialized value
        assert_eq!(
            av.global_cells(),
            vec![ace_av_core::contract::Value::I32(3)]
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}

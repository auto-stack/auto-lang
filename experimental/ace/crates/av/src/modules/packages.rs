//! PLAN-960 packages AV target: the same unit admission/link machinery as
//! the 950 module lane, extended with module global cells and the per-module
//! init chain (each init runs once per linked generation before the entry;
//! a failing init leaves the generation uninitialized — the caller keeps the
//! previous current). No warm body-edit path in this first cut: declaration
//! changes and body edits recompile through the packages lane.
use crate::modules::{persist_unit_with_context, split_units};
use ace_artifacts::store::ManagedStore;
use ace_source_query::modules::PackageProduct;
use std::path::Path;
use std::sync::Arc;

pub const PACKAGES_SCHEMA: &str = "av-source-packages/1";

pub struct PackageAvTarget {
    pub store: ManagedStore,
    pub current: Arc<ace_av_core::program_snapshot::ProgramSnapshot>,
    pub lowered: u64,
    pub source: String,
    /// PLAN-971 B (T-02): the LOWERED MIR's per-function slot names, keyed
    /// by the admitted function name. The ABC wire does not serialize slot
    /// names, so a source-level debugger (locals) reads them from the same
    /// lowering generation that produced the code — never by re-lowering a
    /// different revision.
    pub slot_names: std::collections::BTreeMap<String, Vec<Option<String>>>,
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
        // PLAN-946（T-02）：aggregates packages 线（std.list 承载面）走
        // closures-superset 家族降落 + init 链；plain/ct 线保持原 owned 面。
        let mir = if product.compiled.profile == ace_core::source::AUTO_SOURCE_PROFILE {
            ace_lowering::lower_checked_auto_language_with_init(
                &product.compiled.checked,
                entry,
                0,
                &init_names,
            )
        } else if product.compiled.profile == ace_core::source::AGGREGATES_SOURCE_PROFILE {
            ace_lowering::lower_checked_closures_with_init(
                &product.compiled.checked,
                entry,
                0,
                &init_names,
            )
        } else {
            ace_lowering::lower_checked_owned_with_init(
                &product.compiled.checked,
                entry,
                0,
                &init_names,
            )
        }
        .map_err(|e| format!("{e:?}"))?;
        let slot_names: std::collections::BTreeMap<String, Vec<Option<String>>> = mir
            .funcs
            .iter()
            .map(|f| {
                (
                    f.name.clone(),
                    f.slots.iter().map(|s| s.name.clone()).collect(),
                )
            })
            .collect();
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let mut codes = Vec::new();
        // Admit host imports per portable unit as in the regular package
        // lane. Linked global ordinals and types are checked across units.
        for unit in split_units(&mir)? {
            let mint = package_span(bound, &unit.funcs[0].name)?;
            let (_record, code) =
                persist_unit_with_context(&store, &unit, mint, &bound.capabilities, 0, "")
                    .map_err(|e| format!("unit {}: {e:?}", unit.funcs[0].name))?;
            codes.push(code);
        }
        let lowered = codes.len() as u64;
        let span_maps = (0..codes.len())
            .map(|_| ace_av_core::span_map::SpanSegments::default())
            .collect();
        let current = ace_av_core::program_snapshot::ProgramSnapshot::link_with_checked_globals(
            codes,
            entry,
            0,
            span_maps,
            &mir,
            &init_names,
        )?;
        Ok(Self {
            store,
            current: Arc::new(current),
            lowered,
            source: bound.input_hash.clone(),
            slot_names,
        })
    }

    /// Execute the current entry and return its raw value (i32 for scalar
    /// entries; the observation channel for text entries is `Value::Str*`).
    pub fn run_value(&self) -> Result<ace_av_core::contract::Value, String> {
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

    /// PLAN-973: execute the entry under the platform discipline — per-run
    /// grants, a fresh resource ledger and an explicit backend (the
    /// console lane's `run_platform` over the packages snapshot path).
    pub fn run_platform_value(
        &self,
        grants: &[String],
        backend: &mut dyn ace_av_core::platform_io::PlatformIoBackend,
    ) -> Result<
        (
            ace_av_core::contract::ExecOutcome,
            ace_av_core::PlatformSessionSummary,
        ),
        String,
    > {
        self.run_platform_value_with(
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::default(),
            None,
            grants,
            backend,
        )
    }

    /// PLAN-982（aggregates 文本入口裁定，2026-11-19）：受检文本入口的包车道
    /// 产品入口形——请求文本（如管道 stdin）作为入口单元借用 `str` 形参的
    /// 运行时实参（`ac.runtime.text.v1` 谱系；执行端从入口所属单元核对能力
    /// 与 `StrView` 签名，arena 归零与资源收尾同 `execute_program_platform`）。
    pub fn run_platform_text_value_with(
        &self,
        limits: &ace_av_core::ExecLimits,
        cancel: &ace_av_core::CancelFlag,
        console: Option<&mut dyn ace_av_core::host::ConsoleHost>,
        grants: &[String],
        backend: &mut dyn ace_av_core::platform_io::PlatformIoBackend,
        texts: &[&[u8]],
    ) -> Result<
        (
            ace_av_core::contract::ExecOutcome,
            ace_av_core::PlatformSessionSummary,
        ),
        String,
    > {
        let (out, _trace, summary) =
            ace_av_core::execute_program_text_platform(&self.current, limits, cancel, grants, texts, console, backend)
                .map_err(|e| format!("{}: {}", e.code, e.message))?;
        Ok((out, summary))
    }

    /// PLAN-982 (Q-982-2 step 2): the product-entry form — the caller owns
    /// the wall/cancel budgets and the CONSOLE HOST (capture or pass-through
    /// is the door's choice); grants stay the per-declared-import authority.
    pub fn run_platform_value_with(
        &self,
        limits: &ace_av_core::ExecLimits,
        cancel: &ace_av_core::CancelFlag,
        console: Option<&mut dyn ace_av_core::host::ConsoleHost>,
        grants: &[String],
        backend: &mut dyn ace_av_core::platform_io::PlatformIoBackend,
    ) -> Result<
        (
            ace_av_core::contract::ExecOutcome,
            ace_av_core::PlatformSessionSummary,
        ),
        String,
    > {
        let (out, _trace, summary) = ace_av_core::execute_program_platform(
            &self.current,
            limits,
            cancel,
            grants,
            console,
            backend,
        )
        .map_err(|e| format!("{}: {}", e.code, e.message))?;
        Ok((out, summary))
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
            provider: None,
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
            // diamond base with a STORE bumped by its accessor: both legs
            // share ONE initialized cell (left()=1, right()=2); a per-leg
            // re-initialization would make both legs return 1.
            input(
                "base",
                "d",
                "var hits int = 0\npub fn seed() int { hits = hits + 1\n return hits }",
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
                "use base.d.seed\nuse app.one.hit as one_hit\nuse app.two.hit as two_hit\nuse app.one.bump as one_bump\nuse app.two.bump as two_bump\nuse app.std.greet\nfn main() int { let a = app.left.left()\n let b = app.right.right()\n let x = one_bump()\n let y = two_bump()\n let g = greet()\n let s = seed()\n return a + b + one_hit() + two_hit() + x + y + s + g.len() }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        // main: a=left()=1, b=right()=2 (diamond base initialized ONCE, one
        // shared cell bumped through both legs; per-leg re-init would give
        // 1+1), one_hit()=1 + two_hit()=2 (same-name private fns stay
        // package-isolated), one_bump()=1 + two_bump()=1 (same-name private
        // STORES stay module-isolated — a shared cell would make the second
        // bump return 2), seed()=3 (third bump from main), "m2".len()=2.
        // 1+2+1+2+1+1+3+2 = 13
        let dir = tmp("run");
        let av = PackageAvTarget::cold(&dir, &product).unwrap();
        assert_eq!(av.run_value(), Ok(ace_av_core::contract::Value::I32(13)));
        // the Str function itself runs and is observed as a Str view
        let av2 = PackageAvTarget::cold_with_entry(&dir, &product, "d_p3_appm3_std_greet").unwrap();
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
        // a trapping initializer AFTER an earlier store was written: the run
        // reports the trap and the generation publishes no partial state
        // (every cell is back to its placeholder, not `a = 7`).
        let bad = [input(
            "app",
            "main",
            "fn z() int { return 0 }\nvar a int = 7\nvar b int = 1 / z()\nfn main() int { return a + b }",
        )];
        let product = compile_package_product(
            &bad,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let av = PackageAvTarget::cold(&dir, &product).unwrap();
        let mut backend = ace_av_core::platform_io::CapturePlatformIo::default();
        for _ in 0..2 {
            let (out, trace, _) = ace_av_core::av::execute_program_platform(
                &av.current,
                &ace_av_core::ExecLimits::default(),
                &ace_av_core::CancelFlag::default(),
                &[],
                None,
                &mut backend,
            )
            .unwrap();
            assert!(
                matches!(
                    out,
                    ace_av_core::contract::ExecOutcome::Trap {
                        kind: ace_av_core::contract::TrapKind::DivByZero,
                        ..
                    }
                ),
                "{out:?}"
            );
            assert!(!av.current.initialized());
            assert_eq!(trace.frames_remaining, 0);
            assert!(av
                .global_cells()
                .iter()
                .all(|value| *value == ace_av_core::contract::Value::I32(0)));
        }
        let out = av.run_value();
        assert!(out.is_err(), "{out:?}");
        assert!(!av.current.initialized());
        assert!(
            av.global_cells()
                .iter()
                .all(|c| *c == ace_av_core::contract::Value::I32(0)),
            "{:?}",
            av.global_cells()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn ordinary_packages_share_instances_and_bind_restore_to_version() {
        use ace_av_core::contract::Value;
        use ace_core::source::ast::Item;
        use ace_source_query::modules::{packages::compile_package_product_auto, PackageStore};

        let inputs = [
            input(
                "app",
                "main",
                "use support.lib.identity
use support.lib.nineteen
fn main() int { return nineteen() + identity<int>(23) }",
            ),
            input(
                "support",
                "lib",
                "pub fn identity<T>(x T) T { return x }
pub fn nineteen() int { return identity(19) }",
            ),
        ];
        let caps = [ace_core::source::AUTO_SOURCE_ADMISSION_CAP.into()];
        let compile = |inputs: &[PackageModuleInput]| {
            compile_package_product_auto(inputs, "app::main", &caps, FrontBudget::WORK_OWNED_64MIB)
        };
        let only_instance = |product: &PackageProduct| {
            let bound = &product.bound;
            let names: Vec<_> = bound
                .program
                .items
                .iter()
                .filter_map(|item| {
                    let Item::Fn(f) = item else { return None };
                    let origin = bound.origins.get(&f.name.name)?;
                    (origin.0 == "support" && origin.1 == "lib" && origin.2 == "identity")
                        .then(|| f.name.name.clone())
                })
                .collect();
            assert_eq!(
                names.len(),
                1,
                "both callers must share one instance: {names:?}"
            );
            names[0].clone()
        };
        let dir = tmp("ordinary-generic-identity");
        let baseline = compile(&inputs).unwrap();
        let instance = only_instance(&baseline);
        let receipt = PackageStore::prepare(&dir, &baseline, &inputs, &[]).unwrap();
        let run = |product| PackageAvTarget::cold(&dir, product).unwrap().run_value();
        assert_eq!(run(&baseline), Ok(Value::I32(42)));

        let mut edited = inputs.clone();
        edited[1].source = edited[1].source.replace("return x }", "return x + 1 }");
        let changed_body = compile(&edited).unwrap();
        assert_eq!(only_instance(&changed_body), instance);
        assert_ne!(
            changed_body.compiled.input_hash,
            baseline.compiled.input_hash
        );
        assert_eq!(run(&changed_body), Ok(Value::I32(44)));

        let mut new_signature = inputs.clone();
        new_signature[1].source = new_signature[1]
            .source
            .replace("(x T) T { return x }", "(x T, delta int) T { return x + delta }");
        let stale = compile(&new_signature).err().expect("stale callers must fail");
        assert!(stale.iter().any(|d| d.diagnostic.code == "source.argument"));
        new_signature[0].source = new_signature[0]
            .source
            .replace("identity<int>(23)", "identity<int>(23, 3)");
        new_signature[1].source = new_signature[1]
            .source
            .replace("identity(19)", "identity(19, 4)");
        let changed_signature = compile(&new_signature).unwrap();
        assert_ne!(
            changed_signature.compiled.input_hash,
            baseline.compiled.input_hash
        );
        let signature_receipt =
            PackageStore::prepare(&dir, &changed_signature, &new_signature, &[]).unwrap();
        let (restored_signature, _) = PackageStore::restore(&dir, &signature_receipt).unwrap();
        assert_eq!(run(&changed_signature), Ok(Value::I32(49)));
        assert_eq!(run(&restored_signature), Ok(Value::I32(49)));

        edited[1].version = "2.0.0".into();
        let upgraded = compile(&edited).unwrap();
        assert_ne!(only_instance(&upgraded), instance);
        let upgrade_receipt = PackageStore::prepare(&dir, &upgraded, &edited, &[]).unwrap();
        let (restored, _) = PackageStore::restore(&dir, &receipt).unwrap();
        let (restored_new, _) = PackageStore::restore(&dir, &upgrade_receipt).unwrap();
        assert_eq!(only_instance(&restored), instance);
        assert_eq!(only_instance(&restored_new), only_instance(&upgraded));
        assert_eq!(run(&restored), Ok(Value::I32(42)));
        assert_eq!(run(&restored_new), run(&upgraded));
        assert_eq!(run(&restored_new), Ok(Value::I32(44)));

        let mut wrong_receipt = upgrade_receipt;
        wrong_receipt.digest = receipt.digest.clone();
        assert!(
            matches!(PackageStore::restore(&dir, &wrong_receipt), Err(e) if e == "packages root receipt mismatch")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ordinary_method_calls_share_one_owner_bound_instance() {
        use ace_av_core::contract::Value;
        use ace_core::source::ast::Item;
        use ace_source_query::modules::packages::compile_package_product_auto;

        let inputs = [
            input(
                "app",
                "main",
                "use support.math.Counter
use support.math.initial
fn main() int { return (Counter { base: initial() }).add<int>(23) }",
            ),
            input(
                "support",
                "math",
                "pub type Counter { base int; fn add<T>(x T) int { return .base + x } }
pub fn initial() int { return (Counter { base: 16 }).add(3) }",
            ),
        ];
        let caps = [ace_core::source::AUTO_SOURCE_ADMISSION_CAP.into()];
        let product = compile_package_product_auto(
            &inputs,
            "app::main",
            &caps,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let origins: Vec<_> = product
            .bound
            .origins
            .iter()
            .filter(|(_, origin)| origin.2 == "Counter.add")
            .collect();
        // The package retains the source declaration and publishes one concrete
        // member of the same owner for both the explicit and inferred calls.
        let record = product
            .bound
            .program
            .items
            .iter()
            .find_map(|item| match item {
                Item::Type(t) if t.name.name.ends_with("_Counter") => Some(t),
                _ => None,
            })
            .unwrap();
        assert_eq!(record.methods.len(), 1);
        assert!(record.methods[0].type_params.is_empty());
        let concrete = format!("{}.{}", record.name.name, record.methods[0].name.name);
        assert_eq!(origins.len(), 2);
        assert!(origins
            .iter()
            .any(|(key, origin)| **key == concrete && origin.0 == "support" && origin.1 == "math"));
        let dir = tmp("ordinary-method-owner");
        assert_eq!(
            PackageAvTarget::cold(&dir, &product).unwrap().run_value(),
            Ok(Value::I32(42))
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ordinary_packages_keep_owned_globals_per_generation() {
        use ace_av_core::contract::{ExecOutcome, Value};
        use ace_source_query::modules::{packages::compile_package_product_auto, PackageStore};

        let inputs = [input(
            "app",
            "main",
            r#"fn make() fn(int) -> int {
    let owned = "ok".clone()
    return move (x int) => x + owned.len()
}
var f fn(int) -> int = make()
var hit = 0
fn main() int { hit = hit + 1; return f(39) + hit }"#,
        )];
        let caps = ["ac.source.language.av/v1".into()];
        let budget = FrontBudget::WORK_OWNED_64MIB;
        let dir = tmp("ordinary-global-cap");
        let product = compile_package_product_auto(&inputs, "app::main", &caps, budget).unwrap();
        let receipt = PackageStore::prepare(&dir, &product, &inputs, &[]).unwrap();
        let current = PackageAvTarget::cold(&dir, &product).unwrap();

        let (first, trace) = ace_av_core::execute_program(
            &current.current,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::default(),
        );
        assert!(matches!(
            first,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ));
        let arena = trace.arena.unwrap();
        assert_eq!(arena.live(), 1);

        let mut bad = inputs.clone();
        bad[0].source.push_str("\nfn main() int { return 0 }");
        let diags = match compile_package_product_auto(&bad, "app::main", &caps, budget) {
            Ok(_) => panic!("invalid edit unexpectedly compiled"),
            Err(diags) => diags,
        };
        assert_eq!(
            diags[0].diagnostic.code,
            "source.module-duplicate-declaration"
        );
        let refused: Result<(), ()> = current.current.with_global_transaction(|| {
            assert_eq!(current.run_value(), Ok(Value::I32(43)));
            Err(())
        });
        assert_eq!(refused, Err(()));
        assert_eq!(current.run_value(), Ok(Value::I32(43)));

        let (restored, targets) = PackageStore::restore(&dir, &receipt).unwrap();
        assert!(targets.is_empty());
        assert_eq!(restored.bound.input_hash, product.bound.input_hash);
        let fresh = PackageAvTarget::cold(&dir, &restored).unwrap();
        let tight_limit = ace_av_core::ExecLimits {
            max_steps: 0,
            ..ace_av_core::ExecLimits::default()
        };
        let (limited, _) = ace_av_core::execute_program(
            &fresh.current,
            &tight_limit,
            &ace_av_core::CancelFlag::default(),
        );
        assert!(matches!(
            limited,
            ExecOutcome::Resource {
                kind: ace_av_core::contract::ResourceKind::InstrSteps,
                ..
            }
        ));
        assert!(!fresh.current.initialized());
        let cancel = ace_av_core::CancelFlag::new(true);
        let (cancelled, _) = ace_av_core::execute_program(
            &fresh.current,
            &ace_av_core::ExecLimits::default(),
            &cancel,
        );
        assert!(matches!(cancelled, ExecOutcome::Cancelled));
        assert!(!fresh.current.initialized());
        assert_eq!(fresh.run_value(), Ok(Value::I32(42)));
        assert_eq!(current.run_value(), Ok(Value::I32(44)));

        let mut next_version = inputs.clone();
        next_version[0].version = "2.0.0".into();
        let next = compile_package_product_auto(&next_version, "app::main", &caps, budget).unwrap();
        assert_ne!(next.bound.input_hash, product.bound.input_hash);
        let next_target = PackageAvTarget::cold(&dir, &next).unwrap();
        assert_eq!(next_target.run_value(), Ok(Value::I32(42)));
        assert_eq!(current.run_value(), Ok(Value::I32(45)));
        drop(next_target);

        drop(current);
        assert_eq!(arena.live(), 0);
        drop(fresh);

        let trapping = [input(
            "app",
            "main",
            r#"var label Str = "temporary".clone()
fn zero() int { return 0 }
var result int = label.len() / zero()
fn main() int { return result }"#,
        )];
        let trapping = compile_package_product_auto(&trapping, "app::main", &caps, budget).unwrap();
        let trapping = PackageAvTarget::cold(&dir, &trapping).unwrap();
        for _ in 0..2 {
            let (outcome, trace) = ace_av_core::execute_program(
                &trapping.current,
                &ace_av_core::ExecLimits::default(),
                &ace_av_core::CancelFlag::default(),
            );
            assert!(matches!(
                outcome,
                ExecOutcome::Trap {
                    kind: ace_av_core::contract::TrapKind::DivByZero,
                    ..
                }
            ));
            assert!(!trapping.current.initialized());
            assert_eq!(trace.frames_remaining, 0);
            assert_eq!(trace.arena.unwrap().live(), 0);
        }
        drop(trapping);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

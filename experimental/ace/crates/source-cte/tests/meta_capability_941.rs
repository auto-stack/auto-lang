//! Current-request admission must precede every CT sink/AV effect.
use ace_av_core::{CancelFlag, ExecLimits};
use ace_core::atom_text::Stage;
use ace_core::source::{cst::*, resolve_with, SourceProfile};
use ace_source_cte::envelope::EnvelopeError;
use ace_source_cte::{
    compile_scalar_ct, run_entry_on_av, CtEnvelopeSink, CtExecutedV1, CtProduct, CtProductError,
    CtService, ExecKey, MemoryEnvelopeSink,
};
use std::{cell::RefCell, rc::Rc};

const META2: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at");
const META3: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta3.at");
const CHILD: &str =
    include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/pick40.at");
const HAS: &str = "type Pair { x int\ny int }\nfn pick(b bool) int { if b { return 40 }; return 42 }\nfn main() int { return #{ pick(ct_has_field(Pair, \"z\")) } }";

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Effects {
    probes: usize,
    keys: Vec<ExecKey>,
}
struct Sink {
    inner: MemoryEnvelopeSink,
    effects: Rc<RefCell<Effects>>,
}
impl CtEnvelopeSink for Sink {
    fn probe(&self, key: &ExecKey) -> Result<Option<CtExecutedV1>, EnvelopeError> {
        self.effects.borrow_mut().probes += 1;
        self.inner.probe(key)
    }
    fn publish(&mut self, key: &ExecKey, env: &CtExecutedV1) -> Result<(), EnvelopeError> {
        self.inner.publish(key, env)?;
        self.effects.borrow_mut().keys.push(key.clone());
        Ok(())
    }
}
fn service() -> (CtService<Sink>, Rc<RefCell<Effects>>) {
    let effects = Rc::new(RefCell::new(Effects::default()));
    (
        CtService::new(Sink {
            inner: MemoryEnvelopeSink::new(),
            effects: effects.clone(),
        }),
        effects,
    )
}
fn caps(meta: bool) -> Vec<String> {
    let mut caps = vec![CT_CAPABILITY.into(), INSTANCES_CAPABILITY.into()];
    if meta {
        caps.push("cte.meta.fields/v1".into());
    }
    caps
}
fn compile(svc: &mut CtService<Sink>, src: &str, meta: bool) -> Result<CtProduct, CtProductError> {
    compile_scalar_ct(
        svc,
        src,
        "main.at",
        &caps(meta),
        "plan941",
        "meta-admission",
    )
}
fn value(product: &CtProduct) -> i32 {
    let (value, _) = run_entry_on_av(
        &product.checked,
        "d_main",
        941,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .unwrap();
    match value {
        ace_av_core::contract::Value::I32(n) => n,
        other => panic!("{other:?}"),
    }
}
fn core_diagnostics(src: &str, meta: bool) -> Vec<ace_core::atom_text::Diagnostic> {
    let FrontOutcome::Ok(parsed) = parse_front_with_caps(
        FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &caps(meta),
    ) else {
        panic!("fixture must parse")
    };
    resolve_with(&parsed.program, &caps(meta), SourceProfile::Scalar)
        .err()
        .unwrap_or_default()
}
fn rejected_without_effects(src: &str, meta: bool, code: &str, spelling: &str) {
    let ds = core_diagnostics(src, meta);
    let d = ds
        .iter()
        .find(|d| d.code == code)
        .expect("existing typed core cause");
    assert_eq!(d.stage, Stage::Bind);
    let start = src.find(spelling).unwrap();
    assert_eq!(d.span.start, start);
    let end = if spelling == "fn " {
        src.find('\n').unwrap()
    } else {
        start + src[start..].find(')').unwrap() + 1
    };
    assert_eq!(d.span.end, end);
    let (mut svc, effects) = service();
    let before = *svc.counters();
    let result = compile(&mut svc, src, meta);
    assert_eq!(
        *effects.borrow(),
        Effects::default(),
        "reject before even probing the sink"
    );
    assert_eq!(*svc.counters(), before);
    assert!(svc.exec_log().is_empty());
    let Err(CtProductError::Diagnostics(ds)) = result else {
        panic!("expected admission diagnostics, got {result:?}")
    };
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].stage, Stage::Bind);
    assert_eq!(ds[0].code, "source.cte.bind");
    assert_eq!(ds[0].span.start, 0);
    assert_eq!(ds[0].span.end, 0);
    // Existing product aggregation is formed from the typed Resolve error.
    assert_eq!(
        ds[0].message,
        ace_source_query::apply::BindFactsError::Resolve(core_diagnostics(src, meta)).to_string()
    );
}

#[test]
fn cold_missing_meta_capability_has_no_effects_941() {
    rejected_without_effects(
        META2,
        false,
        "source.cte.meta-capability",
        "ct_field_count(Pair)",
    );
}

#[test]
fn cold_has_field_missing_capability_has_no_effects_941() {
    rejected_without_effects(
        HAS,
        false,
        "source.cte.meta-capability",
        "ct_has_field(Pair",
    );
}

#[test]
fn nested_and_callee_demands_require_current_capability_941() {
    let nested = META2.replace("ct_field_count(Pair)", "#{ ct_field_count(Pair) }");
    rejected_without_effects(
        &nested,
        false,
        "source.cte.meta-capability",
        "ct_field_count(Pair)",
    );
    let callee = "type Pair { x int\ny int }\nfn helper() int { return #{ ct_field_count(Pair) } }\nfn main() int { return #{ helper()+40 } }";
    rejected_without_effects(
        callee,
        false,
        "source.cte.meta-capability",
        "ct_field_count(Pair)",
    );
    let (mut svc, _) = service();
    assert_eq!(value(&compile(&mut svc, &nested, true).unwrap()), 42);
    assert_eq!(value(&compile(&mut svc, callee, true).unwrap()), 42);
}

#[test]
fn revoked_capability_cannot_probe_but_regrant_hits_old_page_941() {
    for src in [META2, HAS] {
        let (mut svc, effects) = service();
        let original = compile(&mut svc, src, true).unwrap();
        assert_eq!(value(&original), 42);
        let before = *svc.counters();
        let old_effects = effects.borrow().clone();
        let log_len = svc.exec_log().len();
        svc.set_forbid_av(true);
        let revoked = compile(&mut svc, src, false);
        assert!(
            matches!(revoked, Err(CtProductError::Diagnostics(_))),
            "{revoked:?}"
        );
        assert_eq!(*svc.counters(), before);
        assert_eq!(*effects.borrow(), old_effects);
        assert_eq!(svc.exec_log().len(), log_len);
        assert_eq!(value(&original), 42, "old checked product remains usable");
        let hit = compile(&mut svc, src, true).unwrap();
        assert_eq!(value(&hit), 42);
        assert_eq!(svc.counters().ct_execute, before.ct_execute);
        assert_eq!(
            svc.counters().ct_freeze_publishes,
            before.ct_freeze_publishes
        );
        assert_eq!(svc.counters().ct_hits, before.ct_hits + 1);
        assert_eq!(effects.borrow().keys, old_effects.keys);
        assert!(svc.exec_log().last().unwrap().from_cache);
    }
}

#[test]
fn ordinary_names_without_meta_and_protected_declarations_941() {
    for name in ["ct_field_count", "ct_has_field"] {
        for (params, body, args) in [("x int", "x+7", "35"), ("", "42", "")] {
            let src = format!(
                "fn {name}({params}) int {{ return {body} }}\nfn main() int {{ return #{{ {name}({args}) }} }}"
            );
            let (mut svc, _) = service();
            let product = compile(&mut svc, &src, false).unwrap();
            assert_eq!(value(&product), 42);
            assert_eq!(product.served.meta_queries, 0);
            assert_eq!(svc.counters().ct_execute, 1);
            rejected_without_effects(&src, true, "source.cte.meta-name-conflict", "fn ");
        }
    }
}

#[test]
fn declared_user_functions_cannot_be_folded_as_meta_selectors_941() {
    // Without the meta grant these are ordinary, ill-typed function calls:
    // the record name cannot silently turn their arguments into a selector.
    // The old scheduler instead folded both calls and returned 2 / true.
    for src in [
        "type Pair { x int\ny int }\nfn ct_field_count(x int) int { return x+7 }\nfn main() int { return #{ ct_field_count(Pair) } }",
        "type Pair { x int\ny int }\nfn ct_has_field(x int, y int) bool { return true }\nfn main() int { if #{ ct_has_field(Pair, \"x\") } { return 42 }; return 40 }",
    ] {
        let (mut svc, effects) = service();
        let result = compile(&mut svc, src, false);
        assert!(result.is_err(), "ordinary invalid call must not become a meta result: {result:?}");
        let Err(CtProductError::Plan(ace_source_cte::PlanError::Demand { err, .. })) = result else {
            panic!("expected the ordinary call's existing frontend projection")
        };
        assert_eq!(err.code, "cte.frontend/bind");
        assert!(!err.av_reached);
        assert_eq!(svc.counters().ct_execute, 0);
        assert_eq!(svc.counters().ct_freeze_publishes, 0);
        assert!(effects.borrow().keys.is_empty());
    }
}

#[test]
fn protected_template_names_are_checked_before_concretization_941() {
    let src = "fn ct_field_count<T>(x T) T { return x }\nfn main() int { return #{ ct_field_count(42) } }";
    rejected_without_effects(src, true, "source.cte.meta-name-conflict", "fn ");
    let (mut svc, effects) = service();
    assert_eq!(value(&compile(&mut svc, src, false).unwrap()), 42);
    let before = *svc.counters();
    let old_effects = effects.borrow().clone();
    let rejected = compile(&mut svc, src, true);
    assert!(
        matches!(rejected, Err(CtProductError::Diagnostics(_))),
        "{rejected:?}"
    );
    assert_eq!(*svc.counters(), before);
    assert_eq!(*effects.borrow(), old_effects);
}

#[test]
fn admission_seam_accepts_unserved_hash_and_ordinary_names_941() {
    for (src, meta) in [(META2, true), (HAS, true), ("fn ct_field_count(x int) int { return x+7 }\nfn main() int { return #{ ct_field_count(35) } }", false)] {
        assert!(core_diagnostics(src, meta).is_empty(), "{:#?}", core_diagnostics(src, meta));
    }
    // A Hash's child must go through the ordinary iterative expression
    // binder too (not its leaf-only arm). Admission must accept arithmetic
    // and unary nodes before the scheduler structurally serves them.
    for expression in [
        "40+2".to_string(),
        "-(-42)".to_string(),
        (0..128).fold("42".to_string(), |s, _| format!("#{{ {s} }}")),
    ] {
        let src = format!("fn main() int {{ return #{{ {expression} }} }}");
        assert!(core_diagnostics(&src, false).is_empty());
    }
}

#[test]
fn valid_shape_edits_and_instance_child_still_execute_941() {
    let (mut svc, effects) = service();
    assert_eq!(value(&compile(&mut svc, META2, true).unwrap()), 42);
    assert_eq!(value(&compile(&mut svc, META3, true).unwrap()), 43);
    let has_z = HAS.replace("y int", "y int\nz int");
    assert_eq!(value(&compile(&mut svc, HAS, true).unwrap()), 42);
    assert_eq!(value(&compile(&mut svc, &has_z, true).unwrap()), 40);
    assert_eq!(effects.borrow().keys.len(), 4);
    assert_eq!(effects.borrow().keys[0].args_canonical, "i32:2,i32:40");
    assert_eq!(effects.borrow().keys[1].args_canonical, "i32:3,i32:40");
    assert_eq!(effects.borrow().keys[2].args_canonical, "bool:false");
    assert_eq!(effects.borrow().keys[3].args_canonical, "bool:true");
    let child = compile(&mut svc, CHILD, false).unwrap();
    assert_eq!(value(&child), 40);
    assert_eq!(child.instances.len(), 1);
    assert_eq!(value(&compile(&mut svc, CHILD, true).unwrap()), 40);
    assert_eq!(
        value(
            &compile(
                &mut svc,
                &CHILD.replace("return true", "return false"),
                false
            )
            .unwrap()
        ),
        42
    );
}

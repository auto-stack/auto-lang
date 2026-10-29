//! Independent oracle at the real source -> shared MIR -> ABC -> AV seam.
use auto_av_probe::collections_864_support::compile_and_execute;

fn assert_42(source: &str) {
    let run = compile_and_execute(source, "d_main").expect("full collections chain");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
    assert!(
        run.allocs >= 2,
        "list and owned text must actually allocate"
    );
}

#[test]
fn named_variant_in_owned_list_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/list_variant.at"
    ));
}

#[test]
fn nested_forward_variant_cross_function_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/nested_variant.at"
    ));
}

#[test]
fn optional_variant_in_owned_list_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/optional_variant.at"
    ));
}

#[test]
fn owned_text_clone_runs_and_both_buffers_release() {
    let run = compile_and_execute("fn main() int { let a Str = \"中\"\n let b = a.clone()\n return a.len() * 40 + b.len() * 2 }", "d_main").unwrap();
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, 2);
    assert_eq!(run.frees, 2);
}

#[test]
fn borrowed_copyable_record_signature_and_return_agree() {
    let run = compile_and_execute("type Pair { x int; y int }\nfn bump(p Pair) Pair { return Pair(p.x + 1, p.y) }\nfn main() int { let a = Pair(19, 22)\n let b = bump(a)\n return a.x + b.y + 1 }", "d_main").unwrap();
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
}

#[test]
fn single_function_collection_units_preserve_clone_and_borrowed_signatures() {
    for source in [
        "fn main() int { let a Str = \"中\"\n let b = a.clone()\n return a.len() * 40 + b.len() * 2 }",
        "type Pair { x int; y int }\nfn bump(p Pair) Pair { return Pair(p.x + 1, p.y) }\nfn main() int { let a = Pair(19, 22)\n let b = bump(a)\n return a.x + b.y + 1 }",
    ] {
        let compiled=auto_ac_prototype::source::compile_collections_source(source,"p889.at",&[]).unwrap();
        let bundle=compiled.checked.bundle();
        let names:Vec<_>=bundle.modules[0].defs.iter().map(|d|d.id_text.clone()).collect();
        let units:Vec<_>=names.iter().map(|name|(name.as_str(),auto_mir_lower::lower_function_collections_unit(bundle,name,&[],0).unwrap())).collect();
        for (_,unit) in &units { auto_av_core::verify_mir(unit).unwrap(); }
        let mut session=auto_av_probe::collections_session::CollectionsUpdateSession::new(None,"d_main");
        let result=session.apply_revision(&units).unwrap();
        assert_eq!(result.value,Some(42)); assert_eq!(result.live_after_release,0);
        assert_eq!(result.allocs,result.frees); assert!(result.gate.pass());
    }
}

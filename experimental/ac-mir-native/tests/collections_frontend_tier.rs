mod common;
use ac_mn::collections_frontend::cold;
#[test]
fn bounded_cold_original_sources_and_supplementary_owned_cases_run_native() {
    for (id, value) in [
        ("L01", 42),
        ("L02", 42),
        ("L03", 0),
        ("L04", 42),
        ("L05", 42),
        ("L06", 70),
        ("L07", 42),
        ("V01", 42),
        ("V02", 42),
        ("V03", 3),
        ("V04", 42),
        ("V05", 42),
        ("V06", 42),
        ("V07", 42),
        ("LSTR864", 51),
        ("FIN864A", 42),
        ("FIN864B", 53),
    ] {
        let source = std::fs::read_to_string(
            common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at")),
        )
        .unwrap();
        let result = cold(&source, "p889.at").unwrap_or_else(|e| panic!("{id}: {e:?}"));
        assert_eq!(result.real_body_plans, result.units.len());
        let run = common::collections::run_units(
            &result.units,
            "d_main",
            &format!("bounded-{id}"),
            Some(&source),
        );
        assert_eq!(
            run.exit_code,
            Some(value),
            "{id}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        common::collections::assert_balanced(&run);
        if id == "L06" {
            assert_eq!(
                ac_mn::collections_observe::decode(run.exit_code, &run.stderr)
                    .unwrap()
                    .outcome,
                ac_mn::collections_observe::Outcome::Trap(21)
            );
        }
    }
}
#[test]
fn bounded_cold_named_nested_optional_and_resource_caller_trap() {
    for (tag,source,exit) in [
        ("list-variant",include_str!("../../ac-core/tests/fixtures/collections_889/list_variant.at"),42),
        ("nested",include_str!("../../ac-core/tests/fixtures/collections_889/nested_variant.at"),42),
        ("optional",include_str!("../../ac-core/tests/fixtures/collections_889/optional_variant.at"),42),
        ("recursive", "fn main() int { return count(42) }\nfn count(n int) int { if n > 0 { return count(n-1)+1 }\n return 0 }",42),
        ("text-call", "fn make() Str { return \"中\" }\nfn read(s Str) int { return s.len() }\nfn main() int { let s=make()\n return read(s)+41 }",42),
        ("record-view-call", "type Item { name Str; code int }\nfn read(p Item) int { return p.name.len() }\nfn main() int { let p=Item(\"hello\",37)\n return read(p)+p.code }",42),
        ("caller-trap","enum Token { Data(Str)\nEnd }\nfn fail() int { var xs=List<int>.new()\n return xs.get(0) }\nfn main() int { let t=Token.Data(\"caller\")\n var xs=List<Str>.new()\n xs.push(\"still-live\")\n return fail() }",70),
    ] {
        let result=cold(source,"p889.at").unwrap();
        let run=common::collections::run_units(&result.units,"d_main",&format!("bounded-{tag}"),Some(source));
        assert_eq!(run.exit_code,Some(exit),"{tag}: {}",String::from_utf8_lossy(&run.stderr));
        common::collections::assert_balanced(&run);
        if exit==70 {assert_eq!(ac_mn::collections_observe::decode(run.exit_code,&run.stderr).unwrap().outcome,ac_mn::collections_observe::Outcome::Trap(21));}
    }
}

#[test]
fn bounded_cold_rejects_over_input_cap_before_frontend_work() {
    let source = " ".repeat(ac_mn::config::MAX_INPUT_BYTES + 1);
    let (result, work) =
        auto_ac_prototype::body_work::capture_body_work(|| cold(&source, "p889.at"));
    let errors = result.err().expect("over-cap input must reject");
    assert!(errors.iter().any(|d| d.code == "mn.resource.input-size"));
    assert!(work.is_empty());
}

#[test]
fn bounded_units_do_not_bind_unreferenced_function_interfaces() {
    let a = cold(
        "fn unused(p int) int { return p }\nfn main() int { return 42 }",
        "p889.at",
    )
    .unwrap();
    let b = cold(
        "fn unused(p bool) bool { return p }\nfn main() int { return 42 }",
        "p889.at",
    )
    .unwrap();
    let main = |units: Vec<auto_av_core::mir::MirModule>| {
        units
            .into_iter()
            .find(|m| m.funcs[0].name == "d_main")
            .unwrap()
    };
    let am = main(a.units);
    let bm = main(b.units);
    assert!(am.imports.is_empty() && bm.imports.is_empty());
    assert_eq!(ac_mn::cache::unit_fp(&am), ac_mn::cache::unit_fp(&bm));
    let runtime = ac_mn::collections_runtime::Identity {
        abi: ac_mn::collections_runtime::ABI.into(),
        build_key: "test".into(),
        content_hash: "test".into(),
    };
    assert_eq!(
        ac_mn::cache::CoffMeta::for_collections_unit(&am, ac_mn::cache::unit_fp(&am), &runtime)
            .unwrap(),
        ac_mn::cache::CoffMeta::for_collections_unit(&bm, ac_mn::cache::unit_fp(&bm), &runtime)
            .unwrap()
    );
    assert_eq!(
        ac_mn::collections_lower::emit_unit_coff(&am).unwrap().bytes,
        ac_mn::collections_lower::emit_unit_coff(&bm).unwrap().bytes
    );
}

#[test]
fn bounded_cold_frozen_workload_real_peak_and_result() {
    let load = ac_mn::collections_workload::generate(0);
    let baseline = auto_av_probe::mempeek::sample_private_bytes();
    let sampler = ac_mn::resources::TreePeakSampler::start(5);
    let started = std::time::Instant::now();
    let result = cold(&load.source, "p889.at").unwrap();
    assert_eq!(result.units.len(), 116);
    assert_eq!(result.real_body_plans, 116);
    let run = common::collections::run_units(
        &result.units,
        "d_main",
        "bounded-frozen-load",
        Some(&load.source),
    );
    assert_eq!(
        run.exit_code,
        Some(load.expected),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    common::collections::assert_balanced(&run);
    let elapsed = started.elapsed().as_millis();
    let peak = sampler.stop();
    let delta = peak.own_peak.saturating_sub(baseline);
    println!("bounded-cold wall_ms={elapsed} private_base={baseline} private_peak={} delta={delta} tree_peak={} ticks={} child_ticks={}",peak.own_peak,peak.tree_peak,peak.ticks,peak.child_ticks);
    assert!(baseline > 0 && peak.ticks > 0 && peak.own_peak > 0);
    assert!(
        delta <= 128 << 20,
        "cold worker delta {delta} exceeds unchanged128MiB limit"
    );
}

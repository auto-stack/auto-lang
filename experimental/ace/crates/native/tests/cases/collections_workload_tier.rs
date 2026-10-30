use ace_native::collections_workload::*;
#[path = "../common/mod.rs"]
mod common;
#[test]
fn frozen_collection_workload_keeps_original_scale_and_real_owned_helper_edit() {
    let a = generate(0);
    let b = generate(1);
    let back = generate(0);
    assert!(a.source.len() >= ace_native::workload::MIN_SOURCE_BYTES);
    assert_eq!(a.source.matches("fn ").count(), FUNCTIONS);
    assert_eq!(a.source, back.source);
    assert_ne!(a.source, b.source);
    assert_eq!(b.expected, a.expected + 1);
    let tail = |s: &str| s[s.find("fn fn_leaf_1").unwrap()..].to_owned();
    assert_eq!(
        tail(&a.source),
        tail(&b.source),
        "main and old115 function bodies must remain byte-identical"
    );
    let compiled =
        ace_core::source::compile_collections_source(&a.source, "frozen889.at", &[]).unwrap();
    let m = ace_lowering::lower_checked_collections(&compiled.checked, "d_main", 0).unwrap();
    ace_av_core::verify_mir(&m).unwrap();
    let units = ace_av::cache::split_units(&m).unwrap();
    assert_eq!(units.len(), FUNCTIONS);
    for unit in &units {
        ace_native::collections_lower::admit_unit(unit).unwrap();
    }
    let helper = units
        .iter()
        .find(|u| u.funcs[0].name == "d_collections_helper")
        .unwrap();
    let instructions = format!("{:?}", helper.funcs[0]);
    for op in [
        "Clone",
        "ListNew",
        "ListPush",
        "ListRemove",
        "VariantPack",
        "Aggregate",
    ] {
        assert!(instructions.contains(op), "missing {op}");
    }
    println!(
        "source_bytes={} functions={} expected={} source_blake3={}",
        a.source.len(),
        units.len(),
        a.expected,
        blake3::hash(a.source.as_bytes())
    );
}

#[test]
fn frozen_workload_first_native_pe_matches_independent_mirror() {
    let load = generate(0);
    let baseline = ace_av::mempeek::sample_private_bytes();
    let sampler = ace_native::resources::TreePeakSampler::start(5);
    let started = std::time::Instant::now();
    let run = common::collections::run_source(&load.source, "frozen-load-first");
    let elapsed = started.elapsed().as_millis();
    let peak = sampler.stop();
    assert_eq!(
        run.exit_code,
        Some(load.expected),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    common::collections::assert_balanced(&run);
    assert!(
        baseline > 0 && peak.ticks > 0 && peak.own_peak > 0,
        "sampler unavailable"
    );
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    println!("feasibility-only profile={profile} wall_ms={elapsed} worker_private_base={baseline} worker_private_peak={} worker_delta={} tree_peak={} ticks={} child_ticks={}",peak.own_peak,peak.own_peak.saturating_sub(baseline),peak.tree_peak,peak.ticks,peak.child_ticks);
}

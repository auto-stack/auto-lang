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

#[test]
fn cold_batches_are_exact_single_body_mir_coff_and_coordinates() {
    use auto_ac_prototype::source as ac;
    let mut sources = vec![ac_mn::collections_workload::generate(0).source];
    for id in [
        "L01",
        "L02",
        "L03",
        "L04",
        "L05",
        "L06",
        "L07",
        "V01",
        "V02",
        "V03",
        "V04",
        "V05",
        "V06",
        "V07",
        "LSTR864",
        "FIN864A",
        "FIN864B",
        "N889-list-variant",
        "N889-nested-variant",
        "N889-optional-variant",
    ] {
        let path =
            common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at"));
        sources.push(std::fs::read_to_string(path).unwrap());
    }
    sources.push("fn a() Str { return \"中\" }\nfn b() int { return a().len() }\nfn c(n int) int { if n > 0 { return c(n-1)+1 }\n return 0 }\nfn main() int { return b()+c(41) }".into());
    for src in sources {
        let p = ac::parse_program_with(&src, ac::SourceProfile::Collections).unwrap();
        let mut env = ac::resolve_env(&p, &[], ac::SourceProfile::Collections).unwrap();
        env.typecheck_all().unwrap();
        let names = ac::env_fn_names(&env);
        let batched = cold(&src, "p889.at").unwrap();
        for (i, name) in names.iter().enumerate() {
            let (parts, _) = ac::emit_function_body(&env, i, &src, "p889.at").unwrap();
            let single = auto_mir_lower::hir_adapter::lower_function_collections_unit(
                &parts.bundle,
                &format!("d_{name}"),
                &[],
                auto_av_core::contract::fnv1a64(src.as_bytes()),
            )
            .unwrap();
            assert_eq!(
                auto_av_core::abc::encode_abc(&single).unwrap().to_flat(),
                auto_av_core::abc::encode_abc(&batched.units[i])
                    .unwrap()
                    .to_flat(),
                "all MIR fields including coordinates: {name}"
            );
            assert_eq!(
                ac_mn::cache::unit_fp(&single),
                ac_mn::cache::unit_fp(&batched.units[i])
            );
            assert_eq!(
                ac_mn::collections_lower::emit_unit_coff(&single)
                    .unwrap()
                    .bytes,
                ac_mn::collections_lower::emit_unit_coff(&batched.units[i])
                    .unwrap()
                    .bytes,
                "COFF {name}"
            );
        }
        let chosen: Vec<_> = (0..names.len().min(4)).rev().collect();
        let ((parts, r), work) = auto_ac_prototype::body_work::capture_body_work(|| {
            ac::emit_collections_function_batch(&env, &chosen, &src, "p889.at").unwrap()
        });
        assert_eq!(r.real_body_plans, chosen.len());
        assert_eq!(r.scaffold_fns, names.len() - chosen.len());
        assert_eq!(work.len(), chosen.len());
        for &i in &chosen {
            assert_eq!(work[names[i]].plan, 1);
            assert_eq!(work[names[i]].bind, 1);
            let actual = auto_mir_lower::hir_adapter::lower_function_collections_unit(
                &parts.bundle,
                &format!("d_{}", names[i]),
                &[],
                auto_av_core::contract::fnv1a64(src.as_bytes()),
            )
            .unwrap();
            assert_eq!(
                auto_av_core::abc::encode_abc(&actual).unwrap().to_flat(),
                auto_av_core::abc::encode_abc(&batched.units[i])
                    .unwrap()
                    .to_flat(),
                "unordered batch {i}"
            );
        }
    }
}

#[test]
fn cold_batch_boundary_and_large_body_fallback() {
    use auto_ac_prototype::source as ac;
    let src = format!(
        "fn main() int {{ let s Str=\"{}\"\n return s.len() }}\nfn other() int {{ return 42 }}",
        "x".repeat(17 * 1024)
    );
    let (result, work) =
        auto_ac_prototype::body_work::capture_body_work(|| cold(&src, "p889.at").unwrap());
    assert_eq!(result.real_body_plans, 2);
    assert_eq!(
        result.scaffold_declarations, 2,
        "large body uses original single path"
    );
    for counts in work.values() {
        assert_eq!(
            (counts.resolve, counts.typecheck, counts.plan, counts.bind),
            (1, 1, 1, 1)
        );
    }
    let mut env = result.env;
    for indices in [vec![], vec![0, 0], vec![2], vec![0, 1, 0, 1, 0]] {
        assert!(ac::emit_collections_function_batch(&env, &indices, &src, "p889.at").is_err());
    }
    env.close();
    assert!(ac::emit_collections_function_batch(&env, &[0], &src, "p889.at").is_err());
    let p =
        ac::parse_program_with("fn main() int { return 42 }", ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&p, &[], ac::SourceProfile::Scalar).unwrap();
    env.typecheck_all().unwrap();
    assert!(ac::emit_collections_function_batch(
        &env,
        &[0],
        "fn main() int { return 42 }",
        "p889.at"
    )
    .is_err());
}

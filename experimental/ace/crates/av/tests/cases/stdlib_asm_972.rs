//! PLAN-972 T-02 assembled stdlib conformance: the frozen assembled std set
//! (`std.list` — the 946 slice superset + closure-HOF family
//! reduce/find/any/all/filter/map) through the packages-aggregates-ct lane
//! (`assemble_stdlib_aggregates_inputs` + `compile_assembled_aggregates_package`)
//! observed on the AV end with independent hand-computed oracles; std identity
//! reservation (tampered source / provider layers) rejects pre-run; closure
//! params stay rejected on the plain aggregates lane (lane isolation).
//! Four-end producer legs, store roundtrip and fault paths are T-03/T-04.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, PackageImplLayer, PackageModuleInput, ProviderChoice,
    ProviderKind, TargetOs, STDLIB_PACKAGE, STDLIB_VERSION,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs,
    compile_package_product_aggregates, stdlib_aggregates_inputs,
};

/// The V-DETAIL-01 program shape over the HOF family:
/// reduce=15, find(>3)=Some(4), all(>0)=true, any(<0)=false,
/// filter(>1)=[2,3,4,5] (len=4 sum=14), map(*10) sum=140
/// → 15*10^5 + 4*10^4 + 4*10^3 + 14*10 + 140 + 1 = 1544281.
const HOF_MAIN: &str = r#"
use std.list.reduce
use std.list.find
use std.list.any
use std.list.all
use std.list.filter
use std.list.map
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = reduce(xs, 0, (acc int, x int) => { return acc + x })
    let f = find(xs, (x int) => { return x > 3 })
    var got = 0
    is f {
        Some(v) -> { got = v }
        None -> { got = 0 }
    }
    let ys = filter(xs, (x int) => { return x > 1 })
    let zs = map(ys, (x int) => { return x * 10 })
    var ys_sum = 0
    for y in ys {
        ys_sum = ys_sum + y
    }
    var zs_sum = 0
    for z in zs {
        zs_sum = zs_sum + z
    }
    var code = s * 100000 + got * 10000 + ys.len() * 1000 + ys_sum * 10 + zs_sum
    if all(xs, (x int) => { return x > 0 }) {
        code = code + 1
    }
    if any(xs, (x int) => { return x < 0 }) {
        code = code + 2
    }
    return code
}
"#;

const SLICE_MAIN: &str = r#"
use std.list.sum
use std.list.find_first_gt
use std.list.pop
use std.list.total_len
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = sum(xs)
    let f = find_first_gt(xs, 3)
    var got = 0
    is f {
        Some(v) -> { got = v }
        None -> { got = 0 }
    }
    let p = pop(xs)
    var popped = 0
    is p {
        Some(v) -> { popped = v }
        None -> { popped = 0 }
    }
    var ss = List<Str>.new()
    ss.push("ab")
    ss.push("cde")
    return s * 1000 + got * 100 + popped * 10 + total_len(ss)
}
"#;

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/asm972/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn asm_lane_product(
    tag: &str,
    main_src: &str,
) -> ace_source_query::modules::packages::PackageProduct {
    let mut ctx = PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(
        &[("main.at".to_string(), main_src.to_string())],
        "app::main",
        &mut ctx,
    )
    .unwrap_or_else(|e| panic!("{tag} assemble: {e}"));
    compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|ds| {
            panic!(
                "{tag} compile: {} {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        })
}

fn run_av(tag: &str, product: &ace_source_query::modules::packages::PackageProduct) -> Value {
    let av =
        PackageAvTarget::cold(&scratch(&format!("av-{tag}")), product).unwrap_or_else(|e| panic!("av {tag}: {e}"));
    av.run_value().unwrap_or_else(|e| panic!("run {tag}: {e}"))
}

/// The HOF family + slice superset on AV×Auto through the assembled
/// aggregates lane, with hand-set oracles (1544281 / 15455).
#[test]
fn stdlib_asm_972_hof_and_slice_av() {
    let p = asm_lane_product("hof", HOF_MAIN);
    assert_eq!(run_av("hof", &p), Value::I32(1544281));

    let p = asm_lane_product("slice", SLICE_MAIN);
    // sum=15, find_first_gt(3)=4, pop=5, total_len(["ab","cde"])=5
    // → 15*1000 + 4*100 + 5*10 + 5 = 15455.
    assert_eq!(run_av("slice", &p), Value::I32(15455));
}

/// `std` identity reservation on the assembled lane: a tampered `std.list`
/// source, and a provider layer riding `std.list`, both reject BEFORE any
/// compile/run with the named reservation code.
#[test]
fn stdlib_asm_972_std_identity_reserved() {
    use ace_core::source::packages::bind_packages_assembled_aggregates;
    use ace_source_query::modules::packages::assembled_aggregates_caps;
    let ctx = PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let caps = assembled_aggregates_caps();
    let files = ace_core::source::packages::stdlib_assembled_files();
    let frozen_list = files
        .iter()
        .find(|(n, _)| n == "list.at")
        .map(|(_, c)| c.clone())
        .expect("frozen assembled list module");
    let app = PackageModuleInput {
        package: "app".into(),
        version: STDLIB_VERSION.into(),
        id: "main".into(),
        source: "fn main() int { return 0 }".into(),
        provider: None,
    };

    // tampered std.list source
    let tampered = PackageModuleInput {
        package: STDLIB_PACKAGE.into(),
        version: STDLIB_VERSION.into(),
        id: "list".into(),
        source: format!("{frozen_list}\npub fn evil(x int) int {{ return x }}"),
        provider: None,
    };
    let e = bind_packages_assembled_aggregates(
        &[app.clone(), tampered],
        "app::main",
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_err();
    assert_eq!(e.diagnostic.code, "ace.stdlib-reserved", "{e:?}");

    // provider layer riding std.list (frozen public source)
    let layered = PackageModuleInput {
        package: STDLIB_PACKAGE.into(),
        version: STDLIB_VERSION.into(),
        id: "list".into(),
        source: frozen_list,
        provider: Some(PackageImplLayer {
            kind: ProviderKind::At,
            source: "fn never(x int) int { return x }".into(),
        }),
    };
    let e = bind_packages_assembled_aggregates(
        &[app, layered],
        "app::main",
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_err();
    assert_eq!(e.diagnostic.code, "ace.stdlib-reserved", "{e:?}");
}

/// Lane isolation: closure-typed params stay a named syntax rejection on the
/// plain aggregates lane (packages-aggregates without the ct/asm face; the
/// composed packages-aggregates-ct lane admits them — see the HOF test above).
#[test]
fn stdlib_asm_972_closure_params_still_rejected_on_plain_lanes() {
    use ace_core::source::packages::PACKAGES_CAPABILITY;
    const CLOSURE_SRC: &str = "fn apply(f fn(int) -> int, v int) int { return f(v) }\nfn main() int { return apply((n int) => { return n + 1 }, 41) }";
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: CLOSURE_SRC.into(),
        provider: None,
    });
    let ds = match compile_package_product_aggregates(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("closure-typed params unexpectedly admitted on the plain aggregates lane"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].diagnostic.code, "source.syntax", "{ds:?}");
}

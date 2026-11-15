//! PLAN-977 B-PY-I: the legacy a2p public-surface probe through the ACE
//! Python lane (Checked HIR -> a2x -> `ace-a2x-python` -> real OS Python).
//!
//! The legacy `trans/python.rs` corpus (test/a2p/**: hello/array/func/
//! struct/enum/method/if/for/is/str/option/result/generics/collections)
//! is BYTE-GOLDEN emit on the legacy AST; this suite carries its PUBLIC
//! OBLIGATIONS through the new lane with INDEPENDENT execution oracles
//! (hand-computed entry values observed as `value:<n>` on stdout — the
//! new lane's frozen observation face; the legacy print-per-line form is
//! an old-emitter artifact, string obligations ride `.len()`/scalar
//! arithmetic through the admitted domains).
//!
//! Each case names the legacy corpus family it preserves; a construct
//! the lane rejects is a NAMED gap (never silently dropped) — see the
//! PLAN-977 §5 B-PY-I freeze for the gap ledger.

use std::path::PathBuf;

use ace_a2x_python::{
    python_runtime_source, verify_python_fileset_manifest, PythonTargetProvider, PY_FILESET_SCHEMA,
    PY_PRODUCER, PY_RT_FILESET_SCHEMA,
};
use ace_a2x_store::A2xSession;
use ace_a2x_worker::pyrun::{resolve_python, run_py_fileset};
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::SourceProfile;

fn out_root(sub: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-out-p977")
        .join(format!("{sub}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn py_tool() -> ace_a2x_worker::pyrun::ResolvedPython {
    let rt = python_runtime_source(&[]);
    let rt_hash = sha256_hex(rt.as_bytes());
    resolve_python(&rt_hash).expect("OS Python interpreter required (missing Python cannot PASS)")
}

enum Route {
    Session,
    PackagesAggregates,
}

/// One legacy public-surface obligation, ported to the admitted ACE lane.
struct PyCase {
    route: Route,
    /// legacy a2p corpus family preserved (e.g. "010_if/011_for")
    family: &'static str,
    name: &'static str,
    profile: SourceProfile,
    caps: &'static [&'static str],
    src: &'static str,
    /// hand-computed independent oracle (stdout of the real Python run)
    expected: &'static [u8],
}

const CASES: &[PyCase] = &[
    // 010_if + 011_for: conditional + loop accumulation (oracle: 0+1+2+3+4=10)
    PyCase {
        family: "010_if/011_for",
        name: "if_for_sum",
        route: Route::Session,
        profile: SourceProfile::Scalar,
        caps: &[],
        // the scalar profile's loop is the CONDITIONAL `for <cond>` form
        src: "fn main() int {\n    var s = 0\n    for s < 10 { s = s + 1 }\n    if s == 10 { return s }\n    return 0\n}",
        expected: b"value:10\n",
    },
    // 003_func: plain function calls and shadow-free locals (oracle: 8*8+1=65)
    PyCase {
        family: "003_func",
        name: "func_calls",
        route: Route::Session,
        profile: SourceProfile::Scalar,
        caps: &[],
        src: "fn square(x int) int { return x * x }\nfn main() int {\n    let a = square(8)\n    return a + 1\n}",
        expected: b"value:65\n",
    },
    // 08_generics: the legacy #[with(T)] identity obligation rides the
    // ACE template surface (`<T>`); the SCALAR profile under the
    // PLAN-935 instances capability carries the desugared concrete
    // program (`ac.source.instances.scalar/v1`) — the caps-blind trims
    // and the packages binder reject `<` by design (named routing
    // facts, not language gaps)
    PyCase {
        family: "08_generics",
        name: "generic_identity",
        route: Route::Session,
        profile: SourceProfile::Scalar,
        caps: &["ac.source.instances.scalar/v1"],
        src: "fn id<T>(x T) T { return x }
fn main() int {
    let a = id(42)
    return a
}",
        expected: b"value:42
",
    },
    // 008_method/11_methods: static + view methods on a record
    // (implicit-self `.field`; oracle: 42 + 1 = 43)
    PyCase {
        family: "008_method/11_methods",
        name: "record_methods",
        route: Route::Session,
        profile: SourceProfile::TextRecord,
        caps: &[],
        src: "type Counter {\n    value int\n    static fn new(n int) Counter { return Counter(n) }\n    fn read() int { return .value }\n}\nfn main() int {\n    let x Counter = Counter.new(42)\n    return x.read() + 1\n}",
        expected: b"value:43\n",
    },
    // 007_enum: single-payload enum construction + variant pattern
    // (oracle: "中" 1 scalar + code 41 -> 1*100+41 = 141)
    PyCase {
        family: "007_enum",
        name: "enum_variant_pattern",
        route: Route::Session,
        profile: SourceProfile::Aggregates,
        caps: &[],
        src: "type Item { name Str; code int }\nenum Token { Data(Item)\nEnd }\nfn main() int {\n    let t = Token.Data(Item(\"中\", 41))\n    is t {\n        Token.Data(v) -> { return v.name.len() * 100 + v.code }\n        Token.End -> { return 0 }\n    }\n}",
        expected: b"value:141\n",
    },
    // 015_str/04_strings: string concat + scalar length (oracle: "ab"+"cd"=4 -> 4*3+2=14)
    PyCase {
        family: "015_str/04_strings",
        name: "str_concat_len",
        route: Route::Session,
        profile: SourceProfile::TextRecord,
        caps: &[],
        src: "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a + \"cd\"\n    return b.len() * 3 + a.len()\n}",
        expected: b"value:14\n",
    },
    // 006_struct: record construction + field projection (oracle: 3*10+4=34)
    PyCase {
        family: "006_struct",
        name: "record_fields",
        route: Route::Session,
        profile: SourceProfile::TextRecord,
        caps: &[],
        src: "type Point { x int; y int }\nfn main() int {\n    let p Point = Point { x: 3, y: 4 }\n    return p.x * 10 + p.y\n}",
        expected: b"value:34\n",
    },
    // 002_array/10_collections: list build + length + indexed read (oracle: 1+2+3 -> 3 -> 30+3=33)
    PyCase {
        family: "002_array/10_collections",
        name: "list_build_read",
        route: Route::PackagesAggregates,
        profile: SourceProfile::Aggregates,
        caps: &[],
        // the List surface rides the PLAN-946 aggregates packages lane
        // (std.list carrier; oracle: len 3*100 + sum 6 + get(2) 3 = 309)
        src: "use std.list.sum
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    let s = sum(xs)
    return xs.len() * 100 + s + xs.get(2)
}",
        expected: b"value:309
",
    },
    // 09_option_result/012_is: option construction + is-pattern (oracle: some(7)+none(0) -> 70+2=72)
    PyCase {
        family: "09_option_result/012_is",
        name: "option_is",
        route: Route::Session,
        profile: SourceProfile::Aggregates,
        caps: &[],
        // the aggregates domain's pattern face is the is-EXPRESSION with
        // match arms + the `??` coalesce (control-effects corpus forms)
        src: "fn main() int {\n    let a = Some(7)\n    let g = is a {\n        Some(v) -> v\n        None -> 0\n    }\n    let b = None ?? 2\n    return g * 10 + b\n}",
        expected: b"value:72\n",
    },
];

/// Every ported obligation runs green through the real OS Python lane;
/// failures name the family + the lane rejection (never silent).
#[test]
fn p977_legacy_public_surface_runs_green_on_python() {
    let py = py_tool();
    let mut failures: Vec<String> = Vec::new();
    for case in CASES {
        let caps: Vec<String> = case.caps.iter().map(|s| s.to_string()).collect();
        let asm = match &case.route {
            Route::Session => {
                let store = out_root(&format!("p977_{}_store", case.name));
                let mut session = A2xSession::open(
                    &store,
                    "p977",
                    "probe",
                    64 * 1024 * 1024,
                    PythonTargetProvider,
                )
                .expect("open a2x session");
                session
                    .cold_assemble_profile(case.src, case.profile, &caps, "main", false)
                    .map_err(|e| {
                        failures.push(format!(
                            "{}[{}] ASSEMBLE REJECTED: {e:?}",
                            case.family, case.name
                        ));
                        e
                    })
                    .ok()
            }
            Route::PackagesAggregates => {
                let mut inputs = ace_core::source::packages::stdlib_aggregates_inputs();
                inputs.push(ace_core::source::packages::PackageModuleInput {
                    package: "app".into(),
                    version: "1.0.0".into(),
                    id: "main".into(),
                    source: case.src.to_string(),
                    provider: None,
                });
                let product =
                    ace_source_query::modules::packages::compile_package_product_aggregates(
                        &inputs,
                        "app::main",
                        &[ace_core::source::packages::PACKAGES_CAPABILITY.to_string()],
                        ace_core::source::cst::FrontBudget::WORK_OWNED_64MIB,
                    );
                match product {
                    Ok(product) => {
                        let store = out_root(&format!("p977_{}_store", case.name));
                        let target =
                            ace_a2x_store::modules::packages::PackageA2xTarget::cold_aggregates(
                                &store,
                                "p977",
                                "probe",
                                64 * 1024 * 1024,
                                PythonTargetProvider,
                                &product,
                            );
                        match target {
                            Ok(target) => Ok(ace_a2x_store::Assembly {
                                files: target
                                    .current
                                    .files
                                    .iter()
                                    .map(|(a, b)| (a.clone(), b.clone()))
                                    .collect(),
                                manifest: target.current.manifest.clone(),
                            }),
                            Err(e) => {
                                failures.push(format!(
                                    "{}[{}] PACKAGE TARGET: {e}",
                                    case.family, case.name
                                ));
                                Err(())
                            }
                        }
                    }
                    Err(ds) => {
                        failures.push(format!(
                            "{}[{}] AGGREGATES COMPILE REJECTED: {:?}",
                            case.family,
                            case.name,
                            ds.iter()
                                .map(|d| format!("{}/{}", d.module, d.diagnostic.code))
                                .collect::<Vec<_>>()
                        ));
                        Err(())
                    }
                }
                .ok()
            }
        };
        let asm = match asm {
            Some(asm) => asm,
            None => continue,
        };
        let schema = match (&case.route, case.profile) {
            (_, SourceProfile::TextRecord) => PY_RT_FILESET_SCHEMA,
            (Route::PackagesAggregates, _) | (_, SourceProfile::Aggregates) => {
                ace_a2x_python::PY_COLL_FILESET_SCHEMA
            }
            _ => PY_FILESET_SCHEMA,
        };
        let run_dir = out_root(&format!("p977_{}_run", case.name));
        for (rel, content) in &asm.files {
            std::fs::write(run_dir.join(rel), content).unwrap();
        }
        std::fs::write(run_dir.join("manifest.json"), &asm.manifest).unwrap();
        if let Err(e) =
            verify_python_fileset_manifest(&asm.manifest, &asm.files, PY_PRODUCER, schema)
        {
            failures.push(format!("{}[{}] MANIFEST: {e}", case.family, case.name));
            continue;
        }
        let run = match run_py_fileset(&py, &run_dir, &[]) {
            Ok(run) => run,
            Err(e) => {
                failures.push(format!("{}[{}] RUN: {e:?}", case.family, case.name));
                continue;
            }
        };
        if run.stdout.as_slice() != case.expected {
            failures.push(format!(
                "{}[{}] STDOUT: expected {:?} got {:?} (exit {:?}, stderr {})",
                case.family,
                case.name,
                String::from_utf8_lossy(case.expected),
                String::from_utf8_lossy(&run.stdout),
                run.exit_code,
                String::from_utf8_lossy(&run.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "p977 legacy public-surface gaps (lane rejections are NAMED gaps, never silent):\n{}",
        failures.join("\n")
    );
}

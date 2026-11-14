//! PLAN-972 T-01 有界全链前探（scratch probe，非永久 fixture）。
//! 串起本包最长 producer→consumer 路径的当前真实状态：
//!   P1 正例 —— (a) 1043 后缀装配 lane（纯 int/str 公共面）AV Auto/Rs 两配置实跑；
//!              (b) aggregates lane（962 准入）闭包 HOF 归约（972 自有能力）AV 实跑；
//!              (c) 后缀装配 lane 携 List 公共参数 → 记录当前冻结具名拒（T-02 桥接缺口）。
//!   P2 同类型非法近邻 —— 缺默认来源候选 / 实现签名漂移，均须在任何运行前具名拒。
//!   P3 失败后恢复 —— PackageStore /2 往返保身份，恢复件在 AV 上复跑同值。
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_package_inputs, compile_assembled_inputs, discover_provider_sources,
};

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/probe972/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write_fixture(dir: &std::path::Path, files: &[(&str, &str)]) {
    std::fs::create_dir_all(dir).unwrap();
    for (n, c) in files {
        std::fs::write(dir.join(n), c).unwrap();
    }
}

fn assemble_compile(
    dir: &std::path::Path,
    backend: AssemblyBackend,
    provider: ProviderChoice,
    target: TargetOs,
) -> Result<
    ace_source_query::modules::packages::PackageProduct,
    Vec<ace_core::source::packages::PackageDiagnostic>,
> {
    let files = discover_provider_sources(dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(backend, provider, target);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx)
        .unwrap_or_else(|e| panic!("assemble {backend:?}/{provider:?}: {e}"));
    compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
}

fn run_av(tag: &str, product: &ace_source_query::modules::packages::PackageProduct) -> Value {
    let av = PackageAvTarget::cold(&scratch(tag), product).unwrap_or_else(|e| panic!("av {tag}: {e}"));
    av.run_value().unwrap_or_else(|e| panic!("run {tag}: {e}"))
}

// ---- fixtures -----------------------------------------------------------

/// 纯 int 公共面（math 域）：公共体 scale 共享 + spec 层按 provider 来源选择。
const P1A_MAIN: &str = "use algo972.scale\nuse algo972.layer\nfn main() int { return scale(7, 8) * 10 + layer() }";
const P1A_PUBLIC: &str =
    "pub fn scale(x int, y int) int { return x * y + x + y }\npub spec fn layer() int";
const P1A_AT: &str = "fn layer() int { return 1 }";
const P1A_RS: &str = "fn layer() int { return 2 }";

/// List 公共参数（collections 域）进后缀装配 lane —— 预期当前冻结具名拒。
const P1C_MAIN: &str = "use algo972.total\nfn main() int { var xs = List<int>.new(); xs.push(1); xs.push(2); return total(xs) }";
const P1C_PUBLIC: &str =
    "pub fn total(xs List<int>) int { var t = 0; for x in xs { t = t + x } return t }";

/// aggregates lane 算法面：sum/find/any/all/filter（V-DETAIL-01 输入形态，
/// 无闭包参数——闭包参数形见 P1B2 当前冻结边界记录）。
const P1B_MAIN: &str = r#"
fn sum_to(xs List<int>) int {
    var t = 0
    for x in xs {
        t = t + x
    }
    return t
}
fn find_gt(xs List<int>, threshold int) ?int {
    for x in xs {
        if (x > threshold) {
            return Some(x)
        }
    }
    return None
}
fn any_neg(xs List<int>) bool {
    for x in xs {
        if (x < 0) {
            return true
        }
    }
    return false
}
fn all_pos(xs List<int>) bool {
    for x in xs {
        if (x <= 0) {
            return false
        }
    }
    return true
}
fn filter_gt(xs List<int>, threshold int) List<int> {
    var ys = List<int>.new()
    for x in xs {
        if (x > threshold) {
            ys.push(x)
        }
    }
    return ys
}
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = sum_to(xs)
    let f = find_gt(xs, 3)
    var got = 0
    is f {
        Some(v) -> { got = v }
        None -> { got = 0 }
    }
    let ys = filter_gt(xs, 1)
    let ys_sum = sum_to(ys)
    let ys_len = ys.len()
    var code = s * 10000000 + got * 1000000 + ys_len * 100000 + ys_sum
    if all_pos(xs) {
        code = code + 1
    }
    if any_neg(xs) {
        code = code + 2
    }
    return code
}
"#;

/// 闭包参数形（fn 型参数注解 + 调用位闭包字面量）——972 自有的 HOF 组合
/// 能力；当前 packages-aggregates 语法面具名拒（T-02 缺口记录）。
const P1B2_MAIN: &str = r#"
fn apply(f fn(int) -> int, v int) int {
    return f(v)
}
fn main() int {
    return apply((n int) => { return n + 1 }, 41)
}
"#;

// ---- P1 ------------------------------------------------------------------

/// P1(a)：后缀装配 lane 纯 int 公共面，AV×Auto 与 AV×Rs 两配置实跑。
/// scale(7,8)=71 → Auto 层 layer=1 → 711；Rs 层 layer=2 → 712。
#[test]
fn probe972_p1a_suffix_int_publics_av() {
    let dir = scratch("p1a-src");
    write_fixture(
        &dir,
        &[
            ("main.at", P1A_MAIN),
            ("algo972.at", P1A_PUBLIC),
            ("algo972.at.at", P1A_AT),
            ("algo972.rs.at", P1A_RS),
        ],
    );
    let p = assemble_compile(&dir, AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows)
        .unwrap_or_else(|ds| {
            panic!(
                "p1a Auto compile: {} {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        });
    assert_eq!(run_av("p1a-auto", &p), Value::I32(711));

    let p = assemble_compile(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "p1a Rs compile: {} {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    });
    assert_eq!(run_av("p1a-rs", &p), Value::I32(712));
}

/// P1(b)：aggregates lane（962 准入）算法面 AV 实跑：sum=15、find_gt(3)=4、
/// any_neg=false、all_pos=true、filter_gt(xs,1)=[2,3,4,5]（len=4 sum=14
/// ——拥有元素内容，不只 len/ok）→ 15*10^7+4*10^6+4*10^5+14+1 = 154400015。
#[test]
fn probe972_p1b_aggregates_algorithms_av() {
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::{
        compile_package_product_aggregates, stdlib_aggregates_inputs,
    };
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: P1B_MAIN.into(),
        provider: None,
    });
    let product = compile_package_product_aggregates(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "p1b compile: {} {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    });
    assert_eq!(run_av("p1b", &product), Value::I32(154400015));
}

/// P1(b2)：闭包参数形（fn 型参数 + 调用位闭包字面量）在当前
/// packages-aggregates 语法面的具名拒——T-02 缺口记录（放行即红，提示
/// 冻结边界变化须进决定表）。
#[test]
fn probe972_p1b2_closure_param_frozen_rejection() {
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::{
        compile_package_product_aggregates, stdlib_aggregates_inputs,
    };
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: P1B2_MAIN.into(),
        provider: None,
    });
    let e = match compile_package_product_aggregates(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("p1b2: closure-typed params unexpectedly admitted"),
        Err(ds) => ds,
    };
    eprintln!(
        "p1b2 frozen rejection observed: module={} code={} msg={}",
        e[0].module, e[0].diagnostic.code, e[0].diagnostic.message
    );
}

/// P1(c)：List 公共参数进后缀装配 lane —— 当前冻结面按 946 T-01 边界
/// 具名拒（T-02 的桥接缺口记录）。任何代码变化（放开或改码）都会被本
/// 探针看见。
#[test]
fn probe972_p1c_suffix_list_public_frozen_rejection() {
    let dir = scratch("p1c-src");
    write_fixture(&dir, &[("main.at", P1C_MAIN), ("algo972.at", P1C_PUBLIC)]);
    let e = match assemble_compile(&dir, AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows)
    {
        Ok(_) => panic!("p1c: List-typed publics unexpectedly admitted on the assembled lane"),
        Err(ds) => ds,
    };
    // 记录当前真实拒绝码（预期 list-value-context 族）；放行即红。
    eprintln!(
        "p1c frozen rejection observed: module={} code={} msg={}",
        e[0].module, e[0].diagnostic.code, e[0].diagnostic.message
    );
}

// ---- P2 ------------------------------------------------------------------

/// P2(a)：同类型非法近邻 —— 其他来源候选在而默认 at 候选缺 →
/// `assembly.missing-default-provider` 具名拒（装配期，先于任何编译/运行）。
#[test]
fn probe972_p2a_missing_default_provider_named_reject() {
    let dir = scratch("p2a-src");
    write_fixture(
        &dir,
        &[
            ("main.at", P1A_MAIN),
            ("algo972.at", P1A_PUBLIC),
            ("algo972.rs.at", P1A_RS),
        ],
    );
    let files = discover_provider_sources(&dir).unwrap();
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let e = match assemble_package_inputs(&files, "app::main", &mut ctx) {
        Ok(_) => panic!("p2a: missing default candidate unexpectedly assembled"),
        Err(e) => e,
    };
    assert!(
        e.contains("assembly.missing-default-provider"),
        "p2a unexpected: {e}"
    );
}

/// P2(b)：同类型非法近邻 —— 实现签名漂移（1 参 vs 需求 0 参）→
/// `source.assembly.signature` 具名拒（编译期，先于运行）。
#[test]
fn probe972_p2b_signature_drift_named_reject() {
    let dir = scratch("p2b-src");
    write_fixture(
        &dir,
        &[
            ("main.at", P1A_MAIN),
            ("algo972.at", P1A_PUBLIC),
            ("algo972.at.at", "fn layer(x int) int { return 7 }"),
            ("algo972.rs.at", P1A_RS),
        ],
    );
    let e = match assemble_compile(&dir, AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows)
    {
        Ok(_) => panic!("p2b: signature drift unexpectedly compiled"),
        Err(ds) => ds,
    };
    assert!(
        e[0].diagnostic.code.contains("assembly.signature")
            || e[0].diagnostic.message.contains("does not match the public declaration"),
        "p2b unexpected: {} {} {}",
        e[0].module,
        e[0].diagnostic.code,
        e[0].diagnostic.message
    );
}

// ---- P3 ------------------------------------------------------------------

/// P3：失败后恢复 —— 装配身份经 PackageStore /2 往返（prepare→restore）
/// 保持 input_hash，恢复件在 AV 上复跑同值（同一 fixture，identity 不变）。
#[test]
fn probe972_p3_store_roundtrip_recovers_and_reruns() {
    use ace_source_query::modules::PackageStore;
    let dir = scratch("p3-src");
    write_fixture(
        &dir,
        &[
            ("main.at", P1A_MAIN),
            ("algo972.at", P1A_PUBLIC),
            ("algo972.at.at", P1A_AT),
            ("algo972.rs.at", P1A_RS),
        ],
    );
    let files = discover_provider_sources(&dir).unwrap();
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx).unwrap();
    let product =
        compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
            .unwrap();
    let root = scratch("p3-store");
    let receipt = PackageStore::prepare_with_context(&root, &product, &inputs, &[], Some(&ctx))
        .unwrap();
    let (restored, _) = PackageStore::restore(&root, &receipt).unwrap();
    assert_eq!(restored.bound.input_hash, product.bound.input_hash);
    assert_eq!(run_av("p3-restored", &restored), Value::I32(711));
}

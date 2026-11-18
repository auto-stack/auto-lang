//! PLAN-982 / V-B-C（**新版 A2C**）：跨模块限定调用经 `ace-a2x-c` 生成 C，再由 MSVC
//! 编译、链接、运行。判据按用户裁定：产物必须来自 `CTargetProvider`（新版），
//! 而不是 legacy `CTrans`。
//!
//! 源：包 `app` 两个模块——`util` 提供 `pub fn add`，`main` 用**带类型标注**的
//! 限定调用 `util.add(2, 3)`；期望编译产物打印 `multi:5`。

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs, PackageProduct};
use std::path::PathBuf;

const UTIL: &str = "pub fn add(a int, b int) int {\n  return a + b\n}\n";
const MAIN: &str = "use util\nfn main() int {\n  var r int = util.add(2, 3)\n  return r\n}\n";

fn modules() -> Vec<PackageModuleInput> {
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "util".into(),
        source: UTIL.into(),
        provider: None,
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: MAIN.into(),
        provider: None,
    });
    inputs
}

fn compile() -> Result<PackageProduct, String> {
    compile_package_product(
        &modules(),
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|ds| {
        format!(
            "{}: {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    })
}

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/plan982/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn plan982_new_a2c_lowers_a_qualified_cross_module_call() {
    let product = match compile() {
        Ok(p) => p,
        Err(e) => panic!("the two-module package compiles: {e}"),
    };
    let store = scratch("store");
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg982",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .expect("the NEW A2C emits a C fileset");
    let files: Vec<(String, String)> = c.current.files.clone();
    assert!(
        files.iter().any(|(n, _)| n.ends_with(".c")),
        "the fileset has C sources: {:?}",
        files.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>()
    );

    let dir = scratch("build");
    for (path, content) in &files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &c.current.manifest).unwrap();
    let exe = dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run");
    let out = String::from_utf8_lossy(&run.stdout).to_string();
    eprintln!("plan982 new-a2c: status={:?} stdout={out:?}", run.status);
    assert!(
        out.contains("value:5") || out.contains("multi:5"),
        "the compiled C program prints the cross-module result: {out:?}"
    );
}
#[test]
fn plan982_new_a2c_infers_the_cross_module_return_type() {
    // 去掉类型标注：跨模块返回类型推断也必须成立（用户指令第 2 项）。
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "util".into(),
        source: UTIL.into(),
        provider: None,
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "use util\nfn main() int {\n  var r = util.add(2, 3)\n  return r\n}\n".into(),
        provider: None,
    });
    let product = compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|ds| {
        format!(
            "{}: {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    })
    .unwrap_or_else(|e| panic!("the inferred two-module package compiles: {e}"));
    let store = scratch("store-inf");
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg982i",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .expect("the NEW A2C emits a C fileset with an inferred return type");
    let dir = scratch("build-inf");
    for (path, content) in &c.current.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &c.current.manifest).unwrap();
    let exe = dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = std::process::Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run");
    let out = String::from_utf8_lossy(&run.stdout).to_string();
    eprintln!(
        "plan982 new-a2c (inferred): status={:?} stdout={out:?}",
        run.status
    );
    assert!(
        out.contains("value:5"),
        "the inferred cross-module result is 5: {out:?}"
    );
}

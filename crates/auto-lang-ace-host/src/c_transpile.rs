//! PLAN-982（982/C，宿主 A2C 渲染器）：**包级 A2C 的宿主驱动面**。
//!
//! 产品侧（`auto build` 的 C 目标）目前仍走旧 `transpile_c`（legacy `CTrans`）。
//! 本模块是两者之间的**唯一边界**（与 `rust_transpile` 同层）：用户模块 + 冻结
//! 标准库 → `compile_package_product`（受检包编译）→ `PackageA2xTarget::cold`
//! + `CTargetProvider::v1()`（`ace-a2x-c`，HIR→C11）→ 产出 fileset
//! （`main.c` + 运行时头 + `generated.c` + manifest）。
//!
//! 合同（与 A2R 驱动一致）：
//! - 跨模块限定调用（`use util` + `util.add(...)`）**只在包级成立**——逐文件
//!   渲染无法解析跨模块引用，因此本驱动只服务"整工程一次编译"；
//! - 每个失败都是**具名**的（`ace.c/compile`、`ace.c/emit`），绝不返回空
//!   fileset 或静默降级到旧转译；
//! - 同输入**逐字节确定**（store 目录每次冷启动，无 fragment 复用）。

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

/// 一个产出文件（相对文件名 + 文本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceCFile {
    pub name: String,
    pub text: String,
}

/// 一次包级 A2C 渲染的完整结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceCFileSet {
    pub files: Vec<AceCFile>,
    /// 冻结 schema 的 manifest（`auto.a2x.c.fileset/1`）。
    pub manifest: String,
    /// 入口模块名（源名 `main` ⇒ def `d_main`）。
    pub entry: String,
}

/// 包级 A2C 渲染：`modules` = `(模块名, 源码)` 列表（工程全部 `.at`），
/// `entry` = 入口模块名（如 `"main"`）。
///
/// 标准库（`std.math`/`std.text`）由本驱动按冻结输入注入，调用方只给用户模块。
pub fn transpile_package_c(
    modules: &[(String, String)],
    entry: &str,
) -> Result<AceCFileSet, (&'static str, String)> {
    let mut inputs = stdlib_inputs();
    for (id, source) in modules {
        inputs.push(PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: id.clone(),
            source: source.clone(),
            provider: None,
        });
    }
    let entry_id = format!("app::{entry}");
    let product = compile_package_product(
        &inputs,
        &entry_id,
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|diags| {
        let first = diags.first();
        let message = match first {
            Some(d) => format!(
                "package '{}': {} {}",
                d.module, d.diagnostic.code, d.diagnostic.message
            ),
            None => "the package compile failed without a diagnostic".to_string(),
        };
        ("ace.c/compile", message)
    })?;

    // store 冷启动：同输入 ⇒ 同 fileset（无 fragment 复用的隐式差异）。
    let store = std::env::temp_dir().join(format!("ace-a2c-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    std::fs::create_dir_all(&store).map_err(|e| {
        (
            "ace.c/emit",
            format!("scratch store dir {}: {e}", store.display()),
        )
    })?;
    let mut target = PackageA2xTarget::cold(
        &store,
        "auto-a2c",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .map_err(|e| ("ace.c/emit", e))?;
    let files: Vec<AceCFile> = target
        .current
        .files
        .iter()
        .map(|(name, text)| AceCFile {
            name: name.clone(),
            text: text.clone(),
        })
        .collect();
    let fileset = AceCFileSet {
        files,
        manifest: target.current.manifest.clone(),
        entry: entry.to_string(),
    };
    let _ = std::fs::remove_dir_all(&store);
    Ok(fileset)
}

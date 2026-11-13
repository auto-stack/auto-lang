//! PLAN-960 packages module lane: the existing resolver, typecheck,
//! emitter and HIR verifier over the package graph (the same text-record
//! pipeline the single-module lane uses; no parallel semantic analysis).
//! Names/visibility/initialization order are bound by
//! `ace_core::source::packages`; this layer never widens them.
//!
//! The PLAN-934 per-function fact pages only cover the finite scalar
//! portable type set (`PortableTy`), so a package graph carrying text or
//! record types cannot be served by those pages — the lane runs the whole
//! resolver and relies on the binder's reuse ledger (`binding_walks`) for
//! incremental evidence. Fact-page coverage for text/record types is a
//! named query-layer follow-up, not a silent fallback.
use ace_core::atom_text::Diagnostic;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    bind_packages, BoundPackages, PackageDiagnostic, PackageModuleInput,
};
use ace_core::source::{
    emit_resolved_packages_parts, resolve_with, typecheck_with, CompiledSource, SourceProfile,
};

pub struct PackageProduct {
    pub compiled: CompiledSource,
    pub bound: BoundPackages,
}

/// PLAN-945：冻结 ACE 公共标准库包输入（std.math/std.text），由持有其
/// 身份的前门（`ace_core::source::packages`）再导出。
pub use ace_core::source::packages::stdlib_inputs;

fn locate_all(bound: &BoundPackages, ds: Vec<Diagnostic>) -> Vec<PackageDiagnostic> {
    ds.into_iter().map(|d| bound.locate(d)).collect()
}

pub fn compile_package_product(
    inputs: &[PackageModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let bound = bind_packages(inputs, entry, caps, budget).map_err(|e| vec![e])?;
    compile_bound_packages(bound, caps)
}

/// Cold/retained shared compile over an already-bound package graph. The
/// caller retains the graph's identity/visibility admission.
pub fn compile_bound_packages(
    bound: BoundPackages,
    caps: &[String],
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let resolved = resolve_with(&bound.program, caps, SourceProfile::TextRecord)
        .map_err(|ds| locate_all(&bound, ds))?;
    typecheck_with(&resolved, SourceProfile::TextRecord).map_err(|ds| locate_all(&bound, ds))?;
    let parts = emit_resolved_packages_parts(&resolved, &bound, "packages")
        .map_err(|ds| locate_all(&bound, ds))?;
    let checked =
        ace_core::verify::verify_text_record(parts.bundle).map_err(|ds| locate_all(&bound, ds))?;
    Ok(PackageProduct {
        compiled: CompiledSource {
            profile: parts.profile,
            input_hash: parts.input_hash,
            atom_text: parts.atom_text,
            checked,
        },
        bound,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::packages::PACKAGES_CAPABILITY;

    fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: package.into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
            provider: None,
        }
    }

    #[test]
    fn package_graph_reaches_checked_hir_in_the_text_record_domain() {
        let inputs = [
            input(
                "lib",
                "std",
                "pub const BASE int = 40\npub fn greet() str { return \"m2\" }\npub type P { x int\n fn sum() int { return .x } }",
            ),
            input(
                "app",
                "main",
                "use lib.std.greet\nuse lib.std.P\nuse lib.std.BASE\nvar count int = 0\npub fn bump() int { count = count + 1\n return count }\nfn main() int { let p = P { x: BASE }\n let n = bump()\n return p.sum() + n }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        assert!(product.bound.module_init_fns.contains_key("app::main"));
        let checked = &product.compiled.checked;
        assert!(checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")));
        let syms: Vec<&String> = checked
            .bundle()
            .modules
            .iter()
            .flat_map(|m| {
                m.defs.iter().filter_map(|d| match &d.kind {
                    ace_core::hir::DefKind::Intrinsic { symbol, .. } => Some(symbol),
                    _ => None,
                })
            })
            .collect();
        assert!(
            syms.iter()
                .any(|s| s.as_str() == "module.global.get:g3_libm3_std_BASE"),
            "{syms:?}"
        );
        assert!(
            syms.iter()
                .any(|s| s.as_str() == "module.global.set:g3_appm4_main_count"),
            "{syms:?}"
        );
    }

    #[test]
    fn package_type_and_visibility_failures_keep_source_location() {
        let inputs = [
            input("lib", "std", "pub fn give(x bool) int { return 1 }"),
            input(
                "app",
                "main",
                "use lib.std.give\nfn main() int { return give(1) }",
            ),
        ];
        let ds = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .err()
        .unwrap();
        assert_eq!(ds[0].module, "app::main", "{ds:?}");
        let src = &inputs[1].source;
        assert!(
            ds[0].diagnostic.span.start >= src.find("give(1)").unwrap()
                && ds[0].diagnostic.span.end <= src.len(),
            "{ds:?}"
        );
    }

    #[test]
    fn package_generic_record_templates_reach_checked_hir_963() {
        // PLAN-963（V-G06）：记录模板在 lib 声明、实例在 app 与 lib 两侧使用，
        // 扁平程序级单态化后四端只见具体记录（模板不再抵达 resolve/HIR）。
        let inputs = [
            input(
                "lib",
                "geo",
                "pub type Point<T> { x T\n fn get() T { return .x } }\npub fn mk(v int) Point<int> { return Point { x: v } }",
            ),
            input(
                "app",
                "main",
                "use lib.geo.Point\nuse lib.geo.mk\nfn main() int { let q Point<int> = Point { x: 2 }\n let r Point<int> = mk(40)\n return q.get() + r.get() }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let text = &product.compiled.atom_text;
        assert!(text.contains("t3_libm3_geo_Point_int"), "instance missing");
        assert!(!text.contains("Point<T>"), "template residue in atom text");
        assert!(product
            .compiled
            .checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")));
    }
}

// ---------------------------------------------------------------------------
// PLAN-1043（T-04）：装配 lane——候选发现（宿主扫描）、按上下文组装输入、
// 显式装配编译入口。选择模型/绑定/求值都在 ace-core packages owner；本层
// 只做盘上候选闭包与入口编排，不建第二 loader/binder。
// ---------------------------------------------------------------------------

use ace_core::source::packages::{
    classify_provider_file_name, select_provider_source, PackageAssemblyContext,
    PackageDiagnostic as AsmPackageDiagnostic, PackageImplLayer, PackageModuleInput as AsmInput,
    ProviderFileKind, ProviderKind, ProviderSourceError, IN_CAPABILITY, PACKAGES_CAPABILITY,
};
use std::collections::{BTreeMap, BTreeSet};

/// 装配 lane 的能力集（packages 面 + `#in` 编译期命名空间）。
pub fn assembled_caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string(), IN_CAPABILITY.to_string()]
}

/// 扫描一个平铺候选目录 → (文件名, 内容) 闭包（按名排序；非本合同文件
/// 与旧 `.vm.at` 目标层被分类器忽略）。无递归、无搜索路径猜测。
pub fn discover_provider_sources(dir: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("scan {}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| classify_provider_file_name(n).is_some())
        .collect();
    names.sort();
    let mut out = Vec::with_capacity(names.len());
    for n in names {
        let content =
            std::fs::read_to_string(dir.join(&n)).map_err(|e| format!("read {}: {e}", n))?;
        out.push((n, content));
    }
    Ok(out)
}

/// 由文件闭包 + 显式上下文组装包输入：公共 `.at` 成模块，实现候选按
/// backend/provider/target 选择挂到对应模块（选择错误具名上抛）。
/// `available` 读集同步进 ctx（增删候选即失效身份）。
pub fn assemble_package_inputs(
    files: &[(String, String)],
    entry: &str,
    ctx: &mut PackageAssemblyContext,
) -> Result<Vec<AsmInput>, String> {
    let mut publics: BTreeMap<String, String> = BTreeMap::new();
    let mut candidates: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for (name, content) in files {
        match classify_provider_file_name(name) {
            Some((ProviderFileKind::Public, base)) => {
                if publics.insert(base.clone(), content.clone()).is_some() {
                    return Err(format!("duplicate public module file for `{base}`"));
                }
            }
            Some((ProviderFileKind::Provider(kind), base)) => {
                let slot = candidates.entry(base.clone()).or_default();
                let label = kind.label().to_string();
                if slot.insert(label, content.clone()).is_some() {
                    return Err(format!(
                        "duplicate `{}` provider candidate for `{base}`",
                        kind.label()
                    ));
                }
            }
            None => {}
        }
    }
    // 候选存在性读集（进 ctx 身份）。
    ctx.available = candidates
        .iter()
        .map(|(base, kinds)| {
            (
                base.clone(),
                kinds
                    .keys()
                    .filter_map(|l| match l.as_str() {
                        "at" => Some(ProviderKind::At),
                        "c" => Some(ProviderKind::C),
                        "rs" => Some(ProviderKind::Rs),
                        "py" => Some(ProviderKind::Py),
                        _ => None,
                    })
                    .collect::<BTreeSet<ProviderKind>>(),
            )
        })
        .collect();
    let entry_module = entry.rsplit("::").next().unwrap_or(entry);
    if !publics.contains_key(entry_module) {
        return Err(format!(
            "entry module `{entry_module}` has no public source"
        ));
    }
    let mut inputs = Vec::new();
    for (base, source) in &publics {
        let provider = match candidates.get(base) {
            None => None,
            Some(kinds) => {
                let set: BTreeSet<ProviderKind> = kinds
                    .keys()
                    .filter_map(|l| match l.as_str() {
                        "at" => Some(ProviderKind::At),
                        "c" => Some(ProviderKind::C),
                        "rs" => Some(ProviderKind::Rs),
                        "py" => Some(ProviderKind::Py),
                        _ => None,
                    })
                    .collect();
                let sel = select_provider_source(base, &set, ctx.backend, ctx.provider, ctx.target)
                    .map_err(|e: ProviderSourceError| format!("{}: {}", e.code(), e.message()))?;
                sel.provider.map(|kind| PackageImplLayer {
                    kind,
                    source: kinds.get(kind.label()).cloned().unwrap_or_default(),
                })
            }
        };
        inputs.push(AsmInput {
            package: entry.split("::").next().unwrap_or(entry).to_string(),
            version: "1.0.0".to_string(),
            id: base.clone(),
            source: source.clone(),
            provider,
        });
    }
    Ok(inputs)
}

/// PLAN-1043：显式装配编译入口（文件闭包 → 组装 → 绑定 → 既有管线）。
/// 调用方保留组装后的 inputs 与 ctx（进 store 身份/恢复）。
pub fn compile_assembled_package(
    files: &[(String, String)],
    entry: &str,
    budget: FrontBudget,
    ctx: &mut PackageAssemblyContext,
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let inputs = assemble_package_inputs(files, entry, ctx).map_err(|e| {
        vec![PackageDiagnostic {
            module: entry.to_string(),
            diagnostic: Diagnostic::new(
                ace_core::atom_text::Stage::Bind,
                "assembly.discovery",
                e,
                ace_core::atom_text::Span { start: 0, end: 0 },
            ),
        }]
    })?;
    compile_assembled_inputs(&inputs, entry, budget, ctx)
}

/// 已组装输入的装配编译（恢复/再编译共用）。
pub fn compile_assembled_inputs(
    inputs: &[AsmInput],
    entry: &str,
    budget: FrontBudget,
    ctx: &PackageAssemblyContext,
) -> Result<PackageProduct, Vec<PackageDiagnostic>> {
    let caps = assembled_caps();
    let bound =
        ace_core::source::packages::bind_packages_assembled(inputs, entry, &caps, budget, ctx)
            .map_err(|e| vec![convert_asm_diag(e)])?;
    compile_bound_packages(bound, &caps)
}

fn convert_asm_diag(d: AsmPackageDiagnostic) -> PackageDiagnostic {
    d
}

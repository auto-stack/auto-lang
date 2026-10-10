//! PLAN-755 T-01/T-03：HTTP 公共面契约覆盖走查（SD-01 覆盖完整性守卫）。
//!
//! `stdlib/auto/http.at` 公共层每个 pub fn/method 按
//! `validate::public_native_name` 求 native 名并解析 id；凡「解析成功且 shim
//! 已绑定」的符号——即真实程序引用时走 `verify_core_reference` 的被选中
//! callee——必须在 `NativeInterface::production()` 最终接口上有独立适配契约
//! 且 `signature_matches` 通过。resolved-but-unbound 符号登记为知情项（被引
//! 用时走 PROVIDER_CLAIM_NO_CALLEE，属 shim 缺口而非契约缺口，由 plan738
//! t04 冻结诚实状态，不计入本走查失败面）。

use crate::stdlib_assembly::model::{LayerKind, ParseStatus, SymbolKind};

/// registry 就绪 + production() 接口（生产 boot 同款次序，见 plan738 t04
/// production_surfaces 与 vm/engine.rs AutoVM::new）。
fn production_interface() -> crate::vm::native::NativeInterface {
    let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
    let repo_root = stdlib_root
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(&repo_root).unwrap();
    crate::vm::native_registry::register_builtin_natives();
    std::env::set_current_dir(orig).unwrap();
    crate::vm::native::NativeInterface::production()
}

#[test]
fn plan755_http_public_face_contract_coverage() {
    let interface = production_interface();
    let registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();

    let declaration = "stdlib/auto/http.at".to_string();
    let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
    let text = std::fs::read_to_string(stdlib_root.join("http.at")).unwrap();
    let layer = crate::stdlib_assembly::loader::parse_layer(&text, LayerKind::Public, declaration);
    assert!(
        !matches!(layer.parse, ParseStatus::Failed { .. }),
        "http.at 公共层解析失败"
    );

    let mut missing: Vec<String> = Vec::new();
    let mut drift: Vec<String> = Vec::new();
    let mut unbound: Vec<String> = Vec::new();
    let mut covered = 0usize;
    for sym in layer
        .symbols
        .iter()
        .filter(|s| s.is_pub && matches!(s.kind, SymbolKind::Fn | SymbolKind::Method))
    {
        let native_name = crate::stdlib_assembly::validate::public_native_name("http", sym);
        let Some(id) = registry
            .peek_qualified(&native_name)
            .or_else(|| interface.resolve(&native_name))
        else {
            continue; // 无 native 面符号（VM 侧实现/语法层），不属本走查
        };
        if interface.get(id).is_none() {
            unbound.push(format!("{} -> {} #{}", sym.name, native_name, id));
            continue;
        }
        match interface.contract(id) {
            None => missing.push(format!(
                "{} -> {} #{} (arity {})",
                sym.name, native_name, id, sym.arity
            )),
            Some(contract) => {
                match crate::stdlib_assembly::validate::signature_matches(
                    "http",
                    sym,
                    contract,
                    LayerKind::Public,
                ) {
                    Ok(()) => covered += 1,
                    Err(reason) => {
                        drift.push(format!("{} -> {} #{}: {reason}", sym.name, native_name, id));
                    }
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "HTTP 公共面缺独立适配契约（PLAN-755 SD-01；修复=native.rs production() 补 declare）:\n{}",
        missing.join("\n")
    );
    assert!(
        drift.is_empty(),
        "HTTP 公共面契约签名漂移:\n{}",
        drift.join("\n")
    );
    // 走查空转守卫：http.at 公共面绑定量必须达到量级下限。
    assert!(covered >= 20, "covered={covered} 异常偏小，走查疑似空转");
    if !unbound.is_empty() {
        eprintln!(
            "[plan755] http 公共面 resolved-but-unbound（知情登记，shim 缺口另案）:\n{}",
            unbound.join("\n")
        );
    }
}

/// AC-02：补齐后的 HTTP 契约经 `AutoVM::new` 同款 CFFI merge 后仍存活
/// （merge 的静态 id 重注册会撤销契约——19973b089 语义；生产 boot 必
/// 不可把公共面契约抹掉）。
#[test]
fn plan755_http_contracts_survive_cffi_merge() {
    let mut interface = production_interface();
    let registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
    let sampled: Vec<(String, u16)> = [
        "auto.http.get",
        "auto.http.upload_receive",
        "auto.http.response_redirect",
    ]
    .iter()
    .filter_map(|name| {
        registry
            .peek_qualified(name)
            .or_else(|| interface.resolve(name))
            .map(|id| (name.to_string(), id))
    })
    .collect();
    assert!(sampled.len() == 3, "采样 native 名解析不全: {sampled:?}");
    for (name, id) in &sampled {
        assert!(
            interface.contract(*id).is_some(),
            "merge 前即缺契约: {name} #{id}"
        );
    }

    // AutoVM::new 同款：merge CFFI_GLOBAL 快照后契约仍在。
    {
        let cffi = crate::vm::codegen::CFFI_GLOBAL.lock().unwrap();
        interface.merge(cffi.native_interface());
    }
    for (name, id) in &sampled {
        assert!(
            interface.contract(*id).is_some(),
            "CFFI merge 撤销了公共面契约: {name} #{id}"
        );
    }
}

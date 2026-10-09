//! Rust producer evidence is read from the selected producer's Rust AST.
//! Routing alone and provider catalog claims cannot establish a signature.
use super::model::{LayerKind, ParseStatus, VerificationLevel};
use super::reference::ReferenceProof;

#[derive(Debug, Clone, Copy, Default)]
pub enum RustRuntime {
    #[default]
    Standalone,
    Embedded,
}

pub fn verify_rust_reference(
    module: &str,
    symbol: &str,
    emitted: &str,
    runtime: RustRuntime,
) -> crate::AutoResult<Option<ReferenceProof>> {
    if !super::validate::CORE_MODULES.contains(&module) {
        return Ok(None);
    }
    let root = super::loader::repo_stdlib_root()?;
    let declaration = format!("stdlib/auto/{module}.at");
    let text = std::fs::read_to_string(root.join(format!("{module}.at")))?;
    let layer = super::loader::parse_layer(&text, LayerKind::Public, declaration.clone());
    if let ParseStatus::Failed { error } = &layer.parse {
        return Err(format!("STDINV.PARSE_FAIL: {declaration}: {error}").into());
    }
    let Some(public) = layer
        .symbols
        .iter()
        .find(|entry| entry.is_pub && entry.name == symbol)
    else {
        return Ok(None); // Private legacy helpers do not inherit a public claim.
    };
    let mut expression: syn::Expr = syn::parse_str(emitted)
        .map_err(|e| format!("STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: emitted expression cannot be inspected: {e}"))?;
    // §5.7：async 是签名证据的一部分。async 上下文的 Call 臂会自动追加
    // `.await`；解包该等待点，随后与 producer asyncness 双向核对（错配=漂移），
    // 不做一刀切拒绝。
    let mut awaited = false;
    if let syn::Expr::Await(await_expr) = expression {
        expression = *await_expr.base;
        awaited = true;
    }
    let syn::Expr::Call(call) = expression else {
        return Err(format!("STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: adapter expression requires independent proof").into());
    };
    let syn::Expr::Path(path) = *call.func else {
        return Err(format!("STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE: {module}.{symbol}").into());
    };
    let names: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|p| p.ident.to_string())
        .collect();
    if names.len() != 3 || names[0] != "a2r_std" || names[1] != module {
        return Err(format!(
            "STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE: {module}.{symbol}: selected {names:?}"
        )
        .into());
    }
    let callee = &names[2];
    let (producer_id, producer_text, nested) = match (runtime, module) {
        (RustRuntime::Standalone, "json") => (
            "crates/a2r-std/src/json.rs",
            include_str!("../../../a2r-std/src/json.rs"),
            false,
        ),
        (RustRuntime::Standalone, "http") => (
            "crates/a2r-std/src/http.rs",
            include_str!("../../../a2r-std/src/http.rs"),
            false,
        ),
        (RustRuntime::Standalone, "sse") => (
            "crates/a2r-std/src/sse.rs",
            include_str!("../../../a2r-std/src/sse.rs"),
            false,
        ),
        (RustRuntime::Embedded, _) => (
            "crates/auto-lang/src/a2r_std.rs",
            include_str!("../a2r_std.rs"),
            true,
        ),
        _ => {
            return Err(format!(
                "STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: no inspected Rust producer"
            )
            .into())
        }
    };
    let file = syn::parse_file(producer_text)
        .map_err(|e| format!("STDASSEMBLY.PARSE_FAIL: {producer_id}: {e}"))?;
    let items = if nested {
        file.items
            .iter()
            .find_map(|item| match item {
                syn::Item::Mod(item) if item.ident == module => {
                    item.content.as_ref().map(|(_, items)| items.as_slice())
                }
                _ => None,
            })
            .unwrap_or(&[])
    } else {
        file.items.as_slice()
    };
    let producer = resolve_producer(producer_id, items, callee.as_str(), REEXPORT_DEPTH)
        .ok_or_else(|| {
            format!(
                "STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE: {module}.{symbol}: {producer_id}::{callee}"
            )
        })?;
    let producer_async = producer.1.sig.asyncness.is_some();
    if producer_async != awaited {
        return Err(format!(
            "STDASSEMBLY.SIGNATURE_DRIFT: {module}.{symbol}: async mode mismatch (producer async={producer_async}, emission awaited={awaited}): {}",
            producer.0
        )
        .into());
    }
    if !producer.1.sig.generics.params.is_empty() {
        return Err(format!(
            "STDASSEMBLY.SIGNATURE_UNVERIFIED: {}::{callee}: generic adapter unavailable",
            producer.0
        )
        .into());
    }
    let mut parameters = Vec::new();
    let mut adapters: Vec<&'static str> = Vec::new();
    for parameter in &producer.1.sig.inputs {
        let syn::FnArg::Typed(parameter) = parameter else {
            return Err(format!(
                "STDASSEMBLY.SIGNATURE_UNVERIFIED: {}::{callee}: receiver",
                producer.0
            )
            .into());
        };
        if is_asref_str_view(&parameter.ty) {
            // §5.3：兼容 lowering 的参数适配变体单独记录，不冒称同形。
            adapters.push("impl AsRef<str> view adapter");
        }
        parameters.push(logical_type(&parameter.ty).ok_or_else(|| {
            format!(
                "STDASSEMBLY.SIGNATURE_UNVERIFIED: {}::{callee}: parameter representation",
                producer.0
            )
        })?);
    }
    let mut returns = match &producer.1.sig.output {
        syn::ReturnType::Default => "void".into(),
        syn::ReturnType::Type(_, ty) => logical_type(ty).ok_or_else(|| {
            format!(
                "STDASSEMBLY.SIGNATURE_UNVERIFIED: {}::{callee}: return representation",
                producer.0
            )
        })?,
    };
    let mut error_shape = "Rust return value".to_string();
    // The selected parse implementation uses serde_json::Value::Null as its
    // nullable payload. This representation is identified explicitly.
    if module == "json" && symbol == "parse" && callee == "parse" && returns == "JsonValue" {
        returns = "JsonValue?".into();
        error_shape = "Value::Null sentinel on parse failure".into();
    }
    if producer_async {
        error_shape.push_str("; async producer consumed at call-site await");
    }
    for adapter in &adapters {
        error_shape.push_str("; ");
        error_shape.push_str(adapter);
    }
    let contract = crate::vm::native::NativeContract {
        parameter_modes: parameters.iter().map(|_| "View".into()).collect(),
        parameters,
        returns,
        producer: format!("{}::{callee}", producer.0),
        receiver: None,
        is_static: false,
        generics: Vec::new(),
        error_shape,
    };
    if call.args.len() != contract.parameters.len() {
        return Err(format!(
            "STDASSEMBLY.SIGNATURE_DRIFT: {module}.{symbol}: emitted arity / Rust producer arity"
        )
        .into());
    }
    super::validate::signature_matches(module, public, &contract, LayerKind::Public).map_err(
        |reason| {
            format!(
                "STDASSEMBLY.SIGNATURE_DRIFT: {declaration}: {module}.{symbol} / {}: {reason}",
                contract.producer
            )
        },
    )?;
    Ok(Some(ReferenceProof {
        module: module.into(),
        symbol: symbol.into(),
        declaration,
        declaration_hash: layer.content_hash,
        producer: contract.producer.clone(),
        native_id: None,
        callee: names.join("::"),
        verification: VerificationLevel::SignatureChecked,
        public_signature: public.signature.clone().unwrap(),
        selected_adapter: contract,
    }))
}

/// 再导出跟随的最大深度：内嵌镜像 → 独立 crate 模块层 → 定义文件。
const REEXPORT_DEPTH: usize = 3;

fn direct_public_fn<'a>(items: &'a [syn::Item], callee: &str) -> Option<&'a syn::ItemFn> {
    items.iter().find_map(|item| match item {
        syn::Item::Fn(function)
            if function.sig.ident == callee
                && matches!(function.vis, syn::Visibility::Public(_)) =>
        {
            Some(function)
        }
        _ => None,
    })
}

/// 独立 a2r-std crate 的闭世界模块文件映射（路径段 → 源身份与文本）。
fn standalone_source(path: &[String]) -> Option<(&'static str, &'static str)> {
    let segments: Vec<&str> = path.iter().map(String::as_str).collect();
    match segments.as_slice() {
        ["json"] => Some((
            "crates/a2r-std/src/json.rs",
            include_str!("../../../a2r-std/src/json.rs"),
        )),
        ["sse"] => Some((
            "crates/a2r-std/src/sse.rs",
            include_str!("../../../a2r-std/src/sse.rs"),
        )),
        ["http"] => Some((
            "crates/a2r-std/src/http.rs",
            include_str!("../../../a2r-std/src/http.rs"),
        )),
        ["http", "server_file"] => Some((
            "crates/a2r-std/src/http/server_file.rs",
            include_str!("../../../a2r-std/src/http/server_file.rs"),
        )),
        ["http", "server_upload"] => Some((
            "crates/a2r-std/src/http/server_upload.rs",
            include_str!("../../../a2r-std/src/http/server_upload.rs"),
        )),
        ["http", "client"] => Some((
            "crates/a2r-std/src/http/client.rs",
            include_str!("../../../a2r-std/src/http/client.rs"),
        )),
        ["http", "transfer"] => Some((
            "crates/a2r-std/src/http/transfer.rs",
            include_str!("../../../a2r-std/src/http/transfer.rs"),
        )),
        _ => None,
    }
}

fn collect_use_names(tree: &syn::UseTree, prefix: &[String], out: &mut Vec<(Vec<String>, String)>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut nested = prefix.to_vec();
            nested.push(path.ident.to_string());
            collect_use_names(&path.tree, &nested, out);
        }
        syn::UseTree::Name(name) => out.push((prefix.to_vec(), name.ident.to_string())),
        syn::UseTree::Rename(rename) => out.push((prefix.to_vec(), rename.ident.to_string())),
        syn::UseTree::Group(group) => {
            for nested in &group.items {
                collect_use_names(nested, prefix, out);
            }
        }
        // Glob 再导出无法为单一 callee 归因来源，不跟随（如实 NO_CALLEE）。
        syn::UseTree::Glob(_) => {}
    }
}

/// 定位 callee 的真实定义：先找当前文件直接 `pub fn`；否则跟随公开
/// `pub use` 再导出链（内嵌镜像与 http.rs 是文档化的同形转发壳——
/// §5.2 实际导出；真实契约在定义文件）。返回 (定义文件身份, 函数)。
fn resolve_producer(
    producer_id: &str,
    items: &[syn::Item],
    callee: &str,
    depth: usize,
) -> Option<(String, syn::ItemFn)> {
    if let Some(function) = direct_public_fn(items, callee) {
        return Some((producer_id.to_string(), function.clone()));
    }
    if depth == 0 {
        return None;
    }
    for item in items {
        let syn::Item::Use(use_item) = item else {
            continue;
        };
        if !matches!(use_item.vis, syn::Visibility::Public(_)) {
            continue;
        }
        // syn2 的 `ItemUse` 没有独立 path 字段——路径前缀在 UseTree 自身。
        let mut exported = Vec::new();
        collect_use_names(&use_item.tree, &[], &mut exported);
        for (module_path, name) in exported {
            if name != callee {
                continue;
            }
            // `::a2r_std::…` 与 crate 内 `crate::…` 根都指向独立 crate 模块树。
            let mut segments = module_path;
            match segments.first().map(String::as_str) {
                Some("a2r_std") | Some("crate") => {
                    segments.remove(0);
                }
                _ => continue,
            }
            let Some((file_id, text)) = standalone_source(&segments) else {
                continue;
            };
            let file = syn::parse_file(text).ok()?;
            if let Some(hit) = resolve_producer(file_id, &file.items, callee, depth - 1) {
                return Some(hit);
            }
        }
    }
    None
}

/// `impl AsRef<str>` 参数适配变体：公共面声明 str，producer 接受任意
/// AsRef<str> 视图（发射侧传 String/&str 字面量——兼容 lowering）。
fn is_asref_str_view(ty: &syn::Type) -> bool {
    let syn::Type::ImplTrait(impl_trait) = ty else {
        return false;
    };
    if impl_trait.bounds.len() != 1 {
        return false;
    }
    let syn::TypeParamBound::Trait(bound) = &impl_trait.bounds[0] else {
        return false;
    };
    let Some(segment) = bound.path.segments.last() else {
        return false;
    };
    if segment.ident != "AsRef" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    matches!(
        arguments.args.first(),
        Some(syn::GenericArgument::Type(inner))
            if matches!(inner, syn::Type::Path(p) if p.path.is_ident("str"))
    )
}

fn logical_type(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Reference(reference) = ty {
        if reference.mutability.is_some() {
            return None;
        }
        return logical_type(&reference.elem);
    }
    // §5.3 参数适配变体：仅接受恰好 `impl AsRef<str>` 的视图形。
    if is_asref_str_view(ty) {
        return Some("str".into());
    }
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if !matches!(segment.arguments, syn::PathArguments::None) {
        return match segment.ident.to_string().as_str() {
            "Option" => {
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return None;
                };
                let syn::GenericArgument::Type(inner) = arguments.args.first()? else {
                    return None;
                };
                Some(format!("{}?", logical_type(inner)?))
            }
            _ => None,
        };
    }
    Some(match segment.ident.to_string().as_str() {
        "str" | "String" => "str".into(),
        "i32" | "i64" => "int".into(),
        "bool" => "bool".into(),
        "f64" => "float".into(),
        "Value" => "JsonValue".into(),
        // 不透明资源类型按名身份放行（公共 .at 声明同名；物理 layout 属
        // target，manifest 只记录身份，不宣称跨目标共享 ABI）。
        "FileResponse" | "UploadRequest" | "UploadSession" | "UploadReceipt" => {
            segment.ident.to_string()
        }
        _ => return None,
    })
}

#[cfg(test)]
mod plan738_host {
    use super::*;
    #[test]
    fn rust_signature_proof_is_bound_to_actual_emitted_callee() {
        let proof = verify_rust_reference(
            "json",
            "parse",
            "a2r_std::json::parse(\"{}\")",
            RustRuntime::Standalone,
        )
        .unwrap()
        .unwrap();
        assert_eq!(proof.callee, "a2r_std::json::parse");
        assert_eq!(proof.producer, "crates/a2r-std/src/json.rs::parse");
        assert_eq!(proof.selected_adapter.returns, "JsonValue?");
        let changed = verify_rust_reference(
            "json",
            "parse",
            "a2r_std::json::is_valid(\"{}\")",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(changed.to_string().contains("SIGNATURE_DRIFT"));
        let missing = verify_rust_reference(
            "json",
            "parse",
            "a2r_std::json::not_present(\"{}\")",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(missing.to_string().contains("PROVIDER_CLAIM_NO_CALLEE"));
    }
    #[test]
    fn embedded_runtime_has_its_own_producer_identity() {
        let proof = verify_rust_reference(
            "json",
            "parse",
            "a2r_std::json::parse(\"{}\")",
            RustRuntime::Embedded,
        )
        .unwrap()
        .unwrap();
        assert_eq!(proof.producer, "crates/auto-lang/src/a2r_std.rs::parse");
    }

    #[test]
    fn reexport_forwarding_resolves_real_definition() {
        // 内嵌镜像的 http 模块是同形转发壳：file_response 经 `pub use` 链
        // 解析到独立 crate 的定义文件，不以壳文件冒充 producer。
        let proof = verify_rust_reference(
            "http",
            "file_response",
            "a2r_std::http::file_response(\"files\", \"a.txt\", \"{}\")",
            RustRuntime::Embedded,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            proof.producer,
            "crates/a2r-std/src/http/server_file.rs::file_response"
        );
        assert_eq!(proof.selected_adapter.returns, "FileResponse");
        assert_eq!(proof.selected_adapter.parameters, ["str", "str", "str"]);
    }

    #[test]
    fn awaited_async_producer_is_mode_evidence_not_blanket_reject() {
        // async producer + 调用点 await = §5.7 的 mode 证据；缺 await = 漂移。
        let proof = verify_rust_reference(
            "http",
            "upload_receive",
            "a2r_std::http::upload_receive(req, \"r\", \"s\", \"{}\").await",
            RustRuntime::Embedded,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            proof.producer,
            "crates/a2r-std/src/http/server_upload.rs::upload_receive"
        );
        assert_eq!(proof.selected_adapter.returns, "UploadSession");
        assert!(proof
            .selected_adapter
            .error_shape
            .contains("call-site await"));
        assert!(proof
            .selected_adapter
            .error_shape
            .contains("impl AsRef<str> view adapter"));
        let drift = verify_rust_reference(
            "http",
            "upload_receive",
            "a2r_std::http::upload_receive(req, \"r\", \"s\", \"{}\")",
            RustRuntime::Embedded,
        )
        .unwrap_err();
        assert!(drift.to_string().contains("SIGNATURE_DRIFT"), "{drift}");
        assert!(drift.to_string().contains("awaited=false"), "{drift}");
    }

    #[test]
    fn sync_producer_rejects_stray_await() {
        let error = verify_rust_reference(
            "http",
            "upload_metadata",
            "a2r_std::http::upload_metadata(session).await",
            RustRuntime::Embedded,
        )
        .unwrap_err();
        assert!(error.to_string().contains("SIGNATURE_DRIFT"), "{error}");
        assert!(error.to_string().contains("awaited=true"), "{error}");
    }
}

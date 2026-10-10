//! Rust producer evidence is read from the selected producer's Rust AST.
//! Routing alone and provider catalog claims cannot establish a signature.
//!
//! rv3 §5.8.1（T-10）：发射侧适配（数值 cast / 状态侧信道包装 / 三参
//! 元数分派 / async 流 facade）不再因「形状像包装」豁免。每条适配必须
//! 命中 [`ADAPTER_RULES`] 中独立声明的契约，且三方一致才得
//! SignatureChecked：发射结构与契约逐项对拍（callee/元数/await/形状）、
//! producer 实形与契约对拍（async/参数/返回结构）、公共声明与适配投影
//! 对拍。无契约的包装形状报 `SIGNATURE_UNVERIFIED`，与契约不符的漂移
//! 报 `SIGNATURE_DRIFT`；不存在 Resolved-only 放行臂。
use super::model::{LayerKind, ParseStatus, VerificationLevel};
use super::reference::ReferenceProof;

#[derive(Debug, Clone, Copy, Default)]
pub enum RustRuntime {
    #[default]
    Standalone,
    Embedded,
}

impl RustRuntime {
    fn label(self) -> &'static str {
        match self {
            RustRuntime::Standalone => "standalone",
            RustRuntime::Embedded => "embedded",
        }
    }
}

// ===== 发射侧适配契约注册表（独立可检查；每条对应一档冻结的发射形状）=====

/// 观察到的发射形状类别。非 `Plain` 形状必须有对应契约条目。
#[derive(Clone, Copy, PartialEq, Eq)]
enum AdapterShape {
    /// 直接调用（可在调用点携带 `.await`）。
    Plain,
    /// `<callee>(..) as i64` 数值面转换。
    ScalarCast,
    /// `{ let r = <callee>(..)[.await]; set_last_status(r.0); r.1 }`
    /// ——PLAN-724 状态侧信道：body 选中为公共返回，status 进 thread-local。
    StatusSideChannel,
    /// `async { let (s,b,e,k) = <callee>(..).await; HttpResponse{..} }`
    /// ——PLAN-724 T-06 三参认证 tuple 元数分派（历史 HttpResponse 面）。
    ArityDispatchAsyncBlock,
    /// `if <callee>(..) { 1 } else { 0 }`——bool→int 适配壳（公共面 int、
    /// producer bool 的既有语义适配，如 json.has_key）。
    BoolToIntIf,
}

/// producer 返回结构与公共返回的映射方式（契约声明，非类型名猜测）。
enum ProducerReturn {
    /// 可直接逻辑化的标量（`logical_type` 词汇）。
    Scalar(&'static str),
    /// producer 返回 `(int, str)`：body 元素为公共返回，status 进侧信道。
    StatusBodyTuple,
    /// producer 返回 `(int, str, str, str)`：整体重组为 HttpResponse 构造。
    AuthTuple4,
    /// producer 返回 facade 类型，公共面按契约映射（如 AsyncHTTPStream →
    /// HTTPStream：await 消费后同一逻辑流面）。
    Facade { producer: &'static str },
}

enum RuntimeSpec {
    Both,
    Standalone,
    Embedded,
}

struct AdapterRule {
    shape: AdapterShape,
    module: &'static str,
    symbol: &'static str,
    runtime: RuntimeSpec,
    /// 发射内实际调用的 callee 裸名（`a2r_std::<module>::<callee>` 末段）。
    callee: &'static str,
    /// 发射传给 producer 的元数（= producer 参数数）。
    arity: usize,
    /// 发射主调用 `.await` 的存在性。
    awaited: bool,
    /// producer 的 async 性。
    producer_async: bool,
    producer_return: ProducerReturn,
    /// 公共声明参数面（逻辑类型；不含历史扩展位）。
    public_params: &'static [&'static str],
    /// 契约冻结的历史扩展参数位（PLAN-724 三参 post 的 api_key——公共
    /// 声明无此位，适配层单独声明，不复制公共签名充作证据）。
    extension_params: &'static [&'static str],
    /// producer 参数面的 facade 视图（类型名 → 逻辑面），如 async 流
    /// 句柄 `&AsyncHTTPStream` 是公共 `HTTPStream` 资源面的 async 侧
    /// 持有形态——契约声明，不以类型名猜测。
    param_facades: &'static [(&'static str, &'static str)],
    public_return: &'static str,
}

const ADAPTER_RULES: &[AdapterRule] = &[
    // —— PLAN-724 状态侧信道族（body 选中 / status 进 thread-local）——
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "post_sync",
        runtime: RuntimeSpec::Both,
        callee: "post_sync",
        arity: 3,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "post_sync",
        runtime: RuntimeSpec::Both,
        callee: "post_sync_async",
        arity: 3,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "get_sync",
        runtime: RuntimeSpec::Both,
        callee: "get_sync",
        arity: 1,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "get_sync",
        runtime: RuntimeSpec::Both,
        callee: "get_sync_async",
        arity: 1,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str"],
        extension_params: &[],
        public_return: "str",
    },
    // post_bearer 的 async producer 只在内嵌镜像（crates/auto-lang/src/
    // a2r_std.rs）；standalone 无此面——契约限定 Embedded，不冒称 Both。
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "post_bearer",
        runtime: RuntimeSpec::Embedded,
        callee: "post_bearer",
        arity: 3,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "post_bearer_sync",
        runtime: RuntimeSpec::Both,
        callee: "post_bearer_sync",
        arity: 3,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::StatusSideChannel,
        module: "http",
        symbol: "post_bearer_sync",
        runtime: RuntimeSpec::Both,
        callee: "post_bearer_sync_async",
        arity: 3,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::StatusBodyTuple,
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "str",
    },
    // —— 状态读取数值面（producer u32/i32 → 发射 i64 化为公共 int）——
    AdapterRule {
        shape: AdapterShape::ScalarCast,
        module: "http",
        symbol: "last_status",
        runtime: RuntimeSpec::Both,
        callee: "last_status",
        arity: 0,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("int"),
        param_facades: &[],
        public_params: &[],
        extension_params: &[],
        public_return: "int",
    },
    // —— json.len 模块拼写的宽度归一 cast 壳（`(.. as i64)`：公共 int、
    // producer usize——与 last_status 同型的数值面适配）。——
    AdapterRule {
        shape: AdapterShape::ScalarCast,
        module: "json",
        symbol: "JsonValue.len",
        runtime: RuntimeSpec::Both,
        callee: "len",
        arity: 1,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("int"),
        param_facades: &[],
        public_params: &["JsonValue"],
        extension_params: &[],
        public_return: "int",
    },
    // —— PLAN-724 T-06 三参认证 tuple 元数分派：3 参 async post（producer
    // 仅内嵌镜像有）→ (status, body, error, kind) 重组 HttpResponse。公共
    // 声明为两参 post；api_key 位由契约单独冻结。——
    AdapterRule {
        shape: AdapterShape::ArityDispatchAsyncBlock,
        module: "http",
        symbol: "post",
        runtime: RuntimeSpec::Embedded,
        callee: "post",
        arity: 3,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::AuthTuple4,
        param_facades: &[],
        public_params: &["str", "str"],
        extension_params: &["str"],
        public_return: "Response",
    },
    // —— async 流 facade：producer 产出 facade 类型（AsyncHTTPStream），
    // await 消费后对公共面按名映射为 HTTPStream。同步面为同名直接调用，
    // 走通用严格路径，无需契约。——
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "get_stream",
        runtime: RuntimeSpec::Both,
        callee: "get_stream_async",
        arity: 1,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::Facade {
            producer: "AsyncHTTPStream",
        },
        param_facades: &[],
        public_params: &["str"],
        extension_params: &[],
        public_return: "HTTPStream",
    },
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "post_stream",
        runtime: RuntimeSpec::Both,
        callee: "post_stream_async",
        arity: 2,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::Facade {
            producer: "AsyncHTTPStream",
        },
        param_facades: &[],
        public_params: &["str", "str"],
        extension_params: &[],
        public_return: "HTTPStream",
    },
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "post_stream_with_headers",
        runtime: RuntimeSpec::Both,
        callee: "post_stream_with_headers_async",
        arity: 3,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::Facade {
            producer: "AsyncHTTPStream",
        },
        param_facades: &[],
        public_params: &["str", "str", "str"],
        extension_params: &[],
        public_return: "HTTPStream",
    },
    // —— 流消费自由函数的 async 面：句柄经 AsyncHTTPStream 视图持有，
    // next 的 await 消费后同一逻辑流面（公共 stream_* 三件套）。——
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "stream_next",
        runtime: RuntimeSpec::Both,
        callee: "stream_next_async",
        arity: 1,
        awaited: true,
        producer_async: true,
        producer_return: ProducerReturn::Scalar("str"),
        param_facades: &[("AsyncHTTPStream", "HTTPStream")],
        public_params: &["HTTPStream"],
        extension_params: &[],
        public_return: "str",
    },
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "stream_is_done",
        runtime: RuntimeSpec::Both,
        callee: "stream_is_done_async",
        arity: 1,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("int"),
        param_facades: &[("AsyncHTTPStream", "HTTPStream")],
        public_params: &["HTTPStream"],
        extension_params: &[],
        public_return: "int",
    },
    AdapterRule {
        shape: AdapterShape::Plain,
        module: "http",
        symbol: "stream_close",
        runtime: RuntimeSpec::Both,
        callee: "stream_close_async",
        arity: 1,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("void"),
        param_facades: &[("AsyncHTTPStream", "HTTPStream")],
        public_params: &["HTTPStream"],
        extension_params: &[],
        public_return: "void",
    },
    // —— json.has_key bool→int 适配壳（发射臂注释在案的既有语义适配：
    // 公共面 int，Rust producer bool；Value 面与 str-receiver 视图两条）。——
    AdapterRule {
        shape: AdapterShape::BoolToIntIf,
        module: "json",
        symbol: "JsonValue.has_key",
        runtime: RuntimeSpec::Both,
        callee: "has_key",
        arity: 2,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("bool"),
        param_facades: &[],
        public_params: &["JsonValue", "str"],
        extension_params: &[],
        public_return: "int",
    },
    AdapterRule {
        shape: AdapterShape::BoolToIntIf,
        module: "json",
        symbol: "JsonValue.has_key",
        runtime: RuntimeSpec::Both,
        callee: "has_key_str",
        arity: 2,
        awaited: false,
        producer_async: false,
        producer_return: ProducerReturn::Scalar("bool"),
        // str-receiver 视图：首个 &str 参数是公共 JsonValue 接收者的
        // str 键查询形态（§5.3 参数适配变体）。
        param_facades: &[("str", "JsonValue")],
        public_params: &["JsonValue", "str"],
        extension_params: &[],
        public_return: "int",
    },
];

/// 结构观察结果：发射文本中可独立复核的事实。
struct Emission {
    shape: AdapterShape,
    /// 主调用 callee 裸名（`a2r_std::<module>::<name>` 末段）。
    callee: String,
    /// 主调用实参数。
    arity: usize,
    /// 主调用 `.await` 存在性（含包装块内）。
    awaited: bool,
    /// ScalarCast 的目标已逻辑化为 int。
    cast_to_int: bool,
}

fn unproved(module: &str, symbol: &str, detail: &str) -> crate::error::AutoError {
    format!("STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: {detail}").into()
}

fn drifted(module: &str, symbol: &str, detail: &str) -> crate::error::AutoError {
    format!("STDASSEMBLY.SIGNATURE_DRIFT: {module}.{symbol}: {detail}").into()
}

/// 观察发射表达式的结构。任何无法归入已知形状的表达都视为无证明。
fn observe_emission(module: &str, symbol: &str, emitted: &str) -> crate::AutoResult<Emission> {
    let detail = "adapter expression requires independent proof";
    let mut expression: syn::Expr = syn::parse_str(emitted).map_err(|e| {
        unproved(
            module,
            symbol,
            &format!("emitted expression cannot be inspected: {e}"),
        )
    })?;
    let mut shape = AdapterShape::Plain;
    let mut cast_to_int = false;
    // 剥括号外壳（`(x as i64)` 解析为 Paren 包 Cast——括号不透明）。
    while let syn::Expr::Paren(paren) = expression {
        expression = *paren.expr;
    }
    // ScalarCast：`<call>() as i64`（cast 目标必须逻辑化为 int——公共面
    // 即 int；cast 到其它宽度/类型不是本契约的一部分）。
    if let syn::Expr::Cast(cast) = expression {
        match logical_type(&cast.ty).as_deref() {
            Some("int") => cast_to_int = true,
            _ => return Err(unproved(module, symbol, "cast target is not the int face")),
        }
        shape = AdapterShape::ScalarCast;
        expression = *cast.expr;
    }
    // 调用点 await（Plain async 面）。
    let mut awaited = false;
    if let syn::Expr::Await(await_expr) = expression {
        awaited = true;
        expression = *await_expr.base;
    }
    match &expression {
        syn::Expr::Async(async_block) => {
            shape = AdapterShape::ArityDispatchAsyncBlock;
            let call = observe_arity_dispatch_block(module, symbol, &async_block.block)?;
            expression = syn::Expr::Call(call);
            awaited = true;
        }
        syn::Expr::Block(block) => {
            if shape == AdapterShape::Plain {
                shape = AdapterShape::StatusSideChannel;
                let (call, inner_awaited) =
                    observe_side_channel_block(module, symbol, &block.block)?;
                awaited = inner_awaited;
                expression = syn::Expr::Call(call);
            }
        }
        _ => {}
    }
    // BoolToIntIf 适配壳：`if <call> { 1 } else { 0 }`——then/else 恰为
    // 整型字面量 1/0，cond 为唯一主调用。
    if let syn::Expr::If(if_expr) = expression {
        let is_int_lit = |expr: &syn::Expr, value: i64| matches!(expr, syn::Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Int(i) if i.base10_digits() == value.to_string()));
        let then_ok = if_expr.then_branch.stmts.len() == 1
            && matches!(
                &if_expr.then_branch.stmts[0],
                syn::Stmt::Expr(e, None) if is_int_lit(e, 1)
            );
        let else_ok = match if_expr.else_branch.as_ref().map(|(_, expr)| expr.as_ref()) {
            Some(syn::Expr::Block(else_block)) => {
                else_block.block.stmts.len() == 1
                    && matches!(
                        &else_block.block.stmts[0],
                        syn::Stmt::Expr(e, None) if is_int_lit(e, 0)
                    )
            }
            _ => false,
        };
        if !(then_ok && else_ok) {
            return Err(unproved(module, symbol, detail));
        }
        shape = AdapterShape::BoolToIntIf;
        expression = *if_expr.cond.clone();
    }
    let syn::Expr::Call(call) = expression else {
        return Err(unproved(module, symbol, detail));
    };
    let syn::Expr::Path(path) = *call.func else {
        return Err(drifted(
            module,
            symbol,
            "emission calls through a non-path callee",
        ));
    };
    let names: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|p| p.ident.to_string())
        .collect();
    if names.len() != 3 || names[0] != "a2r_std" || names[1] != module {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "selected callee path {:?} is not the a2r_std::{module} face",
                names
            ),
        ));
    }
    Ok(Emission {
        shape,
        callee: names[2].clone(),
        arity: call.args.len(),
        awaited,
        cast_to_int,
    })
}

/// `async { let (s, b, e, k) = <call>(..).await; HttpResponse { s, b, e, k } }`
/// ——块内必须恰为一条 tuple 解构主调用 + 一条 HttpResponse 字段构造，
/// 字段名与解构名一致（shorthand）。返回主调用。
fn observe_arity_dispatch_block(
    module: &str,
    symbol: &str,
    block: &syn::Block,
) -> crate::AutoResult<syn::ExprCall> {
    let detail = "unrecognized arity-dispatch block structure (no independently checkable adapter)";
    let [syn::Stmt::Local(local), syn::Stmt::Expr(tail, None)] = block.stmts.as_slice() else {
        return Err(unproved(module, symbol, detail));
    };
    let syn::Pat::Tuple(pattern) = &local.pat else {
        return Err(unproved(module, symbol, detail));
    };
    let names: Vec<String> = pattern
        .elems
        .iter()
        .filter_map(|elem| match elem {
            syn::Pat::Ident(ident) => Some(ident.ident.to_string()),
            _ => None,
        })
        .collect();
    if names.len() != 4 || pattern.elems.len() != 4 {
        return Err(unproved(module, symbol, detail));
    }
    let init_expr = local.init.as_ref().map(|init| init.expr.as_ref());
    let Some(syn::Expr::Await(await_expr)) = init_expr else {
        return Err(unproved(module, symbol, detail));
    };
    let syn::Expr::Call(call) = &*await_expr.base else {
        return Err(unproved(module, symbol, detail));
    };
    let syn::Expr::Struct(construct) = tail else {
        return Err(unproved(module, symbol, detail));
    };
    if !construct.path.is_ident("HttpResponse") || construct.fields.len() != 4 {
        return Err(unproved(module, symbol, detail));
    }
    for field in &construct.fields {
        // shorthand 字段（无显式值）：成员名即解构名。
        let member = match &field.member {
            syn::Member::Named(ident) => ident.to_string(),
            _ => return Err(unproved(module, symbol, detail)),
        };
        if field.colon_token.is_some() || !names.contains(&member) {
            return Err(unproved(module, symbol, detail));
        }
    }
    Ok(call.clone())
}

/// `{ let r = <call>(..)[.await]; set_last_status(r.0); r.1 }`——块内必须
/// 恰为：绑定主调用、把 tuple 首元写入侧信道、尾表达式取 tuple 次元。
fn observe_side_channel_block(
    module: &str,
    symbol: &str,
    block: &syn::Block,
) -> crate::AutoResult<(syn::ExprCall, bool)> {
    let detail = "unrecognized side-channel block structure (no independently checkable adapter)";
    let [syn::Stmt::Local(local), syn::Stmt::Expr(status_call, Some(_)), tail] =
        block.stmts.as_slice()
    else {
        return Err(unproved(module, symbol, detail));
    };
    let syn::Pat::Ident(binding) = &local.pat else {
        return Err(unproved(module, symbol, detail));
    };
    let bound = binding.ident.to_string();
    let Some(init) = local.init.as_ref() else {
        return Err(unproved(module, symbol, detail));
    };
    let (primary, awaited) = match init.expr.as_ref() {
        syn::Expr::Await(await_expr) => match &*await_expr.base {
            syn::Expr::Call(call) => (call.clone(), true),
            _ => return Err(unproved(module, symbol, detail)),
        },
        syn::Expr::Call(call) => (call.clone(), false),
        _ => return Err(unproved(module, symbol, detail)),
    };
    // set_last_status(<bound>.0) —— 与主调用同一身份的 tuple 首元进侧信道。
    let syn::Expr::Call(status) = status_call else {
        return Err(unproved(module, symbol, detail));
    };
    let is_status_call = matches!(
        status.func.as_ref(),
        syn::Expr::Path(path)
            if path.path.segments.len() == 3
                && path.path.segments[0].ident == "a2r_std"
                && path.path.segments[1].ident == "http"
                && path.path.segments[2].ident == "set_last_status"
    );
    fn tuple_index(field: &syn::ExprField) -> Option<u32> {
        match &field.member {
            syn::Member::Unnamed(index) => Some(index.index),
            syn::Member::Named(_) => None,
        }
    }
    let arg_is_bound_zero = match status.args.first() {
        Some(syn::Expr::Field(field)) => {
            tuple_index(field) == Some(0)
                && matches!(&*field.base, syn::Expr::Path(p)
                    if p.path.is_ident(bound.as_str()))
        }
        _ => false,
    };
    if !is_status_call || status.args.len() != 1 || !arg_is_bound_zero {
        return Err(unproved(module, symbol, detail));
    }
    // 尾表达式 <bound>.1 —— body 元素成为公共返回。
    let tail_selects_body = match tail {
        syn::Stmt::Expr(syn::Expr::Field(field), None) => {
            tuple_index(field) == Some(1)
                && matches!(&*field.base, syn::Expr::Path(p)
                    if p.path.is_ident(bound.as_str()))
        }
        _ => false,
    };
    if !tail_selects_body {
        return Err(unproved(module, symbol, detail));
    }
    Ok((primary, awaited))
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
    let emission = observe_emission(module, symbol, emitted)?;
    let rule = ADAPTER_RULES.iter().find(|rule| {
        rule.module == module
            && rule.symbol == symbol
            && rule.shape == emission.shape
            && rule.callee == emission.callee
            && match (&rule.runtime, runtime) {
                (RuntimeSpec::Both, _) => true,
                (RuntimeSpec::Standalone, RustRuntime::Standalone) => true,
                (RuntimeSpec::Embedded, RustRuntime::Embedded) => true,
                _ => false,
            }
    });
    let Some(rule) = rule else {
        if emission.shape == AdapterShape::Plain {
            // Plain 形状无适配契约也可走通用严格路径（callee/元数/async/
            // 签名全部对拍）；契约只约束带适配的形状。
            return verify_plain_reference(
                module,
                symbol,
                &emission,
                runtime,
                declaration,
                &layer,
                public,
            );
        }
        return Err(unproved(
            module,
            symbol,
            &format!(
                "no declared adapter contract for {} shape / callee {} under {} runtime",
                emission.shape_label(),
                emission.callee,
                runtime.label()
            ),
        ));
    };
    verify_adapted_reference(
        module,
        symbol,
        &emission,
        runtime,
        declaration,
        &layer,
        public,
        rule,
    )
}

impl Emission {
    fn shape_label(&self) -> &'static str {
        match self.shape {
            AdapterShape::Plain => "plain",
            AdapterShape::ScalarCast => "scalar-cast",
            AdapterShape::StatusSideChannel => "status-side-channel",
            AdapterShape::ArityDispatchAsyncBlock => "arity-dispatch-async-block",
            AdapterShape::BoolToIntIf => "bool-to-int-if",
        }
    }
}

/// 契约驱动验证：发射 ↔ 契约 ↔ producer ↔ 公共声明四方一致。
#[allow(clippy::too_many_arguments)]
fn verify_adapted_reference(
    module: &str,
    symbol: &str,
    emission: &Emission,
    runtime: RustRuntime,
    declaration: String,
    layer: &super::model::LayerInventory,
    public: &super::model::SymbolEntry,
    rule: &AdapterRule,
) -> crate::AutoResult<Option<ReferenceProof>> {
    // 1. 发射结构 ↔ 契约。
    if emission.arity != rule.arity {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "emitted arity {} does not match the declared adapter contract arity {}",
                emission.arity, rule.arity
            ),
        ));
    }
    if emission.awaited != rule.awaited {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "await mode (emission awaited={}) does not match the declared adapter contract (awaited={})",
                emission.awaited, rule.awaited
            ),
        ));
    }
    if rule.shape == AdapterShape::ScalarCast && !emission.cast_to_int {
        return Err(drifted(
            module,
            symbol,
            "cast adapter no longer targets the int face",
        ));
    }
    // 2. producer 实形 ↔ 契约。
    let (producer_id, producer) = resolve_runtime_producer(module, &emission.callee, runtime)?;
    let producer_async = producer.sig.asyncness.is_some();
    if producer_async != rule.producer_async {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer async={} does not match the declared adapter contract (async={})",
                producer_async, rule.producer_async
            ),
        ));
    }
    if !producer.sig.generics.params.is_empty() {
        return Err(unproved(
            module,
            symbol,
            "generic producer has no adapter contract",
        ));
    }
    let mut adapters: Vec<&'static str> = Vec::new();
    let mut parameters = Vec::new();
    for parameter in &producer.sig.inputs {
        let syn::FnArg::Typed(parameter) = parameter else {
            return Err(unproved(
                module,
                symbol,
                "producer receiver has no adapter contract",
            ));
        };
        if is_asref_str_view(&parameter.ty) {
            // §5.3：兼容 lowering 的参数适配变体单独记录，不冒称同形。
            adapters.push("impl AsRef<str> view adapter");
        }
        let type_segment = |ty: &syn::Type| -> Option<String> {
            let syn::Type::Reference(reference) = ty else {
                return None;
            };
            let syn::Type::Path(path) = reference.elem.as_ref() else {
                return None;
            };
            path.path.segments.last().map(|s| s.ident.to_string())
        };
        // 契约声明的参数 facade 视图优先（按 producer 类型段名命中——
        // async 流句柄 / str-receiver 键查询等 §5.3 适配变体）。
        let declared_facade = type_segment(&parameter.ty).and_then(|segment| {
            rule.param_facades
                .iter()
                .find(|(producer, _)| *producer == segment)
                .map(|(_, face)| face.to_string())
        });
        let logical = match declared_facade {
            Some(face) => {
                adapters.push("declared param facade view adapter");
                Some(face)
            }
            None => logical_type(&parameter.ty).or_else(|| {
                type_segment(&parameter.ty).and_then(|segment| {
                    rule.param_facades
                        .iter()
                        .find(|(producer, _)| *producer == segment)
                        .map(|(_, face)| {
                            adapters.push("async facade handle view adapter");
                            face.to_string()
                        })
                })
            }),
        };
        parameters.push(logical.ok_or_else(|| {
            unproved(
                module,
                symbol,
                &format!("producer parameter representation: {producer_id}"),
            )
        })?);
    }
    if parameters.len() != rule.arity {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer arity {} does not match the declared adapter contract arity {}",
                parameters.len(),
                rule.arity
            ),
        ));
    }
    // 参数双源：producer 逻辑面 == 公共参数面 + 契约扩展位。
    let mut declared_params = rule.public_params.to_vec();
    declared_params.extend_from_slice(rule.extension_params);
    if parameters != declared_params {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer parameter face {parameters:?} does not match the declared adapter projection {declared_params:?}"
            ),
        ));
    }
    // producer 返回结构 ↔ 契约。
    match &rule.producer_return {
        ProducerReturn::Scalar(expected) => {
            let returns = producer_return_logical(&producer.sig).ok_or_else(|| {
                unproved(
                    module,
                    symbol,
                    &format!("producer return representation: {producer_id}"),
                )
            })?;
            if returns != *expected {
                return Err(drifted(
                    module,
                    symbol,
                    &format!("producer return {returns} does not match the declared adapter scalar {expected}"),
                ));
            }
        }
        ProducerReturn::StatusBodyTuple => {
            check_tuple_return(module, symbol, &producer.sig, &["int", "str"])?;
        }
        ProducerReturn::AuthTuple4 => {
            check_tuple_return(module, symbol, &producer.sig, &["int", "str", "str", "str"])?;
        }
        ProducerReturn::Facade { producer: facade } => {
            let segment = producer_return_segment(&producer.sig).ok_or_else(|| {
                unproved(
                    module,
                    symbol,
                    &format!("producer return representation: {producer_id}"),
                )
            })?;
            if segment != *facade {
                return Err(drifted(
                    module,
                    symbol,
                    &format!("producer return facade {segment} does not match the declared adapter facade {facade}"),
                ));
            }
            adapters.push("async stream facade (awaited consumption)");
        }
    }
    let mut returns = rule.public_return.to_string();
    match rule.shape {
        AdapterShape::StatusSideChannel => {
            adapters.push("(status, body) tuple adapter; body selected, status to side-channel");
        }
        AdapterShape::ArityDispatchAsyncBlock => {
            adapters.push("arity-dispatch adapter; auth tuple reconstructed as HttpResponse");
        }
        AdapterShape::ScalarCast => {
            adapters.push("scalar cast adapter (int width normalization)");
        }
        AdapterShape::BoolToIntIf => {
            adapters.push("bool-to-int if adapter (public int face, bool producer)");
        }
        AdapterShape::Plain => {}
    }
    let mut error_shape = "Rust return value".to_string();
    if rule.producer_async {
        error_shape.push_str("; async producer consumed at call-site await");
    }
    for adapter in &adapters {
        error_shape.push_str("; ");
        error_shape.push_str(adapter);
    }
    // P738-R7：方法符号（`Owner.name`）的 receiver/is_static 来自公共
    // 声明事实——适配路径与通用路径同源。
    let owner = public
        .name
        .split('.')
        .next()
        .filter(|_| public.kind == super::model::SymbolKind::Method)
        .map(str::to_string);
    let is_static = public
        .signature
        .as_ref()
        .map(|sig| sig.is_static)
        .unwrap_or(false);
    let contract = crate::vm::native::NativeContract {
        parameter_modes: parameters.iter().map(|_| "View".into()).collect(),
        parameters,
        returns,
        producer: format!("{}::{}", producer_id, emission.callee),
        receiver: owner,
        is_static,
        generics: Vec::new(),
        error_shape,
    };
    // 3. 公共声明 ↔ 适配投影（扩展位由契约冻结，不与公共声明对拍）。
    check_public_projection(module, symbol, public, &contract, rule)?;
    Ok(Some(ReferenceProof {
        module: module.into(),
        symbol: symbol.into(),
        declaration,
        declaration_hash: layer.content_hash,
        producer: contract.producer.clone(),
        native_id: None,
        callee: format!("a2r_std::{module}::{}", emission.callee),
        verification: VerificationLevel::SignatureChecked,
        public_signature: public.signature.clone().unwrap(),
        selected_adapter: contract,
    }))
}

/// 公共声明与适配投影对拍：参数前缀逐位相等、返回相等；契约扩展位只
/// 记录在 error_shape（公共声明没有该位，不能复制公共签名充作证据）。
fn check_public_projection(
    module: &str,
    symbol: &str,
    public: &super::model::SymbolEntry,
    contract: &crate::vm::native::NativeContract,
    rule: &AdapterRule,
) -> crate::AutoResult<()> {
    let signature = public
        .signature
        .as_ref()
        .ok_or_else(|| unproved(module, symbol, "source logical signature missing"))?;
    let projection = super::model::LogicalSignature {
        generics: Vec::new(),
        parameters: rule.public_params.iter().map(|p| p.to_string()).collect(),
        parameter_modes: rule.public_params.iter().map(|_| "View".into()).collect(),
        returns: rule.public_return.to_string(),
        is_static: false,
        has_self: false,
        attributes: Vec::new(),
    };
    if signature.parameters != projection.parameters
        || signature.parameter_modes != projection.parameter_modes
        || signature.generics != projection.generics
        || signature.attributes.iter().any(|attr| attr != "vm")
    {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "public declaration does not match the adapter projection {:?} -> {:?}",
                signature.parameters, projection.parameters
            ),
        ));
    }
    let source_ret = super::validate::normalize_host_type(&signature.returns);
    let host_ret = super::validate::normalize_host_type(&contract.returns);
    if source_ret != host_ret {
        return Err(drifted(
            module,
            symbol,
            &format!("public return {source_ret} does not match the adapter return {host_ret}"),
        ));
    }
    if public.kind == super::model::SymbolKind::Method {
        let owner = public.name.split('.').next().unwrap_or("");
        if signature.is_static || contract.receiver.as_deref() != Some(owner) {
            return Err(drifted(
                module,
                symbol,
                "independent receiver/static contract unavailable",
            ));
        }
    }
    if public.arity != rule.public_params.len() {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "public arity {} does not match the adapter projection arity {}",
                public.arity,
                rule.public_params.len()
            ),
        ));
    }
    Ok(())
}

/// Plain 形状的通用严格路径：无适配契约也必须完整对拍 callee/async/
/// 元数/签名——不存在 Resolved-only 放行。
#[allow(clippy::too_many_arguments)]
fn verify_plain_reference(
    module: &str,
    symbol: &str,
    emission: &Emission,
    runtime: RustRuntime,
    declaration: String,
    layer: &super::model::LayerInventory,
    public: &super::model::SymbolEntry,
) -> crate::AutoResult<Option<ReferenceProof>> {
    let (producer_id, producer) = resolve_runtime_producer(module, &emission.callee, runtime)?;
    let producer_async = producer.sig.asyncness.is_some();
    if producer_async != emission.awaited {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "async mode mismatch (producer async={producer_async}, emission awaited={}): {}",
                emission.awaited, producer_id
            ),
        ));
    }
    if !producer.sig.generics.params.is_empty() {
        return Err(unproved(
            module,
            symbol,
            &format!(
                "{producer_id}::{}: generic adapter unavailable",
                emission.callee
            ),
        ));
    }
    let mut adapters: Vec<&'static str> = Vec::new();
    let mut parameters = Vec::new();
    for parameter in &producer.sig.inputs {
        let syn::FnArg::Typed(parameter) = parameter else {
            return Err(unproved(
                module,
                symbol,
                &format!("{producer_id}::{}: receiver", emission.callee),
            ));
        };
        if is_asref_str_view(&parameter.ty) {
            adapters.push("impl AsRef<str> view adapter");
        }
        parameters.push(logical_type(&parameter.ty).ok_or_else(|| {
            unproved(
                module,
                symbol,
                &format!(
                    "{producer_id}::{}: parameter representation",
                    emission.callee
                ),
            )
        })?);
    }
    let mut returns = producer_return_logical(&producer.sig).ok_or_else(|| {
        unproved(
            module,
            symbol,
            &format!("{producer_id}::{}: return representation", emission.callee),
        )
    })?;
    let mut error_shape = "Rust return value".to_string();
    // The selected parse implementation uses serde_json::Value::Null as its
    // nullable payload. This representation is identified explicitly.
    // P738-R7：Null 哨兵族——parse 与 JsonValue.get/get_at 的缺键/失败
    // 返回 Value::Null 哨兵而非 Option，与公共 JsonValue? 的既有表示适配。
    let null_sentinel_face = matches!(
        (module, symbol),
        ("json", "parse") | ("json", "JsonValue.get") | ("json", "JsonValue.get_at")
    );
    if null_sentinel_face && returns == "JsonValue" {
        returns = "JsonValue?".into();
        error_shape = "Value::Null sentinel on missing key / parse failure".into();
    }
    if producer_async {
        error_shape.push_str("; async producer consumed at call-site await");
    }
    for adapter in &adapters {
        error_shape.push_str("; ");
        error_shape.push_str(adapter);
    }
    if emission.arity != parameters.len() {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "emitted arity {} / Rust producer arity {} ({producer_id}::{})",
                emission.arity,
                parameters.len(),
                emission.callee
            ),
        ));
    }
    // P738-R5-02：方法符号（`Owner.name`）的 receiver/is_static 来自公共
    // 声明事实（self 方法须有 receiver=Owner），普通 fn 保持 None/false。
    let owner = public
        .name
        .split('.')
        .next()
        .filter(|_| public.kind == super::model::SymbolKind::Method)
        .map(str::to_string);
    let is_static = public
        .signature
        .as_ref()
        .map(|sig| sig.is_static)
        .unwrap_or(false);
    let contract = crate::vm::native::NativeContract {
        parameter_modes: parameters.iter().map(|_| "View".into()).collect(),
        parameters,
        returns,
        producer: format!("{}::{}", producer_id, emission.callee),
        receiver: owner,
        is_static,
        generics: Vec::new(),
        error_shape,
    };
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
        callee: format!("a2r_std::{module}::{}", emission.callee),
        verification: VerificationLevel::SignatureChecked,
        public_signature: public.signature.clone().unwrap(),
        selected_adapter: contract,
    }))
}

/// PLAN-738 T-11：具名导入闭包——`use auto.<core>: name` / `use <core>: name`
/// 发射 `use a2r_std::<core>::<name>`，导入语句本身即对该符号的真实引用。
/// 绑定（producer 存在）与公共签名必须成立；纯适配面符号（无裸名可用
/// 形状，如 post_sync 侧信道族）在此拒绝并指明限定拼写——裸名拼写不能
/// 复现适配壳。调用点的元数/async 证据由 call 收集器的裸名闭包补齐。
pub fn verify_rust_import(
    module: &str,
    symbol: &str,
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
        return Ok(None); // No public claim for this name (private/unknown).
    };
    let plain = verify_plain_reference(
        module,
        symbol,
        &Emission {
            shape: AdapterShape::Plain,
            callee: symbol.to_string(),
            arity: public.arity,
            awaited: false,
            cast_to_int: false,
        },
        runtime,
        declaration,
        &layer,
        public,
    );
    match plain {
        Ok(proof) => Ok(proof),
        Err(error) => {
            // 纯适配面符号：公共面只能经限定拼写的适配壳满足——裸名导入
            // 无法复现（侧信道/cast/元数分派），给出限定拼写指引。
            let adapter_only = ADAPTER_RULES
                .iter()
                .any(|rule| rule.module == module && rule.symbol == symbol)
                && !ADAPTER_RULES.iter().any(|rule| {
                    rule.module == module
                        && rule.symbol == symbol
                        && rule.shape == AdapterShape::Plain
                });
            if adapter_only {
                return Err(format!(
                    "STDASSEMBLY.SIGNATURE_UNVERIFIED: {module}.{symbol}: adapter face has no bare-name spelling; call it qualified as {module}.{symbol}(...)"
                )
                .into());
            }
            Err(error)
        }
    }
}

/// 公共方法分母归属（R5-02）：发射 callee 名在该模块公共层的方法面。
/// 含发射侧适配拼写别名（类型名冲突变体 value_type→type、str-receiver
/// 视图 *_str→基础名）；无公共面映射返回 None（legacy 边界，不冒称证明）。
pub fn public_method_symbol(module: &str, callee: &str) -> Option<String> {
    const ALIASES: &[(&str, &str)] = &[
        ("value_type", "type"),
        ("as_string_str", "as_string"),
        ("as_int_str", "as_int"),
        ("as_bool_str", "as_bool"),
        ("len_str", "len"),
        ("has_key_str", "has_key"),
    ];
    let method = ALIASES
        .iter()
        .find(|(from, _)| *from == callee)
        .map(|(_, to)| *to)
        .unwrap_or(callee);
    let Ok(root) = super::loader::repo_stdlib_root() else {
        return None;
    };
    let Ok(text) = std::fs::read_to_string(root.join(format!("{module}.at"))) else {
        return None;
    };
    let layer =
        super::loader::parse_layer(&text, LayerKind::Public, format!("stdlib/auto/{module}.at"));
    layer
        .symbols
        .iter()
        .find(|entry| entry.is_pub && entry.name.ends_with(&format!(".{method}")))
        .map(|entry| entry.name.clone())
}

/// 公共面查询（T-11 wildcard 裸名归属）：模块公共 .at 是否声明该符号。
/// 只读事实查询，不产生证明。
pub fn public_symbol_exists(module: &str, name: &str) -> bool {
    let Ok(root) = super::loader::repo_stdlib_root() else {
        return false;
    };
    let Ok(text) = std::fs::read_to_string(root.join(format!("{module}.at"))) else {
        return false;
    };
    let layer =
        super::loader::parse_layer(&text, LayerKind::Public, format!("stdlib/auto/{module}.at"));
    layer
        .symbols
        .iter()
        .any(|entry| entry.is_pub && entry.name == name)
}

/// 按运行形态定位 producer 源并解析真实定义（含 `pub use` 再导出链）。
fn resolve_runtime_producer(
    module: &str,
    callee: &str,
    runtime: RustRuntime,
) -> crate::AutoResult<(String, syn::ItemFn)> {
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
            return Err(unproved(
                module,
                callee,
                "no inspected Rust producer for this module/runtime",
            ))
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
    let producer =
        resolve_producer(producer_id, items, callee, REEXPORT_DEPTH).ok_or_else(|| {
            format!(
                "STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE: {module}.{callee}: {producer_id}::{callee}"
            )
        })?;
    Ok(producer)
}

fn producer_return_logical(signature: &syn::Signature) -> Option<String> {
    match &signature.output {
        syn::ReturnType::Default => Some("void".into()),
        syn::ReturnType::Type(_, ty) => logical_type(ty),
    }
}

fn producer_return_segment(signature: &syn::Signature) -> Option<String> {
    match &signature.output {
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// producer 返回必须是逻辑形状逐位匹配的 tuple（契约声明，非猜测）。
fn check_tuple_return(
    module: &str,
    symbol: &str,
    signature: &syn::Signature,
    expected: &[&str],
) -> crate::AutoResult<()> {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer return is not the declared {}-tuple",
                expected.len()
            ),
        ));
    };
    let syn::Type::Tuple(tuple) = ty.as_ref() else {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer return is not the declared {}-tuple",
                expected.len()
            ),
        ));
    };
    if tuple.elems.len() != expected.len() {
        return Err(drifted(
            module,
            symbol,
            &format!(
                "producer tuple arity {} does not match the declared adapter tuple {}",
                tuple.elems.len(),
                expected.len()
            ),
        ));
    }
    for (index, (element, expected)) in tuple.elems.iter().zip(expected).enumerate() {
        let logical = logical_type(element).ok_or_else(|| {
            unproved(
                module,
                symbol,
                &format!("producer tuple element {index} representation"),
            )
        })?;
        if logical != *expected {
            return Err(drifted(
                module,
                symbol,
                &format!(
                    "producer tuple element {index} is {logical}, declared adapter expects {expected}"
                ),
            ));
        }
    }
    Ok(())
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
            // P738-R6-01：序列面 Vec<T> → []T（keys 的 Vec<String> 即公共
            // []str 的 Rust 形态——列表资源面按名身份，同 §5.3 适配变体）。
            "Vec" => {
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return None;
                };
                let syn::GenericArgument::Type(inner) = arguments.args.first()? else {
                    return None;
                };
                Some(format!("[]{}", logical_type(inner)?))
            }
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
        "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" => "int".into(),
        "bool" => "bool".into(),
        "f64" => "float".into(),
        "Value" => "JsonValue".into(),
        // 不透明资源类型按名身份放行（公共 .at 声明同名；物理 layout 属
        // target，manifest 只记录身份，不宣称跨目标共享 ABI）。
        "FileResponse" | "UploadRequest" | "UploadSession" | "UploadReceipt" => {
            segment.ident.to_string()
        }
        // http 客户端/服务端资源面（公共声明同名；句柄经 wire 物化）。
        "Response" | "HTTPStream" | "RequestBuilder" | "FileTransfer" => segment.ident.to_string(),
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

    #[test]
    fn side_channel_wrapper_needs_declared_contract_and_matches_it() {
        // 合法形状：post_sync 异步面（三参 + .await + set_last_status + __resp.1）。
        let proof = verify_rust_reference(
            "http",
            "post_sync",
            "{ let __resp = a2r_std::http::post_sync_async(\"u\", \"b\", \"k\").await; a2r_std::http::set_last_status(__resp.0); __resp.1 }",
            RustRuntime::Standalone,
        )
        .unwrap()
        .unwrap();
        assert_eq!(proof.callee, "a2r_std::http::post_sync_async");
        assert_eq!(proof.selected_adapter.returns, "str");
        assert_eq!(proof.selected_adapter.parameters, ["str", "str", "str"]);
        assert!(proof.selected_adapter.error_shape.contains("side-channel"));
        assert_eq!(proof.verification, VerificationLevel::SignatureChecked);
        // 契约元数漂移：包装内只传 2 参。
        let arity = verify_rust_reference(
            "http",
            "post_sync",
            "{ let __resp = a2r_std::http::post_sync_async(\"u\", \"b\").await; a2r_std::http::set_last_status(__resp.0); __resp.1 }",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(arity.to_string().contains("SIGNATURE_DRIFT"), "{arity}");
        assert!(
            arity
                .to_string()
                .contains("declared adapter contract arity"),
            "{arity}"
        );
        // 契约 await 漂移：异步 callee 缺 .await（旧 Resolved-only 放行位）。
        let await_drift = verify_rust_reference(
            "http",
            "post_sync",
            "{ let __resp = a2r_std::http::post_sync_async(\"u\", \"b\", \"k\"); a2r_std::http::set_last_status(__resp.0); __resp.1 }",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(
            await_drift.to_string().contains("SIGNATURE_DRIFT"),
            "{await_drift}"
        );
        assert!(
            await_drift.to_string().contains("await mode"),
            "{await_drift}"
        );
        // 结构漂移：侧信道写入 tuple 次元（__resp.1）而非首元。
        let structure = verify_rust_reference(
            "http",
            "post_sync",
            "{ let __resp = a2r_std::http::post_sync_async(\"u\", \"b\", \"k\").await; a2r_std::http::set_last_status(__resp.1); __resp.1 }",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(
            structure.to_string().contains("SIGNATURE_UNVERIFIED"),
            "{structure}"
        );
    }

    #[test]
    fn unknown_wrapper_shape_has_no_contract_and_is_rejected() {
        // 无契约的包装：get 不是侧信道符号——块形状查无契约（旧路径会以
        // Resolved-only 放行，现按 §5.8.1 拒绝）。
        let error = verify_rust_reference(
            "http",
            "get",
            "{ let __resp = a2r_std::http::get(\"u\"); __resp }",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("SIGNATURE_UNVERIFIED"),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("unrecognized side-channel block structure"),
            "{error}"
        );
    }

    #[test]
    fn arity_dispatch_post_proves_only_under_embedded() {
        let emitted =
            "async { let (status, body, error, kind) = a2r_std::http::post(\"u\", \"b\", \"k\").await; HttpResponse { status, body, error, kind } }";
        // Embedded：3 参 async post producer 存在——完整证明 + 扩展位记录。
        let proof = verify_rust_reference("http", "post", emitted, RustRuntime::Embedded)
            .unwrap()
            .unwrap();
        assert_eq!(proof.callee, "a2r_std::http::post");
        assert_eq!(proof.producer, "crates/auto-lang/src/a2r_std.rs::post");
        assert_eq!(proof.selected_adapter.returns, "Response");
        assert_eq!(proof.selected_adapter.parameters, ["str", "str", "str"]);
        assert_eq!(proof.public_signature.parameters, ["str", "str"]);
        assert!(proof
            .selected_adapter
            .error_shape
            .contains("arity-dispatch adapter"));
        assert_eq!(proof.verification, VerificationLevel::SignatureChecked);
        // Standalone：a2r-std crate 无 3 参 post——不再 Resolved-only 放行。
        let standalone =
            verify_rust_reference("http", "post", emitted, RustRuntime::Standalone).unwrap_err();
        assert!(
            standalone.to_string().contains("SIGNATURE_UNVERIFIED"),
            "{standalone}"
        );
        assert!(
            standalone
                .to_string()
                .contains("no declared adapter contract"),
            "{standalone}"
        );
        // 契约元数漂移：块内只传 2 参（对 Embedded producer 也是漂移）。
        let drift = verify_rust_reference(
            "http",
            "post",
            "async { let (status, body, error, kind) = a2r_std::http::post(\"u\", \"b\").await; HttpResponse { status, body, error, kind } }",
            RustRuntime::Embedded,
        )
        .unwrap_err();
        assert!(drift.to_string().contains("SIGNATURE_DRIFT"), "{drift}");
    }

    #[test]
    fn cast_adapter_targets_int_face_only() {
        let proof = verify_rust_reference(
            "http",
            "last_status",
            "a2r_std::http::last_status() as i64",
            RustRuntime::Standalone,
        )
        .unwrap()
        .unwrap();
        assert_eq!(proof.selected_adapter.returns, "int");
        assert_eq!(proof.verification, VerificationLevel::SignatureChecked);
        // cast 到非 int 面（float）：契约不存在该适配。
        let drift = verify_rust_reference(
            "http",
            "last_status",
            "a2r_std::http::last_status() as f64",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(
            drift.to_string().contains("SIGNATURE_UNVERIFIED"),
            "{drift}"
        );
        assert!(drift.to_string().contains("cast target"), "{drift}");
    }

    #[test]
    fn stream_facade_contract_maps_async_producer_to_public_face() {
        let proof = verify_rust_reference(
            "http",
            "get_stream",
            "a2r_std::http::get_stream_async(\"u\").await",
            RustRuntime::Standalone,
        )
        .unwrap()
        .unwrap();
        assert_eq!(proof.selected_adapter.returns, "HTTPStream");
        assert!(proof
            .selected_adapter
            .error_shape
            .contains("async stream facade"));
        assert_eq!(proof.verification, VerificationLevel::SignatureChecked);
        // facade 规则按 callee 命中：同步面走通用严格路径（HTTPStream 直接
        // 逻辑化），不带 facade 记录。
        let sync = verify_rust_reference(
            "http",
            "get_stream",
            "a2r_std::http::get_stream(\"u\")",
            RustRuntime::Standalone,
        )
        .unwrap()
        .unwrap();
        assert_eq!(sync.selected_adapter.returns, "HTTPStream");
        assert!(!sync.selected_adapter.error_shape.contains("facade"));
        // async callee 缺 await（facade 依赖 await 消费）→ 漂移。
        let drift = verify_rust_reference(
            "http",
            "get_stream",
            "a2r_std::http::get_stream_async(\"u\")",
            RustRuntime::Standalone,
        )
        .unwrap_err();
        assert!(drift.to_string().contains("SIGNATURE_DRIFT"), "{drift}");
        assert!(drift.to_string().contains("await mode"), "{drift}");
    }
}

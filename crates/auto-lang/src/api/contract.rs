//! PLAN-734 T-02：API 传输契约——类型身份分类与参数计划的**单源**。
//!
//! 契约速览（决策报告 §2 D1/D9；canonical Spec 由 SD-01 沉淀）：
//!
//! ```text
//! ResponseKind::from_return_string(ret)  ← 所有"这个端点返回什么种类"的判定单源
//! ParamKind::from_param_string(ty)       ← 参数数据种类（VM/生成/back-proxy 共用）
//! EndpointContract::build(module)        ← 生成期完整参数计划（source/required/kind）
//! ```
//!
//! 判定是**类型身份**（裸名/泛型解包后的精确匹配），不是 `contains` 子串——
//! 用户定义 `MyFileResponse` 不命中 File（AC-01 假同名反例）。所有消费面
//! （VM 编组门、back-proxy 守卫、三 targets、api_gen 分支）经本模块分类，
//! contains 字面量退役。纯模块：不依赖 Axum/Tokio/VM/Tauri。

use super::types::{ApiEndpoint, ApiModule, ApiParam, ApiType};

// ============================================================================
// 响应种类
// ============================================================================

/// 端点返回的种类——决定编组分派、传输支持矩阵与生成分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseKind {
    /// 普通 JSON 数据（声明返回非下方专属种类）。
    Json,
    /// `~Iter<T>` / `~Stream<T>`（或裸 Iter/Stream 泛型）——SSE 流。
    Stream,
    /// `FileResponse`（含 `Future<FileResponse>`）——729 文件下载（HTTP 专属）。
    File,
    /// `UploadRequest` 参数或 `UploadReceipt` 返回——730 上传（HTTP 专属）。
    Upload,
    /// `Response`（显式响应对象：status/headers/body）。
    ExplicitResponse,
    /// 无返回（`()`/`void`）。
    Void,
}

impl ResponseKind {
    /// 从返回类型显示串做**身份**分类（exact / 泛型解包 exact——非 contains）。
    /// 输入串来自 `type_to_string`（param 侧 Display）或 `unique_name`（返回侧），
    /// 两者对 User/泛型形态产出同形（`"FileResponse"` / `"Future<FileResponse>"`）。
    pub fn from_return_string(ret: &str) -> ResponseKind {
        let ret = ret.trim();
        // ~T 脱糖为 Future<T>；两种书写都解包。
        let unwrapped = unwrap_future(ret);
        let name = bare_name(unwrapped);
        match name {
            "FileResponse" => ResponseKind::File,
            "UploadReceipt" | "UploadSession" => ResponseKind::Upload,
            "Response" | "HttpResponse" => ResponseKind::ExplicitResponse,
            "Iter" | "Stream" => ResponseKind::Stream,
            "()" | "void" | "Void" | "" => ResponseKind::Void,
            _ => ResponseKind::Json,
        }
    }

    pub fn is_http_exclusive(self) -> bool {
        matches!(self, ResponseKind::File | ResponseKind::Upload)
    }
}

/// 解包一层 `Future<T>`（`~T` 脱糖形态）；非该形态原样返回。
fn unwrap_future(ty: &str) -> &str {
    let trimmed = ty.trim();
    if let Some(inner) = trimmed
        .strip_prefix("Future<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return inner.trim();
    }
    trimmed
}

/// 泛型实例裸名：`Iter<ChatEvent>` → `Iter`；裸类型原样。
fn bare_name(ty: &str) -> &str {
    match ty.find('<') {
        Some(idx) => &ty[..idx],
        None => ty,
    }
}

// ============================================================================
// 参数数据种类
// ============================================================================

/// 参数/字段的数据种类（普通 JSON 支持子集；决策报告 §4）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamKind {
    Str,
    Bool,
    /// Auto `int`（语义 i64 全域）与定点整型（uint/u64/…——范围按 declared_ty 校验）。
    Int,
    Float,
    /// `?T` / `T?`（内层 kind 递归）。
    Optional(Box<ParamKind>),
    /// `[]T`（内层 kind 递归）。
    Array(Box<ParamKind>),
    /// 已解析命名 record（模块 types 命中；字段计划随 contract 携带）。
    Record(String),
    /// 宿主资源/未解析/不支持面（诊断，不降级 String/JSON）。
    Unsupported(String),
}

impl ParamKind {
    /// 从参数类型显示串分类。未知/未解析/宿主资源 → `Unsupported`（携带原串），
    /// 消费面按 transport 诊断——**不当 String**（决策 D1）。
    pub fn from_param_string(ty: &str, module_types: &[ApiType]) -> ParamKind {
        let trimmed = ty.trim();
        // optional：`?T` 前缀 / `T?` 后缀
        if let Some(inner) = trimmed.strip_prefix('?') {
            return ParamKind::Optional(Box::new(Self::from_param_string(inner, module_types)));
        }
        if let Some(inner) = trimmed.strip_suffix('?') {
            return ParamKind::Optional(Box::new(Self::from_param_string(inner, module_types)));
        }
        match trimmed {
            "str" | "String" => return ParamKind::Str,
            "bool" => return ParamKind::Bool,
            "int" | "i64" | "i32" | "uint" | "u32" | "u64" | "usize" | "byte" => {
                return ParamKind::Int
            }
            "float" | "double" | "f32" | "f64" => return ParamKind::Float,
            _ => {}
        }
        // 数组：`[]T` / `[N]T`
        if let Some(inner) = trimmed.strip_prefix("[]") {
            return ParamKind::Array(Box::new(Self::from_param_string(inner, module_types)));
        }
        if let Some(rest) = trimmed.strip_prefix('[') {
            // `[N]T`：长度段到首个 `]`，余下为内层类型。
            if let Some((_, inner)) = rest.split_once(']') {
                return ParamKind::Array(Box::new(Self::from_param_string(inner, module_types)));
            }
        }
        // 命名 record：模块 types 精确命中（假同名由"未在模块声明"自然落 Unsupported）
        let bare = bare_name(trimmed);
        if module_types.iter().any(|t| t.name == bare) {
            return ParamKind::Record(bare.to_string());
        }
        ParamKind::Unsupported(trimmed.to_string())
    }
}

// ============================================================================
// 参数来源
// ============================================================================

/// 参数的传输来源（绑定优先级：Path > Body > Query；Meta/WholeBody/Upload 为
/// 显式兼容/注入标记——决策 D1/D8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamSource {
    Path,
    Body,
    Query,
    /// cookies/auth metadata 约定名（Plan 317 §11；META_PARAM_NAMES 单源在此）。
    Meta,
    /// legacy 单参 whole-body 容忍（Plan 346 事实——显式标记，不再散落）。
    WholeBody,
    /// 730 宿主注入（UploadRequest；绑定先于一切规则）。
    Upload,
}

/// metadata 参数约定名（Plan 317 §11 / Plan 346 stage 4；大小写不敏感）。
pub const META_PARAM_NAMES: [&str; 4] = ["meta", "metadata", "req", "request"];

pub fn is_meta_alias(name: &str) -> bool {
    META_PARAM_NAMES.contains(&name.trim().to_lowercase().as_str())
}

/// 730 上传注入参数的身份判定（精确，非 contains）。
///
/// 输入串有两族来源：API 元数据侧（`type_to_string`/`unique_name` → 裸名
/// `UploadRequest`）与 VM 参数签名侧（ast `Type` 的 Display → User 类型
/// 是 s-expr `(type-decl (name UploadRequest) ...)`——730 期 `contains`
/// 恰好子串命中该形态）。身份判定对两族都按**名字等值**接受，仍拒绝
/// `MyUploadRequest`/`UploadRequestCtx` 等假同名。
pub fn is_upload_param(ty: &str) -> bool {
    let ty = ty.trim();
    if ty == "UploadRequest" {
        return true;
    }
    // s-expr 包装：`(type-decl (name UploadRequest)` 后跟 `)` 或 ` `。
    if let Some(rest) = ty.strip_prefix("(type-decl (name UploadRequest") {
        return rest.starts_with(')') || rest.starts_with(' ');
    }
    false
}

// ============================================================================
// 端点契约（生成期完整计划）
// ============================================================================

/// 单参数计划。
#[derive(Debug, Clone, PartialEq)]
pub struct ParamPlan {
    pub name: String,
    pub source: ParamSource,
    pub kind: ParamKind,
    pub required: bool,
    /// 声明类型原串（诊断/范围校验：uint vs int 的范围差异等）。
    pub declared_ty: String,
}

/// 单端点契约。
#[derive(Debug, Clone, PartialEq)]
pub struct EndpointContract {
    pub fn_name: String,
    pub method: String,
    pub path: String,
    pub params: Vec<ParamPlan>,
    pub response: ResponseKind,
    /// 返回类型原串（编组门按 `ResponseKind::from_return_string` 消费）。
    pub response_decl: String,
    pub is_async: bool,
}

/// 版本化模块契约。
#[derive(Debug, Clone, PartialEq)]
pub struct ApiContract {
    pub schema_version: u32,
    pub module_name: String,
    pub endpoints: Vec<EndpointContract>,
}

impl ApiContract {
    pub fn build(module: &ApiModule) -> ApiContract {
        ApiContract {
            schema_version: 1,
            module_name: module.name.clone(),
            endpoints: module
                .endpoints
                .iter()
                .map(|ep| EndpointContract::build(ep, &module.types))
                .collect(),
        }
    }
}

impl EndpointContract {
    pub fn build(ep: &ApiEndpoint, module_types: &[ApiType]) -> EndpointContract {
        let method = ep.method();
        let path = ep.path();
        let response_decl = ep.return_type.trim().to_string();
        let response = ResponseKind::from_return_string(&response_decl);
        let is_async = response_decl.starts_with("Future<") || response_decl.starts_with('~');
        let params = ep
            .params
            .iter()
            .map(|p| plan_param(p, &method, &path, module_types))
            .collect();
        EndpointContract {
            fn_name: ep.fn_name.clone(),
            method,
            path,
            params,
            response,
            response_decl,
            is_async,
        }
    }
}

fn plan_param(p: &ApiParam, method: &str, path: &str, module_types: &[ApiType]) -> ParamPlan {
    let declared_ty = p.ty.trim().to_string();
    let source = if is_upload_param(&declared_ty) {
        ParamSource::Upload
    } else if path.contains(&format!(":{}", p.name)) || path.contains(&format!("{{{}}}", p.name)) {
        ParamSource::Path
    } else if is_meta_alias(&p.name) && matches!(declared_ty.as_str(), "str" | "String") {
        ParamSource::Meta
    } else if matches!(method, "GET" | "DELETE") {
        ParamSource::Query
    } else {
        ParamSource::Body
    };
    ParamPlan {
        name: p.name.clone(),
        source,
        kind: ParamKind::from_param_string(&declared_ty, module_types),
        required: !p.optional,
        declared_ty,
    }
}

// ============================================================================
// 表驱动测试（身份分类/假同名反例/来源计划）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(ret: &str) -> ResponseKind {
        ResponseKind::from_return_string(ret)
    }

    #[test]
    fn plan734_response_kind_identity_matrix() {
        assert_eq!(kinds("FileResponse"), ResponseKind::File);
        assert_eq!(kinds("Future<FileResponse>"), ResponseKind::File);
        assert_eq!(kinds("~FileResponse"), ResponseKind::Json); // 非法书写不猜
        assert_eq!(kinds("UploadReceipt"), ResponseKind::Upload);
        assert_eq!(kinds("Future<UploadReceipt>"), ResponseKind::Upload);
        assert_eq!(kinds("Response"), ResponseKind::ExplicitResponse);
        assert_eq!(kinds("Future<Stream<ChatEvent>>"), ResponseKind::Stream);
        assert_eq!(kinds("Iter<int>"), ResponseKind::Stream);
        assert_eq!(kinds("Future<Iter<str>>"), ResponseKind::Stream);
        assert_eq!(kinds("()"), ResponseKind::Void);
        assert_eq!(kinds("int"), ResponseKind::Json);
        assert_eq!(kinds("User"), ResponseKind::Json);
        assert_eq!(kinds("?User"), ResponseKind::Json);
        // AC-01 假同名反例：contains 会命中，身份匹配不命中。
        assert_eq!(kinds("MyFileResponse"), ResponseKind::Json);
        assert_eq!(kinds("FileResponseLike"), ResponseKind::Json);
        assert_eq!(kinds("UploadReceiptLog"), ResponseKind::Json);
        assert_eq!(kinds("List<FileResponse>"), ResponseKind::Json);
        assert_eq!(kinds("MyResponse"), ResponseKind::Json);
        assert_eq!(kinds("FileResponseX<Stream<a>>"), ResponseKind::Json);
    }

    #[test]
    fn plan734_param_kind_matrix() {
        let types = vec![ApiType {
            name: "Note".into(),
            fields: vec![],
            doc: None,
        }];
        let k = |t: &str| ParamKind::from_param_string(t, &types);
        assert_eq!(k("str"), ParamKind::Str);
        assert_eq!(k("String"), ParamKind::Str);
        assert_eq!(k("int"), ParamKind::Int);
        assert_eq!(k("i64"), ParamKind::Int);
        assert_eq!(k("bool"), ParamKind::Bool);
        assert_eq!(k("float"), ParamKind::Float);
        assert_eq!(k("double"), ParamKind::Float);
        assert_eq!(
            k("?Note"),
            ParamKind::Optional(Box::new(ParamKind::Record("Note".into())))
        );
        assert_eq!(
            k("[]Note"),
            ParamKind::Array(Box::new(ParamKind::Record("Note".into())))
        );
        assert_eq!(k("[3]int"), ParamKind::Array(Box::new(ParamKind::Int)));
        assert_eq!(k("Note"), ParamKind::Record("Note".into()));
        // 未声明/宿主资源 → Unsupported（不降 String）。
        assert_eq!(
            k("FileResponse"),
            ParamKind::Unsupported("FileResponse".into())
        );
        assert_eq!(
            k("Map<str, int>"),
            ParamKind::Unsupported("Map<str, int>".into())
        );
        assert_eq!(k("Unknown1"), ParamKind::Unsupported("Unknown1".into()));
    }

    #[test]
    fn plan734_meta_and_upload_identity() {
        assert!(is_meta_alias("meta"));
        assert!(is_meta_alias("REQ"));
        assert!(is_meta_alias("request"));
        assert!(!is_meta_alias("note"));
        assert!(is_upload_param("UploadRequest"));
        assert!(!is_upload_param("UploadRequestCtx"));
        assert!(!is_upload_param("MyUploadRequest"));
    }
}

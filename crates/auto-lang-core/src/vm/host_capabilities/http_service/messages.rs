//! Owned HTTP bridge messages: no VM or protocol-framework objects.

/// Compiler-published service route, owned independently of any VM tables.
#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub fn_name: String,
}

/// Declared language parameter name and type display string.
#[derive(Debug, Clone)]
pub struct ApiParamSig {
    pub name: String,
    pub ty: String,
}

/// Immutable program handoff. No AST, Store, heap reference or VM pointer.
pub struct HttpProgramMetadata {
    pub params: std::collections::HashMap<String, Vec<ApiParamSig>>,
    pub returns: std::collections::HashMap<String, String>,
    pub shadowed: std::collections::HashSet<String>,
    pub routes: Vec<HttpRoute>,
}

/// Owned, `Send` request handed from the HTTP network layer to the VM owner
/// thread. Protocol objects (hyper types, sockets) never cross the bridge;
/// the owner re-runs the same routing/binding/middleware/response semantics
/// as the legacy inline path (AC-02: no `Rc<AutoVM>`, no raw pointers).
pub struct ApiRequest {
    pub method: String,
    /// Raw request target (path + query string), as received.
    pub path: String,
    /// All request headers (names lowercased, values as received).
    pub headers: Vec<(String, String)>,
    /// Fully framed body bytes (bounded by the transport's body limit).
    pub body: Vec<u8>,
    /// PLAN-730 T-05：上传路由的原始 body（bridge 不消费，授权后由
    /// receive 增量读取；非上传路由恒 None——`body` 语义不变）。
    pub raw_upload_body: Option<a2r_std::http::UploadBodyStream>,
    pub peer: Option<std::net::SocketAddr>,
}

impl ApiRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Reply body payload: plain bytes, SSE frames streamed from a producer
/// task on the VM owner thread, or a file response descriptor (PLAN-729).
pub enum ApiBody {
    Text(Vec<u8>),
    Sse(tokio::sync::mpsc::Receiver<String>),
    /// PLAN-729 T-04：文件响应描述符（编组单次取出；打开/协议决策/发送
    /// 在 transport 侧的共享宿主服务，不在 VM owner）。status/headers 由
    /// 宿主 serve 决定，`Full` 携带的占位被 File 臂忽略。
    File(FileReplySeed),
}

/// [`ApiBody::File`] 载荷：owned 描述符 + 请求 id（日志/CORS 追加用）。
pub struct FileReplySeed {
    pub descriptor: a2r_std::http::FileResponse,
    pub request_id: String,
}

/// Structured reply produced by the VM owner for one request.
pub enum ApiReply {
    Full {
        status: u16,
        headers: Vec<(String, String)>,
        body: ApiBody,
    },
    /// `#[api]` route matched an `Upgrade: websocket` request — the network
    /// layer writes the 101 (accept value precomputed here) and serves the
    /// raw echo loop on the upgraded IO. Capability boundary unchanged:
    /// simplified echo, not general WebSocket (Design 33 §7.3).
    WebSocket { accept: String },
}

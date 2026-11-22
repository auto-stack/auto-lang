//! PLAN-1119 AutoWeb 只读导航协议/缓存层永久 case（V-02 语义面）。
//!
//! 传输为脚本化 [`ScriptTransport`]（记录请求、按序返回响应）——网络
//! 执行与真实 TLS 链的证据面在 auto-ui `autoscape_autoweb_readonly`
//! （真 rustls 测试服务器）；本文件钉协议语义：
//! 地址/别名映射、v1 描述严格解析、同源/重定向/降级拒绝、媒体/状态/
//! 限额准入、ETag/`304`/`no-store`/`Vary` 缓存轨迹、只读准入扫描、
//! 取消与期限。

use ase_m0::autoweb::{
    admit_media, admit_readonly_document, parse_descriptor, parse_web_address, AddressSpelling,
    AutowebCache, AutowebTransport, CacheKey, NavLimits, NavOutcome, NavStage, ReadonlyNavigator,
    Representation, TransportFailure, TransportRequest, TransportResponse, ACCEPT_DESCRIPTOR,
    ACCEPT_DOCUMENT, DESCRIPTOR_MEDIA, DOCUMENT_MEDIA,
};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// 脚本化传输
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct LoggedRequest {
    url: String,
    accept: String,
    if_none_match: Option<String>,
}

#[derive(Clone)]
struct ScriptedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl ScriptedResponse {
    fn ok(content_type: &str, body: &str, etag: Option<&str>) -> Self {
        let mut headers = vec![("Content-Type".to_string(), content_type.to_string())];
        if let Some(etag) = etag {
            headers.push(("ETag".to_string(), etag.to_string()));
        }
        ScriptedResponse {
            status: 200,
            headers,
            body: body.as_bytes().to_vec(),
        }
    }
}

/// 按序出队响应；耗尽即 panic（脚本面比期望请求短=用例写错）。
struct ScriptTransport {
    script: Vec<ScriptedResponse>,
    /// 每个响应出队前的服务延时（毫秒；模拟慢链路，期限/取消用例）。
    delays_ms: Vec<u64>,
    log: Vec<LoggedRequest>,
    cancel_after: Option<usize>,
    seen: usize,
    cancelled: bool,
}

impl ScriptTransport {
    fn new(script: Vec<ScriptedResponse>) -> Self {
        let len = script.len();
        ScriptTransport {
            script,
            delays_ms: vec![0; len],
            log: Vec::new(),
            cancel_after: None,
            seen: 0,
            cancelled: false,
        }
    }
}

impl AutowebTransport for ScriptTransport {
    fn fetch(&mut self, req: &TransportRequest) -> Result<TransportResponse, TransportFailure> {
        self.seen += 1;
        if let Some(limit) = self.cancel_after {
            if self.seen > limit {
                self.cancelled = true;
            }
        }
        if self.cancelled {
            return Err(TransportFailure::Cancelled);
        }
        self.log.push(LoggedRequest {
            url: req.url.clone(),
            accept: req
                .headers
                .iter()
                .find(|(k, _)| *k == "Accept")
                .map(|(_, v)| v.clone())
                .unwrap_or_default(),
            if_none_match: req
                .headers
                .iter()
                .find(|(k, _)| *k == "If-None-Match")
                .map(|(_, v)| v.clone()),
        });
        let delay = self.delays_ms.get(self.seen - 1).copied().unwrap_or(0);
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        let response = self.script.remove(0);
        Ok(TransportResponse {
            status: response.status,
            headers: response.headers,
            body: response.body,
        })
    }

    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---------------------------------------------------------------------------
// 站点素材（与协议 §2 示例同型）
// ---------------------------------------------------------------------------

const DOC_BODY: &str =
    "# Gallery 指南\n\n只读文档正文。含 [显式链接](https://example.com/next)。\n";

fn doc_sha_hex(body: &str) -> String {
    let digest: [u8; 32] = Sha256::new()
        .chain_update(body.as_bytes())
        .finalize()
        .into();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn descriptor_json(doc_href: &str, sha_hex: &str, title: &str) -> String {
    format!(
        r#"{{"protocolVersion":1,"appId":"auto.demo.guide","release":"1.0.0","title":"{title}","entry":{{"kind":"autodown","href":"{doc_href}","sha256":"{sha_hex}"}},"assets":[],"services":[]}}"#
    )
}

fn descriptor_response(doc_href: &str, title: &str, etag: Option<&str>) -> ScriptedResponse {
    ScriptedResponse::ok(
        DESCRIPTOR_MEDIA,
        &descriptor_json(doc_href, &doc_sha_hex(DOC_BODY), title),
        etag,
    )
}

fn document_response(etag: Option<&str>) -> ScriptedResponse {
    ScriptedResponse::ok(DOCUMENT_MEDIA, DOC_BODY, etag)
}

/// `304 Not Modified`（可选 ETag）。
fn not_modified(etag: Option<&str>) -> ScriptedResponse {
    ScriptedResponse {
        status: 304,
        headers: etag
            .map(|e| vec![("ETag".to_string(), e.to_string())])
            .unwrap_or_default(),
        body: Vec::new(),
    }
}

fn navigate(
    transport: &mut ScriptTransport,
    cache: &mut AutowebCache,
    address: &str,
) -> NavOutcome {
    let parsed = parse_web_address(address).expect("address parses");
    let mut navigator = ReadonlyNavigator::new(transport, cache, NavLimits::default());
    navigator.navigate(&parsed)
}

fn refused(outcome: NavOutcome) -> (NavStage, String) {
    match outcome {
        NavOutcome::Refused(refusal, _) => (refusal.stage, refusal.reason),
        NavOutcome::Success(_) => panic!("expected refusal, got success"),
    }
}

// ---------------------------------------------------------------------------
// 地址与别名（V-01 地址面）
// ---------------------------------------------------------------------------

#[test]
fn p1119_addr_https_and_alias_map_to_same_origin() {
    let https = parse_web_address("https://Demo.Example/gallery/guide/").unwrap();
    assert_eq!(https.spelling, AddressSpelling::Https);
    assert_eq!(https.host, "demo.example");
    assert_eq!(https.https_url(), "https://demo.example/gallery/guide/");
    assert_eq!(https.origin(), "https://demo.example");

    let alias = parse_web_address("autoweb://demo.example:8443/gallery/guide/?v=1").unwrap();
    assert_eq!(alias.spelling, AddressSpelling::Autoweb);
    assert_eq!(
        alias.https_url(),
        "https://demo.example:8443/gallery/guide/?v=1"
    );
    assert_eq!(
        alias.display_address(),
        "autoweb://demo.example:8443/gallery/guide/?v=1"
    );
    // 规范 https 写法与别名共用同一来源。
    let plain = parse_web_address("https://demo.example:8443/gallery/guide/?v=1").unwrap();
    assert_eq!(alias.https_url(), plain.https_url());
    assert_eq!(alias.origin(), plain.origin());
}

#[test]
fn p1119_addr_fragment_stripped_and_defaults_applied() {
    let address = parse_web_address("autoweb://demo.example#section").unwrap();
    assert_eq!(address.https_url(), "https://demo.example/");
    let address = parse_web_address("https://demo.example?a=1#frag").unwrap();
    assert_eq!(address.https_url(), "https://demo.example/?a=1");
}

#[test]
fn p1119_addr_named_refusals() {
    for (input, needle) in [
        ("http://demo.example/", "insecure scheme http"),
        ("ftp://demo.example/", "unsupported scheme"),
        ("not-an-address", "not a network address"),
        ("https://user:pw@demo.example/", "userinfo"),
        ("https:///", "empty host"),
        ("https://demo.example:0/", "port out of range"),
        ("https://demo.example:99999/", "port out of range"),
        ("https://[::1]/", "IPv6"),
        ("https://BAD_HOST/", "lowercase"),
    ] {
        let error = parse_web_address(input).unwrap_err();
        assert!(
            error.to_string().contains(needle),
            "{input}: {} lacks {needle:?}",
            error
        );
    }
}

// ---------------------------------------------------------------------------
// 描述严格解析（V-01/V-03）
// ---------------------------------------------------------------------------

#[test]
fn p1119_descriptor_strict_v1_domain() {
    let base = parse_web_address("https://demo.example/gallery/guide/").unwrap();
    let ok = parse_descriptor(
        descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Guide").as_bytes(),
        &base,
    )
    .unwrap();
    assert_eq!(ok.protocol_version, 1);
    assert_eq!(ok.app_id, "auto.demo.guide");
    assert_eq!(ok.entry_url, "https://demo.example/gallery/guide/guide.ad");

    // 相对解析变体：根相对与 `../`。
    let root_rel = parse_descriptor(
        descriptor_json("/docs/guide.ad", &doc_sha_hex(DOC_BODY), "Guide").as_bytes(),
        &base,
    )
    .unwrap();
    assert_eq!(root_rel.entry_url, "https://demo.example/docs/guide.ad");
    let up_rel = parse_descriptor(
        descriptor_json("../guide.ad", &doc_sha_hex(DOC_BODY), "Guide").as_bytes(),
        &base,
    )
    .unwrap();
    assert_eq!(up_rel.entry_url, "https://demo.example/gallery/guide.ad");

    // 重复键 / 未知字段 / 缺必需字段 / 非法 UTF-8 / 超限。
    let dup = r#"{"protocolVersion":1,"protocolVersion":1,"appId":"a","release":"1","title":"t","entry":{"kind":"autodown","href":"./g.ad","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"assets":[],"services":[]}"#;
    for (bytes, needle) in [
        (dup.as_bytes(), "strict v1"),
        (
            br#"{"protocolVersion":1,"appId":"a","release":"1","title":"t","entry":{"kind":"autodown","href":"./g.ad","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"assets":[],"services":[],"extra":1}"#,
            "strict v1",
        ),
        (
            br#"{"protocolVersion":1,"appId":"a","release":"1"}"#,
            "strict v1",
        ),
        (b"\xff\xfe", "UTF-8"),
    ] {
        let refusal = parse_descriptor(bytes, &base).unwrap_err();
        assert!(refusal.reason.contains(needle), "{:?}", refusal.reason);
    }

    // 版本/入口类型/assets 非空/坏摘要/跨源入口。
    for (json, needle) in [
        (
            descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Guide").replacen(
                r#""protocolVersion":1"#,
                r#""protocolVersion":2"#,
                1,
            ),
            "protocolVersion 2",
        ),
        (
            descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Guide").replacen(
                r#""kind":"autodown""#,
                r#""kind":"autoui""#,
                1,
            ),
            "entry kind",
        ),
        (
            descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "Guide").replacen(
                r#""assets":[]"#,
                r#""assets":[{"href":"x"}]"#,
                1,
            ),
            "empty assets",
        ),
        (descriptor_json("./guide.ad", "nothex", "Guide"), "sha256"),
        (
            descriptor_json(
                "https://other.example/guide.ad",
                &doc_sha_hex(DOC_BODY),
                "Guide",
            ),
            "cross-origin",
        ),
        (
            descriptor_json(
                "http://demo.example/guide.ad",
                &doc_sha_hex(DOC_BODY),
                "Guide",
            ),
            "not https",
        ),
    ] {
        let refusal = parse_descriptor(json.as_bytes(), &base).unwrap_err();
        assert!(refusal.reason.contains(needle), "{:?}", refusal.reason);
        assert_eq!(refusal.stage, NavStage::DescriptorParse);
    }
}

// ---------------------------------------------------------------------------
// 媒体准入（V-03）
// ---------------------------------------------------------------------------

#[test]
fn p1119_media_admission_exact_with_charset_param() {
    admit_media(DESCRIPTOR_MEDIA, DESCRIPTOR_MEDIA, false).unwrap();
    admit_media("TEXT/VND.AUTODOWN", DOCUMENT_MEDIA, true).unwrap();
    admit_media("text/vnd.autodown; charset=utf-8", DOCUMENT_MEDIA, true).unwrap();
    assert!(
        admit_media("text/vnd.autodown; charset=latin1", DOCUMENT_MEDIA, true)
            .unwrap_err()
            .reason
            .contains("charset")
    );
    assert!(admit_media("text/vnd.autodown; x=y", DOCUMENT_MEDIA, true)
        .unwrap_err()
        .reason
        .contains("parameter"));
    assert!(admit_media("text/html", DESCRIPTOR_MEDIA, false)
        .unwrap_err()
        .reason
        .contains("HTML"));
    assert!(admit_media("application/json", DESCRIPTOR_MEDIA, false)
        .unwrap_err()
        .reason
        .contains("unexpected media type"));
}

// ---------------------------------------------------------------------------
// 只读准入（V-03 渲染前负控）
// ---------------------------------------------------------------------------

#[test]
fn p1119_readonly_admission_domain() {
    admit_readonly_document("# 标题\n\n正文 **加粗**、`code` 与 [链接](https://ok.example/a)。\n")
        .unwrap();
    admit_readonly_document("相对链接 [a](./x) 与锚 [b](#sec) 与 [c](autoweb://ok.example/)")
        .unwrap();
    for (body, needle) in [
        ("![图片](https://evil.example/x.png)", "image"),
        ("<script>alert(1)</script>", "HTML"),
        ("正文 <b>加粗</b>", "HTML"),
        ("$$x^2$$", "`$`"),
        ("$embed(src=https://evil.example/x)", "`$`"),
        ("$query(x)", "`$`"),
        ("[x](javascript:alert(1))", "whitelist"),
        ("[x](file:///C:/secret)", "whitelist"),
        ("[x](data:text/html,hi)", "whitelist"),
        // RF-3：词法洞回归——前导空格、实体编码、定义行。
        ("[x]( javascript:alert(1))", "whitelist"),
        ("[x](javascript&#58;alert(1))", "entity"),
        ("[ref]: javascript:alert(1)", "whitelist"),
        ("  [ref]: file:///C:/secret", "whitelist"),
    ] {
        let refusal = admit_readonly_document(body).unwrap_err();
        assert_eq!(refusal.stage, NavStage::DocumentAdmission);
        assert!(
            refusal.reason.contains(needle),
            "{body}: {:?}",
            refusal.reason
        );
    }
}

// ---------------------------------------------------------------------------
// 完整导航：协商 + 摘要 + 轨迹（V-01）
// ---------------------------------------------------------------------------

#[test]
fn p1119_nav_full_chain_sends_negotiated_accept_and_verifies_sha() {
    let mut transport = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "Gallery 指南", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    let mut cache = AutowebCache::new();
    let outcome = navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/gallery/guide/",
    );
    let NavOutcome::Success(success) = outcome else {
        panic!("expected success, got {outcome:?}")
    };
    assert_eq!(success.title, "Gallery 指南");
    assert_eq!(success.release, "1.0.0");
    assert_eq!(success.document, DOC_BODY);
    assert_eq!(success.document_sha256_hex, doc_sha_hex(DOC_BODY));
    assert_eq!(success.canonical_url, "https://demo.example/gallery/guide/");
    // 协商与请求面：描述/文档各自的 Accept；条件头缺席。
    assert_eq!(transport.log.len(), 2);
    assert_eq!(transport.log[0].accept, ACCEPT_DESCRIPTOR);
    assert_eq!(transport.log[1].accept, ACCEPT_DOCUMENT);
    assert!(transport.log.iter().all(|r| r.if_none_match.is_none()));
    assert_eq!(
        transport.log[1].url,
        "https://demo.example/gallery/guide/guide.ad"
    );
    assert_eq!(success.trace.requests.len(), 2);
}

#[test]
fn p1119_nav_document_sha_mismatch_refused() {
    let wrong_sha = "0".repeat(64);
    let mut transport = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./guide.ad", &wrong_sha, "Guide"),
            None,
        ),
        document_response(None),
    ]);
    let mut cache = AutowebCache::new();
    let (stage, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/guide/",
    ));
    assert_eq!(stage, NavStage::DocumentIntegrity);
    assert!(reason.contains("mismatch"), "{reason}");
    // 摘要不符的文档不入可复用提交（Success 从不产生）。
}

#[test]
fn p1119_nav_status_and_limit_refusals() {
    for (script, address, needle) in [
        (
            vec![ScriptedResponse {
                status: 404,
                headers: vec![],
                body: Vec::new(),
            }],
            "https://demo.example/missing/",
            "404",
        ),
        (
            vec![ScriptedResponse {
                status: 406,
                headers: vec![],
                body: Vec::new(),
            }],
            "https://demo.example/guide/",
            "406",
        ),
        (
            vec![ScriptedResponse {
                status: 500,
                headers: vec![],
                body: Vec::new(),
            }],
            "https://demo.example/guide/",
            "server error",
        ),
        (
            vec![ScriptedResponse::ok("text/html", "<html></html>", None)],
            "https://demo.example/guide/",
            "HTML",
        ),
        (
            vec![ScriptedResponse::ok("application/json", "{}", None)],
            "https://demo.example/guide/",
            "unexpected media type",
        ),
        (
            vec![ScriptedResponse::ok(
                DESCRIPTOR_MEDIA,
                &"x".repeat(65 * 1024),
                None,
            )],
            "https://demo.example/guide/",
            "exceeds limit",
        ),
    ] {
        let mut transport = ScriptTransport::new(script);
        let mut cache = AutowebCache::new();
        let (_, reason) = refused(navigate(&mut transport, &mut cache, address));
        assert!(reason.contains(needle), "{reason}");
    }
}

#[test]
fn p1119_nav_document_limit_and_admission_refusals() {
    // 描述合法，但文档体超限（脚本注入小限额——直接构造超长文档）。
    let big_body = "x".repeat(1024 * 1024 + 1);
    let mut transport = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./g.ad", &doc_sha_hex(&big_body), "G"),
            None,
        ),
        ScriptedResponse::ok(DOCUMENT_MEDIA, &big_body, None),
    ]);
    let mut cache = AutowebCache::new();
    let (stage, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/g/",
    ));
    assert_eq!(stage, NavStage::DocumentMedia);
    assert!(reason.contains("exceeds limit"), "{reason}");

    // 准入拒绝：文档含图片。
    let bad_doc = "# t\n\n![i](https://e.example/x.png)\n";
    let mut transport = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./g.ad", &doc_sha_hex(bad_doc), "G"),
            None,
        ),
        ScriptedResponse::ok(DOCUMENT_MEDIA, bad_doc, None),
    ]);
    let mut cache = AutowebCache::new();
    let (stage, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/g/",
    ));
    assert_eq!(stage, NavStage::DocumentAdmission);
    assert!(reason.contains("image"), "{reason}");
}

// ---------------------------------------------------------------------------
// 重定向策略（V-03：降级/跨源/超跳在下一次请求前拒）
// ---------------------------------------------------------------------------

fn redirect(location: &str) -> ScriptedResponse {
    ScriptedResponse {
        status: 302,
        headers: vec![("Location".to_string(), location.to_string())],
        body: Vec::new(),
    }
}

#[test]
fn p1119_nav_same_origin_redirect_followed_cross_origin_refused_before_request() {
    // 同源跳一次 → 成功。
    let mut transport = ScriptTransport::new(vec![
        redirect("/final/"),
        descriptor_response("./guide.ad", "G", None),
        document_response(None),
    ]);
    let mut cache = AutowebCache::new();
    assert!(matches!(
        navigate(&mut transport, &mut cache, "https://demo.example/start/"),
        NavOutcome::Success(_)
    ));
    assert_eq!(transport.log[1].url, "https://demo.example/final/");

    // 跨源重定向：只发 1 个请求（对 evil 的请求从未发出）。
    let mut transport = ScriptTransport::new(vec![redirect("https://evil.example/guide/")]);
    let mut cache = AutowebCache::new();
    let (stage, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/s/",
    ));
    assert_eq!(stage, NavStage::DescriptorFetch);
    assert!(reason.contains("cross-origin"), "{reason}");
    assert_eq!(transport.log.len(), 1);

    // 降级重定向（http）。
    let mut transport = ScriptTransport::new(vec![redirect("http://demo.example/guide/")]);
    let mut cache = AutowebCache::new();
    let (_, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/s/",
    ));
    assert!(reason.contains("https"), "{reason}");
    assert_eq!(transport.log.len(), 1);

    // 跳数上限：3 跳允许、第 4 跳拒。
    let mut transport = ScriptTransport::new(vec![
        redirect("/a/"),
        redirect("/b/"),
        redirect("/c/"),
        descriptor_response("./guide.ad", "G", None),
        document_response(None),
    ]);
    let mut cache = AutowebCache::new();
    assert!(matches!(
        navigate(&mut transport, &mut cache, "https://demo.example/loop/"),
        NavOutcome::Success(_)
    ));
    let mut transport = ScriptTransport::new(vec![
        redirect("/a/"),
        redirect("/b/"),
        redirect("/c/"),
        redirect("/d/"),
    ]);
    let mut cache = AutowebCache::new();
    let (_, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/loop/",
    ));
    assert!(reason.contains("hops"), "{reason}");
}

// ---------------------------------------------------------------------------
// 缓存与条件验证（V-02）
// ---------------------------------------------------------------------------

#[test]
fn p1119_cache_revalidation_reuses_304_without_body_redownload() {
    let url = "https://demo.example/guide/";
    let mut cache = AutowebCache::new();
    // 首访：描述+文档全量。
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, url),
        NavOutcome::Success(_)
    ));
    // 再访：两个资源都带 If-None-Match，服务器 304 → 复用，无正文下载。
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse {
            status: 304,
            headers: vec![("ETag".to_string(), "\"d1\"".to_string())],
            body: Vec::new(),
        },
        ScriptedResponse {
            status: 304,
            headers: vec![("ETag".to_string(), "\"b1\"".to_string())],
            body: Vec::new(),
        },
    ]);
    let NavOutcome::Success(success) = navigate(&mut second, &mut cache, url) else {
        panic!("expected success on revalidation")
    };
    assert_eq!(success.document, DOC_BODY);
    assert_eq!(second.log[0].if_none_match.as_deref(), Some("\"d1\""));
    assert_eq!(second.log[1].if_none_match.as_deref(), Some("\"b1\""));
    assert_eq!(success.trace.descriptor_bytes_downloaded, 0);
    assert_eq!(success.trace.document_bytes_downloaded, 0);
    assert!(success.trace.cache_reused);
    assert_eq!(cache.reuse_hits, 2);
    assert_eq!(cache.revalidations, 2);
}

/// RF-2 回归：`304` 复用必须通过身份（ETag）与摘要（当前描述声明）双
/// 复验——描述/文档发布偏斜时拒绝复用。
#[test]
fn p1119_cache_304_reuse_revalidates_identity_and_digest() {
    let url = "https://demo.example/guide/";
    // RF-2-b：304 自身 ETag 与缓存条目不符 → 复用拒（fetch 阶段）。
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, url),
        NavOutcome::Success(_)
    ));
    let mut second = ScriptTransport::new(vec![
        not_modified(Some("\"d1\"")),
        ScriptedResponse {
            status: 304,
            headers: vec![("ETag".to_string(), "\"unrelated-tag\"".to_string())],
            body: Vec::new(),
        },
    ]);
    let (stage, reason) = refused(navigate(&mut second, &mut cache, url));
    assert_eq!(stage, NavStage::DocumentFetch, "{reason}");
    assert!(reason.contains("does not identify"), "{reason}");

    // RF-2-a：描述更新声明新正文摘要，文档却 304 复用旧字节 → 摘要复验拒。
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, url),
        NavOutcome::Success(_)
    ));
    let new_doc = "# v2 body\n";
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./guide.ad", &doc_sha_hex(new_doc), "G2"),
            Some("\"d2\""),
        ),
        not_modified(Some("\"b1\"")),
    ]);
    let (stage, reason) = refused(navigate(&mut second, &mut cache, url));
    assert_eq!(stage, NavStage::DocumentIntegrity, "{reason}");
    assert!(reason.contains("revalidated"), "{reason}");
}

#[test]
fn p1119_cache_new_version_replaces_after_full_validation() {
    let url = "https://demo.example/guide/";
    let new_doc = "# 新版\n\n更新后的正文。\n";
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, url),
        NavOutcome::Success(_)
    ));
    // 版本更新：描述 200 新 ETag+新摘要，文档 200 新正文 → 新内容提交。
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./guide.ad", &doc_sha_hex(new_doc), "G2"),
            Some("\"d2\""),
        ),
        ScriptedResponse::ok(DOCUMENT_MEDIA, new_doc, Some("\"b2\"")),
    ]);
    let NavOutcome::Success(success) = navigate(&mut second, &mut cache, url) else {
        panic!("expected success")
    };
    assert_eq!(success.document, new_doc);
    assert_eq!(success.document_sha256_hex, doc_sha_hex(new_doc));
}

#[test]
fn p1119_cache_bad_update_refused_old_entry_retained() {
    let url = "https://demo.example/guide/";
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, url),
        NavOutcome::Success(_)
    ));
    // 坏更新：描述改了（新 ETag），但文档字节与描述摘要不符 → 整次拒；
    // 旧缓存条目保留（描述旧条目被新 200 替换，文档旧条目保留）。
    let tampered = "# 被篡改的内容\n";
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "G"),
            Some("\"d2\""),
        ),
        ScriptedResponse::ok(DOCUMENT_MEDIA, tampered, Some("\"b2\"")),
    ]);
    let (stage, _) = refused(navigate(&mut second, &mut cache, url));
    assert_eq!(stage, NavStage::DocumentIntegrity);
    let key = CacheKey {
        url: format!("{url}guide.ad"),
        representation: Representation::Document,
    };
    assert_eq!(cache.entry(&key).unwrap().body, DOC_BODY.as_bytes());
    // 旧提交的保留由外壳层负责（提交只在 Success 时发生）。
}

#[test]
fn p1119_cache_no_store_bypasses_cache_entirely() {
    let url = "https://demo.example/nostore/";
    let mut cache = AutowebCache::new();
    for _ in 0..2 {
        let mut transport = ScriptTransport::new(vec![
            ScriptedResponse {
                status: 200,
                headers: vec![
                    ("Content-Type".to_string(), DESCRIPTOR_MEDIA.to_string()),
                    ("Cache-Control".to_string(), "no-store".to_string()),
                ],
                body: descriptor_json("./guide.ad", &doc_sha_hex(DOC_BODY), "G").into_bytes(),
            },
            document_response(Some("\"b\"")),
        ]);
        assert!(matches!(
            navigate(&mut transport, &mut cache, url),
            NavOutcome::Success(_)
        ));
        // no-store 的描述从不入缓存：描述请求永不带条件头。
        let descriptor_log = transport
            .log
            .iter()
            .find(|r| r.accept == ACCEPT_DESCRIPTOR)
            .unwrap();
        assert!(descriptor_log.if_none_match.is_none());
    }
    assert!(cache
        .entry(&CacheKey {
            url: url.to_string(),
            representation: Representation::Descriptor
        })
        .is_none());
}

#[test]
fn p1119_cache_mismatched_304_refused() {
    let mut cache = AutowebCache::new();
    // 冷缓存收到 304（无先条目、无 If-None-Match）→ 拒。
    let mut transport = ScriptTransport::new(vec![ScriptedResponse {
        status: 304,
        headers: vec![],
        body: Vec::new(),
    }]);
    let (stage, reason) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/g/",
    ));
    assert_eq!(stage, NavStage::DescriptorFetch);
    assert!(reason.contains("304"), "{reason}");

    // 首访无 ETag 条目 → 再访不发条件头；此时服务器仍回 304 → 拒。
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", None),
        document_response(None),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, "https://demo.example/g/"),
        NavOutcome::Success(_)
    ));
    let mut second = ScriptTransport::new(vec![ScriptedResponse {
        status: 304,
        headers: vec![],
        body: Vec::new(),
    }]);
    let (_, reason) = refused(navigate(&mut second, &mut cache, "https://demo.example/g/"));
    assert!(reason.contains("304"), "{reason}");
}

#[test]
fn p1119_cache_vary_accept_separates_representations() {
    // 同一 URL 两种 Accept（描述/文档）→ 两个独立条目互不串。
    let url = "https://demo.example/dual";
    let mut cache = AutowebCache::new();
    let descriptor_key = CacheKey {
        url: url.to_string(),
        representation: Representation::Descriptor,
    };
    let document_key = CacheKey {
        url: url.to_string(),
        representation: Representation::Document,
    };
    let body = DOC_BODY;
    let mut transport = ScriptTransport::new(vec![
        ScriptedResponse::ok(
            DESCRIPTOR_MEDIA,
            &descriptor_json(url, &doc_sha_hex(body), "G"),
            Some("\"desc\""),
        ),
        // 文档请求打到同一 URL（描述 href 指回自身）。
        ScriptedResponse::ok(DOCUMENT_MEDIA, body, Some("\"doc\"")),
    ]);
    let parsed = parse_web_address(url).unwrap();
    let mut navigator = ReadonlyNavigator::new(&mut transport, &mut cache, NavLimits::default());
    assert!(matches!(
        navigator.navigate(&parsed),
        NavOutcome::Success(_)
    ));
    assert_eq!(
        cache.entry(&descriptor_key).unwrap().body,
        descriptor_json(url, &doc_sha_hex(body), "G").as_bytes()
    );
    assert_eq!(cache.entry(&document_key).unwrap().body, body.as_bytes());
    // 再访：各自的条件头互不串（描述带 "desc"，文档带 "doc"）。
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse {
            status: 304,
            headers: vec![],
            body: Vec::new(),
        },
        ScriptedResponse {
            status: 304,
            headers: vec![],
            body: Vec::new(),
        },
    ]);
    let mut navigator = ReadonlyNavigator::new(&mut second, &mut cache, NavLimits::default());
    assert!(matches!(
        navigator.navigate(&parsed),
        NavOutcome::Success(_)
    ));
    assert_eq!(second.log[0].if_none_match.as_deref(), Some("\"desc\""));
    assert_eq!(second.log[1].if_none_match.as_deref(), Some("\"doc\""));
}

// ---------------------------------------------------------------------------
// 拼写共享缓存（V-01：两种写法共用 https origin）
// ---------------------------------------------------------------------------

#[test]
fn p1119_alias_spelling_shares_https_cache() {
    let mut cache = AutowebCache::new();
    let mut first = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", Some("\"d1\"")),
        document_response(Some("\"b1\"")),
    ]);
    assert!(matches!(
        navigate(&mut first, &mut cache, "https://demo.example/guide/"),
        NavOutcome::Success(_)
    ));
    // 同一资源换原生专用写法：命中同一缓存（描述+文档 304 复用，无新下载）。
    let mut second = ScriptTransport::new(vec![
        ScriptedResponse {
            status: 304,
            headers: vec![],
            body: Vec::new(),
        },
        ScriptedResponse {
            status: 304,
            headers: vec![],
            body: Vec::new(),
        },
    ]);
    let NavOutcome::Success(success) =
        navigate(&mut second, &mut cache, "autoweb://demo.example/guide/")
    else {
        panic!("expected success")
    };
    assert_eq!(success.display_address, "autoweb://demo.example/guide/");
    assert_eq!(success.canonical_url, "https://demo.example/guide/");
    assert_eq!(success.trace.document_bytes_downloaded, 0);
}

// ---------------------------------------------------------------------------
// 取消与期限
// ---------------------------------------------------------------------------

#[test]
fn p1119_nav_cancellation_mid_chain() {
    let mut cache = AutowebCache::new();
    let mut transport = ScriptTransport::new(vec![
        descriptor_response("./guide.ad", "G", None),
        document_response(None),
    ]);
    transport.cancel_after = Some(1); // 描述完成后取消。
    let (stage, _) = refused(navigate(
        &mut transport,
        &mut cache,
        "https://demo.example/g/",
    ));
    assert_eq!(stage, NavStage::Cancelled);
    assert_eq!(transport.log.len(), 1, "文档请求从未发出");
}

#[test]
fn p1119_nav_deadline_enforced_between_hops() {
    let mut cache = AutowebCache::new();
    let mut transport = ScriptTransport::new(vec![
        redirect("/a/"),
        redirect("/b/"),
        redirect("/c/"),
        redirect("/d/"),
        redirect("/e/"),
        redirect("/f/"),
    ]);
    // 慢首跳（60ms）+ 3ms 总期限：首请求完成后下一跳前到期拒。
    transport.delays_ms[0] = 60;
    let parsed = parse_web_address("https://demo.example/slow/").unwrap();
    let mut limits = NavLimits::default();
    limits.nav_deadline_ms = 3;
    let mut navigator = ReadonlyNavigator::new(&mut transport, &mut cache, limits);
    let (stage, _) = match navigator.navigate(&parsed) {
        NavOutcome::Refused(refusal, _) => (refusal.stage, refusal.reason.clone()),
        NavOutcome::Success(_) => panic!("expected deadline refusal"),
    };
    assert_eq!(stage, NavStage::Deadline);
    assert_eq!(transport.log.len(), 1, "期限在下一跳前拒绝");
}

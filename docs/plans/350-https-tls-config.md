# Plan 350: HTTPS 证书配置

> **状态**：设计文档 / TODO
> **优先级**：🔴 高
> **难度**：低
> **依赖**：无

## 背景

reqwest 默认通过 native-tls 支持 HTTPS，`https://` URL 已经能工作。但缺少：
- 自定义 CA 证书（企业内网、自签名证书）
- 跳过证书验证（开发/测试环境）
- 客户端证书（mTLS 双向认证）

## 方案

### 新增 native（Client 侧）

#### `RequestBuilder.tls_ca_cert(path: String) -> RequestBuilder`
加载 PEM 格式的自定义 CA 证书。用于企业内网、自签名证书场景。
```rust
// shim: 读 PEM 文件，加入 reqwest ClientBuilder 的 add_root_certificate
```

#### `RequestBuilder.tls_skip_verify(skip: bool) -> RequestBuilder`
跳过 TLS 证书验证。开发/测试环境用。
```rust
// shim: reqwest ClientBuilder.danger_accept_invalid_certs(skip)
```

#### `RequestBuilder.tls_client_cert(cert_path: String, key_path: String) -> RequestBuilder`
设置客户端证书（mTLS）。用于需要双向认证的 API。
```rust
// shim: reqwest ClientBuilder.identity(pkcs12/der)
```

### 实现要点

1. **RequestBuilder 扩展**：`HttpRequestBuilderData` 增加 `ca_cert: Option<PathBuf>`、`skip_verify: bool`、`client_cert: Option<(PathBuf, PathBuf)>` 字段。
2. **send 时构建 Client**：`request_builder_send` 当前每次创建 `reqwest::blocking::Client::new()`。改为在创建 Client 时检查这些字段，用 `ClientBuilder` 配置 TLS。
3. **注册 native**：`register_shim_by_name` 注册三个新方法。

### 关键文件
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — `HttpRequestBuilderData` + 三个新 shim
- `crates/auto-lang/src/vm/native_catalog.rs` — 注册新 native ID

### 用法示例
```auto
let resp = http.request("GET", "https://internal-api.corp.local/data")
    .tls_ca_cert("/etc/ssl/corp-ca.pem")
    .send()
```

## 不在范围
- Server 侧 TLS（AutoVM server 目前用裸 TCP，不支持 TLS；生产环境用反向代理）
- 证书引脚（Certificate Pinning）
- OCSP 装订

# Plan 351: 文件上传（multipart/form-data）

> **状态**：设计文档 / TODO
> **优先级**：🟡 中
> **难度**：中
> **依赖**：无

## 背景

当前 HTTP client 只能发送 JSON body。文件上传（图片、文档、CSV 导入等）需要 multipart/form-data 格式。reqwest 已有 `.multipart()` 支持，只需暴露为 native。

## 方案

### 新增 native（Client 侧）

#### `http.upload(url: String, file_path: String) -> Response`
最简单的单文件上传。自动用 multipart/form-data，文件字段名为 `file`。
```rust
// shim: reqwest multipart::Form::new().file("file", path)
```

#### `http.upload_with_fields(url: String, file_path: String, fields_json: String) -> Response`
文件 + 表单字段上传。`fields_json` 是 JSON 对象，如 `{"name": "avatar", "type": "image"}`。
```rust
// shim: 先从 JSON 解析字段，逐个 .text() 加入 Form，再加 .file()
```

#### `RequestBuilder.multipart_file(field_name: String, file_path: String) -> RequestBuilder`
链式 API：在 RequestBuilder 上附加文件。
```rust
// shim: HttpRequestBuilderData 增加 multipart_files: Vec<(String, PathBuf)>
// send 时构造 multipart::Form
```

#### `RequestBuilder.multipart_text(field_name: String, value: String) -> RequestBuilder`
链式 API：在 multipart 表单中附加文本字段。

### 实现要点

1. **HttpRequestBuilderData 扩展**：增加 `multipart_files: Vec<(String, PathBuf)>` 和 `multipart_texts: Vec<(String, String)>`。
2. **send 时检测**：如果有 multipart 数据，用 `reqwest::blocking::multipart::Form` 构造 body，而非 `.body()`/`.json()`。
3. **Content-Type**：multipart 自动设置 `Content-Type: multipart/form-data; boundary=...`，不需要手动设。
4. **错误处理**：文件不存在时返回错误 Response。

### 关键文件
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — 4 个新 shim + HttpRequestBuilderData 扩展
- `crates/auto-lang/src/vm/native_catalog.rs` — 注册新 native ID

### 用法示例
```auto
// 简单上传
let resp = http.upload("https://api.example.com/upload", "/path/to/file.png")

// 带字段上传
let resp = http.upload_with_fields(
    "https://api.example.com/upload",
    "/path/to/avatar.png",
    "{\"user_id\": \"123\", \"type\": \"avatar\"}"
)

// 链式 API
let resp = http.request("POST", "https://api.example.com/upload")
    .header("Authorization", "Bearer xxx")
    .multipart_file("file", "/path/to/file.png")
    .multipart_text("description", "My profile photo")
    .send()
```

## 不在范围
- 流式上传（大文件分块上传）—— 需要更复杂的 chunked transfer
- 多文件同时上传（可链式 `.multipart_file()` 多次实现，但当前设计支持）
- 上传进度回调 —— 见 Plan 352

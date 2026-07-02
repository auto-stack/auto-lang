# Plan 352: 大文件下载 + 断点续传 + 进度回调

> **状态**：设计文档 / TODO
> **优先级**：🟡 中
> **难度**：中
> **依赖**：Plan 348（非阻塞 yield 机制复用）

## 背景

当前 `http_stream.get_stream` 能分块读取响应体，但缺少：
- 断点续传（HTTP Range header，从中断处继续下载）
- 下载进度回调（用于 UI 进度条）
- 直接写入文件（避免内存中拼接大量数据）

## 方案

### 新增 native（Client 侧）

#### `http.download(url: String, file_path: String) -> bool`
最简单的下载到文件。阻塞式，内部用 `reqwest::blocking` 流式写入文件。
```rust
// shim: reqwest GET → response.bytes_stream() → 逐 chunk 写入 File
// 返回 true 成功 / false 失败
```

#### `http.download_resume(url: String, file_path: String, offset: i64) -> bool`
断点续传下载。发送 `Range: bytes={offset}-` header，从已有文件的 offset 位置追加。
```rust
// shim: 检查文件大小确定 offset，或用传入的 offset
// 打开文件 append 模式，流式写入
```

#### `http.download_with_progress(url: String, file_path: String) -> iterator_id`
带进度的下载。返回迭代器，每次 yield 一个进度事件（JSON `{"downloaded": 1024, "total": 4096, "percent": 25}`）。
```rust
// shim: 独立线程跑下载（复用 Plan 341/348 的独立线程 + channel 模式）
// 每个 chunk 写入文件后，经 channel 推送进度事件
// shim_iterator_next 用 Plan 348 的非阻塞 yield 机制
```

用法：
```auto
for progress in http.download_with_progress(url, "/path/to/file") {
    print("Downloaded: " + progress + "%")
}
```

### 实现要点

1. **断点续传**：HTTP 1.1 的 `Range: bytes=START-` header。服务器返回 `206 Partial Content`。需要检查 `Content-Range` header 确认服务器支持。
2. **进度计算**：从 `Content-Length` header 获取总大小，累加已下载 chunk 大小。
3. **文件写入**：用 `std::fs::File` + `write_all` 逐 chunk 写入，避免在内存中缓存整个文件。
4. **非阻塞进度**：复用 Plan 348 的 `AsyncHttpStream` + `waiting_sse_stream_id` 机制——下载线程推送进度事件到 channel，`shim_iterator_next` 非阻塞 yield。

### 进度迭代器复用 Plan 348 机制

```
http.download_with_progress(url, path)
  → spawn 独立线程：reqwest GET → 流式读 chunk → 写文件 → channel 推 {downloaded, total, percent}
  → 创建 AsyncHttpStream 迭代器（同 sse_get_stream）
  → shim_iterator_next 非阻塞 yield（Plan 348）
```

### 关键文件
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — 3 个新 shim + AsyncStreamEvent 扩展
- `crates/auto-lang/src/vm/native_catalog.rs` — 注册新 native ID

### 用法示例
```auto
// 简单下载
http.download("https://example.com/large-file.zip", "/downloads/file.zip")

// 断点续传（从已下载的 1024 字节处继续）
http.download_resume("https://example.com/large-file.zip", "/downloads/file.zip", 1024)

// 带进度条
for progress in http.download_with_progress("https://example.com/large-file.zip", "/downloads/file.zip") {
    let p = json.to_value(progress)
    print("Downloaded: " + p["percent"].to_string() + "%")
}
```

## 不在范围
- 多线程并发下载（分段 Range + 多线程合并）—— 复杂度高
- 上传进度回调 —— 原理类似，可后续加
- 下载速度限制（throttle）
- 校验和验证（MD5/SHA256）

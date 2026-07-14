# Auto 标准库 IO/FS 模块设计

> **状态**：调研与设计文档
> **对比**：Rust std::fs / std::io、Python pathlib / io、Go os / bufio

## 一、现有能力盘点

### File 操作（`auto.file.*` / `File.*`）

| Native | 说明 | 异步？ | 流式？ |
|--------|------|:---:|:---:|
| `File.read_text(path)` | 读全部文本 | ❌ | ❌ |
| `File.write_text(path, content)` | 写文本 | ❌ | ❌ |
| `File.append_text(path, content)` | 追加文本 | ❌ | ❌ |
| `File.read_bytes(path)` | 读全部字节 | ❌ | ❌ |
| `File.write_bytes(path, bytes)` | 写字节 | ❌ | ❌ |
| `File.exists(path)` | 文件存在 | — | — |
| `File.delete(path)` | 删除文件 | — | — |
| `File.copy(src, dst)` | 复制文件 | — | — |
| `File.size(path)` | 文件大小 | — | — |
| `File.is_dir(path)` | 是否目录 | — | — |
| `File.create_dir(path)` | 创建目录 | — | — |
| `File.walk(path)` | 递归遍历（返回 JSON 字符串） | — | — |
| `File.read_lines(path)` | 读所有行（返回 JSON 数组字符串） | ❌ | ❌ |

### FS 路径操作（`auto.fs.*`）

| Native | 说明 |
|--------|------|
| `auto.fs.temp_dir()` | 临时目录 |
| `auto.fs.temp_file()` | 临时文件 |
| `auto.fs.rename(from, to)` | 重命名/移动 |
| `auto.fs.read_dir(path)` | 列目录 |
| `auto.fs.canonical(path)` | 规范化路径 |
| `auto.fs.ext(path)` | 扩展名 |
| `auto.fs.stem(path)` | 文件名（无扩展名） |
| `auto.fs.walk_files(path)` | 递归列出文件 |
| `auto.fs.walk(path)` | 递归遍历（含目录） |
| `auto.fs.metadata(path)` | 元信息（大小/修改时间/类型） |
| `auto.fs.copy_recursive(src, dst)` | 递归复制 |
| `auto.fs.filename(path)` | 文件名 |
| `auto.fs.parent(path)` | 父目录 |
| `auto.fs.join(a, b)` | 路径拼接 |

### 文件句柄（`auto.file.create_handle` / `open_handle`）

| Native | 说明 |
|--------|------|
| `auto.file.create_handle(path)` | 创建文件（返回句柄） |
| `auto.file.open_handle(path)` | 打开文件（返回句柄） |
| `auto.file.write_handle(handle, data)` | 写句柄 |
| `auto.file.try_clone(handle)` | 克隆句柄 |
| `auto.file.read_handle(handle)` | 读句柄（隐式，通过 RustStdlibObject 分发） |

### IO（`auto.io.*`）

| Native | 说明 |
|--------|------|
| `io.read_line()` | 从 stdin 读一行（阻塞） |

### Path 操作（`auto.path.*`）

| Native | 说明 |
|--------|------|
| `path.parent(path)` | 父目录 |
| `path.extension(path)` | 扩展名 |
| `path.filename(path)` | 文件名 |

### TCP（`auto.net.*`）

| Native | 说明 |
|--------|------|
| `net.tcp_bind` / `tcp_connect` / `tcp_stream_read` / `tcp_stream_write` | 原始 TCP |

## 二、与 Rust/Python/Go 对比

### Rust std::fs + std::io

| 能力 | Rust | Auto | 差距 |
|------|------|------|------|
| 一次性读写 | `fs::read_to_string` / `fs::write` | ✅ `read_text` / `write_text` | — |
| 字节读写 | `fs::read` / `fs::write` | ✅ `read_bytes` / `write_bytes` | — |
| 按行读 | `BufReader::lines()` → iterator | ❌ `read_lines` 返回全量 JSON | **缺流式迭代** |
| 带缓冲读写 | `BufReader` / `BufWriter` | ❌ | **缺缓冲层** |
| Seek | `Seek::seek` / `seek_from` | ❌ | **缺随机定位** |
| 文件追加 | `OpenOptions::append` | ✅ `append_text` | — |
| 异步 IO | `tokio::fs` / `tokio::io` | ❌ | **缺异步** |
| 读目录迭代器 | `fs::read_dir()` → iterator | ❌ `read_dir` 返回 JSON | **缺迭代器** |
| 文件锁 | `Lock` / `flock` | ❌ | 缺（低优先级） |
| 文件监听 | `notify` crate | ❌ | 缺（低优先级） |

### Python pathlib + io

| 能力 | Python | Auto | 差距 |
|------|--------|------|------|
| Path 对象（链式操作） | `Path("/a/b").parent / "c.txt"` | ❌ 只有分离函数 | **缺 Path 对象** |
| 上下文管理 | `with open(...) as f:` | ❌ | **缺资源管理** |
| 按行迭代 | `for line in f:` | ❌ | **缺流式迭代** |
| StringIO/BytesIO | 内存流 | ❌ | 缺（低优先级） |
| tempfile | `tempfile.NamedTemporaryFile` | ✅ `fs.temp_file` | — |

### Go os + bufio

| 能力 | Go | Auto | 差距 |
|------|-----|------|------|
| Scanner | `bufio.NewScanner(f)` → 按行/token | ❌ | **缺 Scanner** |
| 带缓冲读写 | `bufio.Reader` / `Writer` | ❌ | **缺缓冲层** |
| 文件信息 | `os.Stat()` → `FileInfo` | ✅ `fs.metadata` | — |

## 三、缺失能力分析

### 🔴 核心缺失

| # | 能力 | 说明 | 场景 |
|---|------|------|------|
| 1 | **按行流式读取** | `for line in file.lines(path)` | 大文件逐行处理（日志分析、CSV） |
| 2 | **异步文件 IO** | `await file.read_text(path)` | 不冻结 UI 的大文件读写 |
| 3 | **带缓冲读写** | `BufReader` / `BufWriter` | 高效逐块读写 |
| 4 | **Seek/随机定位** | `file.seek(offset)` / `file.position()` | 二进制文件随机访问 |
| 5 | **read_dir 迭代器** | `for entry in fs.read_dir(path)` | 遍历大目录不爆内存 |
| 6 | **Path 对象** | `Path("/a/b").join("c.txt").exists()` | 链式路径操作 |

### 🟡 中等缺失

| # | 能力 | 说明 |
|---|------|------|
| 7 | **Scanner（分词读取）** | 按分隔符/token 读取（CSV、空格分隔） |
| 8 | **文件监听（watch）** | 目录变化通知 |
| 9 | **stdin 非阻塞读取** | 交互式终端输入 |
| 10 | **标准错误输出** | `eprint` / `eprintln` |
| 11 | **文件权限** | `chmod` / `metadata.permissions` |
| 12 | **符号链接** | `symlink` / `readlink` |

### 🟢 低优先级

| # | 能力 |
|---|------|
| 13 | 内存流（StringIO/BytesIO） |
| 14 | 文件锁（flock） |
| 15 | 管道（pipe） |
| 16 | 内存映射（mmap） |

## 四、设计方案

### 设计原则

1. **分层**：底层同步原语 → 缓冲层 → 异步层 → 迭代器层
2. **渐进式**：不破坏现有 `File.read_text` 等 API
3. **复用**：异步层复用 Plan 348 的非阻塞 yield 机制
4. **对称**：读和写 API 对称设计

### 模块结构

```
auto.file.*      — 一次性操作（现有，保留）
auto.fs.*        — 文件系统操作（现有，保留）
auto.io.*        — 流式 IO（新增）
auto.path.*      — Path 对象（增强）
```

### 4.1 流式按行读取（最高优先级）

**API**：
```auto
// 按行迭代（复用 Plan 348 的 AsyncHttpStream + 非阻塞 yield 机制）
for line in io.lines("/path/to/large.log") {
    print(line)
}

// 带缓冲的块读取
for chunk in io.chunks("/path/to/file.bin", 4096) {
    process(chunk)
}
```

**实现**：
- `io.lines(path)` native：spawn 独立线程打开文件，用 `BufReader::lines()` 逐行读，经 channel 推送
- `io.chunks(path, size)` native：spawn 线程用 `read_exact` 逐块读
- 复用 `AsyncHttpStream` 迭代器 + `waiting_sse_stream_id` 非阻塞 yield

### 4.2 异步文件 IO

**API**：
```auto
// 异步读取（不冻结 UI）
let content = await io.read_text_async("/path/to/file")
let bytes = await io.read_bytes_async("/path/to/file")
io.write_text_async("/path/to/file", content)
```

**实现**：
- 复用 Plan 349 step 7 的异步 HTTP 模式（`ASYNC_HTTP_RESULTS` → `ASYNC_IO_RESULTS`）
- spawn 线程读写文件，完成后存入结果表
- `run_task_loop` 加第六唤醒源（IO 结果就绪）

### 4.3 缓冲读写

**API**：
```auto
let writer = io.buffered_writer("/path/to/output.txt")
writer.write_line("Hello")
writer.write_line("World")
writer.flush()
writer.close()

let reader = io.buffered_reader("/path/to/input.txt")
let line = reader.read_line()
```

**实现**：
- 句柄存入 `heap_objects`（RustStdlibObject 包装 `std::io::BufWriter<File>`）
- `write_line` / `write` / `flush` / `close` 方法通过 CALL_SPEC 分发

### 4.4 Seek / 随机定位

**API**：
```auto
let f = io.open("/path/to/file.bin")
f.seek(1024)           // 定位到 1024 字节
let bytes = f.read(100) // 读 100 字节
let pos = f.position()  // 当前位置
f.close()
```

**实现**：
- `io.open(path)` 返回 File 句柄（`std::fs::File` 包装）
- `seek(offset)` / `read(n)` / `write(bytes)` / `position()` / `close()` 方法

### 4.5 read_dir 迭代器

**API**：
```auto
for entry in fs.entries("/path/to/dir") {
    print(entry.name + " " + entry.is_dir.to_string())
}
```

**实现**：
- `fs.entries(path)` 返回迭代器，每次 yield 一个 JSON 对象 `{"name":"x","is_dir":false,"size":123}`
- 复用 AsyncHttpStream + channel 模式

### 4.6 Path 对象

**API**：
```auto
let p = Path.new("/usr/local/bin")
let parent = p.parent()           // "/usr/local"
let name = p.filename()           // "bin"
let joined = p.join("app")        // "/usr/local/bin/app"
let ext = p.extension()           // "" (无扩展名)
if joined.exists() {
    print("found")
}
```

**实现**：
- `Path.new(path)` 返回堆对象（类似 RequestBuilder）
- 方法链式调用，返回新 Path 或基本类型
- `exists()` / `is_file()` / `is_dir()` / `size()` 方法

### 4.7 Scanner（分词读取）

**API**：
```auto
let scanner = io.scanner("/path/to/data.csv")
for token in scanner.scan(",") {
    print(token)
}
scanner.close()
```

**实现**：
- 基于 BufReader + 分隔符切割
- `scan(delimiter)` 返回下一个 token

### 4.8 标准错误输出

**API**：
```auto
eprint("error message")
eprintln("error with newline")
```

**实现**：
- 直接写 `std::io::stderr()`

## 五、实施路线

| 阶段 | 能力 | 难度 | 优先级 |
|------|------|------|--------|
| 1 | 按行流式读取 (`io.lines`) | 中 | 🔴 高 |
| 2 | 标准错误输出 (`eprint`/`eprintln`) | 低 | 🔴 高 |
| 3 | read_dir 迭代器 (`fs.entries`) | 中 | 🟡 中 |
| 4 | 缓冲读写 (`io.buffered_writer/reader`) | 中 | 🟡 中 |
| 5 | Seek / 随机访问 (`io.open/seek/read`) | 中 | 🟡 中 |
| 6 | 异步文件 IO (`io.read_text_async`) | 中 | 🟡 中 |
| 7 | Path 对象 (`Path.new().join().exists()`) | 高 | 🟢 低 |
| 8 | Scanner (`io.scanner`) | 低 | 🟢 低 |

## 六、与现有 API 的关系

- **现有 `File.read_text` 等保留**，不废弃
- **新增 `io.*` 为流式/异步/缓冲层**
- **现有 `fs.*` 路径操作保留**，`Path` 对象为上层封装
- **现有 `auto.net.tcp_*` 保留**，`io.*` 不涉及网络层

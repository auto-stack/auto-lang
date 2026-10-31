# auto-vm / zip-natives（PLAN-897）

状态：delivered（随 PLAN-897 交付）· 日期：2026-10-31
消费方：auto-reader READER-003（最小 EPUB）及一切需要只读 zip 容器的 .at 程序。

## 能力表

| FFI 名 | .at 形态 | 成功 | 失败（全部为归一返回，无 raise） |
|---|---|---|---|
| `zip.list_json` | `zip.list_json(path str) str` | 中央目录顺序 JSON 数组 `[{"name":str,"size":int,"comp_size":int,"method":string},...]`；`method` 为 zip 3.0 `CompressionMethod` 的 Debug 名（如 `"Deflated"`/`"Stored"`） | `""`（缺文件/坏档案/条目读失败） |
| `zip.read_text` | `zip.read_text(path str, name str, max_bytes int) str` | 条目解压后**严格 UTF-8** 文本 | `""`（缺文件/缺条目/非 UTF-8/**超 max_bytes**/IO/`max_bytes ≤ 0`） |
| `zip.extract_entry` | `zip.extract_entry(path str, name str, dest str, max_bytes int) int` | `0`（dest 截断写单文件） | `-1` 一般失败（含 `max_bytes ≤ 0`）；`-2` 解压字节超 `max_bytes` |

## 语义约定

1. **预算硬闸**：`max_bytes ≤ 0` 非法；读侧 `take(max_bytes)` + 截断探测——
   解压字节超过上限即失败（-2/""），**不信任 ZIP 头声明的解压大小**。
   恰好等于上限 = 成功。
2. **条目名**：以 ZIP manifest 原名精确匹配（`by_name`），不做 unicode
   归一、不做路径规范化。列名顺序 = 中央目录顺序。
3. **写侧边界**：`extract_entry` 只写显式 `dest` 单文件（截断语义），
   不做目录重组、不从条目名物化路径（zip-slip 面不存在于本 native；
   条目名准入是调用方职责）。
4. **UTF-8 严格**：`read_text` 非法 UTF-8 字节序列 → `""`；二进制条目
   请用 `extract_entry`。
5. **递归嵌套 zip 不解析**：内嵌 zip 对本 native 是普通条目。

## 非目标

zip 写侧、CFI/EPUB 语义、a2r/Rust 后端轨映射、VM 列表/字符串语义
（N-003-1..6）的根修。

## 实现锚点

- shim：`crates/auto-lang-core/src/vm/ffi/stdlib/zip_io.rs`
  （`#[auto_macros::rust_fn]` + inventory 自动注册）
- 声明：`stdlib/auto/zip.at` + `stdlib/auto/zip.vm.at`
- 测试：`crates/auto-lang-core/src/vm/ffi/stdlib/zip_io.rs`
  `plan897_tests`（进程内三测）；`tests/probe/plan897_zip_probe.as`
  （VM 轨端到端，PLAN-897 T-03）

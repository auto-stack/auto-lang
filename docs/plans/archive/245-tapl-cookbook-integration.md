# Plan 245: TAPL 书中集成 Cookbook 实战示例

> **Status**: ✅ Completed — 所有 20 个 listing 文件 + markdown 引用 + 中英文版均已就位(2026-06)

## Context

TAPL（《The Auto Programming Language》）教程已覆盖 Auto 语言的特性层面（22 章，137 个 listings），但缺乏实战领域的代码配方。Cookbook 的 143 个示例中有约 20-25 个可直接或稍作修改后加入 TAPL，补足书中"知道语法但不知道怎么用"的空白。

TAPL 书位置：`d:/autostack/book/tapl/`
Cookbook 位置：`d:/autostack/auto-lang/crates/auto-lang/test/cookbook/`

## TAPL 各章节现状

| 章 | 主题 | Listings | 薄弱环节 |
|----|------|----------|----------|
| Ch00 | 介绍 | 0 | — |
| Ch01 | 入门 | 2 | — |
| Ch02 | 变量与运算符 | 6 | — |
| Ch03 | 函数与控制流 | 8 | — |
| Ch04 | 集合 | 8 | 缺排序实战 |
| Ch05 | 猜数字项目 | 4 | — |
| Ch06 | 类型与 let | 7 | — |
| Ch07 | 枚举与模式匹配 | 7 | — |
| Ch08 | OOP | 8 | — |
| Ch09 | 错误处理 | 7 | 缺实际错误处理模式 |
| Ch10 | 包与模块 | 5 | 缺进程操作示例 |
| Ch11 | 引用与指针 | 6 | — |
| Ch12 | 内存与所有权 | 6 | — |
| Ch13 | 泛型 | 7 | — |
| Ch14 | 文件处理器项目 | 5 | 文件 I/O 只在项目内，缺独立配方 |
| Ch15 | Actor 并发 | 7 | — |
| Ch16 | 异步 | 6 | — |
| Ch17 | 智能类型转换 | 5 | — |
| Ch18 | 测试 | 5 | — |
| Ch19 | 闭包与迭代器 | 7 | — |
| Ch20 | 编译期元编程 | 5 | — |
| Ch21 | 标准库概览 | 6 | JSON/TOML/编码/时间覆盖不足 |
| Ch22 | 聊天服务器项目 | 6 | 缺 URL 解析等网络基础 |

## 适合加入 TAPL 的 Cookbook 示例

### 加入 Ch04 — Collections（3 个）

| Cookbook 示例 | 改造为 Listing | 说明 |
|---|---|---|
| `algorithms/001_sort_int` | listing-4-9 | 数组排序，基本集合操作 |
| `algorithms/003_sort_struct` | listing-4-10 | struct 排序 + lambda 比较 |
| `science/linear_algebra/001_add_matrices` | listing-4-11 | 2D 数组 + 嵌套 for 循环 |

### 加入 Ch09 — Error Handling（2 个）

| Cookbook 示例 | 改造为 Listing | 说明 |
|---|---|---|
| `errors/001_boxed_error` | listing-9-8 | 实际的 Result 匹配模式 |
| `os/001_env_variable` | listing-9-9 | env 读取 + `.?` 错误传播 |

### 加入 Ch14 — File Processor（4 个）

| Cookbook 示例 | 改造为 Listing | 说明 |
|---|---|---|
| `file/001_read_lines` | listing-14-6 | 逐行读取文件 |
| `file/014_read_lines_temp` | listing-14-7 | 完整文件生命周期（写→读→删） |
| `encoding/003_csv_read` | listing-14-8 | CSV 解析 + 错误处理 |
| `encoding/008_csv_filter` | listing-14-9 | CSV 过滤 + 数据转换 |

### 加入 Ch21 — Stdlib Tour（8 个）

| Cookbook 示例 | 改造为 Listing | 说明 |
|---|---|---|
| `encoding/001_json/json` | listing-21-7 | JSON 序列化/反序列化 |
| `encoding/002_toml/toml` | listing-21-8 | TOML 配置解析 |
| `encoding/004_base64` | listing-21-9 | Base64 编解码 |
| `encoding/005_hex` | listing-21-10 | 十六进制编码 |
| `datetime/001_elapsed_time` | listing-21-11 | 计时 + Duration |
| `text/001_regex_replace` | listing-21-12 | 正则替换 |
| `cryptography/001_sha_digest` | listing-21-13 | SHA 哈希计算 |
| `science/miscellaneous/002_math_functions` | listing-21-14 | 数学函数（pow/sqrt/abs） |

### 加入 Ch22 — Chat Server 前置（3 个）

| Cookbook 示例 | 改造为 Listing | 说明 |
|---|---|---|
| `web/url/001_base` | listing-22-0a | URL 解析基础 |
| `web/url/002_parse` | listing-22-0b | URL 查询参数遍历 |
| `os/002_process_continuous` | listing-22-0c | 子进程管理 |

## 不适合加入的 Cookbook 示例

| 分类 | 原因 |
|------|------|
| `database/*` | 涉及外部数据库服务，不适合入门教程 |
| `web/clients/api/*` | 需要网络访问，模拟桩无教学价值 |
| `web/scraping/*` | 依赖 HTML 解析库，过于特定 |
| `web/mime/*` | 领域过于狭窄 |
| `asynchronous/ftc/*`, `asynchronous/fs/*` | Tier C 模拟桩，只有 print 语句 |
| `concurrency/004_crossbeam_spsc` 等 | 需要第三方库 crossbeam |
| `devtools/005_log_syslog` 等 | 模拟桩或领域特定 |

## 实施步骤

### Phase 1：Ch21 Stdlib Tour 补全（8 个 listing）

Ch21 是最容易集成的章节——stdlib tour 本身就是"展示常用功能"的定位。

1. 从 Cookbook 复制 `.at` 文件到 `listings/ch21/`
2. 按需要精简代码（移除不必要的 `dep` 或 `use.rust` 前置声明，简化到教学最小集）
3. 生成 `.expected.rs`、`.expected.py`、`.expected.c`、`.expected.ts` 输出文件
4. 更新 `ch21-stdlib.md` 和 `ch21-stdlib.cn.md` 正文

### Phase 2：Ch14 File Processor 增强（4 个 listing）

在 File Processor 项目前后增加独立文件配方。

1. 将 4 个 Cookbook 示例改造为教学用 listing
2. 更新 Ch14 正文，增加"文件配方"小节

### Phase 3：Ch04 / Ch09 / Ch22 补充（8 个 listing）

1. Ch04 增加 3 个集合操作 listing
2. Ch09 增加 2 个错误处理实战 listing
3. Ch22 增加 3 个网络前置 listing

### Phase 4：同步中文版

所有英文版改动同步到 `.cn.md` 文件。

## 产出

- 新增约 **20 个 listing**（每个含 main.at + 4 个 expected 输出 + pac.at）
- 更新 **6 个章节** 的英文和中文正文
- TAPL 总 listings 数从 137 → ~157

## 验证

```bash
# 确认所有新 listing 的 .at 文件可运行
for f in $(find d:/autostack/book/tapl/listings -name "main.at" -newer d:/autostack/book/tapl/SUMMARY.md); do
    cargo run --quiet -p auto-lang -- "$f" || echo "FAIL: $f"
done

# 确认 a2r transpile 正常
for f in $(find d:/autostack/book/tapl/listings -name "main.at" -newer d:/autostack/book/tapl/SUMMARY.md); do
    cargo run --quiet -- trans --path "$f" rust || echo "FAIL: $f"
done
```

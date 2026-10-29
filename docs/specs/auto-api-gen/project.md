# auto-api-gen — 通用 API 生成语言 adapter（SD-07，as-built）

> **Status**: active（PLAN-831 C2 切源落定）
> 路径：语言仓 `crates/auto-api-gen`

## 职责与边界

auto-api-gen 是语言服务/API 生成 adapter，物理 owner 留语言仓，服务 UI 与 Man 两类消费者。sidecar 声明自 auto-project-model 读取（SidecarDecl/sidecar_decl_from_root 单源），零 Man 管理实现依赖（闭包内无 auto-man-core）。adapter 不拥有第二份 SidecarDecl 或根声明解析，不持有 UI 宿主状态。

## 接口与兼容

真实接口与旧入口兼容由定向语义及类型消费验证（tauri fixture 预存红如实归因保留）。生成器资源与外部工具前置按实际消费者闭包登记。

## Rust 服务生成运行时契约（PLAN-885 实测固化）

生成 `--server rust` 服务的后端产物（`auto-blog-back/` 形态：`api.rs`/`db.rs`/`works.rs`/`types.rs`/`main.rs`）满足以下可复验边界：

- **runtime 路径**：back 模块转译按 Embedded 运行形态收集 producer 引用，产物经 `qualify_a2r_std` 通用改写为 `auto_lang_core::a2r_std::*`（`a2r_std::sys::`/`a2r_std::sqlite` 保持独立 crate 面）；内嵌 facade 必须提供生成代码引用的全部模块（含 `time`，epoch i64，与独立 crate 同源）。
- **back 模块导入解析**：伴生 back 模块转译须传入源目录（`source_dir`=back 目录），使 `use works: …` 导入的函数签名/字段类型可解析（字符串值/借用与模块类型路径正确生成）。
- **伴生同名类型**：与 api 类型同形同名的 struct 提升为 `pub use crate::types::X;`（字段名序列一致才替换），db 包装与伴生模块返回同一 Rust 类型。
- **参数形状**：`.at int` → `i64`；`str_find` 双参/三参（`str_find(s, needle)` / `str_find_from(s, needle, start)`）返回字符偏移；`parse()` → `.unwrap_or(0)`；String 比较/借用按接收者类型发射。
- **消费者验证边界**：以真实消费者（如 auto-blog）冷生成→`cargo build` exit 0→真实 HTTP 使用（创建/保存/冲突/重启回放）为准；不以 AST/字符串测试代替可编译消费者。

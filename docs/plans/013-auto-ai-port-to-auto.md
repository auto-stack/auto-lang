# Plan 013: auto-ai → Auto 语言移植

> **状态**：已批准，实施中
> **仓库**：auto-lang（Auto 代码）+ auto-ai（Rust 原版参考）
> **前置**：Auto Language Spec v0.2, auto-lang-creator skill

## 目标

将 auto-ai 的 3 个核心 Rust crate 用 Auto 语言复刻，放到 `auto-lang/crates/` 下。
Auto 代码必须能通过 AutoVM 运行，也能通过 a2r 翻译成 Rust，行为与原版一致。

## 移植范围

| Rust crate | Auto crate | 代码量 | 可移植性 |
|---|---|---|---|
| `ai-config` | `crates/ai-config/src/*.at` | ~1220 行 | 高 |
| `auto-ai-client` | `crates/auto-ai-client/src/*.at` | ~478 行 | 中 |
| `auto-ai-agent` | `crates/auto-ai-agent/src/*.at` | ~6898 行 | 中 |

不移植：`auto-ai-daemon`（axum）、`auto-ai-cli`（ratatui TUI）。

## 关键架构决策

1. **spec 即 dyn Trait**：Auto 的 `spec` 自动做动态分发，不需要 `dyn` 关键字。
   `Arc<dyn Client>` → `Arc(Client)`。
2. **serde**：用 stdlib 的 `json.encode[T]` / `json.decode[T]` 或 `use.rust serde_json` 桥接。
3. **async**：`async fn` → `fn ... ~T`。
4. **.at 解析**：桥接 `auto_atom`（已有 Auto 生态）。

## 阶段 1：ai-config

### 文件清单
| Rust | Auto | 状态 |
|---|---|---|
| `tier.rs` | `tier.at` | ✅ 已完成 |
| `wire.rs` | `wire.at` | 待做 |
| `provider.rs` | `provider.at` | 待做 |
| `loader.rs` | `loader.at` | 待做 |
| `validate.rs` | `validate.at` | 待做 |
| `lib.rs` | `lib.at` | 待做 |

### 验收标准
- AutoVM 能运行 parse_name / resolve_key / resolve_model_id
- a2r 能翻译回 Rust 通过 cargo check

## 阶段 2：auto-ai-client
（阶段 1 验收后展开）

## 阶段 3：auto-ai-agent
（阶段 2 验收后展开）

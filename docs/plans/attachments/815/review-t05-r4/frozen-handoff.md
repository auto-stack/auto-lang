# PLAN-815 I-TOOL / auto-ui-tool 交接文档

> T-08 交付物。对象：816 CLI 客户端（M816-B 消费 B 能力）及后续消费者。

## 工具身份与指纹

| 项 | 值 |
|---|---|
| 工具名 | `auto-ui-tool` |
| 二进制 | `crates/auto-ui-tools/src/bin/auto_ui_tool.rs` |
| 协议 crate | `crates/auto-tool-protocol`（纯数据，无 AST/VM/renderer） |
| 协议版本 | `PROTOCOL_VERSION = 1` |
| 交付闭包 | `DELIVERY_CRATES` 26 项（`rust_ui.rs` 常量） |
| 777 域 | `auto-tool-protocol`→data、`auto-ui-tools`→ui |

## 能力面（T-06 后）

| 操作 | cancellable | interactive | 语义 |
|---|---|---|---|
| `describe` | ✗ | ✗ | 能力/版本/生命周期 |
| `validate_project` | ✗ | ✗ | I-PROJECT 元数据/路径/约束校验（诚实边界：不做语言解析/求值） |
| `generate_ui` | ✗ | ✗ | 真实生成到项目生成根（vue/rust/jet/ark/tauri/vscode） |
| `build_ui` | ✓ | ✗ | 真实构建（vue/rust/jet/ark/vscode；vm 解释型无构建步） |
| `run_ui` | ✓ | ✓ | 真实运行（全 7 profile）；交互=Input 通道 → 前台子进程 stdin（VM 会话行 `q` 干净收口） |

## 协议消息（v1 完整面）

```
ClientEnvelope{protocol_version:u32, id:u64, body}
  body = Describe
       | ValidateProject{request: WireRequest}
       | GenerateUi{request}
       | BuildUi{request}
       | RunUi{request}
       | Input{target_id:u64, data:Vec<u8>}     // 协议级控制消息
       | Cancel{target_id:u64}                   // 协议级控制消息

ServerEnvelope{id, body}
  body = DescribeOk{descriptor}
       | ValidateOk{report: ValidateProjectReport}
       | GenerateOk{report: GenerationReport}
       | BuildOk{report: BuildReport}
       | RunOk{report: RunReport}
       | Cancelled{target_id}                    // 取消回执
       | InputAck{target_id, bytes}              // 输入投递回执
       | Event{event: Progress{message,done,total?} | Launched{url}}
       | Error{error: ToolError}
```

- `WireRequest` = `auto_project_model::GenerationRequest` 的 wire codec 编码
  （数据 owner 同源 codec，协议 crate 零复制）。
- 退出码：0=EOF 正常 / 1=内部（控制流写失败即 exit，不等 EOF） / 2=流级致命。

## 消费合同（816 客户端）

1. **只依赖** `auto-tool-protocol` + `auto-project-model`（不静态链接 auto-ui-tools）。
2. 启动 `auto-ui-tool` 子进程，stdin/stdout 帧化 stdio，stderr=日志。
3. `describe` 先行，按能力列表分派（A-only 客户端跳过 build/run/ui/input）。
4. 单飞工作操作：并发第二个工作操作=具名 busy 拒绝。
5. `Input` 字节到达前台子进程 stdin（VM 会话行 `q` 退出）；回执异步
   （`InputAck` 可能与 Progress 交错，按 id 匹配）。
6. `Cancel` 立即回 `Cancelled` 回执；被取消的操作终态为
   `RunOk{outcome: "cancelled"}`（reader 收尾失败披露在 error 字段）。

## 运行架构（内部）

- 协议循环线程 + 工作者线程 + 专属输入写线程（128 深有界队列+可停写）。
- VM 会话进程隔离：`--internal-vm-exec` 内部执行臂（每 VM run 一个子
  进程，iced/winit 单事件循环约束的进程级解法；取消/EOF=终止 owned 树；
  stdin 接 Input 通道；stderr 管道化转发+URL 嗅探=Launched 准确端点）。
- 子进程上下文经**单源请求衍生** env 传递（`AUTO_UI_TOOL_VM_BACK_ROOT`
  绝对后端根、`AUTO_UI_TOOL_VM_THEME_DECL` lite 声明 JSON）。
- 子进程自 `auto-tool-execution`（816 T-02 统一设施）spawn：Windows Job
  Object / Unix 进程组——取消终止整树+有界收尸+reader join（真终态
  oracle：NaturalEof / Stopped / IoError 三分类，非自然 EOF 具名披露）。

## 来源清单

| 来源 | 位置 | 许可 |
|---|---|---|
| gallery 资产 | `crates/auto-ui-tools/assets/gallery/` | 本仓自著 |
| wm 资产 | `crates/auto-ui-tools/assets/wm/` + drawlist 输入 | 本仓自著 |
| shadcn-vue 快照 | `crates/auto-ui-tools/assets/shadcn-ui/`（345 文件） | MIT（上游 registry 快照 2026-08-27，SNAPSHOT.md 记录） |
| Inter 字体 | vue.rs 内嵌（三字重） | SIL OFL 1.1 |

## 规范增量

- SD-01: `docs/specs/auto-ui-tools/project.md`（UI 工程工具包）
- SD-02: `docs/specs/tool-protocol/project.md`（I-TOOL 协议）
- SD-03: `docs/specs/auto-man/project.md`（修改：UI 生产算法退出）
- SD-04: `docs/specs/project-model/project.md`（修改：I-PROJECT 消费）
- SD-05: `docs/specs/auto-lang/ui/overview.md`（修改：工程工具/资产消费 814 公开包）
- SD-06: `docs/specs/repository-boundaries/generated-ui-distribution.md`（修改：新 tool/协议/source 闭包）

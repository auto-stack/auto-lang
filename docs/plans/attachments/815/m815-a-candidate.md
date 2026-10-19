# PLAN-815 M815-A 候选交付记录(T-12)

状态:work 阶段完成、送 T-13 独立阶段审查。本文件是 M815-A 的候选收据(非最终发布收据;`delivery_commit` 须待 T-14 实际合入 master 后由 merge owner 回填方为发布)。

## 1. 身份与冻结位

| 项 | 值 |
|---|---|
| 唯一树 | `D:/autostack/.wt/lang-815/auto-lang`(分支 `codex/plan-815-dev`) |
| 基面 | master `b9f7ab7e1` |
| 代码提交 | `c7bbc96810b46b4c79478be03d3839cfef665d3f`(M815-A 全部代码/测试/fixture) |
| 777 基线提交 | `9f27d54f013119a93ca832a2c43d83413592cb69`(冻结候选 SHA,T-13 审查对象) |
| 依赖交付 | 809/I-PROJECT `b819a602e`(已归档,master 祖先);不依赖 M814-A/814 任何 WIP |

## 2. 交付范围(M815-A,§5.5)

- `crates/auto-tool-protocol`(T-03):I-TOOL 纯数据合同——版本(`PROTOCOL_VERSION=1`)、能力/生命周期描述符、预算常量(8MiB 帧/1MiB 进度/128 队列)、错误分类(unknown-version/unknown-operation/missing-field/invalid-field/budget-exceeded/not-cancellable/internal)、双向信封、共享 framing(`read_frame`/`write_frame`/增量 `parse_frame`)。
- `auto-project-model` 新 `wire` 模块(feature `wire`,数据 owner 同源):`GenerationRequest ↔ JSON` 无损 codec——带标签 Value 编码、未知键拒绝、不可表示变体显式 `UnsupportedValue`。协议/工具/客户端零请求结构复制。
- `crates/auto-ui-tools`(T-12):真实 `auto-ui-tool` 进程,窄操作 `describe` + `validate_project`;版本/能力拒绝先于副作用;`cancel` 按已服务操作诚实回答;日志仅 stderr,stdout 纯帧;退出码 0/1/2。
- `test/fixtures/modularity/ui-tool-consumer`(独立 fixture):闭包证明(无 auto-man/auto-lang/auto-vm/auto-gen)+ 真实子进程 V-05 用例。
- T-02 资源登记:`resources.rs` 资产清单(gallery/wm/shadcn-ui + drawlist 外部输入;owner/来源/许可证/目标),生产 embed 源未动、目标位不存在(防提前复制)。
- 777:`auto-tool-protocol`/`auto-project-model`=data、`auto-ui-tools`=ui;基线刷新 75 边;strict pass。

## 3. 版本/能力表(A 阶段冻结)

| 操作 | cancellable | interactive | 语义 |
|---|---|---|---|
| `describe` | false | false | 返回 ToolDescriptor(名称/版本/协议版本/能力/生命周期/退出码表) |
| `validate_project` | false | false | I-PROJECT 元数据/路径存在性/请求约束校验;report 显式 checked/not_checked,不冒称解析/求值/生成/构建成功 |
| (`cancel`) | — | — | 协议级控制消息:按 target 回 NotCancellable(op) 或 invalid target_id |

不支持的操作(如 `generate_ui`)→ `unknown-operation` 错误、零副作用(测试钉住输出目录未被创建)。

## 4. Spec 候选(SD-02 子集草案;正式沉淀待 T-14 受管窗口)

`docs/specs/tool-protocol/project.md`(A 子集)应陈述:
1. 控制流=stdout 帧化 JSON(4 字节大端长度前缀),日志=stderr;任何非帧字节为协议违例。
2. 请求带 `protocol_version`;未知版本→`unknown_protocol_version` 错误响应,流继续,零副作用。
3. 预算为合同常量(见 §2),帧超限=流致命(exit 2);语义超限=budget_exceeded 错误。
4. I-PROJECT payload 唯一经 `auto-project-model`(wire feature)同源 codec;跨工具边界的类型仅限具名数据。
5. 退出码:0=EOF 关停,1=内部,2=流违例;`--version` 为协议外纯文本。
6. A 能力面=上表;新增能力须走 additive 版本化(SD-02 完整稿含资源/结果/进度/交互/取消的 B 面合同)。

## 5. 测试与门禁证据(work 阶段,冻结 SHA 上)

| 面 | 命令 | 结果 |
|---|---|---|
| 协议 | `cargo test -p auto-tool-protocol` | 13/13 通过 |
| codec | `cargo test -p auto-project-model --features wire --lib` | 44/44(含 6 wire 往返/漂移) |
| 工具+进程 | `cargo test -p auto-ui-tools` | 15/15(11 真实进程 + 4 资产清单) |
| fixture 闭包 | `bash test/fixtures/modularity/ui-tool-consumer/verify-deps.sh` | PASS(闭包无 man/lang/vm/gen) |
| fixture 进程 | `AUTO_UI_TOOL_BIN=... cargo test --manifest-path .../ui-tool-consumer/Cargo.toml` | 4/4 |
| 检查面 | `cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-man` | 通过(零新增警告;proc-macro-error2 future-incompat 为预存传递性) |
| 777 | `python scripts/modularity-audit.py --mode strict` | verdict=pass violations=0 grandfathered=2 |

merge-owned 门禁**pending**(按 2026-10-14 时序裁定,归 T-14 冻结后统一执行):裸 `cargo t` + 新包/协议进程控制档;本轮未触 VM/编译器/trans/UI 生成,预计无需 tv/tt/tu(以 T-13/T-14 实际 diff 复核为准)。

## 6. 792/发行闭包说明

本轮 `rust_ui.rs` 零改动,792 受管发布管线未触及;`DELIVERY_CRATES` 增补 `auto-tool-protocol`/`auto-ui-tools` 留待 T-04(生成主体迁移)一并处理——A 工具不属于 rust 发行闭包。

## 7. 未交付(诚实边界;B 阶段项)

- 完整 UI 工程工具(Vue/Rust/Jet/Ark/Tauri 生成/build/run)——等 M814-A+M815-A 发布后 T-04。
- 协议 B 面:进度事件实际发射、交互 stdin、真取消、运行进程寿命——T-05(等 M814-B)。
- 旧 Man 门面投影、旧算法/重复资源清理——T-04/T-06。
- tool-protocol-consumer 独立 fixture(纯协议 lib 消费)——T-07。
- I-TOOL 协议的 816 真实客户端消费——M816-A(T-14 合入后放行)。

## 8. 下一步

T-13:同树独立阶段审查(对象=冻结 SHA `9f27d54f0` + 本候选 + contract-baseline.md §1/§3/§4),通过后 T-14 串行冻结门与实际 master 合入(不归档、不清树,收据回填 delivery_commit)。

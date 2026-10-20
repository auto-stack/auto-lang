# PLAN-815 M815-A 候选交付记录(T-12)

状态:R1 审查(needs_fix,6 findings)已修复完毕,再送 T-13 复审。本文件是 M815-A 的候选收据(非最终发布收据;`delivery_commit` 须待 T-14 实际合入 master 后由 merge owner 回填方为发布)。

## 0. R1 修复轮(2026-10-20,[review-r1](review-r1/review.md) 全部 6 findings 闭合)

| 项 | 值 |
|---|---|
| 修复提交 | `faea8c9d7`(worktree `codex/plan-815-dev`,基线为 R1 审查对象 `9f27d54f0`) |
| F-01 | `TypeMismatch` 经 `type-mismatch:<field>` 无损往返(解码按 scanner 封闭字段集 members/defines 规范化,编造 payload 拒绝);非 Unicode 路径(未配对 UTF-16)→ `UnsupportedPath` 显式拒绝,不再 `to_string_lossy` 静默改写 |
| F-02 | 全部嵌套层固定键集严格校验(origin/source、identity、display×12、runtime+window、render 各变体、toggles 各变体、theme、dep_roots、package_assets、raw props/nodes/props):未知键拒绝、缺槽要求显式 null |
| F-03 | byte/usize 载荷范围校验(byte=256 拒绝);窗口分量有限+Pac 200..=7680/4320 边界+f32 精确可表示(编解码同规则,1e300/NaN 具名拒绝);`UnknownProtocolVersion.client` 为 u64(4294967297 如实报告) |
| F-04 | 相对路径锚定请求内绝对 `project_root`,禁 cwd;新增 findings:`project-root-not-absolute`/`project-root-is-file`/`input-source-is-directory`/`relative-path-without-root`(无锚定相对路径如实报告"未能校验",绝不探测 cwd);C09/C10 双 cwd 进程对照一致 |
| F-05 | 冻结 `MAX_WIRE_JSON_DEPTH=120`(JSON 层):codec 编/解、write_frame 双向对称强制;生产端超包络具名拒绝(`DepthExceeded`/framing "envelope"),消费端 >128 JSON 层为 Malformed 流致命——合同见 contract-baseline §4 |
| F-06 | 两条新警告清除(未用 import/字段)、12 个新文件 + fixture 全部 fmt 通过;scoped all-targets check 零警告 |
| 回放 | R1 探针:codec C01..C06 闭合;C07/C13 codec 臂现为新具名拒绝语义(探针旧 unwrap panic 属预期,不变量由永久测试 `r1_*`/`r5_*` 覆盖);进程 C08..C11、C13(16 可读/64 超包络拒)、C12(合同允许的输出超限 exit 1)对重建二进制验证 |
| 测试 | 定向 83(workspace: model 49 / protocol 15 / tools 19)+ fixture 4 全绿;新增 12 条永久回归(wire 5、protocol 2、process 3、及 2 处既有断言强化) |

## 1. 身份与冻结位

| 项 | 值 |
|---|---|
| 唯一树 | `D:/autostack/.wt/lang-815/auto-lang`(分支 `codex/plan-815-dev`) |
| 基面 | master `b9f7ab7e1` |
| R1 审查对象 | `9f27d54f0`(needs_fix) |
| 代码提交 | `c7bbc9681`(初版)→ `faea8c9d7`(R1 修复;**当前冻结候选 SHA,T-13 复审对象**) |
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
1. 控制流=stdout 帧化 JSON(4 字节大端长度前缀,预算按载荷字节计、不含前缀),日志=stderr;任何非帧字节为协议违例。
2. 请求带 `protocol_version`;未知版本→`unknown_protocol_version` 错误响应(client 载荷为如实 u64,不截断),流继续,零副作用。
3. 预算为合同常量(§2 + contract-baseline §4):帧 8MiB 载荷、进度 1MiB/op、队列 128、**JSON 深度 120(可传输包络,生产端/消费端对称报告)**;帧超限=流致命(exit 2);语义超限=budget_exceeded 错误。
4. I-PROJECT payload 唯一经 `auto-project-model`(wire feature)同源 codec;跨工具边界的类型仅限具名数据;嵌套各层固定键集(缺槽须显式 null)、数值窄化范围校验、非 Unicode 路径/非有限浮点/超深度均具名拒绝——"无损或明确拒绝",无静默改写。
5. 退出码:0=EOF 关停,1=内部,2=流违例;`--version` 为协议外纯文本。
6. A 能力面=§3 表;新增能力须走 additive 版本化(SD-02 完整稿含资源/结果/进度/交互/取消的 B 面合同)。

`docs/specs/project-model/project.md`(SD-04 A 子集增补,应陈述):
1. `wire` feature(可选 serde_json)提供 `GenerationRequest ↔ JSON` 同源 codec,`WIRE_FORMAT_VERSION=1`;基 crate 行为零变化(44/44 原测试不动)。
2. 支持面:请求全字段 + 静态子集可产生的 Value 族(标量/Array/Block/Obj/Pair,CStr);拒绝面:不可表示 Value 变体(具名)、非 Unicode 路径、非有限浮点、超 120 JSON 深度、未知/缺失键(各层)、编造的 type-mismatch 字段。
3. 静态扫描器支持域不变;wire 不新增 pac.at 语法解释。

## 5. 测试与门禁证据(R1 修复轮,冻结 SHA `faea8c9d7` 上)

| 面 | 命令 | 结果 |
|---|---|---|
| 协议 | `cargo test -p auto-tool-protocol` | 15/15 通过 |
| codec | `cargo test -p auto-project-model --features wire --lib` | 49/49(含 11 wire 往返/漂移/数值/深度) |
| 工具+进程 | `cargo test -p auto-ui-tools` | 19/19(15 真实进程含 3 组 cwd 对照 + 4 资产清单) |
| fixture 闭包 | `bash test/fixtures/modularity/ui-tool-consumer/verify-deps.sh` | PASS(闭包无 man/lang/vm/gen) |
| fixture 进程 | `AUTO_UI_TOOL_BIN=... cargo test --manifest-path .../ui-tool-consumer/Cargo.toml` | 4/4 |
| 检查面 | `cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-project-model --features auto-project-model/wire --all-targets` | 通过,零警告(F-06 闭合) |
| 格式 | `cargo fmt --check -p auto-project-model -p auto-tool-protocol -p auto-ui-tools`(+ fixture fmt) | 通过(F-06 闭合) |
| 777 | `python scripts/modularity-audit.py --mode strict` | verdict=pass violations=0 grandfathered=2 |
| R1 回放 | review-r1 codec probe + process_probes.py | C01..C06/C08..C11/C13 闭合;C07/C13 codec 臂=新具名拒绝(详 §0) |

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

T-13 复审:同树独立阶段审查(对象=冻结 SHA `faea8c9d7` + 本候选(含 §0 修复轮与 §4 SD-02/SD-04 A 子集)+ contract-baseline.md §1/§3/§4),通过后 T-14 串行冻结门与实际 master 合入(不归档、不清树,收据回填 delivery_commit)。

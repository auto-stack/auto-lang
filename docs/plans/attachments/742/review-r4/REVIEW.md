# PLAN-742 第四次独立 final review

2026-10-14；**needs_fix**。当前仍有一个稳定代码反例 F-742-R4-01，不能批准完整 API handler parity。前轮 17 个输入全部闭合；S10 和 Vue 冒烟的证据缺口已补齐。VM 交互未验证，不据此判产品故障，也不追加固定 19 场景门禁。

## 审核基线与范围

- plan_revision=1；审面 `2c40e1fe4673c2041ae3c366037d7eab38017d81`，基面 `8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`。
- 同一 worktree `D:/autostack/.wt/lang-742/auto-lang`，branch `plan-742-dev`；本轮只有 api_gen.rs 的实现 diff。实施源码提交且干净；未创建新 checkout、未修实现、未合并。
- Codex /root，沿用前三次审核上下文，未参与本轮实施；无额外 reviewer agent。结论从实际生成物、实编与运行重建，未将 work 摘要当证据。
- Notes `1391bd7eb2f6b3b08e9841d30b25125e47ced058`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。Notes 原有未跟踪 front/tests 保留。
- 主检出从 `0a0950bc0` 到 `284acbfb4` 仅并发 PLAN-774 计划簿记；crates、Cargo/config、五份 canonical Specs 无变化。审面、依赖、源码/规范/制品指纹见 [baseline.json](baseline.json)，入口计划冻结为 [plan-input.md](plan-input.md)。
- 已检查 work §9.2 的连续失败诊断：撤除 placeholder、递归遍历嵌套实参的调整有效；新扫描仍以字符串后缀判标准库身份，完整模块身份不变量尚未闭合。

## F-742-R4-01：合法 companion 模块名被标准库扫描误改

**P2 / blocking；原 AC-742-02、04 / T-1、T-2，连带 T-3 的修复版本验证。原模块调用身份保持要求的剩余缺口，不宣称首次由本次提交引入。**

`qualify_a2r_std` 在 api_gen.rs:530 对每个字符执行 `starts_with("a2r_std::")`，未核对左侧 token 边界或完整路径身份。合法 companion `custom_a2r_std.at` 导出的 `ping()` 因名字后缀被当作标准库。

实际输入及控制在 [namespace_suffix](cases/namespace_suffix/api.at) / [namespace_control](cases/namespace_control/api.at)：

```auto
pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var n int = custom_a2r_std.ping()
    if n == 7 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
```

companion 为 `pub fn ping() int { return 7 }`。public generate_api 返回 Ok，但 [api.rs](cases/namespace_suffix/api.rs) 的导入和调用被改成 `crate::custom_auto_lang::a2r_std`；[实际 rustc 输出](cases/namespace_suffix/compile.log) 为 **E0433 cannot find custom_auto_lang in crate**。同结构改用 companion `custom.at` / `custom.ping()` 则实编运行返回 `"ok"`。wrapper 中声明了合法 companion 对应的 Rust 模块，错误不来自缺少测试依赖。

修复应按真实 token/模块路径身份限定标准库，保留较长标识符和 companion crate 路径；补成永久实编控制，继续覆盖原生 sys/sqlite、已限定路径与 use 行正例。不用模块名黑名单规避。此输入沿用既有 companion 点调用形态，没有添加语法、规模或新命名承诺。

## 旧例闭合与覆盖账

第三次 17 个 api.at/repository.at 的 SHA256 与本轮逐一相同，全部实编/运行对拍通过。前轮 marker_literal 与 nested_borrow 均通过，F-742-R3-01/02 的具体旧例关闭；本轮另外两层/三层借用组合通过。完整模块限定不变量因 R4-01 仍 partial。

本轮 [generated-results.json](generated-results.json) 计 **20 个有效用例：19 实编运行通过，1 编译失败**，包含旧 17、nested_twice、新反例与命名控制。原 fixture 本轮 public generate_api 返回 Ok（probe.log）；其既有实编 oracle 包含在目标测试中，不将本轮生成日志单独称为完整编译证据。上一轮探索材料不纳入当前计数，也不追加语法要求。

复现：在实施树运行证据包 `replay.ps1 -Build`，再运行 `execute-generated.ps1`。前者从 Cargo JSON 唯一 dev-profile auto_man rlib 及 native link paths 构建探针，调用真正 public generate_api；后者编译生成的 handler 并实际 poll/运行、对拍值。它验证 handler 发射，不冒称启动完整 HTTP 服务。脚本、原输入、生成物和失败日志均在本包。

| AC | 任务与结果 | 证据 / 限制 |
|---|---|---|
| AC-742-01 集合构造与业务值保持 | T-1/T-2 pass（已验形态） | List/brace/module/borrow/marker 字符串及构造对照实编运行；旧输入未换样 |
| AC-742-02 companion 导入/调用/借用 | T-1/T-2 **fail** | 遮蔽、借用、嵌套已验形态通过；合法模块名后缀 R4-01 E0433 |
| AC-742-03 比较与回归 oracle | T-1 pass（原约定范围） | 单 == 真编译；原历史错误只存类别级归因，无已丢 WIP 行号可补；不扩大链式语法承诺 |
| AC-742-04 返回/handler 实编 | T-1/T-2 **fail** | 原普通值、Option/void/File 旧例通过；当前合法体因错误路径仍不可编译；Upload 沿用源码接线层证据，未冒称实编 |
| AC-742-05 Notes 电池/legacy/联动 | T-3 partial | work 当前 v1 电池与全流程归档；独立 S10、Vue 13/13 通过；通用修复尚未闭合，VM 交互未签 |
| AC-742-06 D7～D9 运行时/主题 | T-2/T-3 partial | 原 Rust 控制与 tu 记录保留，相关源码 hash 未变；当前 Vue 主题实测通过；VM 未验证 |

## Notes 证据与边界

本轮使用 autoui-verifier 标准 VM 驱动及 Notes 既有 Playwright 测试，不改变业务源码/测试。CLI SHA256 `211d5eb87a80f8d21f7564124ee0286b3a20e42199932f8b0caaddac4a5a2e75`。VM 尝试期间后端重建，当前后端 SHA256 `008f004508168802291820233a29df0279024b45bfc6dcc6a2665a382c5a5422`，与 work 早先后端指纹不同；本轮独立检查绑定实际后端，未假借旧制品身份。生成 workspace 的 auto-lang/a2r-std runtime path 指向主检出；这不等于 worktree runtime build，路径配置已冻结指纹。

- **S10 pass**：独立 [s10-replay.py](s10-replay.py) / [s10-independent.json](s10-independent.json) 记录隔离目录 setup、创建 n-1、值读回、强杀 PID44220、退出/拒绝连接、同二进制同目录重启 PID41428、同 note_id/title/body/revision/state 读回。DB 路径相同，前后文件 SHA 分别 `0573a0a7…` 与 `4fa4ab8a…`；文件 hash 不相同，不要求恢复过程不写数据库。仅清理本轮拥有的进程。work 完整转录另存 [work-s10-full.log](work-s10-full.log)。
- **Vue pass**：独立启动后端和既有生成前端，运行 Notes `tests/smoke.spec.ts`；**13/13**，41.5s。包括主题 root dark class、新建/删除、保存、搜索筛选、autosave、console 检查。[vue-smoke.log](vue-smoke.log)、[身份](vue-identity.json)、[可重放驱动](vue-replay.py)。当前代表性用户行为证据成立，不称视觉双端全等。
- **VM partial**：标准 `test_vm_mcp.py --auto-bin <worktree>/target/debug/auto.exe --app-dir D:/autostack/auto-notes --timeout 45`，隔离 NOTES_DATA_DIR/testmode/seed，MCP10104，45秒未就绪，exit1；驱动终止自己的 VM。[vm-mcp.log](vm-mcp.log)。无交互/截图，不确定根因，未判产品回归。**VM19是 Notes 后续交付复验边界，不是新加的固定数量门禁**；不能以启动日志签署已做交互。原计划 §4 的 auto run / v1 / legacy 与扩展 VM 证据范围分别保留。

## 当前门禁与健康检查

| 实际命令 / 检查 | 结果 |
|---|---|
| `cargo test -p auto-man --lib api_gen` | 48 pass，1 ignored，3.72s；[api-gen.log](api-gen.log) |
| `cargo build -p auto-man --message-format=json` | exit0；[制品索引](auto-man-build.jsonl) |
| `cargo t --no-fail-fast` | **5277 pass / 1 fail**，5278 run，105.497s；[daily.log](daily.log) |
| 失败定向 `cargo t lock_serializes_critical_sections` | 1/1 pass，0.204s；[state-lock-retry.log](state-lock-retry.log) |
| `cargo tt --no-fail-fast` | **5649/5649 pass**，169.857s；[trans.log](trans.log) |
| 触面两文件 rustfmt check；基面至 HEAD crates diff whitespace | exit0；[format.log](format.log) |

日常失败为 state_file::tests::lock_serializes_critical_sections（99≠100），源码未改、批量回执 F-762-R2 已记同款不稳定形态。保留首跑红和复跑绿，不将 full daily 改称全绿，不据此归因742。无新增742 dead-code/debug print。存量 warning 不冒称仓库零警告。

未运行 tf（批量档），未运行 aavm/docs_gen（零触面）。本轮只改 api_gen；D7/D9 Rust runtime controls 和 tu 862 pass/2 ignored 属 r1 历史证据，源码 hash 至本轮不变、Cargo.lock 曾变，因此保留来源与范围，不冒称本轮重新运行过。

## 规范增量与下一步

[冻结 spec-delta.md](spec-delta.md) SHA256 **D013A579F74D0861B9FD376C8016A4C7B7E5F229288A93710F5ABD5A0621FAC0**。五目标路径、SD IDs、GOAL-003/007、plan_revision 均不变；SD-742-01/02 不能签完整 parity。未改 canonical Specs/live ledger。D-742-1 继续另案；db旧regex仍是候选，未独立复现，不升格确定故障。

退回 executing，T-1/T-2 重开，T-3 在修复基线上重新确认依赖证据，0/3；已通过事实及所有历史记录保留。下一轮 work 先补“代码区身份与完整模块/token身份不同”的诊断、修复 R4-01、补永久反例/控制并提交同树。已运行的 Vue/S10 不因记账被抹去；VM 未验证边界继续明示。后续 merge-owned 规范沉淀、到期批量档、归档均 pending。

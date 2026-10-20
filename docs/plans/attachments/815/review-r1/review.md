# PLAN-815 r2 / M815-A 独立阶段审查 R1

2026-10-20。**outcome: needs_fix**；review_scope: phase；任务 T-13 / AC-08。

本轮审查对象是已经交接的 M815-A，不是尚未完成的 M815-B。独立审查会话 Codex `/root` 从主检出合同、实际提交及反例重建结论，未继承实施会话的自检结论，未创建代理或第二工作树；不声称未暴露的模型/会话身份。实际使用的技能及合同哈希见 [baseline.json](baseline.json)。

## 冻结基线

- Plan revision: 2；读入 Plan SHA256 `4b93cc2b25898992757eb14a204fc63ed6ee6765a27b0356b717440bdda05ba3`（进度回写前）。
- 唯一实现树 `D:/autostack/.wt/lang-815/auto-lang`；分支 `codex/plan-815-dev`。
- reviewed_commit `9f27d54f013119a93ca832a2c43d83413592cb69`；diff/base `b9f7ab7e1f400073006e096a08b436f933e4a836`；27 个变更文件。
- 809 交付依赖 `b819a602e` 的完整 SHA、主检出 HEAD、所有实现文件/Spec/配置/锁文件哈希见 baseline.json；未使用 814 WIP。
- 真实工具二进制 SHA256 `45b4ab83b829960b59abeeba364d01877f73ae313ddcc3ca93cd6ab1d887c18a`。`cargo build -p auto-ui-tools` 复核产物无需重编，哈希与真实进程反例完全相同。
- 入场和证据采集后实现树均 clean、HEAD 未漂移。审查未修改实现、canonical Specs、ledger，未合入/清理。
- 冻结的规范候选载体：[frozen-m815-a-candidate.md](frozen-m815-a-candidate.md)，SHA256 `5f7760775e85fa79c3141b19e05f5c7564093280c866ff7288437f9006cda700`；候选 §4 为拟发布 SD-02 A 子集。补充冻结 [contract-baseline](frozen-contract-baseline.md)，SHA256 `47b61d404c92c706e79eaad2c68c03af733982fdc51404ae86c0ebc04a57b1f4`。

## Findings（均 OPEN；合同内缺口，不授权转债后延期）

### F-815-R1-01 [P2] 编码路径不能保证已声明的无损交接

AC-02/07，T-03/T-12。`crates/auto-project-model/src/wire.rs:784,800` 将 `TypeMismatch { field }` 编成裸 `type-mismatch`，解码用 `field: "?"` 代替。实际合法构造 `parse_document("members: 7\n") → GenerationRequest`，往返前 `TypeMismatch { field: "members" }`，往返后 `field: "?"`，完整请求不相等。现有 unsupported 测试仅用 interpolation，未覆盖携 payload 的变体。

同一“无损或明确拒绝”不变量的另一条路径：`wire.rs:829` 的 `to_string_lossy()` 静默改变 Windows `PathBuf` 中的未配对 UTF-16；独立 C07 在本机真实构造 `0xD800`，往返变成 U+FFFD，未报错。此处无需承诺所有 OS 路径都可传输，但不可成功返回另一条路径。

修复：同源保留诊断 payload；不可表示路径采用可逆表示或明确拒绝。补 static/dynamic origin、所有 payload arm、project/source/input/output 路径正反控。保留 Unicode/空格路径正控，不靠移除字段规避。

证据：[codec-results.txt](codec-results.txt) C01/C07；可重放源码 [probe/src/main.rs](probe/src/main.rs)。

### F-815-R1-02 [P2] 严格 schema 检查只覆盖请求第一层，嵌套漂移被静默吞掉

AC-02/07，T-03/T-12。`wire.rs:488..535` 只校验 REQUEST_FIELDS，随后 identity/display/runtime/render/toggles/assets 等子对象直接按键取值，`opt_str_field` 又将缺键当 None。

独立 C02 往 `request.identity` 增加 `future_field`，C03 删除 `request.display.title_zh`：公开 codec 均成功，真实进程也回复 `validate_ok / ok=true`。与顶层未知键被拒的正向校验对照相反；违反 wire 模块及候选明确宣称的“未知键拒绝/codec drift fail loudly”，容易把消费者版本漂移伪装为字段缺省。

修复：对各层固定对象/变体按已冻结 shape 校验允许键和必需槽，区分显式 null 与缺键；覆盖 origin、模型字段、theme/dep、raw props/nodes、tag 对象，保持声明中的扩展数据通过 raw 槽传递。若确需某些键可省略，须明确其兼容规则，不能继续声称所有槽严格。

证据：codec C02/C03、[process-results.jsonl](process-results.jsonl) 同名行；CONTROL top_level_unknown 确认不是一概拒绝或探针未触实现。

### F-815-R1-03 [P2] 数值转换会成功接受失真的数据或生成不可读响应

AC-02/03/07，T-03/T-12。

- `wire.rs:201` 直接 `u64 as u8`：tag `byte`、值 256 解成 `Byte(0)`，真实工具回复 ok。相邻 `u8=256` / `i8=-129` 已正确拒绝，`byte=255` 正常，证明遗漏的是该转换分支。
- `wire.rs:1084` 将有限 f64 `1e300` 转成 f32 `inf`；真实 `validate_project` 报 `ok=true`，summary.window 序列化成 `[null,300.0]`。此响应不能反序列化为公开的 `Option<(f32,f32)>`。`window_to_wire` 对本地 NaN 也成功返回含 null 的 wire，随后同源解码失败（C06），与 Value 浮点的明确 NonFiniteFloat 拒绝不一致。
- 同族诊断准确性遗漏：`auto-ui-tools/src/lib.rs:149` 将未知版本 `4294967297` 截断为 1，实际返回 `UnknownProtocolVersion { client:1, server:1 }`。拒绝本身有效，但错误载荷虚报客户端版本。

修复：每个窄化转换做范围和有限性校验；编码和解码对可表示范围一致；u32 版本范围外应准确分类/报告。用边界内/外、NaN/Inf、f32 转换溢出及真实 typed response 解码补永久回归，不仅检查 JSON 输出成功。

证据：codec C04/C05/C06，process C04/C05/C08 与 adjacent-integer-range 控制行。

### F-815-R1-04 [P2] 路径校验依赖工具 cwd，并将文件当作合法工程根

AC-02/07，T-12。`auto-ui-tools/src/validate.rs:41,73,85` 对 output_root/input_sources 直接调用路径探测，未使用请求的 project_root；dep/front 分支才使用 resolve_against_root。

同一绝对 project_root、`input_sources=["front/main.at"]`，实际文件存在：工具从工程目录启动为 ok，从独立 tool-cwd 启动为 input-source-missing。反向对照：工程内已有普通文件 `result`，请求 `output_root="result"`；从工程启动正确失败，其他 cwd 却 ok。另 C11 以普通文件作为 project_root，仍 ok。

修复：遵守 I-PROJECT 已有“相对路径基于请求源根，非进程 cwd”规则，并按路径角色检查文件/目录。无根时按明确合同报告未能校验或受控失败，不静默回落 cwd。补绝对/相对路径、根缺失、角色错误及不同 cwd 对照。无需重新读取/求值 pac.at。

证据：process C09 两行、C10 两行、C11；CONTROL-rooted-front-dep 在独立 cwd 正常，隔离出有问题的分支。

### F-815-R1-05 [P2] 同源写帧器能输出同源读帧器拒绝的有效请求，深度准入未冻结

AC-03/07，T-01/T-03/T-12。`auto-tool-protocol/src/framing.rs:90,144` 默认 serde_json 文本解码带递归限制，写帧器/codec 未作对称检查，冻结预算仅声明字节/进度/队列。

独立用公开 GenerationRequest + Value::Array 构造 16/64 层嵌套数据：两者同源 request_to_wire/request_from_wire 都成功，write_frame 都成功；16 层帧可读，64 层仅 2233 字节却被 read_frame 以 recursion limit 拒绝。真实完整请求同样只需 2302 字节，进程 exit 2，无响应。不是超出已声明 8MiB 预算，也不是非 JSON 或非法标签。

修复：冻结实际可表示/可传输的嵌套包络及错误分类，并保证 codec/写帧/read_frame/parse_frame 一致；若采用显式有界准入，应在对应生产端/消费端都准确报告，而不是让合法写入变成对端“坏 JSON”。覆盖标签数组、raw node children 等实际递归形态与边界两侧。本 finding 只依据以上代表性反例，不增加无限深度或任意规模承诺；若支持域调整构成语义合同变更，按 new 修订合同。

证据：codec C13 same_library_framing；process C13 的 16/64 两个对照。两条文本读路径均已静态检查。

### F-815-R1-06 [P3] 健康门未完成：新增两条警告且 12 个新增 Rust 文件格式不通过

AC-08（阶段健康门）/AC-05（后续完整健康门），T-12。`auto-ui-tools/tests/tool_process.rs:9` 未使用 read_frame；`:24` 的 stderr_text 字段从未读取。定向包测试和 all-targets check 均原样输出这两条新增 warning，不能用普通 lib cargo check 代替所有目标健康检查。

`cargo fmt --check -p auto-project-model -p auto-tool-protocol -p auto-ui-tools` exit 1；差异涉及 wire/wire_tests、protocol 6 文件及 tools 4 文件，共 12 个新增文件。无既有文件格式红混入。

修复：处理新增 warning，格式化新增文件并复跑 scoped fmt/all-targets check；独立 fixture 也检查格式。无需整档回归。

证据：[fmt.log.txt](fmt.log.txt) 与下方真实命令/告警摘录。

## 验证与覆盖

以下是本轮实际执行，不是复述实施者汇总：

| 命令/检查 | 结果与范围 |
|---|---|
| `cargo test -p auto-tool-protocol -p auto-ui-tools -p auto-project-model --features auto-project-model/wire` | exit 0；model 44、protocol 13、tools 4 资源+11 真实进程，共 72 tests；另 0-test targets/doc tests 不计数；两条新增 warning |
| `cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-project-model --all-targets` | exit 0；同两条 warning；未运行整档 |
| `cargo fmt --check -p auto-project-model -p auto-tool-protocol -p auto-ui-tools` | exit 1；12 个新增文件需格式化 |
| Git Bash `test/fixtures/modularity/ui-tool-consumer/verify-deps.sh` | exit 0 / PASS；[完整 tree](dependency-tree.txt)；normal/build 无 Man/lang/VM/gen/media 大包。系统 WSL bash 不可用的 exit127 未当作 pass，已改本机 Git Bash 成功重放 |
| 独立 consumer：设置 `AUTO_UI_TOOL_BIN` 后 `cargo test --offline --manifest-path test/fixtures/modularity/ui-tool-consumer/Cargo.toml` | exit 0；4/4，真实进程；与上述 72 合计 76 个现有定向测试全部通过 |
| `python scripts/modularity-audit.py --mode strict` | exit 0，verdict pass，violations=0，grandfathered=2；cross-check 75 边、无差异；[审计摘要](audit-summary.json) |
| `git diff --check b9f7ab7e1...HEAD` | exit 0 |
| 独立 codec probe / process probe | 观察结果保存在 codec-results / process-results；正反控区分上述缺陷，不能用现有 76 绿覆盖这些未测路径 |

告警摘录：`unused import: read_frame`（tool_process.rs:9）；`field stderr_text is never read`（:24）。本轮未将有意写 stderr 的协议诊断当作 stray debug print，控制流日志分离正控通过。

额外正控：正常 describe/EOF exit0；未知版本或未知能力拒绝后还能 describe；坏前缀/坏 JSON/输入 8MiB+1 全部 exit2；generate_ui 拒绝未创建输出目录；取消已完成操作和未知 target 的既有真实进程测试通过；静态 unsupported 不冒称 evaluation；资源登记/源未提前迁移全部通过。

C12 输出超限是**已验证的合同允许退出**，不是 finding：输入 8,301,703 字节，缺源诊断输出 9,218,878 字节，工具不写半帧、stderr 报预算、exit1（冻结表明确输出超限=Internal）。A 无 progress/交互/真实取消，1MiB progress 和 128 通知队列的实际执行留 B/816；未把未实现 B 能力当 A 新缺陷。

## AC → Task → 实现 → 证据

| AC | 当前结论 | 映射/边界 |
|---|---|---|
| AC-01 | partial；B pending | A 的轻闭包/资产登记通过 T-02 + dependency tree + strict；旧生成器/生产 embed 整体迁移属于未启动 T-04/T-06，未虚报通过 |
| AC-02 | fail（A 当期范围） | T-03/T-12，wire/validate，F01..04；完整项目读取 V-02 仍待 B |
| AC-03 | partial/fail（A 当期范围） | T-03/T-12，真实 host/framing/报告，F03/F05；版本/能力拒绝、流错误和输出超限正控通过；完整生命周期等 B |
| AC-04 | partial；B pending | T-02 资产来源和实际 consumer 通过；792 rust_ui.rs/DELIVERY 表本轮未改，完整 fresh/locked/生产资源失败矩阵仍归 B，未伪造执行 |
| AC-05 | pending | T-09 整体最终审查未到入口；健康缺口见 F06 |
| AC-06 | pending merge | T-10/T-11 未启动 |
| AC-07 | fail | T-01/T-03/T-12 必须修复 F01..06；V-05 正常/负控总数不覆盖反例 |
| AC-08 | fail / needs_fix | T-13 本轮不勾选，无阶段 pass |
| AC-09 | pending merge | T-14 未启动，无下游发布放行 |

## 规范增量核对

当前 canonical 目标版本/哈希见 baseline.json；新增 `auto-ui-tools/project.md`、`tool-protocol/project.md` 当前均不存在，属于 add，不是已发布模块。

| delta | 本阶段审查结果 |
|---|---|
| SD-01 `docs/specs/auto-ui-tools/project.md` | A 只可陈述真实窄 host/资产登记及诚实 checked/not_checked；不得沉淀完整生成/资源 owner 已迁。候选范围描述正确，当前行为缺口须修后再认可 |
| SD-02 `docs/specs/tool-protocol/project.md` | 冻结候选 §4，版本/stdio/退出规则有真实证据；无损、严格字段、可传输包络等 F01..05 尚未成立。预算单位源码明确排除4字节前缀，contract-baseline 的“含前缀后载荷”措辞须统一为 payload bytes，不借修改 Spec 掩盖反例 |
| SD-03 `docs/specs/auto-man/project.md` | B；本轮不退役旧算法，不宣称管理服务脱 UI |
| SD-04 `docs/specs/project-model/project.md` | A 会新增 wire feature/可选 serde_json 与同源 codec；需在修复后的 A 子集草案明确 wire=1、支持与拒绝数据、嵌套准入，不改静态扫描器支持域。候选只列 SD-02，不能让 merge owner隐式发明 SD-04 规则 |
| SD-05 `docs/specs/auto-lang/ui/overview.md` | B；未消费 814 出口，无本轮已交付 UI 主体知识可沉淀 |
| SD-06 `docs/specs/repository-boundaries/generated-ui-distribution.md` | A 新 binary/协议来源只可按阶段实际发布；792 UI发行闭包尚未触及，B 迁移仍 pending |

保留原 frontmatter 的全部真实 repo-relative 路径与 GOAL-003/007（均存在），不因阶段范围删除 B 元数据。本轮未发布 canonical/ledger。修复交接应给精确 A Spec 子集（SD-01/02/04/06 中实际成立部分），审查绑定其新冻结哈希。

## 回放与下一步

同一树实施者一次处理六项，补永久定向测试并提交新 SHA，再送 T-13 受影响面复审。共享反例不能只闭最早的单点：F01 要覆盖 origin payload/路径；F02 要各层 shape；F03 要每个窄化/非有限数及 typed response；F04 要根/路径角色/cwd；F05 要两读路径与写端；F06 要所有新测试目标。若调整支持域，先记录合同修订，不把规则缩减写成完成证据。

回放（源码未动时，在原实现树工作目录执行）：

```text
cargo run --offline --manifest-path D:/autostack/auto-lang/docs/plans/attachments/815/review-r1/probe/Cargo.toml --target-dir target/plan815-review-probe -- D:/autostack/auto-lang/docs/plans/attachments/815/review-r1/seed.json
python D:/autostack/auto-lang/docs/plans/attachments/815/review-r1/process_probes.py D:/autostack/.wt/lang-815/auto-lang/target/debug/auto-ui-tool.exe D:/autostack/auto-lang/docs/plans/attachments/815/review-r1/seed.json
```

probe 的锁内容另存 probe-lock.txt；目标引用原树便于 work 原样复现，未来历史复查时以 baseline 的提交和输入哈希恢复来源，不能只跑某个陈旧二进制。Python 每个进程有10秒截止；所有临时项目均由 TemporaryDirectory 回收。

Plan 保持 executing；重开 T-01（只预算/包络冻结部分）、T-03、T-12，保留 T-02 完成及其余已验证证据，current_step=1/14；T-13 保持未完成。B 项按原依赖 pending，不因本轮再造 blocker。

pending_closeout_gates：阶段审查 pass 后，T-14 串行冻结裸 cargo t + 必要新包/协议/外部消费门；本次 diff 未触 VM/trans/UI生成，暂无 tv/tt/tu 触发；实际合入若变动重新判定。tf 仅主检出到期单实例批量。所有 gate/发布/Spec/ledger收据仍 pending；无合入、归档或清树授权动作发生。

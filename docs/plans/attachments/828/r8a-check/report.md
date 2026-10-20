# PLAN-828 r8a 分段映射独立核查

2026-10-21，**needs_fix**。上轮 F04-o/p/q 的具体组合和 186 格矩阵全部通过；新分段表示在“修改空白、保留 mint token”路径上取得实质修复，但不能据此关闭完整来源不变量。等价 token 重写、历史返回和恢复历史初始化仍有缺口，其中字节相同的 token 重写后继续插入已证实为本轮修复回归。

受审 `e2fc8edc3226cdd57698f716fa342c4c1367ed34`，基面 `099766b83d998b3fae8d41f01fb5415645288f5e`，工作树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`；Plan revision 2、executing、0/11。review_scope=final 中的用户指定受影响 finding/就绪核查，未启动整份计划最终验收。reviewer 为同一 Codex `/root` 审查上下文，未实施 r8a，未另开 agent。使用 auto-plan-review；实施代码、主计划/KNOWN-DEBT、canonical、ledger 均未改。

## 重放与控制

全部构建/测试 cwd 为上述实施树。C 盘 r7a-check 原始输入直接复制为 [original-replay.rs](original-replay.rs)，SHA256 `8d5938acd03ebd5149236e3cdac449247a8f37a9be7624cebcb57c7395c579ce`，与上轮完全一致。版本、输入、规范、运行库和结果哈希见 [baseline](baseline.json.txt)。

| 检查 | 结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 永久 36/36，0 ignored，1.77s，exit0；[raw](permanent-suite-output.txt) |
| 同 manifest `--test function_incremental_828 -- --test-threads=1` | 原 17/17，0 ignored，12.79s，exit0；[raw](original-suite-output.txt) |
| 同 manifest `-p auto-av-core -- --test-threads=1` | core 22/22，0.12s，exit0；另 doc-tests 0 不计覆盖；[raw](core-suite-output.txt) |
| 上轮独立 39 例原样重放 | 39/39，1.89s，exit0，内含原 186 格矩阵全过；[raw](original-replay-output.txt) |
| 扩展公开会话/快照检查 | **40过/5失败**，0 ignored，13.18s，exit101；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 新三步空白区间矩阵 | 528/528；是扩展 runner 中一个 test 的子例，不与 test 数相加 |
| 历史 helper 对照 | 同一输入 r7a 得到正确 `78..83`，r8a 得到 `70..83`，当前断言 exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt) |
| 六个修改 Rust 文件 rustfmt --check | exit1，仅 core/lib.rs 的旧 pub use 换行；基面同文件检查也 exit1；[raw](fmt-output.txt)、[基面输入](base-core-lib.rs)、[基面输出](base-fmt-output.txt) |
| `git diff --check 099766b83d998b3fae8d41f01fb5415645288f5e HEAD` | exit2，新提交的 r7a 旧日志 EOF 空行；[raw](diff-check-output.txt) |

扩展共六个新增 test：三条等价 token/历史来源反例失败、一条恢复映射元数据反例失败、一条恢复实际运行在编辑前即因 span=None 失败，另一个 528 组合矩阵通过。不能把五个失败全部归为本轮引入的五个独立根因。首次扩展 44 例的原始结果也保留：[first raw](first-extended-replay-output.txt)；为避免恢复的早期断言遮蔽初始化路径，随后单独增加独立映射检查，未弱化或删除原失败例。

实际覆盖原三类修复：新增文本整段替换为非空产物、跨新增/原生文本的替换、扩长产物内部及尾端删除、首尾修剪后继续插入/撤销、两处插入的覆盖替换，均通过。原包的旧 snapshot、暂停/resume、递归、intrinsic/授权正反控等仍通过。原 17 例中的进程内时延门本轮通过，不替代 F12 正式 worker/IPC/双线内存门或抹除历史失败。

## F04-r / P1：等价重写的 token 失去来源背衬

位置：`experimental/ac-av-probe/core/src/span_map.rs:43–66` 的 `apply` 与 `75–120` 的 fallback 映射；消费点为 `probe/src/session.rs:1069–1075` 内容命中复用和 `1162–1175` 提交应用。

当前表只保存“仍活着的原始 mint 字节”。被编辑区间覆盖的 mint 字节一律丢弃，替换产物都是无背衬 blob。这足以映射仍然存活的 token，但语义等价的新 token、内容 A→B→A 重引入的 token 仍可能使用旧 FunctionCode，其指令 span 仍引用已被丢弃的 mint 字节。对这些位置，`map()` 回退到前方存活分段的结尾，无法找出本轮真实 token；`map_token_end` 还可能把替换后尾空白纳入表达式。

以下均使用原 TRAP_SRC：

```text
fn helper() int { return 1 }
fn main() int { var x = helper(); return 1 / 0 }
fn tail() int { return 9 }
```

原 trap 为 func#1 `70..75`。

| test / 确切流程 | 实际 → 冷编 |
|---|---|
| `equivalent_expression_replacement_tracks_new_token_coordinates`：以 `  1 / 0  ` 替换当前 `[70,75)` 的 `1 / 0`；断言 edit 成功及 reuse_hit>0；独立 pinned 旧快照仍保持原来源 | `70..79 → 72..77` |
| `identical_token_replacement_then_whitespace_insert_keeps_token_anchor`：先将 `[70,71)` 的 `1` 原字节替换成 `1`；断言源码精确不变、该轮 trap 与冷编相同；再在 70 插入 8 空格 | `70..83 → 78..83` |
| `content_history_return_with_new_trivia_preserves_mint_token_correspondence`：`1 / 0` 的首个 `1` 换成 `2`（新 code 与冷编通过），再把 `2` 换成 `   1`，assert reuse_hit>0 证明返回历史内容路径 | `70..78 → 73..78` |

第一例真实 runtime 表只剩 main 两侧 `[(29,70,29),(75,77,79)]`；原指令需要的整个表达式背衬被移除。第二例只丢了首 token 的 `[70,71)`，后续前置插入再也不能推动它的 start。

**修复回归分类**：第二例的事件序列通过 [历史 helper 对照](historical-map-comparison.rs) 重放上轮冻结 `ProgramSnapshot::compose_region/map_span`，得到正确 `78..83`；新 `SpanSegments` 同序列错误为 `70..83`。这是 helper 层历史对照，加上当前公开会话端到端失败；未伪称历史完整会话二进制已重编执行。其余两例是本轮补测揭示的既有合同缺口，不仅凭“新测例”认定新引入。

修复需为复用代码所引用的来源位置维持当前语义 token/表达式对应关系，包含重新引入的等价文本。不能仅保留原字节存活关系；只对字节相同 edit 增加 no-op 特判也不能修好第一和第三例。需继续保持未改代码 payload/身份、当前来源正确及旧 snapshot 自身坐标。关联 AC05、V02/05，T02/T07，沿用 F-828-R1-04。

## F04-s / P2：恢复历史条目仍初始化为空表

位置：`probe/src/session.rs:566–576`。`warmup` 已为每函数及其 history seed 设置 `identity_range(d0,d1,d0)`；`restore` 替换 seed 时却写回 `SpanSegments::default()`，且其注释所说的后续 origin pass 实际不存在。新表示的空表 `apply` 不会生成分段，与旧空事件表可累加 delta 的语义不同。

独立 metadata 反例 `restored_history_seed_map_tracks_first_equivalent_edit_independently_of_spanless_payload`：

1. 在独立临时 store 上 `warmup_persisted(TRAP_SRC)`，drop，再真实 `restore`，断言 `restore_units_admitted==3`。
2. 在 main 的 return 前插入 32 空格；edit 成功，reuse_hit>0。
3. 冷编当前源得到 `102..107`。公开当前 snapshot 的 main 映射表却为 `SpanSegments { segs: [] }`，对已知 mint `[70,75)` 输出仍是 `(70,75)`；其他未改函数的初始化分段会正常平移。

这是**真实 restore/edit 路径后的公开 snapshot 元数据断言**。当前持久化 payload 的另一来源缺口会使 runtime 没有 span，故本报告不把它描述成已经观察到的 runtime `70..75` trap。需要在恢复代码和 history seed 之间初始化正确 mint 范围/来源映射，内容命中不能用空 seed 覆盖可用的驻留映射；同时与恢复实际来源修复一起验证。关联 AC05/06，V02/07，T07；新分段表示迁移遗漏，按 P2 单列，不能当 F04 已闭的证据。

## 恢复实际运行的独立阻断：span=None

`restored_same_content_edit_initializes_nonempty_source_alignment` 在上述真实 restore 之后、尚未编辑时，就得到 `Trap { kind: DivByZero, span: None }`，与冷编带 `70..75` 的结果不同。因此原 test 未到达后续编辑断言；本轮没有把它误计为一次“已执行但位置错”的插入案例。

另一个独立 metadata test 专门打印该初始实际结果并继续检查 F04-s，两者证据区分保存。该事实位于既有开放的持久化/manifest/来源恢复工作面（F10/F11 及 F04 的来源验收），未做历史完整 restore 运行对比，不归为本次分段更换新引入。`warmup_persisted` 本来保存 spanless ABC，正式来源恢复仍需完成，不能凭已有返回数值的 restore 正控宣称来源正确。既有 F11 保持 OPEN；修完空表并不自动补出 payload 缺失的来源。

## 新结构的组合覆盖与诊断调整

新三步矩阵基线为 `return` 后四空格。首步为“前插8 / 后插8 / 4换12”三选一；第二步为当前 gap 的 `(1,11,3)/(0,12,4)/(4,8,0)/(6,6,5)`；第三步从 `{0,1,len/2,len-1,len}` 去重边界取所有半开区间、替换长度 `{0,1,4}`，保留至少一个合法分隔空格。总 528 条路径，每步均真实 edit、冷编并比较来源，全部通过。这支持“存活 token 周围空白区间手术”修复，而非任意等价重写也已正确。

本轮诊断不是否认分段方案：旧事件的 current/mint 差值与 trim 吸收错误已由结构变化解决。新缺口在“字节存活”与“复用指令的语义来源对应”之间，以及恢复初始化的所有生产入口。此前矩阵只有空白区间，没有覆盖删除/重引入 token；新结构上线后必须补这类路径，不能仅以旧 186 格全绿覆盖替换后的新状态。下一轮应同时重放原39、三条 token 反例、恢复两条独立断言和新528矩阵，保留各失败阶段及历史对照。

## 当前验收与记录

AC05 来源子项 FAIL，V02 FAIL，V05 本轮已测 pinned/resume 子项 PASS，V07 恢复来源子项 FAIL；T02/T07 不得关闭。其余 AC 保持历史状态，未授本轮整项 PASS；AC09 needs_fix、AC10 pending。

Plan 当前 executing 0/11，F01 真函数级前端、F02 安装 payload、F05b 取消/关闭、F09 资源、F10 A07/依赖/GC、F11 changed-only 持久化与 manifest、F12 正式 worker/IPC/内存、F14 文档/SD 等 OPEN 不变。冻结 [Plan](plan-input.md) 与 [SD-01..06 表](spec-delta-input.md)，Spec 输入哈希见 baseline；未完整验收 SD，未发布规范或 ledger。

实施树测试前后 clean，HEAD 固定，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。已有三项 probe warning（SpanInfo、old_src、context_revision）及 shim warning 保留于 raw；lib.rs 旧格式差异与旧日志空行仍属 F14 健康面，未改历史日志规避检查。未执行整档 t/tv/tt/tb/tu/taa/tf、docs_gen、正式 IPC/资源门；最终审查通过后裸 t+tv 及必要 consumer/触面门仍为 merge-owned pending。

主检出属于共享并行活动，本轮不触碰其簿记；结果完整保存在本 C 盘目录，待安全回写 Plan §9 的记录见 [pending-bookkeeping](pending-bookkeeping.md)。next：原树 work 修 F04-r/s 并完成既有恢复来源面，提交后再核受影响路径；未合入或归档。

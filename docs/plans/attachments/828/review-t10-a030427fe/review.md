# PLAN-828 T-10 整份最终验收

2026-10-22；review_scope=final；**needs_fix，仅 T10-H01/P3 格式卫生缺口**。功能/信任/存储/资源/正式双门及 SD 审查未发现新的阻断缺陷；不能把未通过的必需格式检查记为最终 PASS。

## 冻结与边界

- Plan r3；同树 `D:/autostack/.wt/lang-828/auto-lang`、`plan-828-dev`，冻结 `a030427fe15dc043871d3253101f209653cabd9c`；整份 diff 基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`（master merge-base，与最初实施基面一致）。
- 入口/结束 clean、HEAD 未变，521 项输入与依赖一致；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。来源 baseline/final-state、工具版本、交付文档与 frozen Plan/interface 均保留。主检出无关 WIP 未触碰。
- 本 Codex 审核上下文未实施本冻结修复、未启代理，不编造未暴露底层身份。实际读取主 Plan、完整 AC/V/C/任务/SD、主要生产接线/失败出口及断言；本轮无实现写入、无根整档、无合入/归档/清树。

## 唯一待修：T10-H01（P3，必需健康检查）

既有规则：AGENTS 独立审核 Health Check 要求格式清洁；T-09 负责告警/文档收口，T-10 必须核整份冻结实现。

同一冻结根，执行：

```powershell
rustfmt --check --edition 2021 --config skip_children=true experimental/ac-syntax/adapters/core/src/projection_engine.rs experimental/ac-syntax/adapters/core/tests/scalar_projection_.rs
```

预期 exit0；实际 exit1。`projection_engine.rs:726` 的 has_bang 换行，`scalar_projection_.rs:36/80/89/117/223` 等新测试排版不合当前 formatter。它们是 828 新增/修改行，不是已登记 shim 既有格式债。完整整份修改 Rust 文件扫描只这两文件红，见 rustfmt.log/fmt-files.json。

原因已确证到覆盖边界：此前窄轮只扫当轮文件，不能代表整份新代码格式清洁；没有据此推断功能错误。修复限定为两文件 rustfmt 的空白排版，勿改语义/测试 oracle/历史 raw；`format-closure.json` 和两个 `.formatted.txt` 给出从本冻结源直接生成的精确期望。无需重新设计已闭机制。

关闭：两文件格式化后，对全份实际修改 Rust 文件复验 exit0；源/脚本差异卫生 exit0；scalar_projection_ 6/6；确认除了两文件格式无语义差异后新提交冻结，交窄复审。无需重跑三512、正式30轮或整档；旧证据复用须显式验证代码等价、依赖/配置/输入未变。若夹带语义改动，按实际影响重新验收。

## 本轮独立实测

| 面 | 实际结果 |
|---|---|
| 三项512（soak、结构/历史旗舰、完整持久化/GC/restart/sentinel） | **3/3，275.43s**，无 skip |
| auto-av-core / probe lib 真实 linkErr 组件 | 22/22、1/1 |
| 798 cache/MIR/ABC/binding/vertical/native-admission/native-trace/case | 20/8/15/4/3/11/1/5，全过 |
| 798 resources（含32MiB产品压力/GC、双线负控） | 17/17 |
| ac-core hir_verify/scalar_hir/source_frontend/source_hir/scalar_source/text_source/record_source/text_record_source | 9/16/14/12/7/10/22/21，全过（111） |
| scalar_projection_ | 6/6 |
| r23a close/adjacent 原控当前库 | **9/9**，87 项明确 filtered，非零选择 |
| 生产构建 warnings | 上述定向构建零告警；非全仓告警普查 |
| source diff-check / 整份修改 Rust 格式 | exit0 / **exit1，仅上述两文件** |

512 完整存储轴每轮独立值正确、单函数安装、每64轮实际GC、两次171/342重启后全部115单元重验；水位≤初始闭包+8、soft roots≤函数数、history有界、用户哨兵字节不变。旧rev0 pin贯穿保活并在末尾实际执行正确，释放后Weak死亡；不声称每一修订都重新执行了旧pin。

整份 diff-check 的其他红行全部是历史审核报告原始输入尾空白或保留 raw 的末尾空行，不能为过格式检查改写历史证据；当前源码/脚本 diff-check 独立 exit0。

## 同冻结证据复用（明确非本轮重跑）

`evidence-reuse.json`：与 C04 R5 同 HEAD/依赖，**518 个共同源码/配置/Spec输入完全相同、137 个运行产物 hash 相同、92 份证据 hash 相同**；Plan新增收据不改变 r3 验收。复用 `../review-c04-r5-a030427fe/` 中：

- permanent mig9/t078/b214/b111/session18（当轮skip三512，已由本轮3/3补齐）/contract22/t01r4/r186；每项日志及退出码明确。
- 真旧全Program pipeline 同门红、实际 run 内深复制与真实 AV 执行、目录缺观测、后置副本 seq、合法内容命中：judge6/consumer2/identity3/四入口旧pin trace1 等正反控保持。
- 握手4、逻辑接受字节903/no-store2、逐写真实失败4（0/173/425/497全部保留），失败报告落盘规则及完整数据流核对。
- 官方 T08 **5/5，一次调用**：30样本，P50/P95/P99=212.6478/219.4917/221.9354ms，init1300.2778ms；全部值/修订/确切目标/未改身份/resident/seq/结构过；同798 A≤224MiB、B≤128MiB硬门过，worker峰值211775488B/parent22392832B，完整raw/report原文件留存。

不把 work 所报总数冒充独立重跑；不在512并行期间重新测战役。历史一次 work 战役失败未保存具体诊断，**归因仍未确证**；本次不删除记录、不补造日志、不以随后绿证明它不存在。后续测量须逐次独立目录留全部失败raw。该限制与最近同冻结独立正式门通过并存，不另新增功能 finding。

## 全合同核销与遗漏扫描

逐 AC/task 映射、V01..09、C01-01..05/C02-01..05/C05-01..04 见 acceptance-matrix.md；并核实际 gate 的有效/故障正反控与断言。T01..08 已完成；T09仅格式收尾未闭，T10因此不能PASS，T11未执行：Plan **executing 8/11**。AC01..07功能判据PASS；AC08功能/兼容/映射通过但T09健康收尾partial；AC09最终PASS未授；AC10仍 merge-owned pending。

查无新增未授权实现延期或静默whole fallback：受支持body路径只做一body，内容hit复用原对象；非快路径显式拒绝；实际 AV 单元派发无merged旁路；存储 changed-only、required闭包双指纹、有界HIR根与失败接受计量符合合同。可遍历接口目录/来源/绑定元数据，不将它误记为函数语义重编，也不宣称整个编辑O(1)。

coverage的错误恢复树span逐字节等价未承诺为当前消费面；非scalar/堆对象/CTE/活帧迁移、多场景扩样与产品接线仍在明确非目标内。798原域完整准入、旧消费者和正式one-shot保持。shim既有4 dead-code/广泛格式债不偷换为828新文件豁免。前批所有已闭finding保持，T10-H01不重开C04或候选/派发机制。

## 规范与下一步

SD01..06审查通过（proposed），精确审校文本及理由见 spec-delta-reviewed.md/spec-review.md；canonical/ledger未改。

下一责任 **work：仅 T10-H01 两文件格式化，提交新冻结后窄复审**。这不是新的功能开发轮；其他验收结论按等价性复用。通过后才由 review 授最终reviewed，再进入 T11。最终merge-owned裸cargo t+tv、实际consumer/触面、tf到期主检单实例、规范ledger沉淀、归档guard均pending；当前不得合入或宣称D828交付。

# PLAN-815 T-09 全计划终审 R2（F01修复受影响范围）

2026-10-22，**final pass**。**F-815-T09-01 / P2 CLOSED**，无新增 finding。按R1已审完整AC/SD及本轮受影响重审合并裁定，T06/T07/T08复勾，T09完成；Plan经 execution_done14/17→**reviewed15/17**。**T10/T11仍pending，尚未合入或正式发布M815-B。**

## 冻结和独立性

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`；reviewed HEAD **ca9e89b44fea32c923297d943473191f435e3a37**，生产修复 **0f2858e35cb220964d8c23c8d77afce6839db491**，base **a67f114fda8abf52630126d4aa52eeacd6d3248a**。Plan r3，M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。
- reviewer=Codex `/root` 延续独立复审上下文，有早期合同/账本参与限制、未参与当期实现。未创建agent/第二实现树，未修生产代码或外仓、未发布canonical/live ledger、未合入/归档/清树。现成回放只在唯一树target私有目录生成/检查，不执行会写tracked入库位的regen测试。
- [baseline](baseline.json.txt)、[终核](final-baseline.json.txt)：同HEAD clean，5943项源/依赖/配置输入及受审Spec起止无漂移。相对R1仅rust_ui的shell生成臂和入库shell-pack两处源文件变；协议、运行、UI compiler、资源、依赖清单/锁/验证配置均未变。七项Spec与I-PROJECT设计相对R1仍未变。

## F-815-T09-01关闭表

| 关闭条件 | 本轮独立观察 |
|---|---|
| canonical源与原生依赖清单同步 | 壳层头两处、再导出、shell_write/shell_write_vec Value均发canonical；原生Cargo包含auto-ui-runtime（ui-iced）+auto-lang-core，生成库零auto_lang::。无删入口、改输入或新增运行模型。 |
| 使用相同冻结五件输入 | live auto-os/shell与R1冻结副本逐SHA256全部一致，[身份表](input-identity.json.txt)。新私有fixture从冻结副本实际调用公开generate_shell_pack_lib_sources，不借用work修改过的旧consumer清单。 |
| 原样生成→真实消费，零补依赖 | 生成库**97,221B**，SHA256 **e8323aa7934f5577db2a73188f4cf10d47d44d31573b041576477288eeafc871**；原生/布局调整后的依赖键集合完全相同，无人为增加依赖；消费check **exit0**，随后 **--locked exit0**。原107个E0433清零。 |
| 原生源/清单与入库内容一致 | 库和Cargo均与当前入库文件**逐字节相同**，[原生输出对账](native-output-identity.json.txt)。快照净两行改变是当前已发布转译器对字符串JSON投影的既有输出差异；本轮并未改UI compiler。这里以真实再生等价证明，不仅采信“sanctioned”执行叙述。 |
| freshness有效及入库正控 | 原测试体逐字不变，定向freshness **1/1，0.71s**，未skip/ignore/放宽；入库 `cargo check --offline -p shell-pack --lib` **exit0**。 |
| 刷新T08实际源/产物 | handoff新生产冻结位0f2858e35、docs ca9e89b44；新release精确hash/大小/version及四组实际消费均核验，下节；旧1626…明确废止。 |

原始证据：[generate](generate.log.txt)、[native consumer](native-consumer.log.txt)、[locked consumer](locked-consumer.log.txt)、[freshness](freshness.log.txt)、[committed check](committed-check.log.txt)、[consumer结果](consumer-results.json.txt)、[生成源](generated-shell-lib.rs.txt)、[原生Cargo](generated-shell-Cargo.toml.txt)、[消费Cargo](consumer-Cargo.toml.txt)。等价布局调整仍只有本地path定位、serde_json workspace值物化、既有iced_widget patch与独立workspace，不改变依赖集合。

新fixture位置见[probe-root](probe-root.txt)，源码/manifest为[replay.rs](replay.rs)/[replay-Cargo.toml](replay-Cargo.toml.txt)。保留R1冻结输入与旧红/正控为共享对照，不以删除旧失败记录掩盖修复。

## 受影响邻接、新参考artifact及复用

- [scope-and-reuse](scope-and-reuse.json.txt)确认rust_ui的shell段前/后逐字不变；gen_shell_state_access只有shell生成器调用。app/dev/792生成主体、请求读取缝、owned/捕获/会话、协议/wire/预算/能力和资源均未改。旧33/33发行、377资源、真实生命周期/兼容、四回执/raw深度/Linux与字段矩阵证据按原范围复用；不是新冻结树再次执行33个冷构建的声明。新source bundle的实际producer/manifest身份由T10收尾据实绑定。
- 新release参考 **a73b117ff384bb347911e9a1c105bbd46146244626766e10c0368d52cb572781**，**85,116,928B**，`--version`=auto-ui-tool0.1.0。精确文件执行[四组probe](release-probe.json.txt) **4/4**：五操作及cancellable/interactive位、合法validate、协议99 generate零落盘且同流可用、请求压过盘名并真实Vue生成/嵌入button逐字节同源，EOF exit0。[日志](release-probe.log.txt)。未重跑release编译或承诺逐字节可复现。
- rust_ui手写源rustfmt --check、diff --check通过；生成snapshot以原生再生逐字节相等验证，不手工格式化改变字节oracle。编译仅显示既有lib22/runtime235/shell3诊断，本修复无新增未处理诊断；[卫生记录](hygiene.json.txt)。本轮未跑全lib、cargo t/tu等整档，也未把work182/184当成本轮独立总数。

## AC与SD最终裁定

| AC | 最终结果 |
|---|---|
| AC01 | **pass**。R1确认的脱Man依赖/唯一owner/旧门面保持；最后shell公开生成消费缺口本次闭合。 |
| AC02 | **pass**。R1单源/适配器、相对根、字段/主声明冲突与移除证据不受shell修复影响，保持原支持边界。 |
| AC03 | **pass**。协议、生命周期、日志/控制/输入/取消/退出及双平台reader原独立证据保持，新release亦核对能力和真实生成通道。 |
| AC04 | **pass**。原发行/consumer/资源/失败控制保持，加本轮shell原生清单真实消费与locked正控、有效freshness和刷新身份；777 master并集是已批准T10门，仍pending。 |
| AC05 | **pass**。R1完整终审+本轮受影响闭合，当前无未决finding，SD逐项审定，merge-owned门清楚保留。 |
| AC06 / AC07..12 | AC06的B收尾**pending**；A/L历史发布/恢复已审范围保持，不伪造重新执行。 |

[SD01..07已审增量](spec-delta-reviewed.md)绑定[frozen-handoff](frozen-handoff.md)。七目标保留A阶段既有内容，在B受管增补；本轮未直接沉淀canonical或ledger，plan_revision不变。

## 旧债务归类与交接文字纠正

R1对**plan734**作的是基于原始生成/fixture文本、rustfmt归一及814提交来源的“排版金样卫生债”裁定，精确断言仍红，不报绿。**plan593 CSS值金样不是plan734同族，也未被该裁定覆盖**：work fixes.md/提交摘要中的相应归类不准确，本报告明确不采纳该推论。plan593与merged_api仍按P751-D2既有独立开放分项留账；本修复没有改变相应源、测试、CSS或API生成行为，也不据该错误标签放宽任何验收。R1其余未变通过范围保留，不把“余二红”改写成已清偿或全包全绿；T10若运行有关门遇红必须按实际来源归因。

## 状态和下一步

F01关闭后恢复T06/T07/T08，核正work-owned全部完成进入execution_done14/17；T09最终pass勾选后转**reviewed15/17**。这不是merge或M815-B交付完成。

**next=T10**：在最终冻结提交统一跑裸cargo t+UI触面tu及其他实际触面/必要包门、777 master并集和对应发行身份核验，再串行实际合入master；任何需改代码的门禁失败返回work并重审受影响内容。T11随后受管Spec/ledger、发布收据、归档和guard清树。到期批量tf按既有主检出单实例规则另行判定。T10/T11尚未勾选，本轮未提前执行整档、合入、发布、归档或清树。

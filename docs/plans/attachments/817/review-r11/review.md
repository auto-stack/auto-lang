# PLAN-817 R11 final review — needs_fix

2026-10-24；Plan r3；review_scope=final；第11次 final review。当前 Codex 审查上下文独立于实施，未暴露模型/session ID。核对主检出当前 Plan，不使用实现树内旧 r2 计划副本。实现树 `D:/autostack/.wt/lang-817/auto-lang`、分支 `plan-817-dev`、冻结提交 `b29cb23482e6f418138bb73c99595fb2f1aac71e`，修复基面 `e55b59cae`，整体 base `d46e03e8a4986f3d5fed5bd09665c2aeebf0340d`。起止 HEAD 相同、树 clean。没有修改实现。

## R10 finding closure

| Finding | 独立结果 |
|---|---|
| F-817-R10-01 | 原空串数组反例关闭，完整 decode 预算不变量仍 partial，见 R11-01 |
| F-817-R10-02 | closed：真实 AV envelope 经 reserved encode，len=capacity=grant=1163B；最大单次分配1163B。永久回归的容量断言和增长型 encoder 红控通过 |
| F-817-R10-03 | closed：begin_window 同时重置历史峰值/基线，window_readout 不再改基线；内容主导100000B场景的旧读取算法反控与新算法正控通过，原8192B宽松容差造成的不可区分问题已调整 |

R10-01 新审计确实先于 from_slice，成功扫描零分配；空串×10000获1766570B额度，实际decode峰值394094B，在界内；空串×100000/300000分别算出17606570/52806570B，被真实driver的8MiB Hot门拒绝，未触发原3MiB/12MiB解码分配。正常原envelope hit、ct_execute=0。承认这些修复，不重开读取、SQL、编码和观测器的已关闭问题。

## F-817-R11-01 / P1 / AC-05,08 / T-05,T-08：审计的2S未覆盖转义scratch与owned String共存

**原规则与边界**：Plan §3/§5.4、contract-decision §7和R10-01关闭oracle要求编解码前取得覆盖实际容器/瞬态共存的额度，并按分区计量；坏schema/嵌入事实的拒绝路径同样有界。本项是R10-01完整关闭条件的剩余缺口，不是增加语言能力、接受坏缓存或扩大输入规模。

**位置**：`experimental/ac-cte-probe/probe/src/store.rs:242–244,409–412`，driver将审计数用于Hot预留（`driver.rs:685–701`）。

`audit_envelope_budget` 将字符串预算写成2×编码内容S，依据是“字符串字节一次分配，×2覆盖scratch”。但带转义的字符串经过serde_json scratch增长，再由String visitor复制为owned String；scratch可能已接近2×字符串长度，并与1×owned结果同时存活，2S不足。

**已核实原因**：本树锁定serde_json 1.0.154。其 `SliceRead::parse_str_bytes` 在遇转义和结束引号处 `scratch.extend_from_slice`，返回 `Reference::Copied`；serde_core 1.0.229 `StringVisitor::visit_str` 执行 `v.to_owned()`。不是仅凭注释猜测。allocator独立窗口也实测到该共存峰值，最大单次值反而仍在审计额度内，因此只看single指标会漏检。

### 可重复真实输入与生产路径

`allocation-probe.rs`：先用正常driver/AV产出envelope，保留request identity及其余字段；schema设foreign-schema，source_fp设置为 `"x"×N + 换行 + "x"×N`，由serde_json正常编码（单个`\n`转义）。使用公共put在新隔离store保存正确frame/CAS载体，未改文件/数据库或产品实现。随后真实CteDriver恢复同一请求，审计→预留Hot→from_slice→schema拒绝。所有本反例页均小于512KiB；没有增加输入界。

`frozen-replay/result.txt`：

| N | JSON载荷B | 审计Hot额度B | decode最大单次B | decode实际共存峰值B |
|---:|---:|---:|---:|---:|
| 32768 | 66630 | 137708 | 131072 | 197387 |
| 65536 | 132166 | 268780 | 262144 | 393995 |
| 131072 | 263238 | 530924 | 524288 | 787211 |

例如N=65536，长字符串内容131073B，scratch容量增长到262144B，owned字符串与其共存，decode窗口共存393995B > 先行Hot额度268780B。payload在窗口开始前已存在，**decode峰值不包括payload分配**，所以不是把载荷重复归入Hot造成的假红。

三档去掉唯一转义、保持相同解码内容长度的plain反控，decode峰值分别66454/131990/263062B，均在各自审计额度内；正常原信封也通过。三档审计自身均零分配。非法JSON和65层JSON的审计控制均受控拒绝（错误消息小额分配独立记录，不把错误路径声称为零分配）。

**driver证据及限制**：默认cap的 `probe-run/result.txt` 已复现同样审计放行、最终schema Diag且managed_met=true。正式 `frozen-replay` 进一步使用现有测试seam，将Hot cap设成审计额度（其余分区维持默认），仍得到同样结果。不是只离线调用了无需预算的decode convenience API。

本项精确证明**decode/Hot的先行额度不足**。本次默认请求的整个managed台账还持有frame/copy的Staging预留，整个driver峰值未超过台账总峰值；没有据此声称默认32MiB总门、384MiB进程门或512MiB树门已实测超限。实现当前明确把decode额度归Hot，且模型声称包含scratch，不能未经归属/寿命证明把另一分区已释放frame的额度默认为scratch授信。若修复采用明确的scratch分区/额度转移，须证明分配前授权与共存期间归属，而不是只改注释。

### 关闭oracle

建立覆盖实际serde路径的预算模型，分别计算容器容量、逃逸字符串scratch、owned结果及必要瞬态共存，先于分配授信；或采用明确受限的解码机制。保留本次三档escaped/plain配对、R10空串数组、正常多成员manifest、错schema/identity及非法输入控制。测量窗口须包含scratch与owned同时存活，证明实际峰值≤其授权额度；不足额度时先拒绝，失败不发布、不破坏旧条目。核查其他String字段共用此机制的边界，不能只给source_fp样例打补丁或无依据扩大倍数。

## 验证与证据

- PowerShell，在既有plan树运行；resources9项各1/1（decode_audit、allocator_sensitivity、capacity、corrupt_blob、sql、restore_order、oversized、encode、deadline_fast）；cache6/6、freeze7/7，共22项通过，无告警。
- 变更的store/driver/resources测试三个Rust文件rustfmt --check通过，git diff --check通过；release lib构建成功。
- `replay.ps1` 通过本次cargo JSON artifacts选取精确rlib/native路径，编译并运行独立allocator探针。`frozen-replay/result.txt`为最终记录，store下JSON为可直接复用的实际输入；源码/输入/hash见 `baseline.json`。
- 没有跑整档t/tv/tt/tb/tu/taa，没有重跑未变GC/32MiB/512修订/六场景性能重族。依赖树及lock与R10一致；其他未变语义/增量证据只保留原输入、依赖、配置和提交适用边界，不把22项定向当整体交付通过。
- 当前主检出Plan与四个Spec只读核对，完整hash在baseline；Spec delta冻结SHA256 `2C3CD34522EDA02C6BAB5E249BC4537B6BEB4A30CC1741F35D2D0CF045A09C5C`。SD-02的新2S模型仍不能通过规范审查；其余SD本轮无变更。

## AC与路由

AC-04普通缓存/冻结及旧读取具体缺陷复验通过；AC-05因R11-01为fail/partial，AC-08的SD-02为partial，AC-09未通过。AC-01/02/03/06/07/11未改领域的既有证据保留原适用范围，未重新宣称最终整体pass；AC-10是merge-owned pending。原Plan中的规范影响路径保持不变，touched_goals空集说明仍须在T-08终稿收口核定。

**next: work，同树修复R11-01后提交冻结再审。** 保持executing6/11，T-05/T-08/T-11/T-09/T-10开放，其他完成项保留；没有将合同内缺口延期成债务。最终review通过后才由merge执行裸cargo t+必要tv/probe消费面门禁。本轮未改实现、canonical Spec/ledger，未合入/归档。

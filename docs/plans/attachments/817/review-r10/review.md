# PLAN-817 R10 final review — needs_fix

2026-10-24；Plan r3；review_scope=final；当前 Codex 审查上下文独立于实施，未暴露模型/session ID。同一实现树 `D:/autostack/.wt/lang-817/auto-lang`，分支 `plan-817-dev`，冻结提交 `e55b59cae793abb3d551f4af0601f52994003108`，整体 base `d46e03e8a4986f3d5fed5bd09665c2aeebf0340d`，修复基面 `cb0ba5231`。起止 HEAD 一致、实现树 clean；未修改实现。本轮为第十次 final review，审查依据是主检出 Plan 与既有资源关闭 oracle，不因轮次或总测试数放宽。

## 上轮关闭情况

**F-817-R9-01 的读取缓冲区实际缺陷已关闭。** 新固定长度 `vec![0; real_len] + read_exact` 确实避免 read_to_end 扩容；driver 的 frame-only 读取 cap 与 payload copy 分账正确。独立探针跨九个尺寸（500/512/524/1023/1024/1032/2047/2048/2056B）读取真实公共 put 产物，检查实际 frame.capacity 与返回 payload.capacity，总内容缓冲恰等于两笔预留。例如 512B 载荷为 560+512=1072B。512B 对应的旧 1025B 近界文件及 2MiB 文件均在 frame=560B 的物理读取界拒绝。SQL 错误传播、普通缓存/冻结正反控继续通过。unused_mut 告警已清。

但是，既有 AC-05「编解码/坏块全路径先预留」尚未闭合：先前收据的保守 2× decode 假设和精确字节计数 encode 都仍低估实际容器容量。R9 新增永久回归自身也缺少有效的峰值/反控证明。以下均为原合同内问题，不增加语言支持面、输入界或性能规模。

## F-817-R10-01 / P1 / AC-05,08 / T-05,T-08：2× payload 不是 decode 分配上界

位置：`experimental/ac-cte-probe/probe/src/driver.rs:675–678`、`store.rs:244`。

driver 为恢复后的 envelope 预留 `payload_len * 2` Hot，随后 `serde_json::from_slice` 先完整构造 Vec/String，之后才检查 schema/identity。JSON 的短字符串数组可以用少量编码字节产生大 Vec；2× 并未约束这些实际分配。

**独立真实反例**：`allocation-probe.rs` 先经正常 CTE driver/AV 取得成功 envelope，保留 request identity，将 deps.closure 改成空字符串数组、schema 改为 foreign-schema。经公共 `ManagedStore::put_artifact` 存入全新隔离 store，同 layer/key、正确 framing/CAS，不改文件/SQLite/产品代码。再从真实 CteDriver 请求恢复。坏 envelope 本来应受控拒绝，同时满足拒绝前分配有界；不是要求接受它的内容。

`frozen-replay/result.txt`：

| case | 载荷字节 | 先行 Hot 额度 | 实际最大单次分配 | driver 台账峰值 | managed_target_met |
|---|---:|---:|---:|---:|---|
| 正常原 envelope | 1163 | 2326 | decode 窗口 288 | 5313 | true，hit/ct_execute=0 |
| closure 空串×10000 | 31146 | 62292 | 393216 | 125245 | true |
| closure 空串×100000 | 301146 | 602292 | 3145728 | 1205245 | true |
| closure 空串×300000 | 901146 | 1802292 | 12582912 | 3605245 | true |

其中 **301146B 小于原 512KiB 结果页界**，已足以证明额度不足；901146B 进一步覆盖当前 reader 实际允许的 1MiB 异质载体拒绝路径，单次分配甚至超过 Hot 分区 8MiB。两例最后都正确返回 infra/schema 错误，但此时大分配早已发生。不是将最终拒绝误判为错误接受，也没有声称发生了 OOM 或实际总内存超过 32MiB。

直接 decode 窗口与完整 driver 窗口都记录 allocator；较大例 driver 真实分配峰值 13491100B，台账只记 3605245B。证明缺陷在完整生产恢复路径，不仅是离线 serde 实验。永久测试与 Spec delta 中“保守 2×”不能作为该不变量的证据。

关闭 oracle：对所有可解码集合及错 schema/错 identity/错嵌入事实路径，以可靠的容器计量/受限解码在**分配前**取得覆盖额度；不能只在 serde 返回后检查集合长度或再提高任意倍数。保留上述小页反例、1MiB 读取界内异质反例、正常 hit、合法多成员 manifest，并实际测量最大分配与共存峰值，拒绝须先于超过授权的大分配；失败不能发布或破坏旧条目。记录覆盖边界及预算模型依据。

## F-817-R10-02 / P2 / AC-05,08 / T-05,T-08：encode 精确长度仍低估 Vec capacity

位置：`experimental/ac-cte-probe/probe/src/store.rs:310–319`（调用 `serialize_envelope` 的 serde_json::to_vec 在第297行）。

`count_envelope_bytes` 精确算出序列化**长度**，`serialize_envelope_reserved` 为这个长度预留后使用增长型 serde_json::to_vec。实际容量可翻倍，Alloc 事件却只写 payload.len。与 R9 的读取问题属于同一容量/额度不变量的另一条既有路径。

探针直接使用上述真实 AV 产出的正常 envelope，经生产 reserved-encode 入口，Staging cap 恰设为 count（同永久测试的 exactly-enough 正控）：

    ENCODE len=1163 capacity=2048 granted=1163 target_met=true
    max_single=2048 peak_delta=2223

该返回值真实持有 2048B 容量，授信仅1163B；台账仍绿。2223B 还包含额度事件自身的小额开销，不将其全部归给内容缓冲。当前永久测试仅断言 len==count 及手工事件先后，不能证明分配被覆盖。没有从该小例推定默认32MiB已经耗尽，也不假定 source projection 的已归属额度可无凭据重复借给编码。

关闭 oracle：让 count/先行预留覆盖编码实际容量及必要瞬态共存，或采用不会增长超过先行授信的编码方式；回归检查 capacity/allocator，保留不足额度提前拒绝、正常准确编码、2^n 邻域和发布失败保旧条目控制。修改后复核编码到 store 的同时持有边界，不以事件日志中的 len 替代真实分配。

## F-817-R10-03 / P2 / AC-05,09 / T-05,T-08：新增 allocator gate 放过旧错误算法

位置：`experimental/ac-cte-probe/probe/tests/cte_resources_tests.rs:67–80,1228,1291–1299`。

1. `reset_alloc_window` 没有重置 MAX_LIVE_BYTES；它仍是进程历史峰值。
2. 各 case 在读取后调用 reset_alloc_window 获取 single max，同时将 WINDOW_BASELINE 改成操作后的 live 值，然后才计算 live_peak_delta。因此该值不再是开始基线上的本次共存峰值；历史峰值还能造成顺序相关的误红。
3. 所有容量断言添加8192B INCIDENTAL，而九个 payload 最大2056B。所谓“single == framed / 共存≤两笔授信”实际只断言宽松上界，不能区分上轮已确认的容量超额。

独立探针运行**原来的 File.take(1072).read_to_end 算法**，560B frame 实际 capacity1024，再复制512B payload，内容共存1536>1072。把这些观测带入当前永久测试的实际断言，`single_test_pass=true coexist_test_pass=true`。这是对原错误算法和当前阈值的直接敏感性控制；没有冒称修改了实现后跑过整个测试族。原算法可以通过这些新阈值，所以该回归不足以防止已修复缺陷重新出现。

关闭 oracle：正确 reset/snapshot 本次窗口，取结果时保留开始基线；将内容缓冲额度与独立证明的固定杂项分开，选择能区分原错误的尺寸/断言；同一检查器对实际旧算法或等效受控超容量实现必须红、当前算法正常正控绿。保留同长损坏/近界增长/缩短正反控；不能仅提高 epsilon 消除历史峰值误红。

## 验证、证据和限制

所有编译/运行在既有 plan 树，通过 PowerShell。没有执行整档 t/tv/tt/tb/tu/taa，也未重跑未变的 GC/32MiB/512修订/周转重族。

- resources 定向7项，各1/1：capacity_exact、corrupt_blob、probe_sql_error、restore_order、oversized_restore、encode_order、deadline_fast。
- cache 6/6、freeze 7/7：合计20项定向测试通过，无告警。日志逐项保存在本目录。
- 三个变更 Rust 文件 rustfmt --check 通过；git diff --check 通过；release lib 构建成功。代码健康项通过不替代资源正确性。
- `replay.ps1` 可直接重建当前冻结库并重放独立探针。rlib 与 native 路径来自**本次 cargo JSON artifacts**，不按缓存文件时间猜库，不修改 workspace/依赖。allocator 窗口的起止/基线独立于产品测试实现。
- 正式最终复现为 `frozen-replay/result.txt`；输入是 `frozen-replay/store/envelope-{0,10000,100000,300000}.json`，并在该目录真实 store 中存取。此前 final-replay 为扩展小页反例前的探索记录，最终判断以 frozen-replay 为准。
- `baseline.json` 绑定完整 HEAD/base/依赖树/lock、当前主检出四个 Spec 与审查技能版本、输入和探针 hash。Spec delta 冻结 hash：`9992F13182B3E450B360495A208FE92D53D4BF84D7EC993BC5E4E12AA716705F`。

## AC、Spec 与下一步

| AC | 本轮状态/边界 |
|---|---|
| AC-01/02/03 | 本轮无相关语义改动；原首例/Meta/owner/deadline关闭证据保持原修订范围，deadline_fast复验通过；不重新宣称整体通过 |
| AC-04 | 读取/SQL具体旧缺陷关闭，cache/freeze13项复验通过；坏 envelope 资源准入仍受R10-01约束 |
| AC-05 | fail：R10-01/02，R9内容读取机制闭合，永久观测R10-03待修 |
| AC-06/07/11 | 本轮无bench/worker/session代码改动，旧性能/结构证据仅在原输入/依赖/配置等价范围保留；整体未重新授予pass，T-11保持开放 |
| AC-08 | partial：SD-02的2× decode和exact encode描述未被实现证明，须随修复校正，不能靠改规范放松原合同 |
| AC-09 | fail：本轮三个合同内finding开放 |
| AC-10 | merge-owned pending；未合入/归档 |

canonical Spec/ledger保持只读；既有 supersedes/new_spec_components 路径仍是对应落点。touched_goals 空集说明与最终SD/coverage收口由T-08复核，不在本轮借空集授予规范通过。

**next: work，同树修复并提交冻结。** 保持 executing、6/11，T-05/T-08/T-11/T-09/T-10开放，其他已完成任务保留；不把合同内失败延期成债务。merge-owned 裸cargo t+必要tv/probe消费面门禁仍pending，最终review pass后执行。下一次交接须提供编解码真实预算模型、上述复现和共用观测器红绿控制的关闭表，避免再按单条样例修补。

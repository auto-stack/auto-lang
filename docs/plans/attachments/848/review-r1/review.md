# PLAN-848 r2 — R1 review: needs_fix

stage: review; review_scope: final; outcome: needs_fix; 2026-10-25.

审查冻结提交 `cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d`，base
`bf22db3465203f3babd649494f4b37aa2a012027`，唯一树
`D:/autostack/.wt/lang-848/auto-lang` / `plan-848-dev`。SDK
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`、AGP、三锁不变。
代码提交及树clean在测试前后核验。Spec输入、锁、receipt和规范提案hash见
[baseline](baseline.json.txt)，提案冻结副本见[spec-delta](spec-delta.md)。

审查者是当前 Codex 会话；上下文包含此前实施历史，**不是新的独立会话**。
本轮按审查角色停止实现写入，从合同/实际代码/raw证据重建结论并另选反例。
不把角色名称或作者原自检当成上下文独立性，也不据此颁发最终pass；
本轮实际发现下述产品阻塞，结论直接退回work。未启动agent/chat。

## F-848-R1-01 — P1 OPEN：128节点预算依赖候选分布

类型：既有要求缺口，不是新增压力规模或新的CPU阈值。
影响 AC-05、T-02/T-05；T-08 readiness/AC-08覆盖声明随之重开。
违反r2 §5.1/5.5、V-06：128节点宽/深支持控制，两次group写，
2cast+10getter+6AABB=32credits，在原100000steps/120resumes/250ms内完成。
合同没有只支持等深候选、首轴即可剪枝或已有最近命中的数据限制。

### 真实生产入口及最小反例

[完整探针](review848_probe.rs.txt)，[Session原始结果](holdout.log.txt)，
[Host原始结果](host-holdout.log.txt)。探针临时放在同一树driver的
`tests/review848_probe.rs`，结束即删除；产品及永久测试文件没有修改。

正式Host探针：Document::empty → 1个3D group+127个真实builtin Cylinder，
全部visible、finite、同group、默认scale和角度 → Host::new →
SourceSnapshot::from_bytes_for(SegmentV3) → start_user_snapshot →
初始化完成Paused/tick0 → dispatch(Action::default) → frame/pump。
不是手造不可准入的Descriptor，也没有更换SDK或提高预算。

每个callback先把group.x写2再cast，随后写4再cast；两次线段均为
`(0,.5,0) → (800,.5,0)`。各cast后取hit/node/t/point.x/normal.x，最后
6次`ag_aabb_min(127,0)`，恰好32credits，无用户循环额外负载。

| 分布 | 127个叶子的local位置 | 期望 | 正式Host观察 |
|---|---|---|---|
| 合法对照 equal | 全部(10,0,0) | 完成callback，Paused/tick1 | Paused/tick1、无error |
| late-z-miss | 全部(10,0,10) | 两次都是正常miss，发布tick1 | Faulted/tick0；100000累计步耗尽 |
| decreasing-t | 第i叶(500-i,0,0)，i=1..127 | nearest为最后叶，两次成功并发布tick1 | Faulted/tick0；100000累计步耗尽 |

同样分布的actual Session重放：equal成功95次yield；另两例均99次yield后
明确报`cumulative CPU budget exhausted in '_ag_tick' (100000 steps across slices)`。
yield只是调度次数，不是精确执行步。正式Host也复现同一具名budget诊断，
排除了probe-only几何配置、初始化或OS计时噪声的归因。

### 原因与覆盖边界

已确认：v3每个匹配box先完整验证，再X/Y/Z slab；
`if enter>=fraction {continue}`只在存在足够近的已有命中时减掉后续轴。
旧128控制用相同box，首个命中后几乎所有后续候选在X轴后退出。
late-z-miss必须走到Z且一直没有已知hit；decreasing-t每个候选更近，
不能用已有fraction剪掉，故当前总计算量跨过既定预算。
这不是无限循环；预算保护正确地Fault并保last-good，但不能满足合法控制成功要求。
尚未证明某一具体优化方案能修复，review不指定未经验证的实现处方。

探针命令（driver目录）：

```powershell
Copy-Item D:/autostack/auto-lang/docs/plans/attachments/848/review-r1/review848_probe.rs.txt tests/review848_probe.rs
try {
  cargo test --locked --test review848_probe -- --nocapture --test-threads=1
} finally {
  Remove-Item -LiteralPath tests/review848_probe.rs
}
```

注意：诊断探针Host分支**断言当前已知Fault**，所以exit0仅证明反例稳定，
不能作修复验收绿。修复后应把两个Fault断言改成成功发布tick1，并增加
实际miss默认值/最后ordinal nearest及t/point/normal的独立断言。

关闭条件：保留预算、128/32credits、完整几何先验校验和既定语义；
上述两种分布及等深对照通过actual Session与正式Host；永久断言能区分
正常miss/nearest和预算失败。检查共享计算路径的代表2D/3D、宽/深兄弟，
保留bad-geometry-before-prune、parent切换、最终dirty发布及token/credits控制。
按实际修复diff选择定向检查并刷新source/fresh consumer和受影响提案/证据。
不通过减少节点、去掉group写/getters、增加CPU、换更易剪枝的fixture或隐式限制
合法候选分布关闭finding；设计不能满足时先回new有界修订。
完整统一回归仍merge-owned，不挂到每条修复上。

## 逐项验收

| AC → tasks | 结果 | 实际代码/检查与证据 |
|---|---|---|
| AC-01 → T01/08 | pass | diff全部限定driver35文件；固定846/base/SDK/AGP/三锁；370源、681SDK、17local/29edges/479metadata重核通过。[source.log](source.log.txt)、baseline；旧反控原记录保留 |
| AC-02 → T02/04 | pass（数值/几何）；规模成功不在此冒领 | 逐轴kernel/完整box验证/ordinal剪枝代码检查；API8+geometry6本轮重放，closed/parallel/inside/endpoint/ULP、all-axis法线、underflow、nearest及缓存变异有区分断言；规模预算归AC05失败 |
| AC-03 → T03 | pass | Api/profile9→SourceSnapshot→typed Play→Host/CLI/Binding实际路径；旧v1/v2分支与base diff对照，原API/transport/ENTRY输入保持；旧helper合法、新名只v3保留；本轮API身份测试通过，既有CLI/project/raw身份日志按未变面复用 |
| AC-04 → T04/05 | pass | 单槽11全局、每attempt先清token、callback前后清token、序列上限与sticky首错；dirty derive与最后consumer `_ag_field`检查；本轮token/快照/新查询/最终derive删除变异重放通过 |
| AC-05 → T02/05 | **fail** | F-848-R1-01；原均匀宽/深控制有效但覆盖不充分。数值非finite实际producer、配置门、last-good/VM关闭/Starting/Stop安全断言仍有效，并未观察到安全预算被绕过 |
| AC-06 → T03/06 | pass | 用户源而非Rust业务分支；两项目实际Source/Binding/SaveAs/移root/reopen/只改reach路径，脚本和CLI/project断言检查；14原生报告中6正控/4reopen逐raw审核，源SHA与同binary匹配 |
| AC-07 → T05/08 | pass（已审资源及生命周期面） | source/VM/Descriptor弱owner、20轮历史/Run/Fault/Stop及合法pin；GPU Arc、target、上传、扫描同时核内容与counter；5变异拒绝+恢复正控原始结果复用。geometry64次无heap对象增长本轮重放；11标量44B与SDK实际开销分开计量 |
| AC-08 → T07/08 | partial | 原生6正/8反/4reopen及全部part pixel的实际阶段和断言核对；binary075bab…字节hash再核一致。规范提案正确描述目标，但128可行性和readiness的无gap声明被F01否定，须修复后更新 |
| AC-09 → T09 | fail / needs_fix | 尚有合同内阻塞，不能reviewed或宣称最终独立pass |
| AC-10 → T10 | pending | 未合入/归档/清树，所有统一门留merge阶段 |

[api-geometry.log](api-geometry.log.txt)：14个定向测试实际通过，0ignored。
[audit.py](audit.py)核原生raw字段、像素子断言、4重开、owner反控与固定身份，
不是仅数ALL-PASS；原生/GPU及20轮owner在本轮**复用**而未再次执行。
已有runtime预算阻塞，不启动整套原生或完整package回归。

## 规范、遗漏与健康

SD01..04及8个既有Spec目标逐项核对，GOAL-007/008与当前图形方向匹配；
新增segment-query-profile无旧规则覆盖，旧spatial/behavior项目继续显式版本隔离。
拟议描述保留AABB精度、closed/f32 tie、token、weighted credits、原CPU及I2/I3非目标，
未发现通过Spec提案降级要求；但SD01/03的128支持目前无法验收，随F01暂停通过。
canonical Specs/live ledger均未修改。

遗漏/workaround扫描覆盖版本入口、参数/错误、几何/数值、dirty/cache/token、
旧版身份、项目/来源、owner/缓存/上传及GPU反控。发现的是预算覆盖缺口，
没有把它记成可延期债务；其余没有额外阻塞finding。完整原178/新增30等分母
及原生全矩阵仅作为未来merge门，不能由当前定向总数推导已全过。

本轮fmt、实现diff whitespace和冻结source验证通过；未加产品debug输出。
固定SDK原146 warnings不变。临时review probe复用support产生1条dead_code警告，
仅在反例日志中，探针已删除；不是冻结driver新增警告。

## 状态与路由

Plan revision仍2；status→executing；重开T02/T05/T08，保留T01/T03/T04/T06/T07，
current_step=5/10。T09未关闭，T10 pending。已通过的未变数值/兼容/生命周期/
项目/资源证据保留；修复变更到这些面时再定向复核，不无条件全量重跑。
next: work在原唯一树修复F-848-R1-01，提交并更新readiness后重新送审。

pending_closeout_gates：独立最终pass后冻结完整driver/AGP test+build、原图形/
spatial/Step/project/source/owner/native矩阵、producer/fresh own-SDK-own-target consumer、
新segment矩阵；再由merge做Spec/ledger/落地/归档/guard/batch到期判定。
本Plan未触root源码，不跑root t/tv/tt/tb/tu/taa/docs_gen；tf仍主检出批量单实例。
本轮未修产品、未合入、未清树。

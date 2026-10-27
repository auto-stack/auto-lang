# PLAN-821 final R1 — needs_fix

2026-10-25，合同 revision 5。独立审查上下文为当前 Codex 聊天，未参加提交中实现；未创建其他代理。完整绑定见 [baseline.json](baseline.json)。

- 受审树 `D:/autostack/.wt/lang-821/auto-lang`，`plan-821-dev`，HEAD `7c72d657dd16a0865216258684645f342550a808`；差异基面 `b1b8b425e563ba8358e4adc99c3e6c819a8fc137`。入审和出审实施树均 clean，两个邻仓依赖也 clean。
- D820/D828 已合入归档，均在基面祖先链；兄弟依赖 auto-ui `aee499e87093ff727f68ac8467c506e9ecf6ecdc`、auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`。主检出其他会话 WIP 保持。
- [入审 Plan](plan-r5-at-entry.md)、[冻结 SD](spec-delta-frozen.md) 和相关 Spec 的 SHA256 已绑定。语义合同没有修改；本轮只写复审证据、Plan 状态与 debt。
- **结论：9 项 P1、2 项 P2，全部 OPEN，不能发最终 pass。** T-02..06/T-09 重开，保留 T-01，`executing 1/9`。这不是要求新增范围，也不准把原合同缺口列债后结项。

## 实际检查与复现

在受审树运行：

```powershell
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_semantics --test av_owned_821_resources --test av_owned_821_incremental --test av_owned_821_cache_compat
cargo check --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core
cargo fmt --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core -p auto-av-probe -- --check
git diff --check b1b8b425e HEAD
```

现有定向 25+6+4+5=40 测试全过、0 ignored；check/fmt/diff-check 通过。显式包 fmt 记录在 [fmt-scoped.txt](fmt-scoped.txt)、[check-core.txt](check-core.txt)。初次 `cargo fmt --all` 扩展到了根依赖，遇到既有 examples/ui_container.rs 未闭合字符串及大量无关格式漂移；改为显式包检查后通过，未把范围外历史格式问题算作本计划回归。

独立未信任 typed MIR 反例（只依赖真实 core，不修改实施）在主检出运行：

```powershell
cargo run --quiet --manifest-path docs/plans/attachments/821/review-r1/probe/Cargo.toml
```

[源码](probe/src/cases.rs) / [实际输出](probe-output.txt)。除单独的验证内存例外，输入走真实 encode→decode→verify_abc→AV（可执行单元例另走 FunctionCode→ProgramSnapshot）。现有源码合法调用另经当前树 fresh-built debug worker `owned-request --store <unique temp> --src-file <case.at> --entry d_main`；[输出](source-output.txt)、[借入 record](borrowed-record.at)、[mut record](mut-record.at)、[move 正控](move-record-control.at)。

没有跑裸 cargo t、tv/tt/tb/tu/taa、acceptance 聚合、完整 workspace 测试或 batch tf；已知代码要修时不消耗统一回归。

## Findings 与最小关闭 oracle

### F821-R1-01 / P1：RETURN 绕开 Dropped 与终结借用的来源责任

**AC-02/03/04，T-03/04，V-02/06/11。** 合同 §5.1/§5.4 要求所有消费点拒绝已消耗 owner、局部/move-in 来源 view 不外逃。`core/src/ownership.rs:838` 的 Return 只检查 init/moved，未检查 dropped；逃逸检查还用 tokens_live 作为是否追踪来源的开关。

- `return_dropped_Str`：callee Const("abc")→Drop→Return 同槽被真实 ABC 授信，caller 读取结果长度 **0** 并 Success；应在授信拒绝，不能返回 typed zero 充成功。
- `ended_local_escape`：callee 建局部 Str→View→BorrowEnd→Return 该 view，caller 得到长度 **3**；只添 BorrowEnd 就绕开 `mir.borrow-escape`。无 BorrowEnd 的相同输入正被 `mir.borrow-escape` 拒绝，说明反例击中了真实门。
- 原因已定位：Return 漏终态且把“借用已结束”当成“来源合法”。不是共享 buffer 存活就等于语言允许逃逸。
- 关闭：重放两例和合法静态/参数 view 返回、合法 owned return，补 Return/Call/合流等消费点一致的活态断言。仅相关 ownership/return 族；全档仍 pending。

### F821-R1-02 / P1：loan 不变量在调用返回、新文本写入与反向借用处漏传

**AC-02/03/04，T-03/04，V-01/02/06/12。** §5.1 root 保守排他、真实 owner 溯源不能只在部分 opcode 生效。

- `call_result_root_lost`：合法 identity(view) 返回后，结束原 token、Drop 根，再用返回 view；当前准入并得 **3**。Call 分支 `ownership.rs:461` 不为结果建立 root/token，返回 view 的真实 root 丢失。关闭 oracle 应拒绝 Drop，合法延长借用则继续允许；不能把所有跨 callee view 禁掉。
- `live_mut_then_view`：MutRef(s) 活期内 View(s) 并读长度，当前 **Success(3)**；结束 MutRef 后的合法对照也 Success。View 分支 `:678` 没检查现有排他 loan（仅反方向 MutRef 创建时检查）。
- `to_owned_overwrites_borrowed_root`：View(s) 活着时 StrToOwned(dst=s, view)，当前准入并实际新分配，Success。`StrToOwned/StrConcat` 分支 `:758/:787` 没调用 destination conflict 门；Aggregate/FieldSet 的 owned operand transfer 也缺 source active-loan 检查，需一并核实同不变量。
- 关闭：真实 Call 的 provenance（参数→返回与 static）、View/FieldGet 与 MutRef 双向排他、每个 owned def/kill 的源和目的 loan 检查。重放上述例子+结束 loan 后合法对照+相关合流兄弟，不要求无关全测试。

### F821-R1-03 / P1：静态 view 跨单元返回后按 caller 常量表解释

**AC-01/02/03/09，T-03/09，V-06/13。** §5.1.7 明确静态字面量 view 在 callee 返回后有效；D828 单元有独立常量索引域。

`static_view_across_units`：helper 单元常量 0="ABC"，返回静态 StrView；main 单元常量 0=I32(0)，接收 view 后 len。两单元真实准入和 link 均通过，应 **Success(I32(3))**，实际 `Infra("static view references a non-Str const (admission violation)")`。

`av.rs:654` 的 Static{const_idx} 经当前 func_idx 查常量，没有绑定产生该 view 的 owning unit。当前测试只用全模块共享常量域或 owned return，漏了合法跨单元 view。关闭须同时覆盖参数传递、返回、多层调用及 caller 相同索引恰好也是另一 Str 的错误值反控；不以统一常量表掩盖未改单元复用责任。

### F821-R1-04 / P1：Aggregate/FieldSet 声称 move，执行却保留源槽 Rc owner

**AC-02/03/05，T-03/04，V-03/04/06/12。** 冻结 InstructionSpec/ownership 将 owned operands 标作已转移；§5.1.3/4 要求唯一责任及提交后清旧值。

`av.rs:1635/1659` FieldSet/Aggregate 只 clone 源值到聚合，从未 owned_kill_slot。独立 `aggregate_then_explicit_drop_before_B` 先 A→Aggregate(r)→Drop(r)，再分配/Drop B；无 view/share，A 应在显式 Drop(r) 释放、早于 B，实际 free 顺序 **[B(id1), A(id0)]**。终态 live=0 掩盖了 A 被已判 Moved 的 temp 继续持有到帧尾。FieldSet 同样留下源权，旧 buffer 也可能被构造 temp 延迟释放。

关闭：在成功提交点让实际槽责任与静态 kill 一致，失败前保源；分别重放 Aggregate/FieldSet 成功、注入失败、结果转出及显式 Drop 的时间/顺序 oracle，保持合法 clone 的独立 buffer。不只断言最终 live=0。

### F821-R1-05 / P1：验证内存 guard 严重少算 n² root matrix

**AC-04/05，T-03/04，V-07/11/12。** §5.2、contract-decision §5 要求分配前计量 ≤16MiB、有界稀疏路径。

`ownership.rs:189` 的 bound 只按 token_slots 个 roots 计账，但 `:281` 以及 clone_empty_like 实际为 **全部 n 个槽** 各建 n+1 bits，in/out/new/st 多份共存。独立 8192 个 I32 槽、1 块、1 条 Const 的合法有限输入：真实 verify 返回 **Ok，analysis_bytes=4160，work_steps=143**；只在 verify 调用窗用 probe 全局分配器实测 heap peak 增量 **35,176,954 bytes**，已超过 16MiB。单份 root bits 本身约8.45MB，多份并存约33.8MB，结果吻合；probe 未执行 guest、不依赖进程 RSS 估计。

关闭：计量和真实存储布局匹配，构建/clone/scratch 全部在分配前核预算；同时覆盖零 token、大量 token、块数及稀疏 field paths，两侧合法/Resource 分态。高槽输入更大可申请 GB 级内存，本轮未去冒险验证 OOM。

### F821-R1-06 / P1：大字符串 clone 没有按输出字节扣 fuel

**AC-03/05，T-03/04，V-03/04/07/11。** contract-decision §4 明确 CLONE/CONCAT 按输出字节数计量，§5.2 同总预算。

`clone_byte_fuel`：16B 和 **1,048,576B** 的 Str 各做一次 Clone，`max_steps=50` 时两者都 Success、**steps=14**。大输入仍在4MiB wire/8MiB活值范围，应因字节工作超过50而 Resource，不能用相同固定步数通过。`clone_owned_value`/StrConcat 路径没有实际字节成本准入/扣账；表中 fuel=1 不能兑现字节合同。

关闭：Clone/Concat/ToOwned、编码/解码/清理等实际适用遍历按冻结单位记账，长操作支持预算/cancel 的有界点；重放上述大小对照及阈值两侧。另需核实 `clone_owned_value`、`Value::deep_size`、record_set_path 与 Rc drop 的递归实现及 cleanup 临时 Vec/sort，当前并非合同声称的显式栈/预留清理；本轮未做大深度 crash 探针，不能据注释判已通过。

### F821-R1-07 / P1：owned 的全链路增量仍是测试内手工选函数

**AC-09，T-09，V-13(2–4)，SD-07。** 合同明确 parse/resolve/typecheck/HIR/生产计数与 IO，以及强制 whole 的正式 gate 必红。

`probe/tests/av_owned_821_incremental.rs:84` 的 OwnedSession 仅存在测试文件，reproduce_one **每次调用 compile_text_record_source 整模块前端**，再只 lower 用户手工传入的 def_id；`produced` 是测试包装层自增，只计 lower/encode/admit 一次，没观测整前端和真实 HIR 身份。生产 owned_driver 仍整 source request；session.rs 没有 owned consumer 安装通道。

`forced_whole_recompute_is_nonzero_everywhere` 刻意 recompile 所有函数后，断言它们确实全部前进并 **测试 PASS**；未使正式增量 detector/gate 非零。小例 4/4 绿不能证明 V13；继承 scalar campaign 绿也不能替代 owned 的实际入口与新增能力。

关闭：沿 D828 已有真实 frontend/session 私有凭证接入 owned；实际入口自动识别 edit/invalidation，保留 HIR/MIR/code/代际；真实生产点计数，值/清理 oracle 与 IO 同窗；forced whole/对象复制/授权变化通过同一正式 checker 必红并留 raw。保留原 scalar 30修订基线证据，只重跑受影响 owned 路径，不再重放无关基线来掩盖缺口。

### F821-R1-08 / P1：合法 borrow/mut record 源码仍停留 T-03 占位

**AC-01/02/03/04，T-03/06/09，V-02/06。** §1/§5.1、冻结类型/Call/adapter 与 SD-06 承诺接入既有 text-record 支持面；未获批准的支持面缩减不能记债即交付。

真实 worker：`borrowed-record.at` 的 `fn read(p Note) int { return p.n }` 调 `read(a)`，front-end 验证后 **exit4 lower.borrow "record view receivers land with PLAN-821 T-03"**；`mut-record.at` 的 `set(a.mut)` **exit4 "mut borrows land with PLAN-821 T-03"**。同结构 `read(move p Note)`/`read(a.move)` 正控 exit0/I32(7)。实际占位在 `hir_adapter.rs:1632–1644`。

债表将 borrowed record 称为“如需支持须回 new”，但这是现有承诺缺口；mut 占位也没在交付边界披露。record-view/参数模式表达力可能确需合同修订，当前不能由 work 单方面删支持。关闭须完成受约定的源码/参数责任，或经 new 明示修订并获所需授权后重审；保留源码负例与合法多 view/mut 独立源对照。不得用 typed MutRef 句柄“存在”代证源码 mut 读写可用。

### F821-R1-09 / P1：owned 进程/受管内存验收没有父侧硬门

**AC-05/06，T-02/04/06，V-07。** Plan §5.2/§5.5 要求32MiB分区与worker128MiB独立验收，父侧采样瞬态、不能扣前端；canonical D798现已双线，不能用无测量的“沿用”默默化解本 Plan 更严格文字。

verify 脚本 first-case 仅取 worker 自报 `private_bytes_peak` 存 JSON（worker 以自身 PeakSampler 采）；不比较阈值、不父侧采样、不要求样本数、无空载基线/归因。resources 只跑6个 core cap/fault测试与820继承档，没有 owned process/32MiB gate。收据 budget_note 声称后续T04/T06拥有B线门，但这两项已勾完成仍没有此门。~1.7MB小首例和slot cap±1B不能证明进程/受管峰值，旧055提交收据也未覆盖最终新增分析/增量状态。

关闭：先明确适用现行预算与 Plan 文本的一致性（有冲突回合同），在真实 owned worker 冻结输入上父侧采样及自有计数同时执行阈值判断；缺样本、超预算、阈值反控使同一 gate 红。按本机串行资源负载执行，保留归因/整体原值，不能后置 merge 补未完成的work oracle。

### F821-R1-10 / P2：V-08 的“新 worker / 自洽坏内层”并未实际验证

**AC-04/06，T-02/05/06，V-08/11/12。** first_case_same_store_restore 在同一测试进程调用 request_owned→restore_owned；owned-request 每次只重新编译并写store，无 owned 新进程restore入口。cache_compat 的 good_hash_bad_payload 只在内存 clone 信封最后一字节 XOR，再调用 envelope_unwrap，没重建自洽checksum/写回真实store/调用恢复授信，故只证明外壳checksum门。也没有V11/V12状态分支的适用store恢复覆盖。

关闭：真实producer写入，真实新worker从同store恢复（而非重新生产）；把语义非法但结构/hash自洽的ABC实际发布后走restore，断言dispatch=0与深验码，合法非空owned正控仍可用；覆盖相关state/版本轴。复用现存managed-store设施，不另造恢复模型。当前测试名/注释/勾选不算执行证据。

### F821-R1-11 / P2：SD 仍是执行中草案并包含已被反例推翻的现状

**AC-07，T-06/07，SD-01..07。** SD-07 保留“状态：T-09 执行中，完成后补齐”；SD-06 宣称借入view/mut支持而实际占位；SD-01/02宣称预算/生命周期/有界遍历全部落地，与01–09冲突。主Plan §0/§3/§10也仍有T02待开工/入口不存在的当前态文字。

关闭：修复代码后生成精确before/after规则、目标Spec当前版本/hash、AC证据与清楚支持边界，冻结最终SD；不要求本轮发布canonical Spec/ledger。touched_goals=[] 的实验扩面解释成立；目标路径和add/modify分类本身可用。未经批准不能用SD缩小原合同来配合现状。

## AC → Task → 实物/证据映射

| AC | Tasks | 结果 | 实物及当前证据边界 |
|---|---|---|---|
| AC-01 | T02/03/06 | partial | first-case/recipe/782孪生实物存在；源码借入/mut缺口、静态view跨unit失败（03/08），首例不能覆盖全部支持面 |
| AC-02 | T03/04 | fail | ownership + adapter；独立01/02/03/04/08反例在真实授信/执行暴露 |
| AC-03 | T03/04 | fail | arena最终live=0可用，但已drop返回、逃逸/loan、源槽ghost owner和fuel未闭（01–06/08） |
| AC-04 | T03/04/05 | fail | 现有25语义/5缓存绿仅覆盖原样本；非法终态仍授信、内存未预拒、坏内层恢复缺oracle |
| AC-05 | T04/06 | fail | 6资源测试绿；04/05/06/09未兑现责任时序/计量/进程预算 |
| AC-06 | T02/05/06 | partial | producer隔离与原compat测试绿；新worker恢复、好hash坏内层及资源硬门未闭（09/10），V10只有已有result/ledger与3fault证据 |
| AC-07 | T07 | fail | 本轮独立final needs_fix，11项OPEN；SD未可接受 |
| AC-08 | T08 | pending | merge-owned，未跑冻结门、未合入/沉淀/归档/清树 |
| AC-09 | T09 | fail | T09四测试绿，owned新增能力实际frontend/失效/正式反控门缺失；03/07 |

V01/02/03/04/06/07/11/12按上述findings为partial/fail；V05当前只能证整数类型隔离，旧帧/跨store/generation“无消费指令”与冻结必需oracle不等价，保持 **pending contract reconciliation**；V08为partial（10）；V09既有旧consumer拒绝检查通过，本轮未重放全部原语料，不扩大声明；V10已有result/ledger sabotage和worker exit/extra-frame/wrong-hash可见，完整冻结身份/skip/missing集合仍须T06逐子case对账；V13 scalar继承面已有，owned子面fail（07）。未查/未执行的合法大深度与完整oracle不计pass。

## 下一步与状态

同树 work 按稳定finding ID修复，保留这些输入作为永久回放；对表达力/支持合同冲突先形成明确方案并按new修订规则处理。先完成全部修复和定向检查，再提交冻结新SHA及closure表送复审。每项的相关检查如上，全局门不挂到每个finding上重复跑。

merge-owned pending：最终pass后在新冻结提交执行裸 `cargo t` + `cargo tv` +本独立workspace约定完整有限合同档及实际触面门；tf仅主检出到期单实例。当前不能合入、归档、清树或声称交付。没有修改实施代码、canonical Spec或ledger。

# PLAN-798 contract-decision.md — T-01 全路径可行性门的冻结决策

> **r2 对齐（2026-10-16，复审 F798-R1-02 后按 plan r2 重写 §6）**：条目预算语义由
> r1.1 的"每表族"解释性修订升格为计划 r2 正式合同——模块表族与每函数表族各自
> ≤65 536（分配前检查），组合上界落到请求级预算（4 MiB 输入、2M 验证步、5s 墙），
> 三反控（单函数/模块表/组合步数）入 mir_admission_ 测试族。

2026-10-16，work stage T-01。本文件冻结 `av.core-i32/1` profile 的逻辑 Schema、wire
编号/布局、内部调用/返回、entry、预算/计步、资源分类、合法 carrier 与参考负载。
后续 T-02..T-05 在此基础上实现并验收；任何变更须回到本文件并记 revision。

## 1. 可行性门结论

**通过**。最小两函数算术/分支 vertical（`score`/`main`）走真实
Checked HIR → MIR → ABC → Rust AV，与独立手算期望一致；首份 ABC 单元经 783
ManagedStore 写入，并由新进程 worker（`av-probe-worker restore-run`）从磁盘恢复、
重验、执行成功（`tests/cache_tier.rs::av_cache_worker_restore_new_process`）；
同一输入经旧 HIR→Cranelift→COFF→PE 路径产出真实 PE，退出码与独立期望一致
（`tests/vertical.rs::t01_vertical_native_cross_check`）。

依据（PLAN-798 §5.1）：store 作为合法 carrier 成立（本计划自有 LayerSpec +
`ChunkKind::LayoutData` 载体 + 内层 envelope 校验，未改 783 registry/ChunkKind）；
存储/API 未出现不可行反例。

## 2. 逻辑 Schema 与身份

| 项 | 冻结值 |
|---|---|
| MIR schema | `auto.mir.av-probe/1` |
| ABC wire schema | `auto.abc.av-probe/2`（magic `AVAB`，version 2；旧歧义v1隔离） |
| 执行 profile | `av.core-i32/1`（profile id 1） |
| producer | `plan798/av-probe`（envelope 校验必填） |
| 语义版本 | MIR=`1`、ABC=`2`、AV=`1`（任一变化即失效对应层） |
| 信封 | `AVEN` v2，kind∈{1=MIR单元,2=ABC单元,3=模块manifest,4=源映射sidecar}，校验 producer/schema/owner/stage/三元语义版本/snapshot |
| 命名 | 单元/entry 用 HIR def id（`d_main`）；前端声明名（`sf_*`）是内部符号 |

MIR 形状：显式基本块 + 唯一 terminator（Jump/Branch/Return）、有类型可变槽
（Param 先导、Local、Temp）、顺序指令（Const/Copy/Bin/Un/Call/Trace）。
**非 SSA**；块/槽编号是函数内位置身份，非跨修订稳定 id。

槽初始化合同：Param 入帧即初始化；其余槽由指令定义。验证用入口可达 CFG 上的
保守 must-worklist（块入口 = 前驱出口交集，循环从乐观界收敛）：循环内初始化
不外溢（零迭代保守），def-then-use 同轮可用；不可达块参与全部结构/类型检查，
只是不参与初始化数据流。HIR 结构块树（If/Loop 子块）在 lowering 中拍平为 CFG；
scalar `for` 已由前端 HIR 展开，无需特例。

## 3. 求值顺序与短路

- 表达式求值产生 temp 槽，按源码顺序发射；调用实参按 `eval_args` 源序求值后经
  `bindings` 映射为参数序，**绝不按绑定表重排求值**。
- lazy and/or 降低为分支短路：结果 bool 槽在**两臂各定义一次**（skip 臂放短路值，
  taken 臂放 RHS 值），合流靠交集初始化规则——无 phi。
- 未执行分支上的 trap（如 RHS 的除零）保持潜在运行时行为：AV 只在真正执行的
  指令上 raise trap，lowering/验证不提前求值（`av_semantics_unexecuted_rhs_trap_stays_latent`）。

## 4. ABC wire 布局（v2，小端；r6 升代）

```
magic "AVAB"(4) | u16 version=2 | u16 profile=1 | u16 flags(bit0=有sidecar, bit1=FLAG_IS_UNIT) | u16 reserved=0
u32 n_types, n_consts, n_imports, n_funcs, entry
u16 n_caps, 每项 u16 len + bytes（能力随头部传输——admission 数据）
types:    u8 tag(0=I32,1=Bool,2=Fn)；Fn: u32 nparams + params(u32 idx)* + result u32
consts:   u8 tag(0=I32→i32, 1=Bool→u8∈{0,1})
imports:  u16 sym_len + bytes, u8 effect(0/1), u32 sig
funcs:    u32 sig, u16 name_len + bytes, u32 nslots, per slot u8 kind(0/1/2)+u32 ty,
          u32 entry_block, u32 nblocks, per block: u32 ninstrs + instrs + terminator
instrs:   u8 opcode + 定长操作数（u32 槽/常量/块/函数/导入索引，u8 子码）
          01 CONST dst,const | 02 COPY dst,src | 03 BIN sub,dst,lhs,rhs
          04 UN sub,dst,operand | 05 CALL kind,callee,dst,nargs,args* | 06 TRACE event
          BIN 子码 1..11 = add sub mul div rem lt le gt ge eq ne；UN 1..2 = neg not
          TRACE 事件 1=mark_a 2=mark_b
term:     10 JUMP t | 11 BRANCH cond,then,else | 12 RETURN value
sidecar(仅诊断): u32 len + 每函数 u64 snapshot + u32 n + (u32 instr,u32 func,u32 start,u32 end)*
trailer:  u64 FNV-1a 校验和（对全部前序字节）
```

解码顺序：magic/version/profile/flags/reserved → 计数配额（分配前）→ 逐表结构
（未知 tag/opcode/子码/事件、bool 合法性、长度 checked 算术、索引域）→ 尾随字节
→ 校验和。控制流/初始化/签名 admission 在 `verify_abc`（decode 之后），独立
ABC 输入不因"由本编译器生成"跳过验证。分页：编码按 ≤256KiB 页冲刷，解码带
2M 步预算；wire 不携带 owner（信封层身份），重验时以调用方 owner 重盖章。

**known open decision**（非承诺）：此格式是探针冻结件，不构成未来公开字节码 ABI。

## 5. 内部调用/返回与 entry

- CALL 按被调签名逐实参类型/数量/结果类型 admission；单元（unit 模式）只能经
  imports（`fn:<name>`）寻址跨函数调用，闭包装配时按名解析并要求 interned 类型
  等价（结构相等），递归/互递归合法（燃料限制在运行时）。
- RETURN 弹帧并写调用者 dst 槽；根帧返回即 Success。
- entry 合同：模块闭包的 entry 必须零参 i32（`mir.entry-sig`）；单函数单元不适用
  该合同（装配时选择并复查真实 entry）。
- mark_a/mark_b：无参 i32 内建，仅 `hir.test.trace.v1` 能力下解析；AV 映射为
  trace 事件，冻结返回值 1/2（对齐 landed `test-support/marks.rs`：stderr "a"/"b"）。

## 6. 预算与计步（资源分类，§3 冻结值）

| 预算 | 值 | 分类 |
|---|---|---|
| source/HIR/ABC 输入 | ≤4 MiB | `av.resource.input-size` |
| codec 页 | ≤256 KiB | `av.resource.codec-page` |
| 表条目（**r2 正式合同**：**每表族**防护——模块表 types+consts+imports+funcs ≤65 536，且**每个函数** slots+blocks+instrs ≤65 536，均在深检前检查） | ≤65 536/表族 | `av.resource.table-entries` |
| 前门嵌套（token 流，含全部括号种类；Atom 路径守 `[` 深度护递归文档 parser） | ≤128 | `av.resource.frontend-nesting` |
| MIR/ABC 验证 | ≤2M 工作步、wall ≤5s | `av.resource.verify-steps/-wall` |
| **r3**：验证组合分析内存（init 数据流 defs/in/out 位图，blocks×words×8×3，**分配前**核算） | ≤16 MiB（staging 分区份额） | `av.resource.verify-memory` |
| **r3**：解码模块结构堆估算（表项放大守卫，深检前检查） | ≤16 MiB | `av.resource.decode-memory` |
| AV 执行 | ≤1M 指令步、≤1024 帧、活槽 ≤8 MiB、wall ≤2s | `av.resource.instr-steps/frames/slot-bytes/execution-wall` |
| 取消 | 每 ≤1024 指令轮询一次 relaxed load | 独立 `Cancelled` 态 |
| 测试 trace | ≤64 KiB | `av.resource.trace-bytes` |
| native 参考 | build ≤60s / run ≤5s / total ≤70s | infra 阻断，不冒称 AV 预算 |

溢出即上述 resource 分类（非 unknown-opcode/非法语法）；`ExecOutcome` 分态：
Success(value)/Trap(kind+span)/Resource/Cancelled/Infra。trap 集：int-overflow、
div-by-zero、div-overflow(MIN/-1)、rem-overflow(MIN%-1)。

**计步定义**：验证侧 = 逐指令/逐块访问各计 1 步（+块内字数）；执行侧 = 每条
指令 1 步。不可通过不计验证/分配规避——表配额在分配前检查。

**r2 修订记录**（T-05 实测触发 r1.1 解释性修订 → 复审 F798-R1-02 判定须正式
修订 → plan r2 冻结）：冻结的 256 KiB/100 函数参考负载按函数内 ~17 块展开后
整模块 72k 条目，超出 r1"全部表合计≤65 536"的字面读法；负载规模同为 §5.5
冻结项不得缩小，二者不可同时成立。r2 冻结为**每表族**防护（模块表 + 每函数
表各自 ≤65 536）：单表分配守卫不变，整模块/闭包组合规模由 2M 工作步、5s
验证墙与 4 MiB 输入上限显式约束。**三反控**（`mir_admission_`）：
(1) 单函数超族 → `av.resource.table-entries`；(2) 模块表超族（70k consts）→
同分类；(3) 组合超界（40×31k 块闭包）→ `av.resource.verify-steps`。

**r7 修订记录**（F798-R12-04 闭合 → 用户裁定方案 C，2026-10-20，plan r5 §5.9 授权）：worker 内存预算由单一"相对空载增量 ≤128 MiB"拆分为**两条归属明确、均实测、均硬门**的预算线（`validate_and_gate` 失败即 bench 非零退出）：
- **A 线（ac-core owner）**：前端编译瞬态，冻结负载隔离测量（`frontend_peak_bytes`）≤ **224 MiB**——锚定实测 ~200-212 MiB（R12-04 分阶段归属：`descriptor::bind` 单独 ≈158.9 MiB；`compile_parts_with` 公开缝内部强制"atom 渲染→整体重绑定"架构）；ac-core 收敛该架构后**预期收紧本线**，224 不是增长许可。
- **B 线（798 owner）**：探针自有残差 = 总采样峰值（worker+子进程 max）− A 线 ≤ **128 MiB**（数字不变，语义按归属重定义）；**缺归因测量时整峰保守计入 B 线**（无分解即无豁免）。
受管分区 32 MiB（16/8/8）硬门不变；原单一 128MiB 目标字样由本双线取代。永久测试 `av_turnaround_two_line_memory_gate`（A 线越界/B 线越界/双过/无归因保守四态）。当前证据（release `bench-turnaround`，冻结 273 575 B 负载，2026-10-20）：A 线 211 869 696 B=202.1 MiB ✓、B 线 3 637 248 B≈3.5 MiB ✓、`hard_gate_failures=[]`、exit 0。

**r6 修订记录**（复审 F798-R12-01/02/03 判定 → 本轮修复冻结）：
(a) **wire 升代与版本隔离**：`ABC_VERSION` 1→2、`ABC_SCHEMA`
"auto.abc.av-probe/1"→"/2"、`ABC_SEMANTICS` 1→2、A09 层 schema_id
`ac798/a09-abc/v2`。kind 位是 wire 的载荷语义——/1（无 kind）与 /2 是不同
格式代际：raw /1 字节按未知版本受控拒绝（`abc.decode.version`）；A09/A08
查询键携带 codec/schema/语义版本轴（MIR 单元同样以 ABC codec 传输，其键新增
`abc-codec` 轴），旧键产物确定性 miss。当前键下发现异代载荷（K14-06 歧义形）
= **确定性陈旧 miss 重建**（计数 `cache_stale_abc_miss`/`cache_stale_mir_miss`，
O(1) wire 版本门判定）：绝不原地当新语义读、不是硬错误；重建单元重持久化与
陈旧行的 same-key 冲突按 store 语义保留旧行并容忍（计数
`cache_stale_rebuild_conflict`），请求以当次全量重验的重建单元服务。当前代际
载荷的深层 decode/admission 仍在闭包阶段执行——结构性损坏/篡改保持受控拒绝。
(b) **类别→执行凭证**（K14-02/03/05）：`LoadedModule.executable` 现由**类别**
与**imports 闭合**共同决定——unit 类别（即使零 import、entry 形状合法）在
任何公开入口（raw bytes/手工 DecodedAbc/driver 闭包/恢复）都不作为完整入口
执行（dispatch=0，Infra "unit kind"）；unit→闭包的转换唯一路径是 driver 的
闭包装配（整模块 encode→verify_abc 重验）。合法完整 module 的 kind 声明按
完整规则判定（K14-04 正控：同载荷翻位声明 module 且满足完整规则 → 执行）。
带参入口仅在 unit 上合法，故 `execute_entry` 的参数类型准入降为纵深防御
（原"带参 unit 可执行"测试 oracle 已按 r4 合同更正）。
(c) **前门计深 = 真实结合的最长构造路径**：前门内建微型运算符优先级深度求
值器（shunting-yard，按 parser `scalar_prec_of` 优先级表与左结合 `>=` 归约，
`a OP b → 1+max(a,b)`），逐括号层维护操作数路径深度栈——分支兄弟深度按真实
树形取 max 永不相加（尾操作数深调用 60×70 真路径 71 合法执行返回 70，旧法
算 192 误拒）、真实嵌套沿路径组合（首操作数深调用真路径 131 仍拒）、每个
调用层恰计一次（命名单实参调用不再列表层+表达式层双重计数）。括号分组仍各
计一层（保守，与 r5 前一致）。**R13-01 补记**：一元前缀绑定到**完整调用表
达式**——callee 名字在 `(` 处撤回时，折叠进该名字的前缀层数（`ident_unary`）
归还并作用于闭层路径（`-f(..)`/`!f(..)` = `Unary(Call(..))`，每前缀一层）；
R13 反例 20 负号+110 层命名调用（真路径 131）此前漏过 128 门，现两态（带/
不带分组括号，131/132）均拒且诊断数值与真实 HIR 对齐，116/24 低深控制与普
通操作数前缀（`----1`）不变。永久测试
`av_resources_prefix_unary_binds_to_full_call` 七态判别。

**r5 修订记录**（复审 F798-R11-02 判定 → 本轮修复冻结）：ABC wire flags
新增 bit1 `FLAG_IS_UNIT`（§4 布局扩展）——工件类别（可重验 UNIT vs 单函数完整
MODULE）由编码方写入 wire，解码按 flag 读取，**不再按 n_funcs 猜测**：单函数
完整模块（允许直接 self/mutual 调用）往返后仍是 module（`fn main() int { if
false { return main() } else { return 42 } }` fresh 请求执行返回 42，此前被
`mir.unit-callee` 误拒）；split 单元往返仍是 unit。未知 flag 拒绝集合相应扩展。
同轮前门计深规则收敛（F798-R11-01）：**当前 sibling 内部组合**（子路径+自身串
行链相加——深调用作为链首操作数是一条路径）、**已完成 siblings 之间取 max**、
调用层自身 +1；命名冒号只标记 sibling 列表、不折叠未完成的值表达式。

**r4 修订记录**（复审 F798-R6-04/05/07 判定 → 本轮修复冻结）：
(a) **decode 分配预算前置到 decode 自身**：解码器在类型/常量/导入/函数名/
槽/块/指令/span 各表分配**前**累计结构堆估算（元素口径同 heap_estimate），
超 16 MiB 即 `av.resource.decode-memory`——独立 ABC 输入在 decode 内被拒，
不再先完成 >16 MiB 分配后由 verifier 兜底（R6 反例：3.2MB wire → 23MB 结构）。
verify 侧 heap_estimate 检查保留为手工构造模块的第二层。
(b) **前门构造深度跨括号累计**：运算符 run 不再被 `(`/`)`/`[`/`]`/`,` 重置
（`(1)+(1)+…` 与 `f(a,b)+g(c,d)+…` 的嵌套与未加括号链相同）；仅语句/子句
边界（`{`/`}`/`;`/`=`/`:`/关键字）重置；一元 `!` 计入构造。129/400/600 项
带括号链在门前 resource 拒绝。
(c) **native build/total 预算（r8 撤销 r6 轮的降格表述并落实强制语义）**：
r6 轮曾在此把"build≤60s 单一数字"表述为依赖 ac-core owner 的后续工作——
该表述**已撤销**：它未经计划修订授权，且与 §3 冻结的 60/5/70 冲突。现行
实现为全程监督：lower/link/publish 与 run 各阶段在 `av_probe_worker
native-build`/`native-run` 受限子命令内执行，父侧以请求起算的**剩余截止**
驱动 contained executor——执行中的迟滞 linker 在剩余 build 预算处连同子树
被杀（复审复现形状实测 ~1s Budget 杀树），run 截止=min(5s, 剩余总量)，
60/5/70 全部为强制语义而非观察值。冷态 worker 首次准备另列：在请求钟之外、
独立 300s 有界墙（超时杀进程+输出帽），不进入受测请求预算。反控：伪钟
seam（AV798_NATIVE_FAKE_ELAPSED_MS）+ 迟滞 linker 杀树 + 预备墙杀树。

**r3 修订记录**（复审 F798-R3-04/08 判定 → 本轮修复冻结）：
(a) **验证组合分析内存**进入预算表——dense 单元（16 384 槽 × 8 192 块，
24 581 条目、155 738 字节 wire，全部单表族内）在 init 数据流位图上分配
~3×16 MiB 后才被 2M 步拒绝；r3 起位图上界（blocks×words×8×3 + 每块向量
开销）在**分配前**核算，超 16 MiB（staging 分区份额）即
`av.resource.verify-memory`（`mir_verify_dense_analysis_memory_pre_rejected`；
凭证 `VerifyStats.analysis_bytes` 折入驱动 staging 台账）。组合上界的
verify-steps 语义不变，其反控改用内存精简形态（30×2 000 块×4 096 槽，
init 访问步数 ~3.9M > 2M）重钉。
(b) **解码结构堆估算**守卫（`MirModule::heap_estimate`，len 基确定性估算，
计入 staging/hot 分区台账替代 wire 长度冒充），解码放大超 16 MiB 即
`av.resource.decode-memory`。
(c) **前门表达式构造深度**：`a + b + c + …` 平坦运算符链在左结合下每操作
数一层 parse 递归，与括号同属递归风险路径；前门对连续运算符 run
（非运算符 punct 重置）计构造深度，K 个运算符 = K+1 操作数构造，超 128 即
`av.resource.frontend-nesting`（`av_resources_flat_chain_front_door_boundary`：
400 项在门前拒绝，不再由递归 parser 的 512 内限报 Unsupported）。

## 7. 当前前门递归风险核查（§5.5 T-01，r6现行）

源码token入口在旧前端之前，按真实优先级/结合性计算最长表达式构造路径，预算128；括号/块及Atom list深度另作有界检查。callee作为operand后遇`(`撤回时保留其一元前缀，前缀归属完整call；各兄弟实参取路径最大值，不能把宽度/尾操作数深度相加。包含命名/位置实参、普通operand、分组、`-`/`!`及调用组合的超深拒绝/合法浅层正控。旧递归HIR/Atom相关消费受此边界保护，不宣称其已改成增量或全迭代。

当前实现及永久测试：`probe/src/driver.rs` 的前门路径计算；resources_tier的131/132层拒绝、116/24层正控、尾操作数71层通过/首操作数131层拒绝和相邻R7–R13组合反例。§6早期r3“连续run扫描”说明仅保留为演变记录，不覆盖本节现行规则。

## 8. 合法 carrier 与单元/依赖合同（§5.4）

- store：783 `ManagedStore` 公共 API 独立 producer namespace；自有 LayerSpec
  （`ac798/a08-mir/v1`、`ac798/a09-abc/v2`，coverage=Codec——798 自己实现的
  范围，不改 783 registry 的 A08 Future 登记）；载荷走 `ChunkKind::LayoutData`
  通用载体，**永不**标 Hir/Object/DeclSummary。
- 单元：MIR/ABC 按函数单元持久化；模块链接 manifest（JSON，kind 3）映射
  单元名→artifact id+签名内容。跨函数调用在单元内为 `fn:<name>` import；闭包
  装配解析+签名复核+整体重验后才可执行（complete-consistency gate）。
- key（至少区分）：producer/owner/profile/单元名/body+签名内容指纹（HIR 规范
  dump：局部/位置无关，嵌 callee 签名内容）/算法 schema/三语义版本/能力配置。
  **代码内容不含 span**——位置变更共享代码单元，sidecar（kind 4，键含 snapshot）
  重绑诊断。native 参考对象 key 另含 target/CPU/ABI/backend/flags（future，
  本计划仅合同+准入）。
- 恢复（r2 两层接线）：A09 ABC 命中→decode→verify_abc 重验；A08 MIR 命中→
  decode→**verify_mir 重入验证**→供 ABC 发射（不重新 lowering）。按 manifest→
  单元 artifact id 读取（store 地址 admission 护读）→信封逐字段校验。MIR 单元
  wire 复用 ABC 编码器（同一 MIR 内容的忠实序列化，信封 kind/schema/层区分）。
  计数分离（F798-R1-06）：`mir_lower_modules`=整模块 lowering 真实调用数、
  `mir_units_persisted`/`mir_units_restored`、`abc_units_emitted`、
  `closure_encodes`、`peak_staging_bytes`（受管暂存峰值）。不恢复 trusted；
  错 producer/schema/owner/stage/语义版本 = 受控拒绝（非 miss、非覆盖重建）。
  命中仍 av_execute>0 并重新比较（不缓存 PASS/运行结果）。
- 变更失效：callee body 变只重键其单元（caller 代码可复用）；callee 签名变
  连带 caller 指纹变（caller dump 嵌 callee 签名）；producer/语义版本/能力变
  更换 key 空间。

## 9. 参考负载与硬门（§5.5 T-05 冻结）

- 负载：`load256::generate()`——100 个有意义 caller 函数 + 14 leaf + main，
  混合 if/loop/算术块（无空白填充），≈267 KiB（≥256 KiB 上限方向锁定，
  **不得缩小**）；期望值由生成器闭式镜像推导（不经任何后端）。
- 局部编辑位：fn_037 首个乘子 +1（期望随之变）；签名冲突编辑：fn_leaf_1 增参
  （正确 frontend 诊断，非崩溃）。
- 六场景各 ≥30 样本（release，`av-probe-worker bench-turnaround`）：
  fresh / same-process-hot / restart-disk-hot（含跨工程）/ single-body-edit /
  signature-conflict / gc-concurrent。硬门：warm 会话单函数 body 编辑与签名
  冲突编辑两路径"请求入队→当前正确 AV 结果/诊断"P95<1000ms（含前端/验证/依赖/
  cache/MIR/ABC/AV/比较；native 冷构建与全冷另报）。debug 档仅冒烟（3 样本）。

## 10. 实测锚点（2026-10-16，参考 Windows 机）

- 31k 块直链单元：encode 1.2ms / decode 2.3ms / verify 11.4ms（release；
  修 block_base O(n²) 后；debug verify 36ms）。
- 4096 块直链+菱形+循环：验证 <5s 内远低于步预算。
- ≥32 MiB 冷产物：64 模块 × 31k 块单指令（≈31.9→33.5 MiB ABC），分页合规，
  随机只读单元重验通过，512 次修订+GC 后受管水位有界。

## 11. 边界（不因实现进度改口）

- 非目标重申：本 profile 未支持形态（text/record/堆对象/泛型/FFI/回调/Meta/
  CTE）在 lowering/验证前明确 unsupported 拒绝，不降级到旧 VM/源码解释；
  AV 内部调用/ABC 版本不是 797 AutoABI；AJ/发布器/热重载仅合同占位。
- T-06（真实 796 语义 ref 绑定）与 T-07（正式 AutoDown recipe/worker）分别等
  D-796/D-788 合入，本轮不伪造其证据。


## 2026-10-20 Plan r6交付边界更正

以主计划§5.10和受管acceptance.md的r6阶段矩阵为准：本实现有函数持久产物复用，但每请求仍整模块前端，miss时whole MIR lower/verify/split，hit单元重decode/verify后完整closure装载。没有常驻函数代码在线替换。798通过只交付有限执行/缓存基础；真正源码至AV安装的函数级增量由828承接，执行优先级为798完成后828。旧计量不能证明未改函数IR/代码保持不动。sig为前端错误早退，不代表成功签名更新。

旧formal含逐请求初始化，保留原技术门事实与失败样本，不据此判用户稳定会话目标失败；本次不改测试阈值、入口或以bench替代formal。r5双线内存政策和/2 wire/kind准入仍有效；历史摘要旧版本/旧资源目标不作为当前合同。798仍待正式门、最终review及merge-owned冻结门禁。

## 2026-10-20 R16生命周期复核

A08/A09缓存命中逐消费单元持lease，不能用第一个unit的lease冒充manifest反向闭包。无源码恢复在读取manifest之前持manifest lease，required-data边保住全部unit及当前来源sidecar；成功/拒绝返回释放OS锁。新增真实两函数回归检查A08/A09、多单元、manifest恢复与全部lease释放。当前代码d1357870a；性能/资源由同轮收据记录，不以注释授pass。

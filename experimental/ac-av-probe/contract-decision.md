# PLAN-798 contract-decision.md — T-01 全路径可行性门的冻结决策

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
| ABC wire schema | `auto.abc.av-probe/1`（magic `AVAB`，version 1） |
| 执行 profile | `av.core-i32/1`（profile id 1） |
| producer | `plan798/av-probe`（envelope 校验必填） |
| 语义版本 | MIR/ABC/AV 各 `1`（任一变化即失效对应层） |
| 信封 | `AVEN` v1，kind∈{1=MIR单元,2=ABC单元,3=模块manifest,4=源映射sidecar}，校验 producer/schema/owner/stage/三元语义版本/snapshot |
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

## 4. ABC wire 布局（v1，小端）

```
magic "AVAB"(4) | u16 version=1 | u16 profile=1 | u16 flags(bit0=有sidecar) | u16 reserved=0
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
| 表条目（r1.1 修订：**每表族**防护——模块表 types+consts+imports+funcs ≤65 536，且**每个函数** slots+blocks+instrs ≤65 536，均在深检前检查） | ≤65 536/表族 | `av.resource.table-entries` |
| 前门嵌套（token 流，含全部括号种类；Atom 路径守 `[` 深度护递归文档 parser） | ≤128 | `av.resource.frontend-nesting` |
| MIR/ABC 验证 | ≤2M 工作步、wall ≤5s | `av.resource.verify-steps/-wall` |
| AV 执行 | ≤1M 指令步、≤1024 帧、活槽 ≤8 MiB、wall ≤2s | `av.resource.instr-steps/frames/slot-bytes/execution-wall` |
| 取消 | 每 ≤1024 指令轮询一次 relaxed load | 独立 `Cancelled` 态 |
| 测试 trace | ≤64 KiB | `av.resource.trace-bytes` |
| native 参考 | build ≤60s / run ≤5s / total ≤70s | infra 阻断，不冒称 AV 预算 |

溢出即上述 resource 分类（非 unknown-opcode/非法语法）；`ExecOutcome` 分态：
Success(value)/Trap(kind+span)/Resource/Cancelled/Infra。trap 集：int-overflow、
div-by-zero、div-overflow(MIN/-1)、rem-overflow(MIN%-1)。

**计步定义**：验证侧 = 逐指令/逐块访问各计 1 步（+块内字数）；执行侧 = 每条
指令 1 步。不可通过不计验证/分配规避——表配额在分配前检查。

**r1.1 修订记录**（T-05 实测触发）：冻结的 256 KiB/100 函数参考负载按函数内
~17 块展开后整模块 72k 条目，超出"全部表合计≤65 536"的字面读法；而负载规模
同为 §5.5 冻结项、不得缩小。修订为**每表族**防护（模块表 + 每函数表各自
≤65 536）：单表分配守卫不变，整模块规模由 2M 工作步、5s 验证墙与 4 MiB 输入
上限约束。此修订不放宽任何单表安全性质。

## 7. 前门递归风险核查（§5.5 T-01 义务）

- `source::lexer::tokenize`：迭代；`source/parser.rs`：语句/表达式**全迭代**
  （landed R5/R6，"nesting of any shape or depth parses without recursion"）；
  `resolve.rs` 表达式解析迭代（R3-QA-02）。
- `atom_text::tokenize`：迭代；**唯一真实宿主递归**：Atom 文档 parser 的
  `parse_value`/`parse_field`（嵌套 list 递归下降）。
- 结论：前门 = 两路 token 流括号深度扫描（`( [ {` 全种类，非仅花括号；Atom 路
  只认 `[`），>128 在旧递归 parser 之前以 resource 拒绝
  （`av_resources_front_door_nesting_boundary`、`av_resources_atom_front_door_rejects_before_recursive_parser`）。
  表达式递归路径已由 landed 迭代实现覆盖——覆盖完整性成立，可行性门不触发。

## 8. 合法 carrier 与单元/依赖合同（§5.4）

- store：783 `ManagedStore` 公共 API 独立 producer namespace；自有 LayerSpec
  （`ac798/a08-mir/v1`、`ac798/a09-abc/v1`，coverage=Codec——798 自己实现的
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
- 恢复：按 manifest→单元 artifact id 读取（store 地址 admission 护读）→信封
  逐字段校验→decode→verify_abc 重验→装配→再整体验证→执行。不恢复 trusted；
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

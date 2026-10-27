# PLAN-821 contract-decision.md — T-01 冻结决策（r1）

2026-10-25，work stage T-01（用户授权"计划821: /auto-plan-work 实施"）。本文件冻结
`av.owned/1` 独立 owned profile 的身份/版本、类型/指令/访问状态机、预算、oracle 与
case→Scope→真实入口矩阵。后续 T-02..T-09 在此基础上实现并验收；任何变更须回本文件
记 revision。未列出项一律按 plan §5 合同与 [Design44](../../../design/44-av-data-layout-place-and-ownership.md)
I-01..12/§5–7 执行，不在下游实现中临时猜规则。

## 0. 依赖门核对结论（2026-10-25 实测）

| 门 | 结论 | 证据 |
|---|---|---|
| D-820 | ✅ 已交付 | `docs/plans/archive/820-av-typed-data-layout.md` archived 9/9；merge 收据链齐全（Spec 沉淀 `227101fd8`→归档 `81250e0a7`→收口 `c66528e80`，2026-10-25 10:20–10:55）；820 合同 [attachments/820/contract-decision.md](../820/contract-decision.md) r1 全文在档 |
| D828 | ✅ 已交付 | `docs/plans/archive/828-av-function-incremental-session.md` archived 11/11；`resident-function-install.md`/`function-incremental.md` Spec 在档，data profile 已接入 828 增量接口 |
| 817/818 | ✅ 均已合入 | `docs/plans/archive/{817,818}-*.md` 均在 archive；V-09 重放按已合入面 |
| **ac-av-probe 单写窗口** | ❌ **被 827 在途持有** | PLAN-827 executing（R8 修复轮，worktree HEAD `cce6139e8`）；其未合入分支含共享核心写：`core/src/av.rs`(+34)、新增 `core/src/frame_allocation.rs`(+92)、`core/src/lib.rs`(+1)、`probe/Cargo.toml`(+4)（2026-10-25 08:10 `4a069ce7f`），与 820 合入（同日 10:20+）在核心上并行未收敛。按 ACE 合同"公共核心默认一个窗口"，**T-02 冻结：等 827 窗口释放交接**（其 merge 收据或显式不重叠交接记录）；T-01 只读准备不受阻。本计划不得从 827 分支摘抄 |
| 注册 owner | ✅ 820 已随合并释放 | `crates/auto-lang-core/src/test_cases/ac_av.rs` 现有 `ac-av/core-run-v1`(798)+`ac-av/data-run-v1`(820) 两 recipe；827 复用既有 recipe 零注册追加；本计划只追加自己的 recipe/身份，串行窗口同上 |
| 主检出预检 | ✅ 代码路径零 WIP | `git status` 仅 docs/plans、docs/design、roadmap、.autoos 簿记改动 + untracked 构建产物（Cargo.lock 副本、specs.json.bak-*）；crates/test/experimental/src 已跟踪文件干净 |

唯一实施树（本日已创建）：`D:/autostack/.wt/lang-821/auto-lang`，分支 `plan-821-dev`，
基面 master@`c66528e80`。实施/内部检查/最终 review 同树顺序进行；主检出只记计划。

## 1. 身份、版本与路由

| 项 | 冻结值 |
|---|---|
| MIR schema（owned 单元 OwnerScope.schema） | `auto.mir.av-owned/1` |
| ABC wire schema | `auto.abc.av-owned/1`：magic **AVAD**，u16 **version=2**，u16 profile id=**3** |
| 执行 profile | `av.owned/1`（profile id 3；现网 1=core-i32、2=data，无冲突——冻结；822–826 未认领 id 3，822/823/824 计划文仅认领 `av.collections/1`/`av.closures/1`/`av.dynamic/1` 名字） |
| producer | `plan821/av-owned`（store 信封校验必填） |
| 语义版本（owned 轴） | MIR=1、ABC=1、AV=1；core-i32 1/2/1 与 data 1/1/1 三元组**均不变** |
| 信封 | 沿用 783 store `AVEN` v2 校验；owned 单元独立 producer/schema 轴 |
| 路由矩阵 | AVAB(profile1) 与 AVAD v1(profile2) 解码/语义**零改动**；AVAD 解码器扩 v2：遇 version≠2 → `abc.decode.version`（v1 字节按未知版本受控拒，缓存侧沿 798 r6 K14-06 = 确定性陈旧 miss 重建，不当新语义读）；profile≠3 → `abc.decode.profile`；**v1/v2 互不可读**（同族格式代际隔离，798 r6 先例） |
| 缓存键 | owned 单元键携带 producer/schema/语义版本/profile=3/类型表指纹轴；与 core/data 键空间不相交 |

## 2. 类型系统与资源分类（沿 774/782 派生规则）

`MirType` 新增：`Char`（Unicode 标量；域 0..=0x10FFFF 去代理对，verify 域检）、
`Str`（owned UTF-8 buffer）、`StrView`（借用只读视图：root 槽+标量范围+代际）、
`MutRef`（排他写 token：root 槽+代际）。

- **record 字段类型**：六标量 + Char + Str + 已存在 record 索引；**StrView/MutRef 入字段 → `mir.field-borrow`**（782 `verify.field-borrow` 镜像）。空 record 沿 820 `mir.record-empty` 拒。
- **资源分类派生**（不信输入自称）：record 传递字段闭包内任意可达 `Str` ⇒ 资源 record（不可 Copy，只可整根 move/clone/replace）；仅标量/Char/copyable record ⇒ copyable record（820 值语义原样）。copyable 不可能包含资源。
- **类型等价**：820 结构相等规则原样扩展到新类型（Char/Str/StrView/MutRef 为 primitive 种类；record 名义身份含逐字段类型）。StrView/MutRef 是 primitive 值类型（不携带名义身份）。
- **StrView/MutRef 只作局部/参数/返回/临时值**；不入字段（§2 首条）、不入 record 聚合操作数（AGGREGATE 操作数类型含 StrView/MutRef → `mir.type-op` 族拒）。

## 3. 常量与位型（V-06 冻结样本）

`MirConst` 新增 `Char(u32 标量值)`、`Str(Vec<u8> UTF-8)`；wire tag 6=Char(u32)、
7=Str(u32 len + bytes)。Str 常量解码即 UTF-8 校验（非法 → `abc.decode` 结构拒）；
Char 常量域检（代理对/超界 → decode 拒）。冻结样本：空串 `""`、`"A"`、NUL `"\0"`、
组合 `"e\u{0301}"`（len=2 标量/3 字节）、`"😀"`（len=1/4 字节）、`"中"`（3 字节）、
Char `0x0`/`0x1F600`/`0x10FFFF`。i32/bool/宽标量位型样本沿 820 §3 原样重放。

## 4. 指令集与 InstructionSpec 合同表

保留 820 全部 opcode（01–09 + term 10/11/12；owned profile 继承 data 侧 i32/bool
运算与宽标量数据面、FIELD_GET/FIELD_SET/AGGREGATE 形状）。**新增（仅 owned profile，
opcode 避开 term 位 10–12）**：

```
0A CLONE    dst,src                 深克隆（Str/资源record；逐Str字段独立buffer，标量按值，空Str零分配）
0B MOVE     dst,src                 整根转移：src 须 owned（Str/资源record）且 Init 且无活借用；
                                    提交后 src→Moved（move-kill）；copyable → mir.ownership-required
0C VIEW     dst,src                 只读借用：src:Str 槽 → dst:StrView（root=src 槽；注册读锁）
0D MUTREF   dst,src                 排他写借用：src:Str/资源record 根槽 → dst:MutRef；root 保守排他
0E BORROW_END token                 结束 VIEW 或 MUTREF（按 token 槽类型分派）
0F DROP     slot                    显式 drop：owned+Init → 清理一次、slot→Dropped；
                                    Uninit/Moved/Dropped 上 DROP → mir.drop-uninit
13 STR_LEN        dst,view          Unicode 标量数 → I32（"A中😀"=3）
14 STR_LEN_BYTES  dst,view          UTF-8 字节数 → I32（"A中😀"=8）
15 STR_CHAR_AT    dst,view,idx      标量位索引 → Char；越界 → trap av.trap.str-oob（782 ABI -10 镜像）
16 STR_SLICE      dst,view,start,end [start,end) 标量单位子视图；负/反向/end>len → trap str-oob；空范围合法
17 STR_CONCAT     dst,a,b           view×view → owned Str（770 概览面；源码绑定存在性 T-02 对 adapter 绑定表核实，typed fixture 可用）
18 STR_TO_OWNED   dst,view          view → owned Str（770 `to`；深复制范围字节）
19 STR_CMP        sub,dst,a,b       sub 1=eq 2=ne（标量逐位相等语义沿 Unicode 标量比较）→ Bool
1A CHAR_TO_CODEPOINT   dst,char     → I32（770 显式码点转换）
1B CHAR_FROM_CODEPOINT dst,i32      → Char；非法标量 → trap av.trap.char-invalid
```

共享 opcode 在 owned profile 的语义扩展（InstructionSpec 行更新，不改编码）：

- `02 COPY`：仅 copyable 类型（标量/Char/copyable record）+ **StrView（token 句柄复制=多读视图合法，V-02）**；Str/资源record → `mir.copy-owned`；MutRef → `mir.copy-owned`（排他 token 不可复制）。
- `05 CALL`：owned 实参（Str/资源record）= **move-in**（调用点 kill 调用者槽，callee 拥有并负责出口清理或经 return 移出）；StrView 实参=只读共享；MutRef 实参=token 传递（排他随 token 携带）。move-in kill 进入 move-kill 分析。
- `07 FIELD_GET`：Str 字段 → dst:StrView（借用视图，root=base 槽，782"字段读取=视图"）；Char/标量/copyable record 字段 → ReadCopy；资源 record 字段 → `mir.ownership-required`（永不隐式复制 owner）。
- `08 FIELD_SET`（owned=**替换事务**）：root 须 Local/Param 槽且非 Moved/Dropped；Str 字段位=value:Str（先物化）→ 提交 init/move 状态 → 旧 buffer 恰一次 free（V-03 顺序）；标量/Char/copyable 字段沿 820 重建语义；**root 有活 VIEW/MUTREF（未 BORROW_END，合流保守并集）→ `mir.mut-conflict`**；root 已 Moved → `mir.use-after-move`（字段写不复活已移出根）。
- `09 AGGREGATE`：字段表可含 Str（owned 构造）/Char；操作数含 StrView/MutRef → `mir.type-op` 拒。
- def 型指令（CONST/COPY/CALL/AGGREGATE/CLONE/MOVE/STR_CONCAT/STR_TO_OWNED）写 owned 槽 = Initialize 或 Replace（原值 Init ⇒ 先提交新值后清旧值，"先准备、再提交、后清理"）。

InstructionSpec 表（`core/src/instr_spec.rs`）为每个新 opcode 增行：operand read/def/kill
（**MOVE/CALL-owned实参 kill 源；DROP def=无 read=slot**）、类型规则、trap 位
（15/16/1B trap）、fuel（CLONE/CONCAT 按输出字节数计量、DROP 按清理字段数计量——
有界显式遍历）、来源（HIR 映射）、profile∈{owned}；`coverage_check` 漏项即红。

## 5. 访问状态机与借用/清理分析（§5.1/§5.4/§5.5 合同的 MIR 冻结形态）

**槽状态格**：`Uninit → Init → {Moved | Dropped}`；Param 入帧 Init（move-in 语义随 CALL）。
合流规则（§5.5 r3）：Init 取**交集**（must）；Moved/Dropped/活借用取**保守并集**
（任一路径发生即生效：分支一侧 move 合流读 = `mir.use-after-move`；V-11/V-12 冻结此
判别）。循环 back-edge 求有界不动点；零次迭代保守（活 owner 并集保含）。

**move-kill/借用/清理分析**（`core/src/ownership.rs` 新模块，扩 820 must-init worklist）：

- 只 intern 实际引用的根/投影及前缀（MovePath：VIEW/MUTREF/FIELD_GET/FIELD_SET 触达的
  槽+路径），不按每槽展开字段组合；worklist 步数计入 verify 预算（2M 步/5s），状态
  位图内存计入 verify-memory（16MiB，分配前核算）。
- 借用存活：VIEW/MUTREF def 起、BORROW_END 或帧出口止；出口未终结的 VIEW/MutRef
  无清理责任（借用无释放权）但活期阻止冲突写（并集）。**借入参数 view 在 callee
  返回后继续有效**（root 在 caller 帧）；**RETURN StrView 的来源合法域：root ∈ Param
  槽（view/move-in 参数字段链）或 Const 初始化且未被改写的槽**——验证器追踪 view root
  溯源（param|const|local|field-of 四格），非法来源 RETURN → `mir.borrow-escape`
  （782 镜像）；owned 局部/move-in 参数字段出帧 → 同码族。
- **清理义务表**：每出口（RETURN/break/continue/语句尾/trap/cancel/fuel/frame/memory
  耗尽/构造失败）按动态 init/drop guard 对**实际活根**生成逆创建序清理义务（跨类型
  LIFO 交错，782 统一 pending 表语义）；must-init 交集不充当清理表（§5.5 r3）。
  return 先把结果责任转出再清理其余活值。语句尾临时（view 之外的 owned temp）逆序清。
- **reservation**（FIELD_SET 前门）：LHS 根稳定+路径逐段求值一次后、RHS 前，验证器
  对同根静态保守判定——RHS 允许只读（VIEW/FIELD_GET/STR_LEN/STR_CMP/CLONE 读）与
  显式 clone；RHS 含 MOVE/写重叠根/DROP/以 Str·资源record·MutRef 为实参的 CALL（潜在
  move-in/mut effect）→ `mir.reservation-conflict`（保守侧拒，Design44 §5.4）。
  纯 Copy 自赋值合法；owned 自 move/重叠 move 拒。提交到旧值清理之间无用户代码/挂起。
- **运行期句柄域**（`core/src/place.rs` + `owned_text.rs`）：StrView/MutRef 为
  (frame, slot, generation, range) 句柄；每次使用前验 frame 存活+槽代际+类型
  （V-05：普通整数等于活资源 id 仍是整数；旧帧复用/跨 store/代际边界失效句柄在
  读取前受控拒 `av.trap.invalid-handle`，typed AV 层直接探句柄表）。
- **alloc 台账**：每次 owned buffer 分配记 (id, bytes, owner_root)；free 记 (id, 序)；
  live 计数/峰值全出口上 trace。宿主接收 owned 结果（typed entry 返回）持唯一释放
  责任，显式 release 后断言 live=0（首例 oracle）；无 reap_all 掩盖。
- **cleanup 预留**：清理遍历显式栈（无 Rust 递归），预留记账不受正常执行 fuel 耗尽
  取消；清理成本计 fuel 与峰值；cleanup 失败单报 `av.cleanup-failed` 并保留原错误。

## 6. entry 合同与宿主边界

- owned profile 完整模块 entry：参数 ∈ {六标量, Char}（值传入）；返回 ∈ {六标量,
  Char, **owned Str, 资源 record**}（宿主接收 owned 结果=唯一责任，TypedResult 显式
  release；V-01 首例断言）。StrView/MutRef 不出 entry。零参 entry 合法。
- core-i32 零参/i32 合同与 data entry 合同零改动；`execute_entry` 按 profile 分派扩展。
- unit 模式/闭包装配/imports 闭合沿 798 r6 原样；跨函数 CALL 的 owned/view/MutRef
  实参按 §4 CALL 行。
- 宿主测试资源 shim：仅 TypedResult 持有/释放观察（live=0）+ alloc/free 事件只读；
  无任意函数地址/回调（Design44 §10）。

## 7. HIR adapter 支持面（text-record 源域）

- 入口：`experimental/ac-core/src/source/adapter.rs::compile_text_record_source`
  （实存，adapter.rs:133）→ `verify_text_record`（774/782 全套语义/诊断在源域保真，
  源码路径的 `verify.*` 诊断码不变）；owned lowering 只消费其 CheckedModule
  （`HirDomain::TextRecord`）。纯 text 域（`compile_text_source`）Str 语法面同规则接入。
- **支持**（774/782 全部已验证形态）：Str/char 字面量；record（含 Str/Char 字段，位置/
  命名构造，源序求值）；字段读取=视图（len/len_bytes/char_at/slice/eq/ne/to/to_codepoint/
  from_codepoint/concat 以字段为接收者）；`r.clone()` 深克隆；`x.move` 整根转移（实参/
  let/assign/return 值位）；字段替换（Str 字段位，根=var/mut/move-in 参数）；mut 排他对
  （两层包装传播，owner 集=资源根）；完整清理责任（join/循环重入/短路标记/出口 LIFO）；
  move-in 参数（callee 根）；借入参数 view 返回。
- **拒绝**（MIR 层镜像码，typed fixture 防旁路；源码侧为既有 `verify.*` 码）：
  部分字段 move `verify.partial-move`（MIR 形状不可表达——FIELD_GET 不出资源值，
  MOVE 只吃整槽；源域码保持权威）；裸资源读/move 形态非法位 `mir.ownership-required`；
  let/view 根写 `mir.mut-conflict` 域内 `verify.immutable-write`；借 str 字段
  `mir.field-borrow`；借局部逃逸 `mir.borrow-escape`；record 域外的宽标量/Text/Core
  bundle → `lower.domain`。
- typed fixture = 未信任输入：手构 MirModule/ABC 与源码产物走同一 verify_mir/verify_abc
  授信，无旁路。

## 8. 预算与资源（798/820 冻结值零放宽；新增项只增不改旧值）

| 预算 | 值 | 分类 |
|---|---|---|
| 输入 / codec 页 / 表族 / 验证步·墙 / 验证内存·解码堆 | 沿 820 §8 全表原值（4MiB/256KiB/65536 族/2M·5s/16MiB·16MiB） | 同 820 |
| 执行 | ≤1M 步、≤1024 帧、wall ≤2s | 同 820 |
| **活值数据合计** | **≤8 MiB = 槽+聚合+Str buffer+环境+Dyn 合计**（deep_size 扩 Str buffer 字节；分配前拒、peak 全出口 trace） | `av.resource.slot-bytes` |
| Str 常量/值长度 | 单值 ≤8 MiB 自然受合计约束；常量入输入 4MiB | 输入帽 |
| VIEW/MUTREF token 数 | ≤ 槽表族 65536（token 即槽值） | `av.resource.table-entries` |
| CLONE/CONCAT/TO_OWNED 分配 | 真实分配逐次入账（attempt/successful/free/live/peak）；clone 遍历按字段计 fuel | fuel+slot-bytes |
| 清理遍历 | 显式栈；fuel 按清理项计；**预留记账不因执行 fuel 耗尽取消**；计入峰值 | `av.resource.cleanup` |
| 受管 store 分区 / worker 内存 | 32MiB(hot8/staging16/execution8)；双线 A≤224/B≤128MiB 沿 798 r7，owned worker 增量计入 B 线实测 | store/bench 门 |

分态、计步定义、准入错误 dispatch=0、运行期故障按 §5.5（先前合法事件保留、危险目标
访问/提交 0、无 panic）——全部沿 820/798 合同原文。

## 9. 存储、恢复与缓存（沿 783/820 模式）

owned 单元独立 producer `plan821/av-owned`、独立 LayerSpec；恢复重走 decode→verify→
load 门；依赖键含类型表指纹+**清理/借用分析 schema 版本**（ownership.rs 分析规则变
更 ⇒ 语义版本轴 bump ⇒ 键失效）；好 hash 坏 payload 不执行；活帧/借用 token/宿主地址/
alloc 台账不入 cache。同 store 新 worker 恢复重验四态（V-08）沿 820 模式。

## 10. 注册、recipe 与 AutoDown

- 测试前缀 `av_owned_821`（core+probe 两档，`selected>0` 断言）；脚本
  `scripts/verify-av-owned-821.ps1`，Scope `first-case|semantics|admission|resources|
  cache|compatibility|autodown|acceptance`（单次单 Scope）。
- **recipe id 冻结 `ac-av/owned-run-v1`**：plan §3 r1 字面 `av-data-821` 起草于 820
  T-01 冻结实际命名（`ac-av/data-run-v1`）之前；按已交付族式命名（ reconciliation
  记录于本节，非支持面变更）。case fixture 沿 820 实际交付形态内联于
  `experimental/ac-av-probe/probe/tests/av_owned_821*.rs` + typed fixture。
  **r2 修订（2026-10-25 T-02，窗口记录已预告）**：827 交付确立 case 目录新事实
  （`crates/auto-lang/test/cases/av-source-827/*.case.ad`+src/），首例 case 文件按同模式
  落 `crates/auto-lang/test/cases/av-data/821/`（两 case：fixture 深判 + int 孪生源路径
  exit 404；种子 SHA256 收据 README；case 单源约束故拆两文件）。fixture 源仍字节冻结于
  probe 侧 `owned_fixture.rs`，corpus 种子与之字节身份测试锁定。
- 注册：`crates/auto-lang-core/src/test_cases/ac_av.rs` **追加** owned recipe 分派
  （`AvRecipeFamily` 增柄；同 worker `av_probe_worker` 增 owned case-serve 分支，
  `avworker/1` 信封域内以 recipe id 区分不扩字段；不复制 sema worker、不改 v1/v2
  解码、不触他方 recipe 合同）。父侧期望独立判定：owned 结果**逐位+live=0+事件序**
  断言；公共注册写面串行（§0 窗口）。
- 工具身份轴：worker exe sha256 + `av-worker-version` + recipe + profile=3。

## 11. oracle（手审/独立来源，非被测算法生成）

- **首例**（T-02，V-01/08/10 正控）：text-record 源码（含非空 Str 字段 record）——
  从字段读 view（len/char_at 观测）、`clone()` RHS 后 replace 字段、return 整根转移；
  手审 expected（UTF-8 内容/长度/码点序）；AV Success 与 **782 native 链独立对照**
  （`compile_text_record_source`→native.rs 合并 lowering→COFF/PE，退出码/固定文本一致，
  plan §5.2"与旧782 native对照"）+ 同 store 新 worker 恢复重放 + 正式 recipe 父侧判定。
  宿主显式 release 后 alloc 台账 live=0。
- **位型/文本样本**（V-03/06）：§3 冻结样本逐位往返；`"A中😀"` len=3/len_bytes=8、
  `e\u{0301}` len=2、slice 空范围合法、char_at 越界 trap。
- V-01..V-13 期望全部手审冻结于各 fixture；V-10 反控（错 bits/漏 case/skip/错工具
  身份 → 总体非零/ERROR）。
- 期望不由被测布局/清理/借用算法生成；native 参照与 AV 路径相互独立。

## 12. case→Scope→真实入口→oracle 矩阵（必需=本计划验收必跑）

| Case | 子例（冻结输入两侧） | Scope | 真实入口 | 阶段/码 | 区分断言 | 必需/适用 |
|---|---|---|---|---|---|---|
| V-01 | 嵌套字段 replace RHS 读同根（len 视图）成功；RHS move 同根/以 Str 实参 CALL 同根对照 | first-case+semantics | verify+AV+av_owned_821 | `mir.reservation-conflict` | 只读 RHS 成功且 trace 恰一次求值；冲突前门拒、dispatch=0 | 必需 |
| V-02 | move 后读；两次 move；只读写；重叠 mut（MUTREF 活期 FIELD_SET）；合法多 view（多 VIEW+复制） | semantics | verify+AV | `mir.use-after-move`/`mir.double-move`/`mir.mut-conflict` | 各自特定诊断；合法对照不误拒 | 必需 |
| V-03 | 第 n 次 clone/concat/replace 分配失败；构造中途失败；重叠 clone 替换 | resources | alloc 注入 hook | Resource 分态 | 旧 bytes/owner 不变；未提交源未消耗；临时恰清一次；attempt>0 且 successful_alloc=0 | 必需 |
| V-04 | return/break/continue/trap(str-oob)/cancel/fuel/frame/memory 耗尽 | semantics+resources | AV 全出口 | 事件 oracle | 逆创建序唯一 drop（alloc 台账序断言）；结果转出不误释放；取消不吞清理 | 必需 |
| V-05 | 整数=活资源 id；旧帧复用；跨 store；generation 边界 | semantics | typed AV 句柄表 | `av.trap.invalid-handle` | 整数仍整数；失效句柄读取前受控拒 | 必需 |
| V-06 | 空串/NUL/组合 Unicode；view 返回合法来源（view 参数字段/静态字面量）vs 局部/move-in 逃逸；部分 move | semantics+admission | verify+AV | `mir.borrow-escape`/源域 `verify.partial-move` | 沿 774 结果/诊断；owned 返回宿主释放后 live=0 | 必需 |
| V-07 | cap 内/超 1 单位（8MiB 合计 ±1B）；瞬态峰值；故障第 n 次 | resources | ExecLimits+hook | Resource | 预算前拒 attempt=0；注入 attempt 存在 successful=0；真实峰值；无 panic | 必需 |
| V-08 | fresh/同 store 新 worker 重启/改 type·layout·profile/好 hash 坏 payload | cache | 783 恢复→重验→AV | 恢复门 | 合法恢复结果同；依赖变化不复用；非法执行计数 0 | 必需 |
| V-09 | core-i32 重放；已合入 817/818 scalar recipe 重放；owned 送旧 consumer | compatibility | 原 recipe/consumer | 原码位 | 旧结果/诊断不变；未知 profile=3 明确拒 | 必需 |
| V-10 | worker 错结果/错 bits；漏 case/skip；错工具身份 | autodown | 正式 recipe+父侧 | 父侧判定 | 每项错使总体非零/ERROR；缺工具不可绿 | 必需 |
| V-11 | 分支一侧 move 合流读；零次循环 move 状态；move 后 call 实参形态不符；strip debug 后 trap/cancel 清理；稀疏字段路径；MovePath 超预算 | semantics+admission+cache | verify+AV+store 恢复 | use-after-move/冲突授信前拒；Resource | 授信前拒 dispatch=0；合法根 move恰一新 owner 清一次；strip 不漏 drop；稀疏路径按实际增长；好 hash 不绕过 | 必需 |
| V-12 | 分支一侧 move 另侧保留；单臂初始化未发布临时；单臂活 view 合流后 mut；零/多次循环；两臂 end-borrow 后 mut | semantics+cache | verify+AV+恢复重放 | 合流前门拒/合法放行 | 合流 read/mut 不安全拒；两臂终结借用后 mut 可行；分支退出各按真实活根清一次不泄漏/双 drop；未初始化临时不清；准入/运行计数分态 | 必需 |
| V-13 | D828 基线 273575B/115 函数×30 修订；owned 两函数局部 body 更新；签名/layout/授权变化精确失效；强制 whole 前端退化 | acceptance（增量） | 828 session+owned 单元接口 | 计数/身份/IO | 未改对象/代际同、生产阶段/IO=0；P95<1s；退化非零 | 必需（T-09） |

## 13. 写面登记与 T-02 解锁条件（ACE 合同"登记写面"）

- **基面**：master@`c66528e80`（含 820 全部核心成果）；已交付 Spec：`av.data/1`
  (data-layout.md/data-place-profile.md)、828 增量接口（resident-function-install.md）。
- **本计划写面**（均待窗口）：`experimental/ac-av-probe/core/src/{contract,mir,mir_verify,
  instr_spec,abc,abc_verify,av,layout,lib}.rs` 扩展 + 新模块 `{place,ownership,owned_text}.rs`
  （plan §3）；`probe/src/{driver,data_driver,bin/av_probe_worker}.rs` owned 分支；
  `probe/tests/av_owned_821*.rs`；`probe/Cargo.toml`；注册 `crates/auto-lang-core/src/
  test_cases/ac_av.rs` 追加；`scripts/verify-av-owned-821.ps1` 新建。不改：AVAB 编码、
  data profile 语义、v1/v2 解码、他方 recipe、旧 VM、产品面。
- **只读消费者**：817（CTE，`d771712cf` 已为 820 Value 扩展做过最小适配——本计划 Value
  再扩展时同规则 notified 适配，受控拒不扩语义）、818（native，`7c482ceb8` 同类先例）、
  827（复用 recipe，core 消费）。
- **当前 owner**：共享核心窗口 = PLAN-827（在途未合入核心写，§0）。**T-02 解锁条件**：
  (a) 827 实际合入+窗口释放收据，本计划 rebase 到新 master 重核基面（av.rs/lib.rs 冲突
  面）后记录窗口接受；或 (b) 827 显式记录与 §13 写面不重叠的细分交接（其 av.rs 改动
  与本计划 Value/帧扩展重叠可能性高，默认不成立）。持窗等待期间不复制其 WIP。
- **兼容验证义务**（窗口接受后）：core-i32/data 双 profile 全量重放 + 817/818 已合入
  面重放（V-09）+ 827 recipe 面（若有 core 消费）不回归。

## 14. 留 T-02 实现核实的项（有界，非规则缺口）

- 首例 exact 源码文本与 hash（§11 形状为准）。
- `compile_text_record_source` 的 concat 源码绑定存在性（§4 note；MIR opcode 冻结不依赖）。
- worker `native-build --profile` 对 text-record 域的现有支持面（782 native 链实存，
  子命令面 T-02 核实）。
- avworker/1 信封内 owned 分派的最小编码位（recipe id 区分，不扩字段）。
- 828 session owned 单元接缝（`lower_function_owned_unit` 形状）——T-09 按 828 已合入
  接口对接（`FunctionCode::admit_unit`/`ProgramSnapshot::link` 模式）。

任何与本文冲突的实现选择都是 needs_fix；支持面/预算/兼容承诺扩大须回 new 显式 revision。

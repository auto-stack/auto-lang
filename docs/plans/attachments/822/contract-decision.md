# PLAN-822 T-01 合同冻结(contract-decision)

冻结日:2026-10-26。依据:PLAN-822 r5 §5/§6/§8 T-01,主检出 `5009dc629`(master)。
只读调查范围:`experimental/ac-av-probe/`(core 15 模块 + probe)、`crates/auto-lang-core/src/test_cases/`、
`crates/auto-test-case/src/contract.rs`、`crates/auto-lang/test/cases/av-data/821/`、
`scripts/verify-av-{data-820,owned-821,function-incremental-828}.ps1`、归档计划 817/818/820/821/827/828。
本文件冻结支持矩阵、身份、指令、访问/生命周期、预算、oracle 与 V 案例映射;T-02 起实现以本表为准,
偏离须回 new 显式 revision。行号引用基于冻结日 master。

## 0. 依赖门与写窗口核对(T-01 前置义务)

| 门 | 要求 | 实际状态 |
|---|---|---|
| D-821 | 前序 profile 实际交付+review+门禁+合入+收据 | `docs/plans/archive/821-av-place-owned-access.md` status=archived;git log `01747f278`/`5009dc629` 归档收据齐全 ✅ |
| D828 | 828 最终review/门禁/合入/Spec/merge 收据齐全 | `docs/plans/archive/828-av-function-incremental-session.md` status=archived,11/11 ✅ |
| 817/818 owner | 已合入、注册为活代码 | recipe `cte/scalar-query-v1`(`ac_cte.rs:22`)、`ac/mir-native-scalar-v1`(`ac_mn.rs:48`)✅ |
| 共享写面 | 公共核心/注册无他方占用 | 在途 worktree:lang-841(UI 域,executing)、lang-849(图形,reviewed),均不触及 `experimental/ac-av-probe`、`test_cases`、`auto-test-case` ✅ |
| 实验workspace | D 交付的独立 workspace | `experimental/ac-av-probe/` members=[core,probe],package 名 `auto-av-core`/`auto-av-probe` ✅(计划 §3 重核义务完成) |

**单写窗口接受记录(2026-10-26)**:D-821 与 D828 均满足,共享写面空闲,PLAN-822 接受窗口,
唯一实施树 `D:/autostack/.wt/lang-822/auto-lang` 分支 `plan-822-dev`,基面钉 master `5009dc629`。

## 1. Profile 身份(冻结)

新增第四 profile,与既有三族互不授信:

| 项 | 值 | 位置(实现落点) |
|---|---|---|
| AV profile | `av.collections/1` | `contract.rs` 新常量 `AV_COLLECTIONS_PROFILE` |
| MIR schema | `auto.mir.av-collections/1` | `MIR_COLLECTIONS_SCHEMA` |
| ABC schema | `auto.abc.av-collections/1` | `ABC_COLLECTIONS_SCHEMA` |
| producer | `plan822/av-collections` | `COLLECTIONS_PRODUCER_ID` |
| wire magic | `AVAD`(家族沿用;靠 version 隔离) | — |
| wire version | **3**(v3 新 generation;data=v1、owned=v2 与 v3 互不可读,decode `abc.decode.version` 拒) | `ABC_COLLECTIONS_VERSION=3` |
| numeric profile id | **4** | `ABC_PROFILE_COLLECTIONS=4` |
| 语义版本 | MIR/ABC/AV 全 `(1)`,独立轴 | `*_COLLECTIONS_SEMANTICS` |
| capabilities | 沿用 `hir.test.trace.v1`、`ac.runtime.text.v1`(owned Str 面继承) | 不新增 |

与三族无字符串/编号冲突(现值 `av.core-i32/1`=1、`av.data/1`=2、`av.owned/1`=3;schema/producer 无同名)。
collections profile 指令面 = **owned 指令集全集 ∪ 本表 §3 新指令**(owned record/Str/place 语义完整继承);
类型面 = owned 类型全集 ∪ List/Variant(§2)。InstructionSpec `profiles` 增成员 `"collections"`,
owned 行的 profiles 数组加入 `"collections"`。

## 2. 类型系统扩展(冻结)

`MirType` 新增 2 个变体(wire tag 现用 0..12,新从 13 起):

- tag 13 `List(ListTypeRef)`:`ListTypeRef { elem: TypeIdx }` — 名义引用模块类型表;
  模块类型表新增条目形态 `MirTypeDesc::List { elem: TypeIdx }`(与 record 同居一张表,合计入
  MAX_TABLE_ENTRIES)。
- tag 14 `Variant(VariantTypeRef)`:`VariantTypeRef { owner_module, name, arms: Vec<VariantArm> }`,
  `VariantArm { name: String, payload: Option<TypeIdx> }`(有限名义 enum;Option=2 臂
  None/Some、Result=2 臂 Ok/Err,均为语法糖形态,不设内建特权)。

**支持矩阵(元素/字段/payload 三域,首批)**:

| 域 | 支持 | 拒绝(verify 码) |
|---|---|---|
| List elem | i32/bool/i64/u64/f32/f64/char/Str/record(821 owned 语义)/Variant | `mir.type.list-elem`(StrView/MutRef/RecordView/Fn/List 自身/递归) |
| Variant payload | 同上 − Variant 自身首层允许(嵌套 Variant 允许,深度沿布局预算) | `mir.type.variant-payload`(List/借用族/Fn) |
| record 字段 | 821 原面不变(List/Variant **不进** record 字段) | `mir.type.nested-container`(record 含 List/Variant 字段) |
| List< List< _ > > | 拒绝(一层容器组合上限) | 同 `mir.type.list-elem` |

`Option<T>`/`Result<T,E>`/有限 enum 统一为 Variant;disc 为 `u32` 臂序(声明序 0..n,verify 期
名称→disc 映射冻结于类型表)。None≠Null≠Uninit≠false≠0(§5.1.5):payload 未构造分支的槽值
占位 = 类型化零(沿 821 `typed_zero`),与任何合法 Variant 值结构性可区分。

布局:`scalar_layout` 增行 — List 16/8(ptr u64 + len u32 + pad,容量在堆头)、Variant
max(8, payload_align)+8(disc u32 + tag pad;payload 就地 inline,无 payload 臂仅 disc)。
`deep_size` 扩展:List 每元素 `visited+=1` 逐元素计字节;Variant 计 payload。
`type_is_owned`(List/Variant=true)、`record_reaches_str` 泛化为 `type_reaches_owned`
(List/Variant 递归其 elem/arms)。

## 3. 指令集(冻结;opcode 自 0x1D 起,0x10..0x12 仍为 term 保留)

| op | 指令 | 操作数 | 语义要点 |
|---|---|---|---|
| 0x1D | LIST_NEW | dst, list_ty: TypeIdx, cap: u32 | 空列表,cap 检查 ≤MAX_LIST_ELEMS;分配记账 |
| 0x1E | LIST_LEN | dst, src | len → i32 槽 |
| 0x1F | LIST_CAP | dst, src | cap → i32 槽 |
| 0x20 | LIST_GET | dst, base, index | **仅 Copy 元素**(标量/char/bool);owned 元素读取走 LIST_REMOVE 或 LIST_FIELD_GET;负/越界先于访问 |
| 0x21 | LIST_PUSH | root, value | 扩容(倍增 0→4→2x,cap≤MAX_LIST_ELEMS);准备→容量/冲突检查→提交后 kill value(转移);失败旧列表不变 |
| 0x22 | LIST_INSERT | root, index, value | 同 push;index∈[0,len];元素后移按数据量计费 |
| 0x23 | LIST_SET | root, index, value | 原位 replace:旧元素 drop 记账、value 提交后转移;LHS reservation 沿 821 |
| 0x24 | LIST_REMOVE | dst, root, index | 转出 owned 值到 dst;后续元素前移;epoch bump;旧 Place 失效 |
| 0x25 | LIST_FIELD_GET | dst, base, index, path: Vec<u16> | Copy 标量字段读(list[i].f.g);路径 ≤MAX_FIELD_PATH_LEN |
| 0x26 | LIST_FIELD_SET | root, index, path: Vec<u16>, value | 元素内字段写(**首例核心**:list[i].inner.text replace);RHS 先物化;旧字段值 drop、提交后转移源;epoch 不变(元素原位)但元素 gen bump |
| 0x27 | VARIANT_PACK | dst, variant_ty: TypeIdx, disc: u32, payload: Option<SlotId> | 构造;payload 臂必须给槽,None 臂必须不给;提交后 kill payload 源 |
| 0x28 | VARIANT_DISC | dst, src | 当前 disc → i32 |
| 0x29 | VARIANT_GUARD | dst, src, disc: u32 | dst=Bool(当前分支==disc);**记录 guard 事实**(verify 静态格点);运行期恒不 trap |
| 0x2A | VARIANT_PROJECT | dst, root, disc | 需已证 guard;payload **move 到 dst**(责任转出);root 该分支进入 payload-moved 态;再投影/读 = verify 拒 `mir.variant.payload-moved`;Copy payload = 读值(root 分支保持 initialized) |
| 0x2B | VARIANT_SWITCH | root, disc, payload | 切分支:payload 先物化→提交(新 payload 转移入)→旧活分支(未 moved)按责任清理 drop 一次;guard 事实全失效 |

guard 事实失效(§5.5 r3 冻结):variant 重赋值(PACK 到同 root)、SWITCH、或该 root 任意
可能写入调用后,既有 guard 事实作废;BRANCH 复用既有 terminator(不新增 term)。

**陷阱分类(trap,运行期)**:`av.trap.list-index`(负/越界,含 insert 越界)、
`av.trap.stale-place`(epoch 失效后旧 Place/token 访问)、`av.trap.variant-branch`(运行期
错分支投影的纵深防御;静态缺 guard 是 verify 拒绝不是 trap)。资源类沿 `ResourceKind`
(扩 `av.resource.list-elems` — 单列表元素数超限)。

**verify 拒绝码(admission,dispatch=0)**:`mir.type.list-elem`/`mir.type.variant-payload`/
`mir.type.nested-container`/`mir.variant.guard-missing`/`mir.variant.payload-moved`/
`mir.variant.arm-mismatch`(PACK 臂/payload 组合错)/`mir.list.reservation-conflict`(沿 821
`mir.reservation-conflict` 判别族)/`mir.list.place-borrowed`(活借/活 token 下结构变更)。
运行期失败(合法程序)按 §5.5:越界/失效句柄/取消等允许此前合法执行已发生,断言危险
目标访问/提交计数 0 与原值保留,不要求 dispatch=0。

## 4. Place/epoch/生命周期(冻结)

- **epoch**:每 ListBuf 携 `buf_id: u64`(arena mint,沿 StrBuf.id 模式)+ `epoch: u64`;
  扩容(PUSH/INSERT 触发 realloc)、REMOVE(元素移位)epoch+=1。元素级写(LIST_SET/
  LIST_FIELD_SET)bump 所在元素的 per-elem gen(不失效同列表其他合法 Place)。
- **失效断言面**:epoch 变化后,旧 Place 形态(以 root 槽 + epoch 快照表达的访问)再访问
  = `av.trap.stale-place`/verify 拒(静态可见者);REMOVE 后对剩余元素的**重新计算下标**
  list[j](j<新 len)合法且读移位后新值——不是已 move 值(V-12)。
- **LHS/求值一次**:LIST_SET/LIST_FIELD_SET/LIST_REMOVE/VARIANT_SWITCH 的 index/RHS 槽
  只求值一次;RHS 先物化(821 不变量);提交前任何拒绝路径旧列表/源槽完全不变(部分新
  项零泄漏,清理记账不受影响)。
- **构造回滚**:PACK/LIST_PUSH 失败(预算/容量)→ 已 mint 的临时 payload/新 buffer 按
  责任清理后报 Resource/拒绝;成功才转移。
- **清理**:List drop(显式 DROP/frame 退出)按元素逆序责任清理(事件台账逆序断言,V-06);
  Variant drop 清当前活分支 payload 一次(payload-moved 态不重复清);清理循环按元素计
  steps 设取消点(沿 deep_size visited-fuel);cleanup 预算预留独立(沿 821 机制)。
- **821 兼容**:FIELD_GET/FIELD_SET/AGGREGATE/MOVE/CLONE/DROP 等对 List/Variant 类型槽
  的行为 = verify 拒 `mir.type.owned-op-on-collection`(首批显式不支持,不静默降级);
  Str/record 域行为逐字节不变。

## 5. 预算与资源(冻结;沿 D 交集,不放宽旧合同)

复用常量不复制:MAX_INPUT_BYTES=4MiB、MAX_TABLE_ENTRIES=65536、MAX_VERIFY_WORK_STEPS
=2M/5s、MAX_AV_INSTR_STEPS=1M/1024 帧/2s、MAX_AV_SLOT_BYTES=8MiB(deep_size 扩展后
含 List/Variant)、CANCEL_CHECK_INTERVAL、受管 32MiB 分区(hot8/staging16/execution8)、
worker private-bytes 增量 ≤128MiB(父侧采样,798 上限)。

新增仅一条:**MAX_LIST_ELEMS = 65536**(单列表元素数;len/cap/stride 全 checked u32→u64
算术,负下标/溢出不触 buffer)。计费规则(§5.4):PUSH/INSERT/SET/PACK/SWITCH 按新元素/
payload `deep_size` 字节计;REMOVE/投影按移出值计;清理按元素步进;均设取消点;
不以整聚合算一条低成本指令。32MiB/128MiB 两线分别验收(不互扣);front-end 瞬态峰值
Line A=224MiB 沿 bench.rs 现值另报。

## 6. 入口、语料、recipe 与脚本(冻结)

- 测试前缀:`av_collections_822`(probe 集成测试文件
  `probe/tests/av_collections_822_{first_case,semantics,admission,resources,cache_compat,recipe}.rs`;
  T-09 增 `av_collections_822_incremental.rs`)。主仓模块
  `crates/auto-lang/src/tests/av_collections_822_tests.rs`(3 测:corpus_green/
  negative_oracle_fails/worker_faults_are_infra,共享 `AV_CASE_LOCK`)。
- fixture 形态:**程序化 typed fixture**(沿 821 `typed_fixture.rs` 模式)——
  `collections_fixture.rs` 构造 `MirModule`(含新类型/指令),builder 产物是**未信任输入**,
  必须过 `verify_mir_with_deadline`→`encode_abc`→`verify_abc_bytes` 真门后执行;
  不新增 source 语法(§1 非目标),不把 builder 当 Checked。
- recipe(最小接线 9 处,全部沿 821 先例):worker `av_probe_worker.rs` `core_version_for`/
  recipe 判别加 collections 分支 → `auto-test-case/src/contract.rs`
  `REGISTERED_PROBE_RECIPES` 追加 **`ac-av/collections-run-v1`** + op 词汇/ends-with-run/
  av-binding 三处 → `auto-lang-core/src/test_cases/ac_av.rs` 常量
  `AV_COLLECTIONS_RECIPE="ac-av/collections-run-v1"`、`EXPECTED_AV_COLLECTIONS_CORE_VERSION
  ="av.collections/1"`、`RecipeFamily::Collections`、父侧冻结期望表 `COLLECTIONS_CASES`、
  `run_collections_check()` → `mod.rs` dispatch 臂。语料
  `crates/auto-lang/test/cases/av-data/822/{av-data-822.suite.ad(id=av-data-822),*.case.ad,
  README.md,src/}(若无需源文件则 README 记录身份收据)`。store session 沿
  `av798-session-<id>` 命名族(独立 id 域 `av822-*`)。
- 脚本 `scripts/verify-av-collections-822.ps1`:模板 = verify-av-owned-821.ps1 骨架
  (单 Scope 必选、selected>0、无 skip、required-id 恰一次、acceptance 子进程重放)+ 820
  的 GuardSelfTest/TIER_CONTRACT 思路。Scope 集:**first-case|semantics|admission|resources|
  cache|compatibility|autodown|incremental|acceptance**(与计划 §6 拟命令一致 + T-09 增量档)。
- 旧 profile 兼容(V-09):重放 `ac_av_probe`/`cte_probe`/`ac_mn_probe`/`av_owned_821`/
  `av_data_820` 既有入口(821 compatibility scope 模式);collections 工件送旧 consumer
  = 确定性 `abc.decode.version`/未知 profile 拒绝。

## 7. Oracle(冻结;不由被测布局/清理生成)

- **canonical face**(worker 输出):`{result:<canonical JSON>, live_after_release:0,
  allocs:[...], frees:[...], epoch_events:[...], variant_events:[...]}`;canonical JSON
  沿 821 `canonical_owned`(值→稳定文本,List=[...元素序]、Variant={disc,payload})。
- **父侧冻结期望表** `COLLECTIONS_CASES`(ac_av.rs):每 case 冻 result face + alloc/free
  id 序 + 清理事件逆序 + live=0;独立人工审计(README hand-audit 记录)。
- **独立小型容器状态模型**:probe 内**独立参考函数**(不共享 layout/ownership/清理代码)
  按固定规则推导期望序列,仅用于语义族回归对比(V-01/02/04/05/06);与父侧表互相独立。
- **反控 seam**:`AV822_COLLECTIONS_EXPECT_SABOTAGE`(result/ledger 两轴,任意轴必使总体
  非零)+ 沿用 `AV798_FAULT`(exit9/extra-frame/wrong-hash → Error,不是 Skip);
  漏 case/skip/selected=0 均 FAIL(V-10)。
- **不做** native/CTE 对照(822 非目标);821 first-case 的 native 对照不迁移。

## 8. V 案例 → Scope → 入口 → oracle 映射(必需矩阵;含 V-11/12/13)

| Case | Scope | 真实入口 | oracle/断言要点 | 必需性 |
|---|---|---|---|---|
| V-01 类型隔离 | first-case(+semantics) | probe `av_collections_822_first_case`:List<i32/Str/record> 保值 + 同位宽错元素/跨 owner 在执行前拒(admission 档断 dispatch=0) | 独立模型比对 + verify 码 `mir.type.list-elem` | 必需,无 skip |
| V-02 原位/epoch | semantics | `av_collections_822_semantics`:list[i].inner.text 写(LIST_FIELD_SET)、访问后扩容、活 view 下 push | 失效路径 `av.trap.stale-place`/verify 拒;i 只求值一次(步数断言) | 必需 |
| V-03 失败转移 | resources | `av_collections_822_resources`:push/insert 扩容超 1 单位、remove 越界、clone 第 n 项失败注入 | 原 len/内容/源 owner 保留;`attempt` 计数语义(fault 下 attempt≥1 且 successful_alloc=0);部分新项零泄漏 | 必需 |
| V-04 分支投影 | semantics | VARIANT_GUARD/PROJECT 正反控;Some/None、Ok/Err、同名异 enum、无 guard 读 payload | 仅合法 guard 投影;`mir.variant.guard-missing`;运行期错分支=variant-branch trap;inactive bytes 不读(mempeek 无越界访问事件) | 必需 |
| V-05 分支替换 | semantics | Some(Str)→None、Err(record)→Ok、构造新载荷失败 | 只 drop 旧活分支一次(事件序);失败旧分支完整;guard 失效后再投影拒 | 必需 |
| V-06 组合清理 | semantics | 嵌套 List<Result<record,Str>> 空/零长、trap/cancel 中途 | 固定结果 + 逆序清理事件一致;交回并显式 drop 后 live=0 | 必需 |
| V-07 资源边界 | resources | 新 worker + 父侧 mempeek 采样 + fault_at 注入;cap 内/超 1 单位/瞬态峰值/故障第 n 次 | resource 分态先于分配拒绝;危险访问/提交 0;真实峰值 ≤128MiB 与受管分区;无 panic | 必需 |
| V-08 恢复 | cache | 同 store 新 worker 重启(cache.rs collections LayerSpec)、改 type/layout/profile、好 hash 坏 payload | 恢复结果逐 face 相同;依赖变化 clean-miss;坏内层重验拒执行计数 0 | 必需 |
| V-09 旧消费者 | compatibility | 817/818/798/820/821 既有 corpus + collections 工件送旧 consumer | 旧结果/诊断不变;`abc.decode.version` 确定性拒 | 必需 |
| V-10 recipe 反控 | autodown | 主仓 `av_collections_822_tests` + suite av-data-822 + sabotage/fault/漏 case | 合法正控 PASS;每项错使总体非零/ERROR;缺工具不可绿 | 必需 |
| V-11 计量/准入 | admission+resources+cache | §3 全指令 verify/执行/恢复;错分支投影/remove 后旧 Place 与 fresh 索引正反控/扩容旧 Place/取消大型 clone/strip debug 后清理次数不变/稀疏索引预算 | 阶段计数按 §5.5 区分(准入非法 dispatch=0;运行期故障按目标危险操作 0);strip 清理计数不变;数据量计费可观察 | 必需 |
| V-12 guard/索引细目 | admission+semantics+cache | guard 后切 None 再投影 + 新 guard 正控;remove(0) 后旧 token 拒 + 剩余新索引得移位值;len/cap/stride 溢出与空列表 | 失效 guard/旧 Place 危险读前拒;先前合法执行保留;负下标/溢出不触 buffer;空列表构造→清理 live=0 | 必需 |
| V-13 增量四子case | incremental | D828 真实 source edit 基线(load256:273575B/115fn/≥30 修订,结构+P95<1s 双门)+ collections 两函数局部 body 更新 + 接口/layout/helper/授权变化精确失效/拒绝 + 强制 whole/换对象代际 gate 非零 | 未改函数对象/代际不变、生产阶段/IO=0;元数据更新不连坐;gate 非零可观察 | 必需(T-09) |

**cap 两侧输入**(每资源反例必须双值):MAX_LIST_ELEMS(65536/65537)、MAX_AV_SLOT_BYTES
(首例 ~KB 级/超 1 字节构造)、MAX_TABLE_ENTRIES(恰满/超 1)、budget fault(第 n 次注入/
不注入)、MAX_VERIFY_WORK_STEPS(界内/超 1 步)。

## 9. SD 草案映射

SD-01/02(auto-av/collections-variants.md、auto-mir/collection-place-profile.md)按本表 §1–§5
起草;SD-03(cache)按 §6 LayerSpec/恢复轴;SD-04(project)登记 profile 与非目标;SD-05
(test-case architecture)按 §6 recipe;SD-06(resident-function-install)沿 T-09 增量面。
review 按 D 实际 Spec 最终确认差异,merge 才沉淀。

## 10. 决策边界与已知限制

- 本表不改变 §5 语义/预算;实现中发现矩阵不可行项 → 回 new 显式 revision,不在下游猜。
- Programmatic fixture 是未信任输入(真 verify 授信);不冒称新增 source 语法能力(计划 §5.5
  末句);scalar 全链(V-09)仍须重放。
- record 含 List/Variant 字段、List 嵌 List、迭代器、Map/Set = 首批明确拒绝面,后续另案。
- T-02 若首例不可行:记录实际输入/拒绝原因/合同冲突点回规划,不弱化 verifier/oracle。

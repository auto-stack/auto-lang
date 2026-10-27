# PLAN-821 SD 草案（T-06 整理，供 review 核对 / merge 沉淀）

状态：草案（drafts）——canonical Spec 写入由 merge 拥有；本文按 plan §5 规范增量表
与实际交付整理 after 规则与证据锚点。所有"实际交付"均指 plan-821-dev 分支
T-02..T-06 提交（37b51f2a1 / 055ad06a9 / 7e9cceaed / 809fadbca / 3cb7cd5e5 及后续）。

## SD-01 add `docs/specs/auto-av/place-ownership.md`

- before：无本 profile 规则（auto-av Spec 仅有 core-i32 与 data 两 profile 面）。
- after（实际交付面）：
  - 身份：`av.owned/1` = profile id 3；MIR `auto.mir.av-owned/1`；ABC `auto.abc.av-owned/1`
    （AVAD magic、u16 version=2、profile=3；v1↔v2 互不可读=代际隔离）；producer
    `plan821/av-owned`；语义版本 MIR/ABC/AV = 1/1/1 独立轴；缓存键含 producer/schema/
    语义版本/profile=3/类型表指纹（types-fp）轴（V-08：改类型表=clean miss）。
  - 类型/资源分类：MirType 增 Char/Str/StrView/MutRef；record 字段=六标量+Char+Str+
    已存在 record；StrView/MutRef 入字段→`mir.field-borrow`；资源分类不信自称（record
    传递闭包可达 Str ⇒ 资源 record，只可整根 move/clone/replace；copyable 不含资源）。
  - 指令面（InstructionSpec 表 16 新行+共享 opcode owned 语义扩展）：0A CLONE/0B MOVE/
    0C VIEW/0D MUTREF/0E BORROW_END/0F DROP/13..1B 文本九方法+char 转换；02 COPY 仅
    copyable+StrView（token 复制），Str/MutRef→`mir.copy-owned`；05 CALL owned 实参=
    move-in；07 FIELD_GET Str 字段=视图；08 FIELD_SET=替换事务（root 保守）；09
    AGGREGATE 含 Str/Char。
  - 访问状态机：Uninit→Init→{Moved|Dropped}；Init 交集、Moved/Dropped/活借用保守并集；
    循环回边求有界不动点（worklist 防御性访问上限）；VIEW/FIELD_GET 视图按 root=base
    注册 token；MUTREF root 保守排他；DROP owned+Init+非 Moved/Dropped，根上活借用拒。
  - 运行期：arena 台账（alloc/free 事件、live、attempts/successful 分账、fault_at
    测试缝）；宿主接收 owned 结果持唯一释放责任，显式 release 后 live=0；view=774
    快照（Share 延长 buffer 生命周期，成功出口释放序可非严格 LIFO——恰一次+live=0 为
    恒定 oracle）；预算前检（分配前拒 attempt=0）+ 四分配点（Const-Str/Clone/Concat/
    ToOwned）exact deep delta（含槽头部 V）。
  - 预算：沿 798/820 冻结值零放宽（输入 4MiB/表族 65536/验证 2M 步·5s·16MiB/执行
    1M 步·1024 帧·2s）；活值合计 ≤8MiB（cap±1B 实测边界）；32MiB 分区与 128MiB
    worker 增量沿 798 r7（T-02 基线采样 p95 wall 25.6ms/峰值 ~1.7MB 每 fresh 进程）。
  - 拒绝码族：mir.{reservation-conflict,mut-conflict,use-after-move,ownership-required,
    copy-owned,field-borrow,borrow-escape,drop-uninit,type-op,entry-sig…}；运行期
    trap av.trap.{str-index,char-invalid}；av.trap.invalid-handle 无消费指令（冻结指令
    集无句柄解引用）——不可达构造，类型层隔离为实现面。
- 证据：semantics 25 例 + resources 6 例 + first_case 5 例 + corpus 4 check（脚本
  first-case/semantics/admission/resources Scope）。

## SD-02 add `docs/specs/auto-mir/owned-access-profile.md`

- before：无本 profile MIR 合同。
- after：SD-01 的 MIR 侧细化——owned 单元 OwnerScope.schema=`auto.mir.av-owned/1`；
  typed fixture=未信任输入（与源码产物走同一 verify_mir/verify_abc 授信，无旁路）；
  entry 合同（参数六标量+Char；结果可另含 owned Str/资源 record；view/token 不出
  entry）；move-kill/借用/清理分析的 MIR 冻结形态（ownership.rs：MovePath 只 intern
  实际触达根/投影、状态位图内存计入 verify-memory、分析超 16MiB=Resource 分态）；
  部分字段 move 在 MIR 不可表达（源域 `verify.partial-move` 保持权威）。
- 证据：V-11/V-12 矩阵（合流/回边/单臂 init/活 view join mut/两臂终结）。

## SD-03 modify `docs/specs/auto-av/cache-and-conformance.md`

- before：前序 profile（core/data）producer 身份/恢复/失效规则。
- after 增：owned 单元独立 producer `plan821/av-owned`、LayerSpec A21（schema
  `ac821/a21-owned-abc/v1`，key 轴含 types-fp+清理/借用分析 schema 版本）；恢复重走
  envelope→decode→深授信→load 门（restore_owned）；好 hash 坏 payload=checksum 拒；
  身份三向隔离（owned/data/core producer·schema 互拒）；活帧/借用 token/宿主地址/
  alloc 台账不入 cache；同 store 新 worker 恢复同结果（V-08 四态）。
- 证据：av_owned_821_cache_compat 5 例 + first-case 恢复例。

## SD-04 modify `docs/specs/auto-av/project.md`

- before：core-i32 与 data 两 profile 登记。
- after 增：登记 `av.owned/1`（profile id 3）支持面与明确非目标（部分字段 move/完整
  NLL/用户析构/循环共享/GC/List/闭包/Dyn/async/native ABI 不在本 profile；标量
  core-i32 与 data 面不变——三旧语料+mn 24/24 重放零退化）。

## SD-05 modify `docs/specs/auto-test-case/architecture.md`

- before：既有 probe/run recipe 登记（sema/abi/av-core/cte/mn/av-data）。
- after 增：`ac-av/owned-run-v1` recipe（append-only 三缝：REGISTERED_PROBE_RECIPES
  +op/形状/av-binding 围栏；runner 分派；worker case-serve owned 分支——avworker/1
  信封域内 recipe id 区分不扩字段）；owned 面=canonical result（标量 typed-bits 编码/
  Str 精确 UTF-8/record 名义名+声明序字段）+live_after_release+allocs/frees 台账；
  父侧冻结表 OWNED_CASES 独立判决（逐位+live=0+事件序）；V-10 反控缝
  （AV821_OWNED_EXPECT_SABOTAGE=result|ledger）与 worker 故障三态=infra。
- 证据：corpus 4 check + av_owned_821 runner 3 例 + ac_av 判决单测 5 例。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`

- before：HIR 域语义（774/782 text/text-record）与 MIR 消费为 core/data。
- after 增：TextRecord 域 CheckedModule→`lower_checked_owned`（owned MIR）实际消费
  支持/拒绝/资源责任——支持面=774/782 已验证形态（Str/char 字面量、record 构造/字段
  视图/clone/move/替换/mut 排他/清理/move-in/借入 view 返回）；源码路径 `verify.*`
  诊断码保持权威（MIR 镜像码不改源域）；语句尾临时 view 终结（adapter 规范形：RHS
  view 在 FieldSet 提交前 BorrowEnd）；unknown profile/域外输入→受控拒不进 fallback。

## SD-07 modify `docs/specs/auto-av/resident-function-install.md`

- before：D828 函数 IR/代码复用、受检安装/依赖/寿命（828 增量接口）。
- after 增（T-09 落地后回填）：owned 单元经 `lower_function_owned_unit` 接 828 编译/
  装载/snapshot 接口；纯 body 编辑不触发未改函数重编/重装载（V-13 四子 case）；
  签名/依赖/profile 变化精确失效；无活值布局迁移支持时不兼容安装受拒。
  【状态：T-09 执行中——本行证据由 T-09 完成后补齐，merge 前不可缺】

## 交付边界记录（review 关注项）

1. 陈旧读（view 跨根写）在受检 MIR 结构性不可达（root 保守门禁）；值层 Share 快照
   oracle 由 core place.rs/owned_text 固定——非缺口，为 782 root 保守语义的推论。
2. av.trap.invalid-handle 无消费指令（冻结指令集无句柄解引用）——V-05 以类型隔离+
   bin-type 拒+MutRefV{frame,slot,gen} 捕获实现；invalid-handle 保持不可达构造。
3. share 延长下成功出口释放序可非严格 LIFO（774 快照设计）；恰一次+live=0 恒定。
4. 预存债清偿记录：820 resources 两例 cap 重校准（Value 16→32B）；818 mn consumer
   穷尽匹配适配（受控拒，零语义扩面）。

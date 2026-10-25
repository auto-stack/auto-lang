# PLAN-821 SD 草案（R3 修复后重生，供 review 复核 / merge 沉淀）

状态：草案——canonical Spec 写入由 merge 拥有。本文按 R3 修复后的实际交付
（plan-821-dev R3 提交 `<本次提交>`，前置 R2 `18dee14f9` / R1 `e8e1e369d`+`e34a2c999`）
整理 before/after 与证据锚点。当前状态单一事实源：frontmatter `execution_done 7/9`。

## SD-01 add `docs/specs/auto-av/place-ownership.md`

- before：无本 profile 规则。
- after（R3 后实际面）：
  - 身份/版本/缓存键：av.owned/1=profile 3、AVAD v2（v1↔v2 互拒）、producer
    plan821/av-owned、语义版本 1/1/1、types-fp 键轴。
  - 类型：Char/Str/StrView/MutRef/RecordView(RecordTypeRef)（借入 record 参数，
    tag 12）；资源分类派生不变。
  - 指令集（R3 语义修订）：**FIELDMOVE（0x1C，owned-only）为逐字段消费**——
    dst 收字段值（record 为 Rc share、Str 为 handle——share 即转移），字段位
    置为类型化零值，root 代际 +1（root 上陈旧 view 退役）；**root 对其余字段
    保持活**（t_MutRet 包装按 f0+wb* 逐字段消费，双 mut/owned 返回全形状）；
    同路径二次抽取、部分消费后整根读均为 use-after-move。
  - 访问状态机：Return 全终态+逃逸按 provenance；Call 结果 view 溯源；
    MutRef 排他双向；owned def 目的地门；转移源 loan 门；FIELDMOVE 双侧
    borrow 门+逐路径 once。
  - 运行期：arena 台账（attempt/successful/fault_at）；静态 view 随带产出单元
    字节；Aggregate/FieldSet 提交后 kill 源槽；FIELDMOVE 经 `place::field_take`
    （Rc::make_mut 逐层）搬出+零值回填+代际 bump；Const/Clone（字节+字段）/
    Concat/ToOwned/FieldMove fuel；clone 显式工作表迭代（无深度截断）。
  - 文本接收者：`.len()/len_bytes()/char_at/slice` 对 owned Str 操作数先 VIEW
    （与 to_borrowed 同派发）；view 操作数直通。
  - 分析内存：token_roots 稀疏+moved_fields 稀疏（仅 record 槽计入 bound，
    8192 i32 槽零开销）+真实布局 bound。
- 证据：semantics 25 + resources 7 + r1_replay 9（含 R3 三形状+逐字段不变量）
  + first 5 + corpus check。

## SD-02 add `docs/specs/auto-mir/owned-access-profile.md`

- before：无本 profile MIR 合同。
- after：SD-01 的 MIR 侧 + RecordView 编码（单向强制转换/签名驱动调用 kill/
  外部根字段视图）+ **FIELDMOVE 编码（路径解析到 owned 字段类型；dst 等型；
  逐路径 once；稀疏 moved_fields 数据流+合并保守并集）**+ mut 参数写回 MIR
  形状（move-in + t_MutRet 包装结果 {f0:源结果, wb*:按 callee mut-record 参数
  序}+调用点按 **callee 自身 mut 参数位** 驱动的 FIELDMOVE/MOVE 回写——只读
  record 实参与非 record 实参不消耗 wrapper 字段；f0 可复制结果 FIELD_GET、
  owned 结果 FIELD_MOVE 转出）。
- 证据：r1_replay（R3 永久例：混只读 810/双 mut 8917/owned Str 返回 801/
  808 正控）+ incremental stale-link + corpus mut 写回。

## SD-03 modify `docs/specs/auto-av/cache-and-conformance.md`

- before：core/data producer 恢复规则。
- after 增：owned A21 层；worker owned-restore-run 真新进程恢复（信封→decode→
  深授信→执行，跨进程 canonical 面同结果）；自洽坏内层（正确 checksum 重封→
  深验证拒+dispatched=0+exit4）；types-fp clean miss；身份三向隔离。
- 证据：cache_compat 7 例。

## SD-04 modify `docs/specs/auto-av/project.md`

- before：core-i32/data 两 profile。
- after 增：av.owned/1 登记（RecordView 借入+mut 写回全部参数形状+全部 owned
  指令含 FIELDMOVE 逐字段语义）；非目标：部分字段 move（源值层面）/完整 NLL/
  用户析构/循环共享/GC/List/闭包/Dyn/async/native ABI。

## SD-05 modify `docs/specs/auto-test-case/architecture.md`

- before：六 recipe。
- after 增：ac-av/owned-run-v1（三缝注册/owned 面判决/冻结表/sabotage 双面/
  故障三态）+ corpus 借入正控（exit 15）+ mut 写回正控（exit 808）。
- 证据：recipe 3 + corpus check。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`

- before：HIR 域语义与 core/data 消费。
- after 增：TextRecord→owned 支持面——借入 record 参数（RecordView/无 kill/
  调用后有效）；move 参数（move-in）；**mut record 参数（782 ABI-E `*mut`）：
  排他写回全形状落地（单 mut/混只读/双 mut/owned Str 返回——R3 永久例
  808/810/8917/801）**；owned Str 文本接收者 view 派发；语句尾临时 view 终结。
  多参 `.move` 调用标记关联为 782 源域已知缝（KNOWN-DEBT 归 782 owner）。
- 证据：r1_replay R3 形状块。

## SD-07 modify `docs/specs/auto-av/resident-function-install.md`

- before：D828 函数 IR/代码复用。
- after 增（R3 后）：owned 生产安装通道 owned_session——
  - **双通道按编辑真实形状路由**（会话自身结构性证明，非测试声明）：纯单
    body 编辑（恰一个自由函数声明被触及、全部编辑严格在其 span 内、零
    `type` 声明触及）走 **D828 单函数常驻前端**——被编函数切片一次 fn-item
    parse + `prepare_candidate_document` 对保留 CheckedEnv 的 re-check/单 body
    发射（BodyEmitReceipt 生产者证明：real_body_plans=1、其余 body-less
    scaffold）+单单元 lower/verify/encode/admit；**未改单元 parse/recheck/emit
    生产工作=0**。接口/上下文编辑（type-context 变化、recheck_swap 签名漂移
    门拒绝、多声明/声明集变化）走整模块通道——全单元重前端+重产（正确失效
    语义，非静默降级）。
  - **scaffold 模式事实**：单函数文档的 body-less scaffold 无参数模式表，
    t_MutRet 包装 ABI 所需的 callee mut-record 参数位由会话保留的接口事实
    （sig_mode_overrides，冷/整轮捕获）注入 probe 侧 lower（文档类型节不可
    表达）；文档自带 body 恒优先。
  - 候选事务化保持（R2）：前端→失效→逐单元生产→整集受检 LINK→entry RUN
    全部成功后原子发布（EnvStep 分段：单 body 提交 FnCandidate::commit 一次、
    整轮重建环境——均在发布点应用）；任何失败保 source/revision/identity/
    residents/production/可运行性。
  - **Production 六计数**（parse/recheck/emit/lower/encode/admit，逐单元）+
    整模块前端计数；**incremental_gate 通道感知**：单 body 编辑整模块前端
    必须零增、前端三元组仅期望集推进（真整前端在 body 编辑上即红）；整模块
    编辑恰 +1 前端、全单元前端三元组推进、backend 仅失效集；居民 Arc 身份+
    mint 代际双保、成员关系（forced whole、对象换不换代际、漏失效、静默移除、
    虚报期望集都红）。
- 证据：incremental 6 例（含 R3 永久反例 main-only 零未改前端工作+虚报整集
  反控）+ 事务反例（失败候选 revision/production/旧程序三保）。

## 待 review 裁定项（未变）

**V-05 运行期句柄探测**：冻结指令集无句柄解引用指令——当前以类型隔离+
bin-type 拒+MutRefV{frame,slot,gen} 捕获实现；旧帧复用/跨 store/generation
边界运行期探测面不存在（无消费指令）。若冻结矩阵要求运行期探测 oracle，须扩
指令或以故障注入探 MutRefV 域（合同修订路由）。该矩阵行未关闭，属具名必需
反例缺口，待 review 对账——不以"不阻塞"替代裁定。

## 交付边界记录（R3 后）

1. 陈旧读在受检 MIR 结构性不可达（root 保守门禁）；值层 Share oracle 在
   core place.rs。
2. share 延长下成功出口释放序可非严格 LIFO（恰一次+live=0 恒定）。
3. 预存债清偿：820 resources cap 重校准、818 mn consumer 受控拒适配（含 R2
   FIELDMOVE 二次适配）；**R3 追偿：R2 提交漏更新的 820 instr-spec 计数钉
   （12→28 共享表）在兄弟回放中暴露并修正**。
4. 多参 `.move` 调用标记关联错位（782 源域缝，KNOWN-DEBT 归 782 owner）。
5. 单 body 快速道当前仅覆盖顶层自由函数（方法在 type 块内，span 随类型项
   走整模块通道——结构性保守，正确失效语义）；候选文档 span 为切片相对坐标
   （诊断面；身份 span 无关）。
6. 资源门四线（B-line/hot/staging/execution）各按自身量纲判定；store space()
   为磁盘信息列，不代受管内存。

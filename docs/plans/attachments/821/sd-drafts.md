# PLAN-821 SD 草案（R2 修复后重生，供 review 复核 / merge 沉淀）

状态：草案——canonical Spec 写入由 merge 拥有。本文按 R2 修复后的实际交付
（plan-821-dev R2 提交 `18dee14f9`，前置 R1 链 e8e1e369d/e34a2c999）整理
before/after 与证据锚点。当前状态单一事实源：frontmatter `execution_done 7/9`。

## SD-01 add `docs/specs/auto-av/place-ownership.md`

- before：无本 profile 规则。
- after（R2 后实际面）：
  - 身份/版本/缓存键：av.owned/1=profile 3、AVAD v2（v1↔v2 互拒）、producer
    plan821/av-owned、语义版本 1/1/1、types-fp 键轴。
  - 类型：Char/Str/StrView/MutRef/RecordView(RecordTypeRef)（借入 record 参数，
    tag 12）；资源分类派生不变。
  - 指令集（R2 增补）：**FIELDMOVE（0x1C，owned-only）**——破坏性整字段移出
    （dst 收 Rc share，root use-once 终结）；mut record 参数写回的机器指令。
  - 访问状态机：Return 全终态+逃逸按 provenance；Call 结果 view 溯源；
    MutRef 排他双向；owned def 目的地门；转移源 loan 门；FIELDMOVE 双侧
    borrow 门+root 终结。
  - 运行期：arena 台账（attempt/successful/fault_at）；静态 view 随带产出单元
    字节；Aggregate/FieldSet 提交后 kill 源槽；FIELDMOVE 移出+kill root；
    Const/Clone（字节+字段）/Concat/ToOwned/FieldMove fuel；**clone 显式工作表
    迭代（无深度截断——类型链深度由准入表预算界定，R2 关闭 R1 回归）**。
  - 分析内存：token_roots 稀疏+真实布局 bound（8192 槽 0 token 1.6MB bound vs
    1.37MB 实测）。
- 证据：semantics 25 + resources 7 + r1_replay 7 + first 5 + corpus 5 check。

## SD-02 add `docs/specs/auto-mir/owned-access-profile.md`

- before：无本 profile MIR 合同。
- after：SD-01 的 MIR 侧 + RecordView 编码（单向强制转换/签名驱动调用 kill/
  外部根字段视图）+ **FIELDMOVE 编码（路径解析到 owned 字段类型；dst 等型；
  root 记录；use-once）**+ mut 参数写回 MIR 形状（move-in + t_MutRet 包装结果
  {f0:源结果, wb*:末值}+调用点 FIELDMOVE/MOVE 回写）。
- 证据：r1_replay + incremental stale-link + corpus mut 写回。

## SD-03 modify `docs/specs/auto-av/cache-and-conformance.md`

- before：core/data producer 恢复规则。
- after 增：owned A21 层；worker owned-restore-run 真新进程恢复（信封→decode→
  深授信→执行，跨进程 canonical 面同结果）；自洽坏内层（正确 checksum 重封→
  深验证拒+dispatched=0+exit4）；types-fp clean miss；身份三向隔离。
- 证据：cache_compat 7 例。

## SD-04 modify `docs/specs/auto-av/project.md`

- before：core-i32/data 两 profile。
- after 增：av.owned/1 登记（RecordView 借入+mut 写回+全部 owned 指令含
  FIELDMOVE）；非目标：部分字段 move/完整 NLL/用户析构/循环共享/GC/List/闭包/
  Dyn/async/native ABI（mut record 写回已按原合同实现，不再是缩减项）。

## SD-05 modify `docs/specs/auto-test-case/architecture.md`

- before：六 recipe。
- after 增：ac-av/owned-run-v1（三缝注册/owned 面判决/冻结表/sabotage 双面/
  故障三态）+ corpus 借入正控（exit 15）+ **mut 写回正控（exit 808）**。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`

- before：HIR 域语义与 core/data 消费。
- after 增：TextRecord→owned 支持面——借入 record 参数（RecordView/无 kill/
  调用后有效）；move 参数（move-in）；**mut record 参数（782 ABI-E `*mut`）：
  排他写回经 FIELDMOVE 编码落地（callee 写 caller 可见——808 永久例）**；
  语句尾临时 view 终结。多参 `.move` 调用标记关联为 782 源域已知缝
  （KNOWN-DEBT 归 782 owner）。

## SD-07 modify `docs/specs/auto-av/resident-function-install.md`

- before：D828 函数 IR/代码复用。
- after 增（R2 后）：owned 生产安装通道 owned_session——**候选事务化**（前端
  编译→HIR 失效检测→逐单元 lower/verify/encode/admit→整集受检 LINK→entry RUN
  全部成功后原子发布；任何失败保 source/revision/identity/residents/
  production/可运行性）；**incremental_gate 覆盖整个生产面**（每修订恰一次前端
  编译计数/单元生产仅失效集/居民 Arc 身份+mint 代纪双保/成员关系——forced
  whole、对象换不换代际、漏失效、静默移除都红）；GateSnapshot 公共缝；原子
  批量编辑；拒绝中间形回滚。热路径整模块前端仍是当前真实生产点（计数器计量
  可见）；D828 单函数前端通道（incremental_reparse/prepare_candidate_document）
  的 text-record 域接入归 828 owner 语法视图扩展（跨仓写面）。
- 证据：incremental 5 例 + R2 holdout（事务回滚三保）。

## 待 review 裁定项（唯一遗留）

**V-05 运行期句柄探测**：冻结指令集无句柄解引用指令——当前以类型隔离+
bin-type 拒+MutRefV{frame,slot,gen} 捕获实现；旧帧复用/跨 store/generation
边界运行期探测面不存在（无消费指令）。若冻结矩阵要求运行期探测 oracle，须扩
指令或以故障注入探 MutRefV 域（合同修订路由）；不阻塞其余验收。

## 交付边界记录（R2 后）

1. 陈旧读在受检 MIR 结构性不可达（root 保守门禁）；值层 Share oracle 在
   core place.rs。
2. share 延长下成功出口释放序可非严格 LIFO（恰一次+live=0 恒定）。
3. 预存债清偿：820 resources cap 重校准、818 mn consumer 受控拒适配（含 R2
   FIELDMOVE 二次适配）。
4. 多参 `.move` 调用标记关联错位（782 源域缝，KNOWN-DEBT 归 782 owner）。
5. owned 热路径整模块前端（计数器可见，D828 单函数通道跨仓归 828）。

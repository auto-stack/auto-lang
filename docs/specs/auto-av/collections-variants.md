# av.collections/1 类型化集合与 Variant 载荷（PLAN-822）

> Status: experimental（执行域 fixture + **源码域已接通**（PLAN-864：`auto.source.collections.draft` 经共享 lowering 落本 profile，指令/布局/预算零改动；源码形态合同见 [auto-ac/source-collections-variants.md](../auto-ac/source-collections-variants.md)）；实现=`experimental/ace/crates/av-core` + `probe`）。

第四个 AV profile `av.collections/1`：身份四元组 = AVAD wire **v3**（`AVAD` 家族）/ profile id **4** / producer `plan822/av-collections` / 语义版本 (1,1,1)（mir/abc/av 三轴独立）。支持 List/有限名义 Variant/owned Str 与 record 组合的 typed 元素存取，不复制协议/执行器，不依赖旧 core-i32 授信。

## 支持域（冻结；"一层容器"不是 Variant 深度上限）

- **List 元素域**：i32/bool/Str/record/variant（同 owner 名义类型）；禁 StrView/MutRef/Place/List（List 不嵌 List）、禁容器 record 字段。
- **Variant payload 域**：Str/record/variant——**允许任意深度无环嵌套**（受共享预算与资源门约束，无语言深度上限）；禁 List payload、禁未用表项自环/间接环（MIR+ABC 双门拒）。
- record 字段不含容器；Place 仅参照 List 元素（R3 起类型绑定）。
- 首批入口为公开未信任 typed fixture（真 verify/ABC 授信后执行）。
- **PLAN-864 源码绑定**：真实 `.at` 源（List/Option/Result/enum/is/for-in/owned view）经
  `auto.source.collections.draft` 前端 + `auto-mir-lower` collections 模式降级为**且仅为**本 profile
  既有指令序列（无新 opcode；变体槽写走 VARIANT_SWITCH；unit 效果语句无槽；clear=尾删循环展开），
  真三 gates 后执行；typed fixture 通过不冒领源码行——源码行以文件化 17 行
  （`ac-av/collections-source-run-v1`，`test/cases/ace-m1/collections-source-864.suite.ad`）独立判定。
  SEM-02 冻结：唯一索引陷阱码 `av.trap.list-index`（负下标/越界/空容器同码，先于 buffer 访问）；
  旧 VM 负索引差异为 profile 差异，不静默统一。

## 指令面（opcode 0x1D..0x2E 共 18 条）

LIST_NEW/PUSH/INSERT/REMOVE/GET/SET/LEN/CAP、LIST_FIELD_GET/LIST_FIELD_SET、VARIANT_PACK/DISC/GUARD/PROJECT/SWITCH、CLONE/MOVE/DROP，及 R2 补齐的 Index Place 三条 LIST_PLACE/PLACE_GET/PLACE_SET。

- **Index Place**：铸造绑定（frame/slot/buf_id/epoch/index/elem_gen）；结构性变更（push/insert/remove/元素替换）bump epoch 或 per-element generation——旧 place/token 用即 trap（stale-place），fresh 索引读当前值；越界/负下标先于任何 buffer 访问 trap；跨帧提交按激活序定位所有者帧；局部根 place 逃逸/record 字段域在 MIR 拒。
- **Variant**：guard 事实证明后可投影；owned 投影为 move（payload-moved 静态拒，drop 不复位）；SWITCH 构造完整新载荷、提交后旧活分支 drop 恰一次、guard 失效；inactive 载荷不可读。
- **陷阱分类**：list-index / stale-place / variant-branch；资源分类含 list-elems（MAX_LIST_ELEMS=65536）。

## 状态与计量（V-11/§5.4）

- 大 clone/构造/清理**按数据量计费**：CLONE 燃料 = 每节点 1 结构单位（两阶段）+ 每 fresh Str 缓冲输出字节数（拷贝前充，预算拒先于分配尝试）；裸 Str(L) 恰为 L+2。墙钟单坐标系：剩余 allowance 自 helper 入口计量，零 allowance 入口轮询确定性拒。
- 表/图验证与清理**显式遍历**：清理顺序键发现、末主释放与部分克隆回滚（燃料/取消/墙/fault 全部放弃出口）走同一显式工作栈，任意深度无环链不乘宿主栈（256KiB 小栈 3072 层证人）；末引用 Variant 载荷/record 字段经 try_unwrap 取与出，共享句柄为纯责任转移（MOVE/COPY/call move-in 不抽干共享缓冲）。
- 失败转移：变异型指令 precheck→commit，预算拒先于任何变异；list 元素逆序释放、容器 buffer 最后；Place 根集合并入 token_slots 预算分类先于分配。

## 资源与恢复边界

- 沿 798 精确族口径（模块类型+常量+导入+函数合计、每函数槽+块+指令合计均 ≤65536；index 上限另小于表长）；值数据 ≤8MiB、受管 32MiB、worker 增量 128MiB 父侧采样。
- 值恢复身份见 [cache-and-conformance.md](cache-and-conformance.md) A22 节（profile/types-fp/entry/snapshot 四绑定 + owner 重盖章）。

证据与边界：`av_collections_822` 语义/资源/缓存/增量档与 attachments/822 各轮复审记录；touched_goals 为空（有限实验扩面，不核销生产目标）。

## PLAN-915：R02/R03 正式场景证据（2026-11-02）

- **R02（构造/clone/扩容失败注入）**：L04/L05（本 profile 源码域）与 V05 的每个真实分配
  attempt 现由 `ac-av/resource-run-v1` 逐点注入并父侧冻结计数；本 profile 的 List 容量
  扩容是真实分配 attempt（值 seam 覆盖），List/Str 缓冲 attempt 由 arena seam 覆盖。
  指令/布局/预算零改动，生产默认不配置 seam。
- **R03（旧 buffer/活借用）**：Index Place 的 `buf_id/epoch/elem_gen` 使用期校验由
  typed 授信序列（verify_mir→ABC admit→执行）正式见证：push/insert/remove/元素替换后
  旧 token 均 `av.trap.stale-place`。"先于 buffer 访问"由门控计数与 test-only
  access-before-validate 控制测量（正确实现 buffer_reads 只含合法读）；fresh token 与
  无变更 mint→read 是实测正控，活 Share/Static 缓冲的可读性为继承不变量（本计划未新增
  该面证据）。完整合同见
  [resource-fault-and-generation-conformance.md](resource-fault-and-generation-conformance.md)。
- 边界不变：源码不铸造 Place（`source.view-escape`/`source.iterate-mutate` 仍静态拒绝），
  本 profile 仍不承诺通用失效句柄校验。

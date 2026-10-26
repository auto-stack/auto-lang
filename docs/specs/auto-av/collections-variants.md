# av.collections/1 类型化集合与 Variant 载荷（PLAN-822）

> Status: experimental（fixture-only 支持面；实现=`experimental/ac-av-probe/core` + `probe`）。

第四个 AV profile `av.collections/1`：身份四元组 = AVAD wire **v3**（`AVAD` 家族）/ profile id **4** / producer `plan822/av-collections` / 语义版本 (1,1,1)（mir/abc/av 三轴独立）。支持 List/有限名义 Variant/owned Str 与 record 组合的 typed 元素存取，不复制协议/执行器，不依赖旧 core-i32 授信。

## 支持域（冻结；"一层容器"不是 Variant 深度上限）

- **List 元素域**：i32/bool/Str/record/variant（同 owner 名义类型）；禁 StrView/MutRef/Place/List（List 不嵌 List）、禁容器 record 字段。
- **Variant payload 域**：Str/record/variant——**允许任意深度无环嵌套**（受共享预算与资源门约束，无语言深度上限）；禁 List payload、禁未用表项自环/间接环（MIR+ABC 双门拒）。
- record 字段不含容器；Place 仅参照 List 元素（R3 起类型绑定）。
- 首批入口为公开未信任 typed fixture（真 verify/ABC 授信后执行）；无 source 支持面。

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

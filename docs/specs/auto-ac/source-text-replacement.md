# auto.ac source-text 局部替换合同（PLAN-894）

> §5.1 能力扩面的 canonical 合同：拥有 `Str` 局部（`var` 目标）的完整根 `=`
> 替换。本文件只定义该能力；通用文本/资源规则见
> [source-text.md](source-text.md) 与 [data-resource-contract.md](data-resource-contract.md)。

## 准入（verify 层）

- 目标：**局部**（非字段/参数整体）、类型拥有 `Str`、`var`（可变——`let`
  目标仍由结构门 `verify.immutable-write` 拒）、已初始化且 live。
- RHS（沿既有拥有 allowlist，无隐式复制）：拥有字面量提升、concat、显式
  `.clone()`、已有拥有返回、异根显式 `.move`；`c = c + "中"` 合法（RHS 读
  旧值=语句临时视图）。
- text 与 text-record 域同规则；无资源类型与 record 字段/根规则不变。

## 拒绝面（具名码）

| 形态 | 码 |
|---|---|
| 裸 owned 局部 RHS（隐式转移） | `verify.ownership-required` |
| 已移出/drop/作用域退出目标重建（`=` 非构造） | `verify.use-after-move` |
| RHS 消耗目标自身（`c = c.move`、`c = f(c.move)`） | `verify.reassign-conflict` |
| 目标存活借用下替换（根保守） | `verify.mut-conflict` |
| 未初始化目标赋值 | `verify.uninitialized-read` |

## 事务与计划事实

- 顺序：RHS 完整物化（旧目标保持）→ 语句临时视图结束 → 无用户代码的提交段
  安装新 owner。旧缓冲在 RHS 后释放恰一次（`TextResourcePlan.replace_frees`
  空 path 条目=纯 owned 根，沿 buffer_frees 约定）；新值出口释放恰一次。
- 代际：替换后 bump 构造代际——旧构造 backing 读作陈旧（与 fresh `let` 同
  机制）；`c = d.move` 后 d 置 dead（沿 move 规则）。
- 失败边界：RHS 失败=旧目标未提交、部分临时按现行合同清理；不回滚任意 RHS
  副作用。目标 move/mut 静态拒绝（本表），无 NLL/新 unwind/新 ABI。

## 执行层映射

- lowering：局部 Assign 按拥有类型发 `Move`（RHS 先物化、`end_stmt_temp_views`
  后提交；MIR_LOWERING_VERSION≥4）——AV arena 与 native shim 观察旧/新各
  恰一次释放；native 沿既有帧/OS 边界（不扩全栈 unwind）。

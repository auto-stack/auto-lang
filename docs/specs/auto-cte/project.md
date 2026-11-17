# auto-cte 项目卡

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-comptime-contract、ace-comptime-core与ace-comptime位于crates/comptime-*，cte worker协议及adapter保持。普通语言cargo t不构建ACE；cargo t ace选择常规集合，模块常规组排除`::performance::`，显式ace-perf保原专项和适用条件。

> Status: active（experimental 探针面；不接生产默认编译入口）(plan-817)
> 实现：`experimental/ace/crates/{comptime-contract,comptime-core,comptime}` 三个职责包，
> 同属 `experimental/ace/Cargo.toml` 独立workspace，消费其唯一 Cargo.lock；repo根exclude隔离语言构建。

## 定位

AV 驱动的有限 CTE（compile-time evaluation）查询与冻结缓存闭环：
`CT 查询 → 已检查 callee HIR → 798 共享 MIR → ABC → AV → 冻结结果 → consumer 推导/HIR 重验`。
首个可执行切片仅支持 i32/bool 标量；测试/查询接口是版本化 recipe IR
（`ac817/cte-recipe-ir/v1`），不新增 Auto 语言语法，不冒称全语言 comptime。

## 模块

| 组件 | 文档 | 职责 |
|---|---|---|
| scalar-query | [scalar-query-profile.md](scalar-query-profile.md) | 有限真实 AV 查询/能力/Meta 实参/consumer 回灌 |
| frozen-cache | [frozen-cache.md](frozen-cache.md) | A05 真实 producer、依赖/新鲜性/预算顺序/周转门 |

## 边界（非目标）

完整元编程语法、类型/声明生成、type-as-value 执行、用户泛型实例化、runtime 反射、
text/record 运行值、任意 IO/env/时间/随机/网络/FFI、JIT、发布器、自举。只读消费
796/798/783/ac-core 公开 API；MIR/AV 接口零复制（无第二套推导/执行器）。

## 观察事实分账（PLAN-1037）

执行成功后发布失败仍保持 `av_reached=true`：尝试（`ct_execute` 与执行边界日志的
`ok` 条目）、发布（`ct_freeze_publishes`）、命中（`ct_hits`）与语言结果分开记账，
失败发布不重跑 AV、不写页；未写的页不产生幻觉命中，下一次请求真实重执行，修复发布
后的 hit 才闭环。AV 前拒绝（前门/准入/`forbid_av` 冷 miss）保持 `false`。
命名实参与位置/移动实参同属一条 `Arg.value` 子树：发现、闭包、meta、替换、查点与
产品回灌按同一顺序遍历。

## 验证

`powershell -File scripts/verify-ac-cte-817.ps1 -Scope <scope>`
（first-case…closeout|all；resources/release 串行）；测试族
`cte_{first,admit,schedule,cache,freeze,resources,turnaround,adapter,snapshot}_`；
证据矩阵与逐 V 落点：[docs/reports/plan817/acceptance-matrix.md](../../reports/plan817/acceptance-matrix.md)。

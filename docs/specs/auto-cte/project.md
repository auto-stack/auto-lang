# auto-cte 项目卡

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-comptime-contract、ace-comptime-core与ace-comptime位于crates/comptime-*，cte worker协议及adapter保持。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> Status: active（experimental 探针面；不接生产默认编译入口）(plan-817)
> 实现：`experimental/ace/`（contract/core/probe 三包独立 workspace，跟踪
> Cargo.lock；repo 根 `Cargo.toml` exclude 独立解析）。

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

## 验证

`powershell -File scripts/verify-ac-cte-817.ps1 -Scope <scope>`
（first-case…closeout|all；resources/release 串行）；测试族
`cte_{first,admit,schedule,cache,freeze,resources,turnaround,adapter,snapshot}_`；
证据矩阵与逐 V 落点：[docs/reports/plan817/acceptance-matrix.md](../../reports/plan817/acceptance-matrix.md)。

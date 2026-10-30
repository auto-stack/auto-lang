# auto-semantics 项目卡（AC 语义元信息与类型推导探针）

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-semantics-contract、ace-semantics与ace-semantics-store位于crates/semantics-*，仍为实验实现。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> **Status**: experimental（PLAN-796 探针冻结契约；实现面=独立探针 workspace，非生产）
> 来源：PLAN-796（[归档计划](../../plans/archive/796-ac-semantic-metadata-inference-probe.md)）;
> 登记实现 = `experimental/ace/`（contract/core/probe 三 crate，
> 唯一登记源）; 冻结决策 D1–D24 见
> [attachments/796/contract-decision.md](../../plans/attachments/796/contract-decision.md)。
> 设计输入：[Design 42](../../design/42-semantic-metadata-inference-and-comptime.md)。

## 存在理由

让各后端消费**语义事实**，而不是继续共享可变 Universe 或在 Codegen 补推导。
语义库把"声明/类型/推导结果"发布为按内容寻址的不可变事实页，身份、版本、
缓存、失效全部显式化；797（ABI）按同一合同只读消费类型描述。

## 三类数据分离（合同级）

- **语义事实（Facts）**：按声明/body/实例或递归组发布的不可变结果
  （签名、字段集、绑定、来源投影）。
- **推导工作态（Work State）**：infer 变量/替换/worklist 只存在于一次求解
  任务内，**永不持久化**（不缓存 trusted/union-find/指针）。
- **runtime 投影（Runtime Projection）**：target/layout 是下游；不能反过来
  决定名义类型身份（布局归 797）。

## 结构与依赖边界

```
experimental/ace/
  contract/  —— 语义模型/身份/codec（零产品依赖）
  core/      —— 有限推导/query/反射投影（core-ast feature 隔离后端依赖）
  probe/     —— 783 store 消费/预算/IPC/周转（唯一持久化面）
```

- contract/core 默认树**零** auto-lang/UI/VM/Cranelift/auto-man/auto-val/
  auto-cache 依赖（`sema_boundary_` 审计锁定）。
- probe 消费合入版 ac-core 公开 AST（只读 adapter）与 783 ManagedStore；
  不反向引入产品依赖，不改 783/787 实现。
- 子进程复用 788 受控执行设施（semaworker/1 协议、Job containment、
  工具身份四轴）；不新建无限等待执行器。

## 有限支持面（诚实边界）

- 实装：i32/bool 字面量推导、局部 let/var 遮蔽、显式参数/返回、直接调用
  数量+类型准入、`+ * <`、条件/循环约束、赋值一致性；record 声明/字段/
  方法签名/递归组（间接引用）；CT 查询/泛型/**模型**描述（model_compute
  标记，拒绝 vm_execute 冒称）；只读反射投影（查询字段/签名、列举候选）。
- 非目标（不受本计划支持面扩大影响）：完整 Auto 类型系统/HM/trait 求解、
  用户泛型实例化、HIR→ABC 或 AVM/comptime、类型/声明生成、runtime 反射
  访问器、生产 ABI/FFI、生产默认接线、MIR/优化器。合同覆盖 ≠ 能力实现。

## 模块

| 模块 | 职责 | 状态 |
|---|---|---|
| contract | 身份/元信息/覆盖矩阵/codec | experimental（探针冻结） |
| core | 有限推导/查询/反射投影 | experimental |
| probe | store 消费/预算/IPC/周转 | experimental |

详见 [semantic-contract.md](semantic-contract.md)（身份与页合同）、
[inference-profile.md](inference-profile.md)（推导 profile）、
[query-cache.md](query-cache.md)（缓存与周转）。

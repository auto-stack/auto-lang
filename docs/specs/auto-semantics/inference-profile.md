# auto-semantics 推导 profile（有限推导与对照）

> **Status**: experimental（PLAN-796 冻结）。登记实现 =
> `experimental/ace/crates/semantics/src/{infer,query,reflect}.rs`；
> 真实输入 adapter = ac-core 公开 AST（只读）。

## 有限推导面（i32/bool）

- 支持：字面量、局部 let/var 与遮蔽、显式函数参数/返回类型、直接调用
  （签名+**实参数量**+类型三准入）、`+ * <`、条件/循环约束传播、赋值
  一致性、合法前向递归（使用已声明签名）。
- 求解：双向推导 + 显式 worklist；类型变量 occurs 检查、名义相等、期望
  类型传递、候选歧义各有确定性诊断。**未解变量是终态诊断，绝不降 Any**；
  矛盾/未解约束受控诊断。
- 数值/换行/语法遵循现有 core 规范（本库不重写 parse）。

## 与现有 core 的对照（不冒称）

- 真实 core AST 投影进入新 query 探针；推导输出与现有 core 类型检查的
  接受/拒绝及类型事实**差分对照**（`infer_core_parity` 语料），不调用旧
  typecheck 后谎称新推导。
- 未实现语言构造显式 `Unsupported`（保留描述入 canonical），不要求旧
  core 能解析本库所有 meta fixture。

## canonical 完整编码（Ann）

`write_ann` 统一编码 `i32/bool/rec:<name>/?inferred/unsupported:<what>`：
方法签名（含 **record 返回类型**）、字段描述、键、有效版本、读取准入全部
经它——方法返回 `record:A` 与 `record:B` 不可碰撞；49 参数×返回签名与
7 字段描述可区分（矩阵控制）。

## 反射/CT（模型面）

- 只读反射投影：查询字段/签名、列举字段/候选；新增成员/名字由无到有
  使负依赖失效；代码位置变化复用语义内容但更新来源投影。
- CT 与泛型只实现模板描述/替换 key 与模型约束控制（见
  [semantic-contract.md](semantic-contract.md) 接缝节）；不称用户泛型
  可编译、不称 AVM 实际运行。

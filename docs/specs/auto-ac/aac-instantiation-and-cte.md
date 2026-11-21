# AAC 实例化与编译期执行编排 (aac-instantiation-and-cte)

Spec 归 PLAN-1048 (SD-01) 交付。描述已落地的 Auto 侧实例化/身份与 CT 读取集
机制;执行尾仍为 1055 基础 MIR + 1051 AV 桥。

## 1. 实例化与实例身份

- **模板收集**:Auto 模块 `aac/semantics/instances/instances_core.at` 从
  `fn-item` 的 `type_params` 收集声明序参数(类型参数与 `<const N>` 值参数,
  声明类型缺省 u32);`type-param-tail` 中的后续参数同样入表。
- **身份字节**(与 Rust `source-query/ident::sq_hash` 一致):
  - `logical = sq_hash("ac935/instance-logical/v1", [package, module, "tpl",
    template])`,帧为 `SQ_PRODUCER "plan934/source-sq" ‖ 0 ‖ domain ‖ 0 ‖
    {u64le(len) ‖ part}*`,仅 parts 带长度前缀;
  - `key = sq_hash("ac935/instance-key/v1", [logical, "sig", sig_version,
    "body", body_version, 声明序交错 (param, suffix)/(param, "u32", value-hex),
    algo, schema])`。Auto 侧 sig/body 版本为模板签名/体文本在
    `plan934/fn-sig|fn-body` 域上的 blake3(文档化 Auto canonical;logical 与
    Rust 字节一致,key 保证确定性与实参区分)。
- **具体名**:`模板名 + "_" + ty_suffix(i32/bool/text) / "_u32_<值>"`,
 与 Rust 装配拼写一致(`id_i32`/`rep_u32_2`)。
- **调用求解**:`instances_solve.at` 按声明序消费显式实参(`name<...>(args)`)
  或位置实参的字面形状/形参注解;缺参或越权拒为 `instance.arity`;
  同名跨模块以调用者模块优先,歧义拒为 `instance.unsupported`。
- **展开**:`instances_expand.at` 生成编辑表(模板删除、callee 改名、显式
  实参区删除)与按模块追加的具体函数文本;driver 按模块重写并重新解析,
  具体声明与调用走同一 binding/typecheck(生成文本无独立权威)。
- **有界 profile**:类型/值参数与跨模块实例;约束、记录/枚举/方法实例、
  嵌套容器与递归预算为具名后继(1050/1053)。

## 2. 静态 meta 投影

- `aac/semantics/meta/meta.at` 提供纯进程内投影(签名/记录形状/required
  标注),不绑定、不执行 AV;静态 meta 不计作编译期执行。
- `#[required(...)]` 以 `key = value` 属性值拼写解析到声明 owner。

## 3. 编译期执行编排

- CT 目标经 1055 客户端车道执行:`ct-mir` 生产 ABC(受检闭包 → 基础
  Typed MIR)→ bridge → AR01 typed envelope;成功(值)才发布,失败/资源
  超限/坏帧不发布。
- 读取集(`aac/semantics/cte/cte.at`)在 Auto 内投影(闭包引用的模块常量名),
  纯静态;常量变化使闭包体版本变化,新查询重算,旧冻结值不复用。
- 高阶 CT 生成(资源/库/领域节点、`#` 表达式族)为具名后继(1050/1053)。

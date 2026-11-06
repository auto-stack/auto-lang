# native-fs（fs.* 原生族失败语义）

> **Status**: implemented（PLAN-659 SD-01，2026-09-19）

## 范围

VM 臂 `fs.*` 原生 shim（`crates/auto-lang/src/vm/native.rs`）的**失败返回语义**
契约。只约束"失败时返回什么"，不重复各操作的成功语义。

## 契约

- **`fs.canonical(path)`：失败返回 `""`**（`shim_fs_canonical` 失败臂
  `unwrap_or_default`）。
  - 权威依据：语料哨兵——027-file-manager NavTo 以 `can == ""` 判失败
    （"无法打开" toast 分支）。语料语义优先于 shim 实现便利；曾有的
    `unwrap_or(path)`（返回原路径）是 shim 实现侧漂移，会使哨兵分支
    不可达。
  - 族内一致：`Path.canonicalize`
    （`vm/ffi/stdlib.rs::shim_path_canonicalize`）同 `unwrap_or_default`
    ——失败返空是 fs 族统一语义。
  - 成功路径剥 Windows `\\?\` 扩展长度前缀（PLAN-016 T-05：面包屑/
    地址栏/拼接面不消费该形态）。
- 其余谓词型（`fs.exists` / `fs.is_dir` / `file.exists` 等）失败返回
  false——布尔哨兵自洽，不另行约定。
- 读型（`fs.read_dir` 等）失败抛 RuntimeError（中止 handler），非静默
  返回。

## 非目标

- 语料侧 `fs.exists` 前置门控形态（P642-D14② 的备选修向——因全语料
  零 `unwrap_or(path)` 依赖而未采纳，选边依据见归档计划 659 §5.1）。
- vue 臂 fs 语义（浏览器 stub 面另有契约，不在本模块）。

## 回归

`vm_tests.rs::test_plan659_fs_canonical_failure_returns_empty_sentinel`
（非法字符路径确定性失败，golden `[]`）。

## 动态整数列表字节写入（PLAN-988）

`File.write_bytes` 的 VM FFI 参数转换同时接受原生产生的
`ListData<i32>` 与语言数组、`List.new/push`、JSON 数组产生的
`ListData<Value>`。后者仅接受整数标签（Int/Uint/I64/USize/Byte/I8/U8），
并在进入写文件函数前检查 i32 范围；浮点、布尔、字符串等非整数或越界
整数报错，不跳过、不归零，不覆盖已有目标文件。非法句柄仍具名拒绝。
`json.parse` 产生动态 VM 值，未显式注解的解析结果不得沿用历史字符串
返回声明推断；对象的 `bytes` 等字段应读取字段值，不能抢占为字符串内置成员。

合法 0..255 字节逐字保持；写入函数既有 i32→u8 转换行为不因本修复改变，
这不是对任意范围整数新增字节合法性保证。该参数转换也用于 TCP stream
写入和 HTTP Response bytes body 设置；消费者验收分别覆盖，不以文件单测
推断网络行为全部通过。这里不包含 UDP 写入能力。

永久回归：`vm::ffi::convert::plan988_integer_bridge_tests`，含源码数组/
动态列表/JSON→真实文件、全256值、非法元素拒写与原文件保持。

Rust生成消费按生产者的实际表示接入：typed Vec 和 `[]T` 参数的借用 slice
进入 `a2r_std::fs::write_bytes_slice`，`json.parse` 的 Value 进入
`write_bytes_json`，原生读回的 `Vec<Value>` 和尚无元素类型约束的空列表进入
保留具体参数型的 `write_bytes_list`。后者允许空数组经局部变量/分支再传入，
不依赖只识别直接 `[]` 的特殊处理。
若同一无注解空数组既被 `push` 或非空赋值填充又被字节写入消费，按声明槽提前登记
为可推断元素类型的数组，保留 Rust 根据追加值/typed参数产生的元素约束，
写入侧使用借用 slice 入口；追加在写前或写后均一致。其他无注解空列表
保留原上下文推断，不统一强制为动态 Value 桶。
空声明之间的赋值仅关联元素类型，不凭赋值本身产生元素约束。追加/已知
类型或非空来源的约束沿赋值关联传播；无实际约束的空值链/环仍走具体
空列表入口，不能错误选择元素类型尚不确定的泛型入口。`[]T` 函数调用按声明借用参数；写入不
消费调用者的路径或列表，源值可继续使用。生成器两种模块调用表示共用此边界。

三种表示共用写前校验：JSON 必须为数组，所有元素必须为可读整数，再打开
目标。非数组、嵌套数组、非整数返回既有失败码 -1，目标不变；保留原有
i64 取低8位行为。VM FFI仍按其 Vec<i32> 签名检查范围。0..255的承诺在
两端一致，其他范围不宣称新增等价。两端可消费的动态 JSON 字节路径是
`json.parse` 后写入；这不宣称 Rust 的全部 `List.new/push` 构造语法已支持。

永久生产消费回归：`auto-generation-core/tests/plan988_byte_consumers.rs`
运行同一源的 VM，并经生产 transpile/qualify、原样 rustc 编译和执行验证
返回码及真实文件；覆盖空列表/参数传递/JSON正反例/读回/源值复用。

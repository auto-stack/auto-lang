# Legacy Rust (A2R) mut-record 与文本切片证据（PLAN-917）

本文件是 PLAN-917 的规范增量（SD-01），记录旧 production A2R（`transpile_rust_with_source_dir` /
`transpile_rust_project`）在 ace-m1 语料 R07（mut 参数写回）与 T08（str 切片越界）两格上的
真实发射契约、冻结金样与双轴证据边界。

## 发射契约（冻结支持矩阵）

- `mut p T` 形参发射为 `p: &mut T`（名义类型来自声明）；调用点 `f(x.mut, 1)` 发射
  `f(&mut x, 1)`——实参已是 `Expr::Mut` 时调用点不再加 `&mut ` 前缀（不得出现 `&mut &mut x`）。
- 写回经由 `&mut` 别名完成；Auto 记录值复制语义保留（不全局改引用）。
- 逃逸分析（trans/escape）在 `return` 表达式中对**可证明 Copy 的字段投影读**（如 `x.value: int`）
  不再升级绑定 tier（谓词由发射侧以 AST 注解类型 + TypeDecl 字段表构建）；裸 `return x`、
  非 Copy 投影、Index、闭包捕获等保持历史保守升级。返回表达式**内嵌**的裸 `.view/.mut` 实参
  （如 `return wrap(bump(x.mut, 1))`）视为非逃逸（Rust 省略生命周期不可保留）。
- `str.slice(a, b)`（两参）按字符位发射带界块：`{ let s=&(recv); … assert!(a<=b<=char_count,
  "auto str slice out of range"); … }`——接收者与各界均恰求值一次；越界即 panic（冻结的
  slice-oob 语义）。单参 `slice(a)` 形态维持 Plan 019 原发射。
- 真实依赖：std-only（Cargo.toml 无依赖）。

## 当前合法域（如实记录，不静默扩张）

已支持（rustc 真实编译+运行验证）：mut 参数写回链、Copy 投影返回读、返回位嵌套直接调用、
双不同根 mut、`.view` 绑注解局部、`move` 传按值形参、两参切片（含界内近邻）。

不支持：默认实参（producer 丢弃默认值，产物 E0061 级不一致）；mut 路径上的命名/重排实参
（914 准入限纯标量）；同根双 `.mut`（前端未拒绝，产物 rustc E0499——残余准入欠账，
owner=前端/916 准入缝，见 KNOWN-DEBT-AND-RISKS）。

## 双轴证据边界

- 日常轴（固定组 `case-ace-m1-a2r-emit-917`）：真实发射 + 逐字节金样比对，
  **零 Rust 编译**（无 rustc/cargo spawn）；漂移 FAIL 且只产 proposal，不自动覆写金样。
- 显式认证轴（独立域 `ace.m1.a2r-emit917.cert/1`，入口 `a2r-emit-917-cert`）：
  新鲜实际产物经生产入口发射 → 与冻结金样逐字节核对 → 冻结 wrapper 下受限编译+运行
  （CREATE_SUSPENDED + kill-on-close Job，编译 120s / 运行 30s / 每流 1 MiB）→
  手写 reference 同样过 oracle；receipt 严格结构校验（Invalid=硬错误），漂移=诚实重认证。
  与 852 标量域（`ace.m1.cert/1`，oracle/certs/）严格隔离，旧消费者不会误收。
- T08 的 trap oracle 是**冻结行为观察**（Rust panic），不是 Auto trap 认证；
  Auto trap 认证通道保持独立 cert-gap。

## 文本长度/码点域与守卫块恢复（PLAN-994，SD-03）

- **声明型文本局部的源语义（冻结分类规则）**：仅显式 `let`/`var` 声明为
  `str`/`Str`/`str(N)` 的局部（发射侧 `str_typed_stores`/`char_typed_stores`，
  store() 注册、按声明类型维护、重声明改型即除名）获得源语义文本方法：
  `.len()`（零参）=标量字符数（`(name.chars().count() as i64)`）、
  `.len_bytes()`（零参）=UTF-8 字节数（`(name.len() as i64)`）、声明型 `char`
  局部 `.to_codepoint()`=Unicode 标量值（`(name as u32 as i64)`）——均为
  自包含 std 表达式、接收者恰求值一次、不设 a2r_std 依赖（std-only 认证轴）。
- **边界（已批准差异，不统一）**：函数参数（`&str`）、记录字段读、match/is
  模式绑定、推断局部与调用结果的 `.len()`/`.length` 保持既有 remap（Rust
  `len()` 字节/元素计数，如 `(b.name.len() as i64)`/`(e.len() as i64)`）；
  `.len(n)` builder 设置器（带参形态）与用户 record/foreign 同名方法不受
  名称匹配改写；`byte_len`（a2r_std helper）保持原发射。
- **两参 slice/sub 守卫块恢复**：PLAN-956 曾将字符串接收者的 `slice`/`sub`
  发射整体替换为 `a2r_std::str::str_slice` 字节域调用（越界静默空串）且未重冻
  917 金样，造成 case-ace-m1-a2r-emit-917 三红（D-993-2）；PLAN-994 恢复
  `emit_str_slice_chars`：两参守卫字符位块（见上 §发射契约）+ 单参
  `recv.chars().skip((a) as usize).collect::<String>()`，四调用点
  （call_dot_methods 的 slice/sub × dot 与 Bina 镜像）同步，不再设
  a2r_std_used。公共 `a2r_std::str::str_slice` 字节 API 对其显式调用方不变
  （runtime 字节域与 source 字符域的边界即此）。
- **认证与近邻事实**：四格（T01 "A中😀"=3/8→38、T02 slice(1,3)="中😀"=2/7→27、
  T03 clone/move/拼接=2·10+3→23、T04 '😀'=128512）certified，双侧对
  oracle；值近邻（空串 0/0、嵌 NUL 33、emoji 14、ASCII codepoint 65、单参
  slice/sub 27、越界 panic）与字节稳定近邻（字段/模式/参数/List/调用结果）
  由 `a2r_tests.rs` plan994_* 永久单元钉住；3 个 cookbook 冻结期望随声明型
  len 投影具名修订（ASCII 值恒等）。

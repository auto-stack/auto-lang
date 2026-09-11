# shim-metadata

> **Status**: active
> 路径：`crates/shim-metadata`  | 技术栈：Rust（syn / serde / clap）

rustdoc 元信息工具（Plan 430 管线）：从三方 Rust crate 的 rustdoc JSON / 源码 syn AST
提取签名，按规律分类器生成 **FFI method pack**（`.autoos` shim 包），供 AutoVM 经
`use.rs` 直接调用原生 Rust 库。lib 供进程内调用（被 auto-cache 与 auto-lang 依赖），
bin 为离线 CLI。（注释中"不入 auto-lang 依赖树"的说法已过期——`auto-lang/Cargo.toml` 已 path 依赖。）

## 目标与范围

- 签名提取（方法/自由函数/泛型过滤）× 规律分类 → 生成 shim 包（注册/路由/链接三关 + 实体）。
- 与 auto-cache 的方法包缓存（指纹/版本/manifest）配合，实现 `use uuid` 类直连。
- 不做：运行时桥接（那是 auto-lang vm/ffi 的职责）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | 提取 + 分类 + shim 生成（进程内 API） | active |
| bin | 离线 CLI 入口 | active |

## PLAN-591：Option nullable 语义 + manifest v2 布局段

- **classify**：三方 `Option<T>` 返回解 skip → nullable（`Some` 压值 / `None`
  压 null；仅串/句柄槽，标量槽 None→哨兵歧义显式跳过；参数位 Option 显式
  理由跳过；残缺 Option 拒绝）。`Result` Err→错误通道为方法 wrapper 路径
  既有能力；自由函数路径未收口（KNOWN-DEBT P591-D2）。
- **emit_cdylib**：nullable/fallible 组合三臂（`Ok(Some)`/`Ok(None)`/`Err`）；
  manifest v2（`MANIFEST_FORMAT: 2`，装载侧拒载旧版）新增 `layouts` 段——
  探针导出 `auto__shim_layouts`（`offset_of!/size_of/align_of` 实测，与
  wrapper 同 cdylib 编译=同 rustc 实例同布局）；`GENERATOR` v1.3。
- **指纹（C3）payload 增补**：nullable 位、字段清单行（类型.字段|ty）、
  unit-only enum 变体行、features 组合行（排序等价）——同签名集异布局/
  异 features 必异指纹（缓存拒 stale，对抗①②机制链）。
- **rustdoc**：收集全量 pub 字段清单与 unit-only enum 变体（探针/指纹输入；
  深模块路径类型仅短名——误径由 methods pack 剔环类型级归因兜底）。

> 来源：PLAN-591（use-rust-any-crate-direct，r1，9b3639122）； Anchor：
> crates/shim-metadata/src/{classify,types,rustdoc,emit_cdylib}.rs。

## PLAN-596：trait 白名单转发 + 泛型实例化 + callbacks 元数据

- **classify（T3 trait 面）**：trait impl 不再全排除——opt-in 白名单
  （编译期常量表：`std::fmt::Display`/`ToString`/`Clone`/`base64::Engine`）
  命中产出 `TraitPlan`（独立段，不进固有方法面）；白名单版本串入 manifest。
  rustdoc 提取保留 trait impl 归属（`inner.impl.trait`，含裸 `{"path":..}`
  形态处理）；trait impl 方法可见性按认领者裁定（固有需 public，白名单
  trait 不限——E0449 约束）。
- **emit（T3）**：trait 转发 wrapper `auto_<Type>__trait_<Trait>_<method>_<sig>`，
  内体 `<Type as Trait>::method(recv, args)`（编译期定死分发；Clone 返回
  `p` 码新句柄，Display/ToString/encode 走 `s` 码）。
- **泛型实例化（T4）**：泛型项携参不 skip；mono 提示（调用点实参词法推导：
  全整型→i64/全字符串→String）激活 `Exceptions.mono` 通道 + 替换引擎
  （instantiate_method：Generic→具体 Ty），实例条目名带 `__<label>` 后缀
  （wrapper 被调名剥后缀，rustc 从实参推断单态化）；自由函数实例走 212
  wrapper call_name 通道。manifest 增 mono 段（BTreeMap）入 C3 指纹与快路径。
- **callbacks 元数据（T5 原型）**：方法条目增 `callbacks: [{param_idx,
  fn_sig}]`（emit 识别 `Box<dyn Fn(..)>` 形参产出；rustdoc 投影补
  `Box<Fn>` 标记）；wrapper 产 adapter（持令牌 u64）+ 注入式宿主跳板导出
  `auto__register_host_trampoline`。
- **指纹/缓存纪律**：`GENERATOR` v1.4（提取器语义变化两次未入签名集——
  以 GENERATOR 字符串变化兜底全局失效；"提取器语义变化必须升 GENERATOR"
  入 KNOWN-DEBT）。wrapper 装载改覆盖校验扫描（旁路探针读 manifest 全覆盖
  才采用——计数键 v3_N 在 mono 下两侧恒差，P592-D1 的进一步实证）。

> 来源：PLAN-596（dep-rust-v2-trait-generic-callback，r1）；Anchor：
> crates/shim-metadata/src/{classify,rustdoc,emit_cdylib}.rs、types.rs（mono/callbacks 段）。

## plans

- **plan-429** aavm B1 shim inventory ✅ archived——shim 存量盘点（reports/429-b1）
- **plan-430** dep methods 管线 ✅ archived——rustdoc→method pack 全管线（含 430-fixes 四项复审修复）
- **plan-596** dep-rust-v2-trait-generic-callback ✅ archived——trait 白名单 classify/emit（v1.4）+泛型 mono 替换引擎+callbacks 元数据；mono 段入 manifest 与快路径覆盖校验

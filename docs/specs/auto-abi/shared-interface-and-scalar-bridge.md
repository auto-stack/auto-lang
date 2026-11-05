# auto-abi：共享接口事实与标量 ABI 桥（CA-D1）

> **Status**: delivered（PLAN-937；依赖 PLAN-797 candidate/descriptor3 与 PLAN-921 C 标量通道）
> 路径：中立模型 `experimental/ace/crates/foreign-contract`（包 `ace-foreign-contract`，
> deps 仅 `ace-semantics-contract`+serde/blake3）；native 消费 `crates/abi`（`typed_decl`/`typed_emit`/`bridge`/`worker_bridge`）；
> C 消费 `crates/a2x-c`（`abi_scalar`）。SD 来源 PLAN-937 §5.5。

## 目标与范围

backend-free 的接口事实与物理投影，被两个真实消费者共同绑定：
797 native 侧（Cranelift 机器码发射/准入）与新 A2C wrapper（C 标量
导出）。不把无人消费的 schema 当交付；不冒普通源 FFI（源 foreign import
归 938，资源/回调/handle 桥归 939）。

## 逻辑层（foreign-contract `facts`/`typed`）

- 公开 lane 集：`{i32,u32,i64,u64,f64,bool,char}`——**无 f32**（声明即命名拒绝）。
- `InterfaceFacts`：名义 logical id + content version（经真实语义合同
  mint `DeclRef`）、per-函数形式（Pure/Fallible{ctx,params,outs}）、
  显式 error model（status-i32 表：负、唯一、升序）、effects/capabilities、
  **FactsSource**（`Typed` 候选 vs `CheckedSource{module,source_sha256}`，
  两种输入身份不得互相核销）。
- checked-source 纯函数需 `pure:<sym>` 总性证据；普通 checked 运算一律
  fallible。typed 候选绑定冻结族（`ChecksumV1` 位折叠、`FallibleDivV1`）。
- 值级 pre-call 校验：bool=0/1、char=Unicode scalar（代理对/超界命名拒绝），
  f64 按原始位携带（NaN payload/-0.0 位保真）。

## 物理层（`plan`，Windows x64）

- 域/版本/target/profile/packing 沿 797 冻结值
  （`autoabi`/`0.1`/`x86_64-pc-windows-msvc`/`autoabi.win-x64.v0.1-candidate`/`natural-align`）。
- **位置制分类**（实证钉死）：参数位置 0-3 用该位置的寄存器（整数类
  RCX/RDX/R8/R9，F64 XMM0..3），位置 ≥4 一律栈传。MSVC 14.44 代码生成
  与 Cranelift 0.126 `WindowsFastcall`（`get_intreg/get_fltreg` 按
  `arg_idx`）一致；`(int,double,int)` 的 double 在 **XMM1**（按位置，
  非浮点序数）。
- lane 投影：bool→u8（0/1）、char→u32（标量值）；私有 mn/A2C ABI
  （bool=I32、文本=ptr+len）**不与公开 lane 别名**，边界显式转换；
  891 宽浮点整数载体与公开 f64 浮点 lane 分离。
- 严格 codec：`autoabi.contract.facts/1`、`autoabi.contract.plan/1`、
  `autoabi.contract.typed-decls/1`（deny_unknown_fields、错版/超 1MiB/
  截断/篡改均命名拒绝——恢复 fail-closed）。

## 双消费者

- **native（abi）**：typed 冻结族 Cranelift 发射（f64 经栈槽位重解释取
  原始位），三腿真实 PE（自检/native→C/C→native，独立 cside 手写
  oracle），descriptor/3 经投影构造（`descriptor_from_plan`）走 797
  `validate::admit_descriptor` 准入；797 原 wire/canonical/拒绝面不变。
- **C（a2x-c `abi_scalar`）**：纯生成公开 fallible wrapper——显式
  trap 枚举→status 表（-101..-105，未知 -100），`reason` 通道保留原始
  trap 码，成功才提交 `out/reason`；库 fileset（schema
  `auto.a2x.c.library.fileset/1`、recipe `ac-a2c/hir-scalar-library-export-v1`）
  = library.c+abi_export.h+generated.c+runtime+manifest，**无 main.c、
  非认证时删 main 的产物**；旧 exe fileset 原样。CRT 闭包编译用 A2C
  冻结 flags（`/MT`，与 `crun::C_FLAGS` 互钉）。

## 桥组合与增量（abi `bridge`，层 ABI-BR1）

- 四键：facts（身份+签名+来源）、plan（**按 plan 自身 digest 键控**——
  接口不变 body 编辑为纯命中）、fragment（逐函数 sem_fp+provider 身份，
  不混整源 hash：body 编辑只重产该函数）、library fileset（片段
  digest+runtime+exports）。逐键 producer 计数：命中零生产。
- 恢复：严格 codec 解码+digest/来源身份重核，零生产；旧 persisted
  页字节不可变。
- 准入 pre-call：错 target/domain/packing/signature（同宽替换亦拒）/
  unknown symbol/semref 命名拒绝，callee 计数 0。
- 预算沿 797 冻结值（build 60s/run 5s/total 70s、流 1MiB、单产物
  64MiB、lease/GC 沿统一 ManagedStore）。

## 明确非目标

源 foreign import、mn 源码公开导出、AV/A2R foreign 调用、资源/handle/
callback、通用 header importer、公开 f32、动态 loader、跨平台；source
会话恢复首编辑（938 域）。

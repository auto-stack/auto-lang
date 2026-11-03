# auto-a2c

> **Status**: experimental
> 路径：`experimental/ace/crates/a2x-c`（包 `ace-a2x-c`，deps 仅
> `ace-a2x-core`+`ace-core`） | 依据：PLAN-921（CA-A）；C 路线
> [shared-a2x-and-hir-to-c-roadmap](../../design/strategy/shared-a2x-and-hir-to-c-roadmap.md)

受检 HIR → C11 的**标量通道**目标 provider（`CTargetProvider`）：与 Rust
目标共享同一语义事实（AXS1 键/语义指纹）与同一 session/store 服务，片段/
文件集/证书身份完全按 C 目标隔离（层 AXC1、schema `auto.a2x.c.fileset/1`、
recipe `ac-a2c/hir-scalar-emit-v1`、cert `ace.a2c-hir.scalar.cert/1`）。

## 目标与范围

- **支持域**：仅原 core/scalar profile 的 i32/bool 计算面（与 Rust 标量
  通道同准入）；RecordText/Collections 经 C 目标是命名拒绝
  （`a2x.domain`，后继 CA-B/CA-C 不因易表达而放宽）。
- **受管编译**（`a2x-worker::crun`）：cl 定位复用 PLAN-797 冻结序
  （env→vswhere→最高版本；缺工具=infra 非 skip）；vcvars64 从 cl 路径
  推导（无硬编码版本/edition）；冻结 flags
  `-nologo -std:c11 -TC -W4 -WX -O2 -MT`；经 ac-core 受管执行器
  （CREATE_SUSPENDED/Job/120s 编译/30s 运行/1MiB 流帽）。
- **通道形态**：worker `daily-emit-c`（spawn 零：spawned_rustc=0 且
  spawned_cl=0，父侧 golden 字节判）与 `certify-c`（真 cl 双腿：actual
  与手写 reference 各自独立编译运行对同一闭式 oracle，strict 证书）。
- **禁止**：不生成 Rust 再翻译、不调旧 Trans、非 i32/bool 域、宽源码
  （i64/u64/f32/f64）、多模块/stdlib/泛型/通用 FFI——一切以前端/视图
  命名拒绝为先（cl 失败只能 infra）。

## 现状边界

- 首例 TestCase 套件 `a2c-hir-scalar-921`（3 用例 golden 字节判 +
  手写 reference）；固定组 `case-a2c-hir-scalar-921`。
- 61 例冻结语料（745/767 corpus+ace-B+边界合成，含短路极性/陷阱反控/
  trace marks 序）全部经真实 cl 编译运行对 oracle 通过
  （`a2c_scalar_semantics`）。
- 形式采样（`a2c-formal`）：load256 原样 30 样本 P90 全暖门≤2000ms
  （hit/restart/body-edit/restore；restore≈150ms 级）；子进程纯恢复
  见证（P0 计数断言+恢复后编辑可用）。
- 非目标（后继包）：CA-B record/text、CA-C 集合/变体、CA-D 有限 public
  ABI、CA-E 四端/M1 总签章。

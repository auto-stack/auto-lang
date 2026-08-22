# Plan 417: "Auto 作 Rust 脚本层"发布收尾（359 residuals:Phase E + D2/D3 + A1/A2）

> **状态**: 📋 已立项待实施（2026-08-22,源自审计 §5.2 B8 拆粒度;暂缓原因"依赖语言特性(Phase E 各 DIV)"——本计划按 DIV 依赖排序,可分批推进）
> **来源**: Plan 359 真实遗留(2026-08-20 复核):D2 generators 用例、D3 http_client_sync(blocked by DIV-HTTP-LANG-1)、Phase E 五项 open、A1/A2 落地页

---

## 1. Phase E 五项 DIV(known-divergences.md,均 status: open)

依赖链与粒度(每项独立分支,先语言侧后 parity 用例侧):

| DIV | 一句话 | 语言侧入口 | 预估 |
|---|---|---|---|
| E1 CHAR-AT-1 | ✅ 2026-08-22 完成:infer_type_from_expr 把 char_at 移入 Int 臂(golden 007_char_at_infer,string_utils 四处 workaround 注解移除,DIV 翻 fixed);**衍生 E1b ✅ 同日由 D2 根治**:register_import_signatures 在 use_stmt 处理时解析可发现模块源登记导入函数签名(string_utils 三方 22/22 恢复全绿) | trans/rust.rs register_import_signatures | — |
| E2 LANG-1 | 语言级语法/语义分歧点 | 按 known-divergences 条目定位 | 1-2 天 |
| E3 TRAIT-VM-1 | trait 默认方法 VM 分歧 | vm/codegen.rs trait 分发臂 | 2-3 天 |
| E4 TRAIT-VM-2 | trait 关联类型/泛型 impl VM 分歧 | 同上,依赖 E3 | 2-3 天 |
| E5 HTTP-LANG-1 | http 客户端语言级 block 点(同时解锁 D3) | parser.rs async/http 语法 + vm/ffi/http_client | 3-5 天 |

**顺序**: E1(最小)→ E2 → E3 → E4 → E5。每项落地后 known-divergences.md
对应条目 status → fixed,并跑 `parity/crates/auto-parity` 三向验证。

## 2. D2 generators 用例 ✅ 2026-08-22 完成(Plan 417-D2,三方 6/6 一致;VM 带参生成器搬参根治 + a2r ~Iter→Stream 统一降级 + 导入签名登记,E1b 连带根治)

- `parity/libs/generators/` 三向目录(README/auto/tests)——覆盖 `yield`
  迭代器、惰性序列(359 Task D2.2 已有设计:§639 起);
- D2.3 双 demo(教程素材)缺,补齐后 docs/script-to-ship/ 相应章节引用。
- **验收**: parity 三向通过 + parity 仪表盘 L 计数 +1。

## 3. D3 http_client_sync(阻塞于 E5)

- `parity/libs/http_client_sync/`(tokio 见 Plan 347 的既有结论);
- E5 落地后按 D2 同构方式补三向用例 + D3.3 双 demo。

## 4. A1/A2 落地页(website/index.md)

- 补 parity 链接与 hero demo 引用(当前 V3 未达——359 §A 的验收是
  "落地页上线,叙事打动人验证");
- 依赖 D2/D3 的最佳用例产出后做终版;先行版(仅链接 + 仪表盘截图)
  可与 Phase E 并行,0.5 天。

## 5. 165 checkbox 回填(收尾动作,0.5 天)

359 正文 165 个 checkbox 多数已落地未回填(2026-08-20 审计结论)——
按"代码级核验后方可勾选"纪律,finish-plan 流程时统一回填并归档 359。

## 6. 验证矩阵

- 语言侧改动:a2r golden(`RUST_MIN_STACK=33554432`,基线 340/340)+
  auto-ai 三转译零错 + 全量 lib;
- parity 用例:`auto-parity` 三向 runner + 仪表盘重生成;
- 文档:known-divergences.md 状态翻转与 359 回填一致性检查。

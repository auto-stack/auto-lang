# PLAN-815 T-09 R1 修复轮 — F-815-T09-01 关闭记录

2026-10-22，work 阶段交付。对象=[review-t09-r1](../../review-t09-r1/review.md)
唯一 P2；plan734 按 T-09 裁定为 814 来源非阻断排版金样债，不在本轮范围。

## 0. 冻结与身份

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`。
- **生产修复提交 `0f2858e35`**（= 新生产冻结位；此前 e22fc7592 的生产语义
  由本提交推进——shell 再生器模板+清单）。docs/交接随其后 docs 提交叠加。
- 改动面：`crates/auto-ui-tools/src/rust_ui.rs`（生产：五处壳层发射 +
  Cargo 模板）+ `crates/shell-pack/src/lib.rs`（ sanctioned 再生）。

## 1. 根因与修复（不变量级）

- **同族残项**：814 T-05 import 策略把生成物 canonical 化时，app/dev 清单
  臂已随 T-07 子批2（F-815-T07-B2-01）修复，但 shell-pack 再生器
  （contract-baseline §2.1 明确迁入的公开 `generate_shell_pack_lib_sources`）
  被遗漏——组件体经共享转译器已是 canonical（107 处引用），壳层五处仍发
  legacy `auto_lang::ui` 前缀，Cargo 模板只声明 `auto-lang` → 生成工程
  原样消费 107 个 E0433（复审实测）。
- **修复**：五处壳层 canonical 化（头 `use`×2、`shell_projection` 再导出、
  `shell_write`/`shell_write_vec` 的 `auto_ui_runtime::ui::auto_val::Value`
  全限定）+ Cargo 模板改为 `auto-ui-runtime{ui-iced}`+`auto-lang-core` 双
  canonical 直依赖（逐字节镜像入库 814 T-05 形态）。
- **入库快照更新**：按 lib.rs 自身 DO-NOT-EDIT 契约走
  `regen_shell_pack`（#[ignore] sanctioned 测试）再生提交。**live
  auto-os/shell 五件与复审冻结输入逐 hash 一致**（shell.at 64eb707f…/
  desktop.at ac4f99c8…/notification_center.at 52993bd4…/dashboard.at
  9413505f…/switcher.at 789f4c4f…）——live 再生≡冻结回放。净变更恰 2 行
  组件体：为 814 R10-R14 转译器比较投影类型感知演进
  （`60dd597fa` 后转译器继续演进入库快照停在 R9）的**合法快照更新**，
  非手改快照接受错误产物——该演进自带 r14 探针 97 feature+正控 99/99
  exit0 与 compiler 216/216 门（814 在案）。

## 2. 关闭证据（复审 closure 条件逐项）

| 复审关闭条件 | 本轮证据 |
|---|---|
| 真实源/清单与 814 canonical 规则同步 | 五处壳层+清单模板修复（§1）；生成 manifest 与入库逐字节一致 |
| 生成工程凭**自身原生生成的**清单通过消费检查 | 复审 probe（冻结五件输入）重生成→`cargo check`（原 manifest 仅等价布局调整：路径指 producer crate/serde workspace→registry/显式 iced_widget patch，**零补依赖**）**exit 0**——原 107 个 E0433 清零 |
| 相同冻结输入重放公开生成→真实消费 | probe 重放：生成 lib 97,221B；原生清单完整 |
| 产物与应有入库来源一致 | `test_shell_pack_lib_freshness` **1/1 字节一致（红→绿）**；入库正控 `cargo check -p shell-pack --lib` exit0 |
| live auto-os/frozen 输入身份明确 | 五件逐 hash 全一致（§1） |
| freshness 有效、不靠 skip/ignore/放宽断言 | 门原样字节断言，无任何测试放宽 |
| 不让 API 静默拒绝原支持输入 | 输入面零变化（同一五件） |
| 复验受影响生成兼容/邻接正控 | lib no-fail-fast **182/184**：shell_pack 红→绿；余二红=merged_api（P751-D2 在案）+plan593（归 plan734 族，T-09 裁定非阻断） |
| 刷新 T-08 实际生产冻结位/参考 artifact | 生产冻结位→`0f2858e35`；release artifact 自该位重建（SHA256 见 handoff 首表刷新，随 docs 提交） |

## 3. 回放路径（复审可复验）

- probe：`D:/autostack/.wt/lang-815/auto-lang/target/review-t09-debt-…`
  （复审自建 fixture，main.rs/Cargo.toml 在
  [review-t09-r1](../../review-t09-r1/)）：
  `cargo run --offline --manifest-path <probe>/Cargo.toml --target-dir <wt>/target -- <probe>/generated-shell`
  然后 `cargo check --offline --manifest-path <probe>/generated-shell/Cargo.toml --target-dir <wt>/target`。
- freshness：`cargo test -p auto-ui-tools --lib rust_ui::tests::test_shell_pack_lib_freshness -- --test-threads=1`。
- 入库正控：`cargo check -p shell-pack --lib --offline`。

## 4. 状态

T-06/T-07/T-08 复审重开保持（勾选归复审）；plan734 非阻断债不动；
T-10/T-11 pending。merge 门（裸 t+触面档）归 T-10 统一。

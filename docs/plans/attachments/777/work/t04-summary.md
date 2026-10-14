# PLAN-777 T-01..T-04 work 验证证据（2026-10-15）

- 实施树：`D:/autostack/.wt/lang-777/auto-lang`（branch `plan-777-dev`）
- 基面：`master@17bc8431457848b040bffa5c261e13b057771559`（与就绪收据一致）
- 实现冻结提交：`0b5e79950381`（脚本 + 配置 + fixtures，19 文件 +2113 行）
- 基线落库提交：`97df2f997`（`.autoos/module-boundaries-baseline.json`，base_revision=0b5e79950381）
- 类别：Category A——未运行 `cargo t`/`docs_gen`，未编译任何 Rust；`cargo metadata --no-deps --locked` 仅作只读一致性对照。

## 验证矩阵结果

| V | 验证 | 命令 | 结果 |
|---|---|---|---|
| T-01 | help 可用 | `python scripts/modularity-audit.py --help` | exit 0 |
| V-01 | 稳定只读（真实仓） | 冻结提交上两次 `--mode observe`（t04-observe-a/b.json） | payload 字节一致（SHA-256 `54e93962964abb2590abe3785a6fac2ba96ac4868c71ab208b023a5f5f0a5539`，668,766 B）；扫描前后 `git status` 无变化（只读证明） |
| V-02 | 新增禁向边判红 | `--self-test` v02 | fixture-lang→fixture-ui-kit(normal) 非零退出且点名具体边；UI→SDK 反向、dev-only、顶层装配不误红 |
| V-03 | 候选诚实 | `--self-test` v03 | 七处命中分类逐一正确（code1/comment2/doc1/string1/test1/macro1）；别名用法零泄漏；scanner_limitations 显式声明别名/re-export/宏盲区 |
| V-04a | 窄豁免不盖新边 | `--self-test` v04a | 存量边 A grandfather（owner+exit_phase），新边 B 单独判红 |
| V-04b | 撤销边 | `--self-test` v04b | status=revoked 边仍存在 → revoked_edge_present 判红 |
| V-04c | 过期基线 | `--self-test` v04c | base_revision 后 manifest 实际变更 → stale_baseline 判红 |
| V-04d | 宽豁免 | `--self-test` v04d | `[baseline].exempt_paths` 目录级豁免 → 配置加载即拒绝（exit 3） |
| 自测总控 | — | `python scripts/modularity-audit.py --self-test` | 7/7 通过（t03-selftest.txt；每个变异场景先读回证明 fixture 被改） |
| T-04 | strict 真实仓判定 | `python scripts/modularity-audit.py --mode strict` | **verdict=pass，violations=0，grandfathered=2**（auto-man→auto-lang / auto-gen→auto-lang，均带 owner+exit_phase=R5）；exit 0（t04-strict.json） |

## 真实基线要点（@17bc84314 + 本 Plan 文件）

- 规模（口径与调研基线一致）：tracked .rs 2,045 文件 / 945,837 行；auto-lang/src 733 文件 / 710,556 行；UI 五目录 239 文件 / 353,361 行（与 2026-10-13 调研完全一致，期间五目录零变化）；专用 src/tests 212 / 82,788。top 文件 renderer.rs 41,775 / vue.rs 34,045 与调研一致。
- 声明边：22 个 workspace 成员、36 条 path 依赖边、0 提取缺口。`cargo metadata --no-deps --locked` 对照 **36=36 零差异**（锁取主检出 gitignored Cargo.lock 复制为 worktree 未跟踪文件；对照兼容 cargo 1.98 的扁平 kind/target 依赖格式）。
- 文本候选（观察层，非语义边）：core→UI 886（92 文件）；UI→语言 181（15 文件）；man→语言 571（20 文件，与调研的 20 文件面一致）。
- ownership：21 项（调研 §4.3 全部 8 焊点 + §4.5 AURA + §5.3 app_registry + §5.2 拆分 6 项 + 外部 consumer 5 仓），扫描根内路径零缺失，根外 5 项按声明登记。
- 资产根受管文件数：docs/schema 2、stdlib/aura 62、stdlib/auto 146、auto-man/assets 376、packages 300、examples/ui 381、crates/patches 69——与调研 §4.6 数字逐一吻合。
- 已知限制（报告 method/scanner_limitations 显式声明）：文本扫描是启发式候选（别名/re-export/宏为盲区）；cargo 对照在无锁/锁不满足时记 gap 不判红；本工具不声称完整 Rust 语义解析。

## 备注

- strict 判定面 = 当前工作树边集 vs 冻结基线；未提交变更同样在判定面内（新增禁向边无论是否提交都会点名），脏树只在 `--refresh-baseline` 被拒绝（冻结必须绑定干净树 HEAD）。
- 基线新鲜度锚定面 = 根/Cargo.toml、crates/**/Cargo.toml、.autoos/module-boundaries.toml（基线文件自身落库除外）；test/fixtures 下的 fixture manifest 不参与真实根边提取，也不在锚定面内。
- 后续 770/781 合入后：`--refresh-baseline` 重采样并保留旧快照（本目录即历史快照存放处）。

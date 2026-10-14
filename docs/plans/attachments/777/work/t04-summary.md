# PLAN-777 T-01..T-04 工作证据（2026-10-15）

> R2 版（r1 复审 needs_fix 六项修复后重冻）。R1 轮原始证据（t04-observe-a/b.json、
> t04-strict.json、t03-selftest.txt）保留于本目录作历史；本节以 R2 冻结态为准。
> R1 摘要计数勘误：受管 Rust 实为 **2052 文件 / 945894 行**（旧摘要误记 2045/945837，
> 与所附 JSON 不符，已按复审 §5 更正；原始 JSON 未改动）。

- 实施树：`D:/autostack/.wt/lang-777/auto-lang`（branch `plan-777-dev`）
- 基面：`master@17bc8431457848b040bffa5c261e13b057771559`
- R1 实现冻结：`0b5e79950`（实现）→ `97df2f997`（基线）；r1 复审 reviewed_commit=97df2f997
- **R2 修复冻结：`3b50ea63e`（F01–F06 修复 + 永久自测 v05–v09）→ `aaf702011`（基线重冻）**
- 类别：Category A——未运行 `cargo t`/`docs_gen`，未编译任何 Rust；`cargo metadata --no-deps --locked` 仅作只读一致性对照（复审 replay 的 cargo oracle 只在其自有临时副本上允许生成临时锁）。

## R2 修复对账（r1 findings → 不变量 → 修复与永久反例 → 当前结果）

| finding | 不变量 | 修复与永久控制 | 当前结果 |
|---|---|---|---|
| F777-R1-01 多 target 表互相覆盖 | 每个 target 表按自己的依赖表读取 | dep_sections 携带各自表（三元组）；自测 v05 | v05 通过：Windows core→UI 判红、Unix →SDK 不误红；replay C01-multiple-target pass |
| F777-R1-02 workspace 继承 path 基点错误 | 继承 path 相对 workspace 根解析（对齐 Cargo） | path_base 按声明来源取 root/member；自测 v06 | v06 通过：解析到 member、无 unresolved 假缺口；replay C02 pass |
| F777-R1-03 输出可覆盖审计输入 | 报告另存，输入字节不变 | `guard_output_target`/`guard_refresh_target` 写入前守卫（配置/基线/受管文件/根内既有文件；refresh 仅准合法基线目标）；自测 v07 | v07 通过：拒绝先于写入、配置/基线字节不变、合法重冻结可用；replay C03 pass（exit 3 + hash 不变） |
| F777-R1-04 optional 激活冒充存量豁免 | 豁免绑定冻结时完整声明身份 | 基线键扩展为 from/to/kind/target/**optional/to_scope/dep_key**；自测 v08 | v08 通过：optional 控制通过、激活成必选判红 new_forbidden_edge、无 grandfathered；replay C04 pass |
| F777-R1-05 语法/AST/build/CLI 责任漏登 | 调研各类别全覆盖且缺类可判红 | ownership 增至 27 项并分 7 必需覆盖组（`[ownership_coverage]`；补 ast.rs/parser.rs/dialect.rs/dialect/ui.rs/builder/CLI main.rs）；缺组 strict 判红；自测 v09 | v09 通过；7 组全在位、路径零缺失；replay C05 六路径全部 registered |
| F777-R1-06 跨根必选禁令无设计依据 | 禁向判定必须基于依赖方向 | 生产规则改 Design37§2 域限定（语言/infra/runtime 域禁外部 ui/man/man-ecosystem 实现边）；scope-only 无方向规则按惰性处理并记录告警 | replay C06 pass：合法 UI→外部 SDK（显式开发 path）exit 0，惰性规则告警可见；真实仓无 inert_rules |

## 验证矩阵结果（R2 冻结态 aaf702011）

| V | 验证 | 命令 | 结果 |
|---|---|---|---|
| T-01 | help 可用 | `python scripts/modularity-audit.py --help` | exit 0 |
| V-01 | 稳定只读（真实仓） | 两次 `--mode observe`（t04r2-observe-a/b.json） | payload 字节一致（SHA-256 `6fa0e43e549a066e3834e75684b035ee1e2b1ea29c136d2ef9e43e014ab8ec8a`）；前后 `git status` 无变化 |
| V-02..V-04 + R1 反例 | 自测 | `--self-test` | **12/12**（v01–v04d + v05 多 target/v06 继承基点/v07 输出守卫/v08 optional 激活/v09 覆盖缺口；每变异场景先读回证明 fixture 被改） |
| 独立复审重放 | 反例重放 | `python docs/plans/attachments/777/review-r1/replay.py --implementation <worktree>` | **六控全过，exit 0**（C01-control/C01-multiple-target/C02/C03/C04/C05/C06 contract_pass 全 true） |
| T-04 | strict 真实仓判定 | `--mode strict` | **verdict=pass，violations=0，grandfathered=2**（auto-man→auto-lang / auto-gen→auto-lang，owner+R5）；ownership coverage ok；exit 0 |

## 真实基线要点（@3b50ea63e + aaf702011）

- 规模（口径与调研一致）：tracked .rs **2052 文件 / 945894 行**；auto-lang/src 733/710,556；UI 五目录 239/353,361（与调研一致）；专用 src/tests 212/82,788。
- 声明边：22 成员、36 条 path 边、0 提取缺口；`cargo metadata --no-deps --locked` 对照 **36=36 零差异**。
- 文本候选（观察层）：core→UI 886（92 文件）；UI→语言 181（15 文件，**观察范围仅 auto_lang:: 前缀——crate::ast/vm 等核心路径不在本扫描范围，已声明**）；man→语言 571（20 文件）。
- ownership：27 项 / 7 必需覆盖组全在位（core-ui-weld 9、grammar-ast-dialect 4、project-model 1、man-split 7、cli-top 1、assets 7、external-consumers 5），扫描根内路径零缺失，根外 5 项登记。
- 资产根受管文件数：docs/schema 2、stdlib/aura 62、stdlib/auto 146、auto-man/assets 376、packages 300、examples/ui 381、crates/patches 69。
- 已知限制（method/scanner_limitations 显式）：文本扫描为启发式候选（别名/re-export/宏盲区）；cargo 对照无锁/锁不满足记 gap 不判红；不声称完整 Rust 语义解析；scope-only 规则惰性（禁向判定基于依赖方向）。

## 备注

- strict 判当前工作树边集 vs 冻结基线；未提交变更同样在判定面内；脏树只在 `--refresh-baseline` 拒绝（r1 复审 §5 措辞勘误已同步脚本头/METHOD/本摘要）。
- 基线新鲜度锚定面 = 根/Cargo.toml、crates/**/Cargo.toml、.autoos/module-boundaries.toml（基线文件自身落库除外）。
- 后续 770/781 合入后：`--refresh-baseline` 重采样并保留旧快照（本目录即历史快照存放处）。

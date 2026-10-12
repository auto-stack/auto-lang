# auto-lang 目录重新组织分析和方案

> 日期：2026-10-03 · 主检出（master，17292c07e）实测
> 性质：目录组织专项分析（docs 沉淀）；所有执行动作按 §7 分批，走 L0/plan 流程
> 数据口径：`git ls-files` 跟踪清单 × `stat` 字节求和（精确）；磁盘占用 `du`（本地实况）

---

## 0. 现状总览（关键数字）

| 指标 | 数值 |
|---|---|
| git 跟踪文件总数 | 10,379 |
| 跟踪负载大户 | `docs/` 305.9MB（其中 `docs/reports` **202MB**/716 文件）· `website/public` 48.1MB · `examples` 仅 14.6MB |
| 本地可再生堆积 | **≈10.4GB**（§2） |
| 最大 crate | `auto-lang` **635k 行**（ui 216k / vm 91k / ui_gen 80k / tests 60k / trans 47k） |
| 最大单文件 | `ui/iced/renderer.rs` **37,239 行** |
| `docs/plans/archive` | 716 个计划文件 |
| 重复证据 | p715/p718 两张截图**字节级相同** |

---

## 1. 问题一：误入库、其实不用跟踪的目录/文件

判定标准：名字带 tmp/临时语义、构建产物、机器本地安装物、个人配置，却出现在 `git ls-files` 中。

| # | 对象 | 证据 | 建议 |
|---|---|---|---|
| 1-1 | `examples/{jetdemo,jethell}/.gradle/**` | **26 个** Gradle 构建产物（`checksums.lock`、`executionHistory.bin` 等）被跟踪 | `git rm -r --cached` + `.gitignore` 补 `.gradle/` |
| 1-2 | `.agent/skills/auto-brain-archiver`、`auto-stdlib-dev` | **指向 `/d/autostack/auto-skills/` 的符号链接**——机器本地安装物（同 `.agents/skills/auto-plan-*` 已 ignore 的「禁入库」先例）；他机 clone 即断链 | untrack + `.gitignore` 补 `.agent/`（现规则只盖了 `.agents/` 的子项） |
| 1-3 | `.claude/settings.local.json`（+连带 `.claude/skills/commit/SKILL.md`） | `.claude/` 本身已在 ignore，但先跟踪后加规则不生效；`settings.local.json` 按约定就是个人本地配置 | untrack，ignore 规则保持 |
| 1-4 | `.autoos/specs.json.tmp2` | **0 字节**临时文件被跟踪 | `git rm` |
| 1-5 | `docs/plans/reports/assets/031/window-你好世界.bmp`、`window-转换器.bmp` | BMP 未压缩格式 + 中文文件名（跨平台编码风险）双问题 | 转 PNG 重传或删除（历史瘦身见 §8 决策点①） |
| 1-6 | `docs/plans/evidence/p709/diag3-storage.json.lock` | cargo 构建锁文件入库 | `git rm` + ignore 补 `*.lock` 于 evidence 体系（或逐件清） |
| 1-7 | 根目录残留四件：`check_error.txt`（2026-03 错误日志）、`donut_sfc.txt`（调试转储）、`vm-600-callout-box.png`（截图）、`kill-vue-dev.bat`（本机脚本） | 与仓库长期资产无关的走查残留 | 删除，或归 `docs/ai_history`/`scripts/repro` |
| 1-8 | `uncompleted_plans.md`（根） | 2026-05-28 起未更新，引用旧编号体系（011/013），与 `docs/plans/archive/plans-status-audit-*.md` 职能重复 | 删除或移入 archive 作历史快照 |
| 1-9 | `flow.at`（根） | flows DSL 样例，仅两个归档计划引用过 | 移 `docs/examples/` |
| 1-10 | `.vscode/iar-vsc.json` | 个人 IDE（IAR 插件）配置，无 ignore 规则覆盖 | untrack + ignore（该目录仅此一件） |
| 1-11 | `tmp-corpus/`（29 文件 tracked） | 名为 tmp 实为 b3check/j2check 校验语料 + `vm-detail-*.mjs` 探针脚本；仅被两个**归档**计划提及，CI/现行脚本零引用 | 择一：语料归位 `test/`（或 `examples/`）、探针归 `scripts/repro/`；确认无保留价值则整体 untrack |
| 1-12 | lockfile 口径混乱 | 根目录三套并存：`package-lock.json`(tracked) + `pnpm-lock.yaml`(tracked) + `bun.lock`(ignored)；而 `crates/auto-playground/frontend/bun.lock`、`packages/auto-forge-ui/bun.lock` 又是 tracked | 统一包管理器，锁定单一 lockfile 策略 |
| 1-13 | `crates/graphify-out/` | crates/ 内的空残留目录（0 文件） | 删除 |

> 教训注记：1-2/1-3 属「先跟踪、后加 ignore 不生效」型；1-5/1-6 属「证据截图随手 add」型。§7 批次 A 建议顺带在 review checklist 加一条「新增 tracked 文件无构建产物/个人配置」自检。

---

## 2. 问题二：已在 ignore、但本地堆积需要清理的目录

合计 **≈10.4GB**，全部可再生：

| 目录 | 体量 | 说明 |
|---|---|---|
| `target-412/` | **6.2GB** | cargo 探针构建目录（ignore 规则 `target-*/` 已盖，注释自证「可再生」） |
| `examples/**` 的 `gen/`+`dist/`+`deps/`+`node_modules/` | **≈3.0GB** | `gen/` 2.87GB：ui 系每例 130-190MB（`gen/front`+`gen/vue` 双份产物，部分经硬链共享）；重跑 `auto run` 即再生 |
| `packages/*/node_modules/` | ≈570MB | auto-forge-ui 304M / auto-smith-ui 152M / auto-playground-vue 122M（各自 tracked 源仅 <1MB） |
| `tmp/` | 267MB | 2026-04 起堆积的 ps1 调试脚本/截图/转储；ignore 已盖（注意 `tmp/*.at` 特殊规则，清理安全） |
| `parity/target/` | 156MB | parity 子 workspace 独立 target |
| `website/node_modules/` | 152MB | 网站工程依赖 |
| `node_modules/`（根） | 55MB | — |
| `graphify-out/`+`graphify-out-crates/` | 22MB | `/graphify` 再生 |
| `.playwright-mcp/` | 1.4MB | 浏览器自动化 console 日志 |
| 零散 | <1MB | `docs/specs/auto-lang.json.bak`、`scratch/`（132K，按三分归位规约处置） |

特例（**不可直接删**）：`.worktree/auto-down` 是指向 `/d/autostack/auto-down` 的**符号链接**（旧 worktree 布局残留）——移除前必须先跑 `bash D:/autostack/wt-guard.sh`（红线，防递归穿透删除）。

建议：新增 `scripts/clean-local.sh`（默认 `--dry-run`，按上表保守清单执行），并在 AGENTS.md 登记「本地可再生目录清单」，避免每次人工 `du` 排查。

---

## 3. 问题三：重复目录（历史组织漏洞）

| # | 重复组 | 现状与判定 | 归并方案 |
|---|---|---|---|
| 3-1 | `test/` vs `tests/` | **两个都在用**，非死目录：`test/`=.at 特性语料（comptime/generic/txt/ui…，被 `crates/auto-lang/src/tests/stdlib_tests.rs` 引用）；`tests/`=Rust 集成测试 + 更多 .at（`a2r/a2c/vm_tests.at`、`tests/auto/`，被 `crates/auto/src/main.rs`、`tests.rs`、`test_runner.rs` 引用） | 命名混淆属历史漏洞。走 plan 归并为单一 `test/`（.at 语料子树）+ crate 内 `tests/`（Rust）；或最低成本先在两处加 README 声明分工 |
| 3-2 | 计划/报告体系**五处并存**：`docs/plans/{,archive,evidence,reports}` + `docs/reports` + `docs/plan-indices`（16 文件）+ `docs/plan-reports` | `plan-indices`/`plan-reports` 是旧编号时代（01-ast-core）产物，2026-08-27 后冻结 | 并档：移入 `docs/plans/archive/legacy-numbered/`（或 specs `_archive` 同款处理） |
| 3-3 | `auto/lib` vs `auto/lib-legacy` | legacy 快照自 2026-08-28 未动；git 历史即存档 | 删 `lib-legacy` |
| 3-4 | graphify 输出三处：`/graphify-out`、`/graphify-out-crates`、`crates/graphify-out`（空） | 前两者 ignore 在本机堆积，第三处空目录残留 | 删空目录；其余保持 ignore |
| 3-5 | agent 技能目录四套：`.agent/`(2 外链) + `.agents/`(autoui-verifier + 5 个已 ignore 的 auto-plan junction) + `.claude/` + `.codeartsdoer/` | 多助手各有挂载可以理解，但 `.agent` 与 `.agents` 一字之差并存最易混 | AGENTS.md 落约定：仓内技能唯一源 = `.agents/skills/`；其余目录只放各自配置 |
| 3-6 | 画廊 6+1：examples 下 bps/charts/component/jet/vue/widgets-gallery + 根 `dist/`（component-gallery 构建产物） | component/vue/widgets 三者定位重叠明显；产品归并需裁定 | 至少各画廊 README 标注差异；归并走 plan |
| 3-7 | 重复截图：`docs/reports/p715-website-ui/final-v05-en-1440-dark.png` ≡ `p718-website-ui/` 同名文件（**字节级相同**，2.4MB×2） | 两期报告复用同一验收图未去重 | 去重，引用侧指向单份 |
| 3-8 | `packages/` 四个 Vue UI 包：auto-lab-ui / auto-forge-ui / auto-smith-ui / auto-playground-vue | package.json 均无 description，定位是否重叠待澄清 | 逐包补 description + 定位裁定（plan 级） |
| 3-9 | `stdlib/may`、`stdlib/result` | .at 侧全部 `.skip`（禁用态），只余 C 产物 tracked | 恢复或整体冻结并 README 标注状态 |
| 3-10 | 验收截图双体系：`docs/plans/evidence/` vs `docs/plans/reports/assets/` | 同为验收证据两套挂法 | 归一为单一 evidence 体系（§4-2 一并处理） |

---

## 4. 问题四：内容过多、宜拆分的目录

| # | 目录 | 现状 | 拆分方案 |
|---|---|---|---|
| 4-1 | `docs/plans/archive/` | **714 个文件**单层平铺（001 → 7xx），浏览/检索困难 | 按编号段分桶：`archive/000-099/ … 700-799/`，`git mv` 保 rename 检测；补 `archive/INDEX.md` 索引页。一次性做完，避免多次 churn 污染 blame |
| 4-2 | `docs/reports/` | **202MB / 716 文件**，631 PNG + 25 JPG（单图最大 2.4MB），仓库跟踪负载第一大项 | 三层治理：a) 立即去重（3-7）+ 无损压缩/转 WebP（预计 -70%）；b) 防增量——spec-lint 扩展「新增截图 >1MB 拒绝或强制压缩」；c) 存量是否迁 LFS/外链 = 决策点② |
| 4-3 | `docs/plans/evidence/`（42.9MB/256）+ `docs/plans/reports/`（28.4MB/240） | 同 4-2 性质 | 并入同一 evidence 体系，适用同一压缩门 |
| 4-4 | `website/public/` | tracked 48.1MB/341 文件，desktop-showcase 单图 5.7MB | 压缩 + 多尺寸响应式；或截图走 CDN |
| 4-5 | `examples/ui/` | 33 个编号示例单层平铺（001 → 0xx） | 可按主题分组（basic/forms/data/apps），但**编号 ID 被测试矩阵/文档广泛引用，须保持稳定**，仅目录层级变化，谨慎评估收益 |
| 4-6 | `docs/` 整体 | 2,586 文件 / 306MB | 治理后（4-1~4-3 + 3-2）预计降至 <100MB |

---

## 5. 问题五：代码目录（尤其 crate）拆分

### 5-1 `crates/auto-lang`：635k 行一笼统（最优先）

模块行数实测：

```
ui 215,974 │ vm 90,930 │ ui_gen 79,687 │ tests(内嵌) 59,574 │ trans 47,084
aura 9,677 │ ast 9,027 │ infer 5,543 │ autodown 5,171
```

依赖大体单向（ast → infer → trans/vm 独立），具备拆分条件。建议次序（每步一个独立 plan，L2 设计先行）：

1. **`auto-vm` 名实归位**：现 `crates/auto-vm` 是**空壳**——仅 207 行/2 文件（dump 字节码 CLI），真 VM engine 在 `auto-lang::vm`（91k 行）。把 `vm/`（engine/codegen/native_registry/virt_memory…）迁入 auto-vm crate，CLI 一并归位。依赖方向合理（vm 只依赖 ast/dialect 面），是风险最低的第一刀。
2. **`auto-trans`**：`trans/` 47k 行（`trans/rust.rs` 单文件 27k 行）。
3. **`auto-ui`（或 auto-aura）**：`ui/` 216k 行是最大头（`iced/renderer.rs` 37k 行单文件）；先理清与 `ui_gen/`（80k，四生成器）的依赖关系再定边界。
4. **内嵌 `tests/` 60k 行析出为独立 tests crate**（feature-gated），缩短主 crate 增量编译。
   - 收益：增量编译/并行 check/模块边界清晰；成本：feature 矩阵与 nextest 档位（t/tv/tt/tu/ta）需重排——须与 `.config/nextest*.toml`、heavy_gate 同步评审。

### 5-2 `crates/auto-man`：43k 行，单文件巨型化先行

`src/vue.rs` **13,021 行（584KB）**、`rust_ui.rs` 236KB、`api_gen.rs` 196KB。文件级拆分（vue.rs 至少拆 emitter/templates/context 三块）优先于 crate 级拆分；是否析出 `auto-man-ui` 由设计裁定。

### 5-3 巨型单文件台账（>10k 行，建议逐个登记 KNOWN-DEBT）

```
37,239  auto-lang/src/ui/iced/renderer.rs
30,944  auto-lang/src/ui_gen/vue.rs
27,089  auto-lang/src/trans/rust.rs
24,461  auto-lang/src/ui/aura_view_builder.rs
21,433  auto-lang/src/parser.rs
15,983  auto-lang/src/ui_gen/rust.rs
15,145  auto-lang/src/vm/codegen.rs
13,021  auto-man/src/vue.rs
11,962  auto-lang/src/vm/engine.rs
```

拆分随对应模块重构顺带做，不单独立项。

### 5-4 crates/ 内混入的非 crate 内容

- `crates/autoui-skill/`（12 文件技能资产，无 Cargo.toml）→ 迁 `.agents/skills/` 侧或 `tools/`；
- `crates/ac-examples/`（2 文件）→ 迁 `examples/`；
- `crates/graphify-out/`（空）→ 删（见 1-13）；
- `crates/auto-xml/`（disabled，members 已注释）→ 裁定修复或移除。

---

## 6. 汇总：分批落地方案

| 批次 | 内容 | 级别 | 前置 |
|---|---|---|---|
| **A 误入库治理** | §1 全表：`git rm --cached` 清单 + `.gitignore` 补丁（`.gradle/`、`.agent/`、`.vscode/`）+ 残留文件删除/归位 + lockfile 口径统一 | 每个 L0（worktree 执行） | 核查 CI/脚本对 tmp-corpus 的引用（已查：零引用） |
| **B 本地清理** | §2 清单 ≈10.4GB + `scripts/clean-local.sh`（dry-run 默认） | 无 git 改动，立即可做 | `.worktree/auto-down` 先 wt-guard |
| **C 重复归并** | 3-2 计划文档归档、3-3 lib-legacy 删除、3-5 agent 目录约定、3-9/3-10 状态标注、1-8/1-9 根文件退役 | 1-2 个 plan | — |
| **D 大目录治理** | 4-1 archive 分桶、4-2/4-3 截图三层治理、4-4 website 压缩 | plan 级 | 决策点①② |
| **E crate 拆分** | 5-1 四步（vm 归位 → trans → ui → tests）、5-2 文件级拆分、5-4 迁出 | L2 设计 → 多 plan | 建议在无在途 plan 窗口启动 |

**需用户裁定的决策点**：
1. **git 历史瘦身**——BMP/Gradle 产物/大 PNG 已进历史，工作区 untrack 不减 `.git` 体积；是否 `git filter-repo` 重写（需协调所有 worktree/多机 force-push）？
2. **大截图归宿**——LFS、外部 artifact 存储，还是压缩后继续入库？
3. **`test/` vs `tests/`** 是否物理改名归并（涉及代码引用面）？
4. **画廊与 packages 四 UI 包**的产品定位归并。
5. `auto-xml` 修复还是移除。

---

## 7. 风险与红线（执行时必读）

- **`*.json` 全局 ignore 的历史教训**：该规则已引发 ≥5 次 `!` 负规则补漏（27_c_abi manifest、i18n zh.json、drawlist-renderer package.json、style_palette_manifest、website 截图 manifest）。任何 ignore 变更须 fresh clone 抽查上述测试位不缺件。
- **worktree 红线**：禁在 worktree 内建 junction/symlink；清理 `.worktree/auto-down` 链接与任何 worktree 前必跑 `wt-guard.sh`。
- **untrack 前查引用**：CI workflows、`scripts/`、归档计划外的活文档是否引用待清对象（tmp-corpus 已查为零引用）。
- **大量 `git mv` 会稀释 blame**：archive 分桶一次完成；用 rename 检测友好的单步移动。
- **多 agent 并行窗口**：批次 A/E 触及 `.gitignore` 与 crates 结构，会撞所有在途 worktree 的 rebase——在无在途 plan 或 fold 后窗口执行。
- **测试档位联动**：5-1 拆分后 `.cargo/config.toml` 别名、`.config/nextest*.toml`、`heavy_gate`、test-mem-weights 全套须同步重排，复审门禁口径不变。

---

## 附：本次分析所用核查命令（备查复测）

```bash
# 跟踪体积/文件数（stat 求和， immune to xargs 分批坑）
git ls-files -z <dir> | xargs -0 stat -c %s | awk '{s+=$1} END {printf "%.1fMB\n", s/1048576}'
# 可疑跟踪对象
git ls-files | grep -E '\.(bmp|lock|bak|tmp2)$|\.gradle/|^\.agent/|^\.claude/'
# 本地堆积
du -sh --exclude=.git */ | sort -rh | head -15
# 重复文件
cmp -s <a> <b> && echo IDENTICAL
```

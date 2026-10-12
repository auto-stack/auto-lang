# 根目录精简检查（2026-10-11）

本次是检查与整理建议，未删除、移动或取消跟踪任何现有文件；未运行构建或测试。根据本地主检出 `master` 的 HEAD `dde29cf2cdb68d5851aa61c3eb536a04d5aa1fae`、Git 索引、文件内容和仓内引用做静态核查，没有核对远端分支是否与此 HEAD 一致。

## 结论与数量

- 已提交的根目录有 **53 项：25 个目录、28 个文件**。
- 本地根目录有 **124 项**，其中 **71 项没有对应的根目录提交条目**，包括 `.git`、构建缓存、日志、临时探针等。清理这些本地项不会改变 GitHub 的文件列表。
- 建议先整理临时文件、本机配置和散落的文档/脚本：**53 → 34 项**，减少 19 项，不需要搬动核心模块目录。
- 再合并 `tests`、`tmp-corpus`、`tools`、`deploy` 四个目录：**34 → 30 项**。这一阶段涉及引用、脚本相对路径及验收入口，不能直接搬完就算完成。
- 根目录 Node 配置还有进一步精简空间，但存在真实的 npm 命令入口，先保留；统一包管理器后可再少 1 项。

## 1. 可以从仓库移除的临时文件（9 个根目录项）

| 路径 | 检查结果 | 建议 |
|---|---|---|
| `.tmp-p730-asref.py` | 一次性修改 Rust 源文件的替换脚本 | 删除；实现结果已在源文件中，Git 历史保留脚本 |
| `.tmp-p730-dbg.py` | 替换 Plan 730 调试测试的一次性脚本 | 删除 |
| `.tmp-p730-f2fix.py` | 重写 Plan 730 测试的一次性脚本 | 删除 |
| `.tmp-p730-r1e2e.py` | 向 Rust 文件插入测试的一次性脚本 | 删除 |
| `.tmp-p730-race.py` | 修改测试竞态处理的一次性脚本 | 删除 |
| `.tmp-p734-r1a.py` | 修改 api_gen/type 判定的一次性脚本 | 删除 |
| `check_error.txt` | Rust 编译告警输出，没有发现其他受跟踪文件消费它 | 删除 |
| `donut_sfc.txt` | Vite 转换后的 DonutChart SFC 调试导出，含热更新客户端代码 | 删除 |
| `uncompleted_plans.md` | 2026-05-28 起留存的旧计划分类快照；当前已有 `docs/plans/` 和 Specs 账本 | 删除旧快照，当前入口保留在 docs；不把旧状态重新灌回账本 |

另有 **1 个根目录项** `flow.at`：内容是 `feature_X` 的通用流程样例，未发现仓内按该路径加载的入口。建议移到 `docs/design/examples/flow.at`；如果不再保留旧流程设计示例，也可以删除。这一项与上表合计减少 10 个根目录项。

目录内另外发现 `.autoos/specs.json.tmp2`：受跟踪、**0 字节**，没有找到代码/脚本引用，可以删除。它不会减少根目录项数，因为 `.autoos/` 仍需保留。

## 2. 本机配置：取消跟踪，按需要留在本地（4 个根目录项）

| 路径 | 证据 | 建议 |
|---|---|---|
| `.agent/` | 两个 Git 模式 `120000` 的链接项，目标是 `D:/autostack/auto-skills/...`；本机实际表现为存储链接目标文本的普通文件。`.gitignore` 已忽略此目录 | 从 Git 移除；本机安装物按工具需要保留，不迁入公共仓库 |
| `.claude/` | 跟踪了 `settings.local.json` 和 commit 技能；目录已被忽略 | 本机设置取消跟踪。有共享价值的 commit 技能改放 `.agents/skills/`，且先按当前测试/worktree 纪律重写，不能照搬旧的裸 cargo test + push 流程 |
| `.vscode/` | 唯一受跟踪文件 `iar-vsc.json` 指定 `D:\soft\iar\ewriscv-3.40.1` | 本机配置取消跟踪；需要分享时把通用模板放到 docs 或 scripts，而不是跟踪个人安装路径 |
| `.clangd` | 唯一 include 参数是 `/home/visus/.auto/libs/stdlib` | 本机配置取消跟踪，并加根锚定忽略；不强制删除本地可用配置 |

这里的“移除”指不再进入仓库，不等同于清空个人工具配置。已有 ignore 规则不会自动让已跟踪文件退出 Git。

## 3. 归位而非丢弃（5 个根目录项）

| 现路径 | 建议归位 | 需要同步处理 |
|---|---|---|
| `DEBTS.md` | `docs/plans/DEBTS.md`；之后逐项核对是否能并入 `KNOWN-DEBT-AND-RISKS.md` | 本文件仍有拆仓候选和有效风险，不可当成重复副本删除；修正文档/代码注释入口。先迁移可减少根目录项，账本合并不能丢唯一条目或销账记录 |
| `SAFETY.md` | `docs/guides/filesystem-safety.md` | 同步 `scripts/check-junctions.sh` 的提示/引用；保留事故记录及 worktree 禁链接纪律。整理与当前 AGENTS/wt-guard 的差异，避免旧文档的“git 知道 junction 边界”给出误导 |
| `vm-600-callout-box.png` | `docs/plans/evidence/600/vm-600-callout-box.png` | Plan 600 归档文档第 105、118、131、144 行明确引用；`.autoos/specs.json` 也保存了验收描述。应保留图像并修正文档链接，按账本维护规则处理引用 |
| `deploy-website.ps1` | `scripts/deploy-website.ps1` | 原脚本以 `$PSScriptRoot\website` 定位；搬迁后必须重新定位仓根，并更新 `docs/guides/website-deployment-guide.md:74` 起的用法。检查路径即可，本轮整理不触发真实部署 |
| `kill-vue-dev.bat` | `scripts/kill-vue-dev.bat`，或经过用途确认后退役 | 当前脚本实际执行 `taskkill /F /IM node.exe`，会结束全部 Node 进程；迁移不应运行它来“验证” |

特别说明：Design 32 曾把根目录截图归为“不迁”的仓根杂物，但 Plan 600 和 Specs 中仍有明确验收引用。本次建议保留证据并归位，而不是据“杂物”字样直接删除。

## 4. 可合并的目录（最多减少 4 项）

| 现目录 | 建议 | 难度与验收点 |
|---|---|---|
| `tests/`（17 个受跟踪文件） | 合入 `test/`：例如四个总入口放 `test/runners/`，`tests/auto/` 放 `test/auto/`，vue-gen-gap 放 `test/ui/vue-gen-gap/` | `tests/a2r_tests.at` 等是实际 Auto 测试入口；CLI 注释和两条 a2c 构建脚本引用它们。保留 `test/` 这个名称可避免重写大量 Rust 中的 `test/ui/...` 路径。更新运行说明及 vue-gen-gap 的相对依赖路径，保持从仓根运行的语义 |
| `tmp-corpus/`（29 个受跟踪文件） | 合入 `test/fixtures/plan446/`，按 b3check/j2check 子目录保留；可维护探针另放 `scripts/repro/p446/` | `crates/auto-lang/src/tests/plan446_j1_repro_tests.rs:14` 直接定位其 app.at；缺失会 SKIP，不能用“测试没报红”作为迁移成功证据。MJS 还含旧 worktree 绝对路径和 import.meta.url 相对目录，要同步修正 |
| `tools/`（7 个受跟踪文件） | 合入 `scripts/tools/` | 有 Tailwind manifest 生成器、截图差分器、shadcn 快照工具和 native-fixture Rust 独立项目；它们不是垃圾。更新 `.agents/skills/autoui-verifier/SKILL.md` 的差分器路径、工具 usage、根目录计算和 native fixture 构建入口。native-fixture 已有独立 workspace 边界，应保持 |
| `deploy/`（5 个受跟踪文件） | 合入 `scripts/deploy/` | Playground systemd/nginx 和 Plan 736 HTTP 服务模板均需保存；修正文档、模板/探针相对路径，用静态检查及本地夹具验证，不执行公网部署 |

这是有边界的目录归并，不是建议删除这些目录里的源码。相比将所有内容装进一个泛化目录，先消除重复命名和散落入口收益更明确。

## 5. 根目录 Node 配置（暂不整组删除）

`package.json` 有三个真实快捷入口：`build:playground`、`dev:playground`、`start:playground`。不是空文件，也不是 npm workspace 聚合配置。

有两个锁文件，且不一致：

- `package.json` 与 `pnpm-lock.yaml` 都声明 `vue-router ^5.1.0`、`typescript ^6.0.3`。
- 根 `package-lock.json` 仍声明 `vue-router ^5.0.3`、`typescript ^6.0.2`。

建议先统一根工具使用的包管理器：如果选择 pnpm，则移除根 `package-lock.json`；如果选择 npm，则更新 npm 锁并移除根 `pnpm-lock.yaml`。可以减少 **1 项**，但不能以此推断所有子项目也要删除 npm/pnpm 锁。

进一步取消根目录三个 Node 文件，需要把快捷入口迁成 scripts 命令或放到现有 Playground 包，并核查开发脚本/生成工程对根 `node_modules` 的依赖。`scripts/build-playground.mjs` 已在 frontend 目录构建，但这并不自动证明根依赖对其他工作流无用。本次未安装包、构建前端或验证迁移，不直接把整组判为可删。

## 6. 建议保留的根目录入口

完成第 1–4 节之后，保留的 30 项如下（便于核对目标结构）：

```text
.agents/          .autoos/          .cargo/           .config/
.github/          auto/             blueprints/       crates/
docs/             examples/         packages/         parity/
patches/          schema/           scripts/          stdlib/
test/             website/
.gitattributes    .gitignore        .zcodeignore
AGENTS.md         CLAUDE.md         Cargo.toml        LICENSE
README.md         README.cn.md      package.json      package-lock.json
pnpm-lock.yaml
```

上面为 **18 个目录 + 12 个文件 = 30 项**，没有创建新的根目录容器。实施前应重新核对 Git 树，避免检查期间其他会话提交造成清单变化。

保留依据：

- `.agents` 是受维护的仓内验证技能；`.autoos/specs.json` 是规格账本，不能因为被全局 `*.json` 忽略规则命中就删。
- `.cargo` 与 `.config` 有 Cargo 别名、nextest 分层配置和内存权重；`.github` 是 CI 工作流。
- `auto` 是自举库及相关入口；是否退役 aavm 属于功能/架构决策，不能作为根目录美化的一部分直接删。
- `blueprints` 是实际消费的模板规格；`schema/aura.at` 由 `schema_loader.rs:838` 的 `include_str!` 编译期嵌入。
- `stdlib` 是运行库与装配契约；`parity` 是多语言行为对拍资源；`patches/iced_widget` 被根 Cargo.toml 的 `[patch.crates-io]` 使用。
- `crates`、`packages`、`examples`、`docs`、`website` 是不同职责的源码/文档/发布入口，建议保留。将 website 移到 docs 不值得为少一行触发大量发布路径迁移。
- `AGENTS.md` 与 `CLAUDE.md` 是工具自动发现入口；可以把 CLAUDE 的重复规则缩成指向 AGENTS 的短说明，但不建议仅为少一项破坏入口。
- 两份 README 与 LICENSE 保留。`.gitattributes` 保证 shell 脚本 LF，应保留。
- `.zcodeignore` 是工具按约定寻找的根配置；使用 ZCode 时保留，明确弃用 ZCode 后才删除，不能只凭与 gitignore 相似就合并。

## 7. 本地清理与防止再次误提交

本地未提交项中，`.tmp-vm-*`、`.tmp-vite-*`、`.tmp-wg-*`、各类 `*.log`、`a/b/c/s*.json`、`remix*.json`、`nul`、`tmp_test_vm_append.txt`、`c_fs_app_tmp/`、`c_text_app_tmp/` 等可以在对应走查/测试会话结束、证据归档后清理。它们不影响 GitHub 根目录行数。检查时仍有当前 VM 走查脚本/截图，因此本次没有替其他会话清除它们。

- 临时探针与输出统一放到现有、已忽略的 `scratch/<任务>/` 或 `tmp/<任务>/`，不再把每个文件摊到根目录。
- 补充根锚定忽略规则，例如 `/.tmp-*`、`/tmp_test_vm_*`、`/c_fs_app_tmp/`、`/c_text_app_tmp/`、`/.clangd` 和 `*.tmp2` 的适当作用域。正式复现脚本进入 `scripts/repro/`，有价值的证据进入 docs。
- `target/`、`node_modules/`、`.am/` 是可再生内容，但删除会影响构建/缓存；`.zcode/` 和 `scratch/` 可能含个人设置或未归档证据，不能整目录视作垃圾。
- `.worktrees/`、`worktree/`、`auto-shell/` 当前不在根 Git 树中；检查时 `.worktrees/auto-down/autodown` 尚有目录内容、`auto-shell` 有 Cargo.lock。不要用名字推断它们均为空废物。清理前先核查 Git worktree 归属、未提交内容和 reparse points，再按仓规 guard。
- `.git/` 必须保留；本地 Cargo.lock 也不是误提交，本次它未跟踪，是否改为跟踪是单独的可复现构建政策问题。

`git ls-files -ci --exclude-standard` 还暴露了两个方向的问题：

1. 真正的生成物仍被跟踪，如 Python `__pycache__`、Playwright 报告和示例 UI cache；可另做范围明确的清理。
2. 必须保留的源码也被过宽忽略规则命中，如 `crates/auto-lang/src/ui/debug/*.rs` 被 `debug/` 命中，以及多类必要 JSON 被 `*.json` 命中。

所以不能把“已跟踪且被 ignore 命中”的列表整体删除。应收紧 `debug/` 等规则到构建目录，并按实际用途维护 JSON 白名单或更明确的生成物规则。

## 8. 落地边界

本次交付只是检查报告。主检出已有 `docs/plans/KNOWN-DEBT-AND-RISKS.md` 和 `docs/plans/reports/759-post-landing-review-20261011.md` 的未提交修改，未改动这两件。

真正实施目录精简是多文件任务，应按 AGENTS 的 L1 流程立计划、在专用 worktree 实施和独立复审。第一阶段若只删临时件/移文档与脚本，按 Category A 做路径与目标验证，不跑 Rust 全套；第二阶段如修改 Rust 内的夹具定位，则按 Category B 运行日常及触面门禁。迁移后的测试必须确认确实加载新路径，不能接受因资源找不到而静默跳过。

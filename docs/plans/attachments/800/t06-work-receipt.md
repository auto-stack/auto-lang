# PLAN-800 T-06 work 收据（净删/搬迁/未迁 + V01..V05 证据）

worktree `D:/autostack/.wt/lang-800/auto-lang`，branch `plan-800-dev`，
基面 master `b0d6d4aab`；auto-down 兄弟 `D:/autostack/.wt/lang-800/auto-down`
detached `c39ac7f`（依赖仓只读，未修改——跨仓 path 依赖 autodown-core 解析用，
PLAN-726 T-04 布局）。

> **r1 修订（2026-10-17）**：按 review-r1 needs_fix 修复两项——F800-R1-01
> （P2，错误映射描述失实）与 F800-R1-02（P3，新核心未过 rustfmt）；修复提交
> `c50e7a1c6`，冻结代码基线由 `5c1f43b5d` 前进为 `c50e7a1c6`。§2 搬迁表与
> 上文涉及"adapter 映射"的表述已就地修正；本文其余 r0 证据（测试数/闭包/
> 合流对账）在 r1 复验中全部复跑通过。修复明细见文末 §6。

## 1. 净改动（staged diff --stat）

```
 Cargo.toml                                         |  10 +   (member + exclude)
 crates/auto-man/Cargo.toml                         |   4 +   (auto-package-resolver 依赖)
 crates/auto-man/src/resolver.rs                    | 408 +++--（实现段 489 行 → 薄委托 ~290 行；测试全保留+4 新例）
 crates/auto-package-resolver/Cargo.toml            |  17 +   (std-only)
 crates/auto-package-resolver/src/lib.rs            | 599 +++  （实现 ~330 行 + 14 单测）
 docs/specs/auto-man/project.md                     |  25 +   (SD-02：抽取分工节 + resolver 模块行)
 docs/specs/package-resolver/project.md             |  95 +   (SD-01：新建)
 test/fixtures/modularity/package-resolver-consumer/| 335 +   (V01 fixture + verify-deps.sh)
 .autoos/module-boundaries.toml                     |（man-ecosystem 域映射 +1 行）
 12 files changed, 1237 insertions(+), 246 deletions(-)
```

## 2. 搬迁/净删/未迁表

| 项 | 处置 | 证据 |
|---|---|---|
| `find_package_path` 算法 | **搬迁** core（错误改 `PackagePathNotFound`；legacy 调用点 `if let Ok` 静默忽略，**无 adapter 映射**——r1 修正） | auto-man 内 `fn find_package_path` 定义 grep=0 |
| `find_module_in_dep` 算法 | **搬迁** core（segments `&[AutoStr]`→`&[&str]`，AutoStr 转换在 adapter） | 同上 `fn find_module_in_dep` grep=0 |
| dep 未声明错误文本构造 | **搬迁** core `resolve_in_dependency` | adapter 无 format! 该文本 |
| `resolve`/`exists`/`search_paths` 组合规则 | **搬迁** core；trait impl 留 adapter 薄委托 | `impl ModuleResolver for AutoManResolver` 四方法各一行委托 |
| Dependency DTO | **下沉为 core 泛型 `DependencyEntry<M>`**；adapter `pub type Dependency = DependencyEntry<ExecutionMode>`（单一 map，get_dependencies 借用 core） | resolver.rs L22-24 |
| `load_pac_at`/`prepare_env`（pac 行扫描） | **未迁**（adapter 原样，仅 find_package_path 调用改指 core） | T-01 §1 责任表 |
| ExecutionMode 语义 / ModulePath/PathPrefix / ModuleResolver trait | **未迁**（语言边界） | adapter imports 仅 `auto_lang::{resolver,mode,ast}` 三路（799 前/后兼容——旧 AST 路径身份不变） |
| Pac/Super/None 前缀 | **未迁**（继续委托 `FilesystemResolver::with_package_root(project_root)`） | plan800_tests 2 例经生产入口钉住 |
| Target/build/Git/lock/exporter/CLI | **未迁**（795/780 所有权不变） | diff 面零触及 |

## 3. V01..V05 证据（work-owned 定向验收）

| Case | 命令 | 结果 |
|---|---|---|
| V01 | `cargo test --manifest-path test/fixtures/modularity/package-resolver-consumer/Cargo.toml` | **7/7 ok**（fresh 真实目录 resolve/歧义/缺项/候选、consumer 自定义 ConsumerMode 证模式不透明、同名替换立即可见） |
| V01 闭包 | `bash test/fixtures/modularity/package-resolver-consumer/verify-deps.sh` | **PASS**（normal/build 闭包 = 仅 `auto-package-resolver v0.1.0`，无 auto-lang/auto-ast/auto-man/auto-gen/auto-ui/auto-vm/auto-val） |
| V02/V03/V04 | `cargo test -p auto-man --lib resolver::` | **38/38 ok, 1 ignored**（预存 ignore：Config mode changes；34 旧例零改动断言 + 4 新 plan800 例：Pac/Super 生产入口委托、无 pac.at、adapter 级同名替换） |
| V02 核心侧 | `cargo test -p auto-package-resolver` | **14/14 ok**（三臂无盘构造、exists 三差异、组合序、候选优先级、四态判定、错误文本、无序集合比较、泛型不透明） |
| V04 单一实现 | grep `fn find_module_in_dep\|fn find_package_path` crates/auto-man/src/ | **0 定义**（算法只在 core） |
| check | `cargo check -p auto-package-resolver` / `-p auto-man` | 通过，resolver.rs 零告警（auto-man 余留告警=vue.rs:7278 预存，非本计划触面）；`cargo check --workspace` 唯一红=`auto-cosmic-demo` E0063 **预存**（主检出同报，未触本计划文件） |
| 777 边界 | `python scripts/modularity-audit.py` | exit 0、零违规（observe 全仓 + strict 基线外新边=无；`auto-package-resolver`=man-ecosystem 域映射已登记） |

## 4. 合流收据（V05：792 交付闭包对账 + 794 八 native 不变）

- **792（generated-ui-independent-build）**：work 时点未合入——master 上
  `rust_ui.rs`/`automan.rs`/auto CLI main/generated-ui-build fixture 相对取材点
  `30d3e1d2a` **零 diff**；`plan-792-dev` 分支仍在独立 worktree（未合并）。
  按并行合同：**新 crate 不在 master 任何 792 交付闭包中，无 source/manifest
  接线义务**；不伪造 792 验收证据。若 792 后合入，其基于最新 master 重新冻结
  闭包时自行判定 auto-package-resolver 是否入 UI 生成应用闭包（合同"后合入者
  责任"行）。
- **794（scroll-host-provider）**：本计划 diff 面零 auto-lang/auto-vm/UI 文件
  （§1 stat 全列）——八 scroll native/provider/renderer adapter 字节不动。
- **799（frontend-ast-model-extraction，未实施）**：adapter 仅消费旧兼容路径
  `auto_lang::ast::{ModulePath,PathPrefix}`（与抽取前相同三路 import）；
  799 合同保持旧类型身份 → 799 前/后均可编译。
- **798/796/797**：实验/在途线，无共享写面交集（本计划仅触根 Cargo members/
  exclude 精确两行，符合短窗口合同"仅加自己的 member/精确本地依赖"）。

## 5. merge-owned 待办（不在 work 关闭）

- 裸 `cargo t` + 两 crate 定向门（`cargo test -p auto-package-resolver`、
  `cargo test -p auto-man --lib resolver::`）+ consumer fixture 门。
- 未改 VM/trans/book/ui_gen → 不触发 tv/tt/tb/tu（如复审扩大触面再按规则补）。
- 800%5=0 → 合入后批回归到期交接（`/auto-plan:regress` 主检出单实例）。
- specs/ledger 沉淀（`.autoos/specs.json` upsert + spec-index）与归档/guard 清理。

## 6. r1 修复记录（finding → invariant → fix → results）

review-r1（reviewed_commit `5c1f43b5d`，attachments/800/review-r1/REVIEW.md）
needs_fix 两项；修复提交 `c50e7a1c6`（同树同分支）。

| finding | invariant | 受影响路径/状态 | 修复与永久用例 | 当前结果 | 余留缺口 |
|---|---|---|---|---|---|
| F800-R1-01 [P2] SD-01/core 文档/t01/t06 四处声称 adapter 将 `PackagePathNotFound` 映射回 `AutoManError::FileNotFound`；实际 `resolver.rs` 三个调用点（use/dep:/("pkg",mode:) 臂）均为 `if let Ok` 静默忽略 | 知识增量与代码真实边界一致：私有不可观察错误不沉淀为不存在的规范转换；找包失败静默跳过是冻结行为（r0 与旧代码一致，无行为回归） | `docs/specs/package-resolver/project.md` §冻结语义7、`crates/auto-package-resolver/src/lib.rs` PackagePathNotFound 文档、主检出 t01-freeze §1/§2/§3.4、t06 §2 | 四处就地改述为真实语义（legacy 调用点静默忽略、无 adapter 映射、旧私有 FileNotFound 构造已随搬迁删除且无消费方）；按审核裁定不为私有不可观察错误添加无意义映射代码；同因扫描其余文档（SD-02/auto-man spec）无此失实表述 | 措辞与代码一致（grep 无正向"映射回"残留）；行为零变化 | 无 |
| F800-R1-02 [P3] 新核心 `rustfmt --check` exit 1（6 处确定性格式差异） | 新增文件过格式门；不引入新格式债，亦不对历史搬运代码做无关格式改写 | `crates/auto-package-resolver/src/lib.rs`、consumer fixture 三文件、`crates/auto-man/src/resolver.rs` 新代码区域 | 新文件全量 rustfmt（--check exit 0）；adapter 定点修 7 处本计划新代码（import 序、add_dependency 委托、Loaded 日志、segments 收集、plan800_tests 三处）；rustfmt 仍标记的 26 行经内容比对全部逐字存在于 master b0d6d4aab（历史搬运，保留原样） | 新文件 fmt exit 0；核心 14/14、adapter resolver:: 38/38（1 预存 ignore）、consumer 7/7、verify-deps.sh PASS 全部复跑通过 | 无 |

r1 后 SD 冻结 hash（worktree `c50e7a1c6`）：
- SD-01 `docs/specs/package-resolver/project.md`：
  `8BF942D5F5AE1245F2F6AD098C4F39F351F6DD3A9DFE2639FA510C91CEAFAE3A`
- SD-02 `docs/specs/auto-man/project.md`（r1 未触，与 r0 冻结一致）：
  `8FCA03939E763C3B44182AE90D2931CD02F0FF571E96EA4A6ABA582EF0AC725D`

merge 注记（review-r1 §当前合流核对）：792 已合入 main（rust_ui.rs
DELIVERY_CRATES 10 项不含 auto-man，闭包无 auto-man→auto-package-resolver
边）——本新包不入 792 交付闭包；merge 时 SD/auto-man spec 须按**当前** master
版本合并（main 已含 792 新章节，不可覆盖），并履行 metadata 同步核对。

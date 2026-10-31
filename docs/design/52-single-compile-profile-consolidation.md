# 52 · 单一编译 profile 整改（auto-lang 去 UI / feature 拍平 / tm 清退）

状态：target-state r1（2026-10-31 用户六问裁定驱动；实施经四计划 C→A→B（本仓）+ D（auto-ui 仓））
性质：架构（L2）——依赖方向反转 + feature 面退役 + 测试设施概念清退

## 1. 背景与动机

auto-lang 已拆出 auto-ui / auto-man / auto-web，但编译面没有随之收敛：

- **facade crate `auto-lang` 仍默认拉整套 UI 栈**：`default = ["with-file-history", "ui-iced"]`，
  其中 `ui-iced` 经跨仓 path 依赖（`../../../auto-ui/crates/auto-ui-runtime`）把
  iced/wgpu/ash/naga/windows 整个图形闭包编进**本仓** target——"拆仓"只是源码搬家，
  产物归属未变；且每份 plan worktree 组都需要 auto-ui/auto-down 兄弟仓才能构建。
- **多套 feature 组合反复分叉重编**：PLAN-896 固定组方案（`scripts/test-groups.toml`）
  仍按组传 `--features`（109 组中 34 组），同一 crate 在主 target 内存在 2-3 份单元
  （auto_lang_core ×3 @228M、windows ×3 @103M 实测）。
- **旧 tm/tier 概念已死未清**：`cargo tm` exit 2（PLAN-896 r4），但别名、引擎脚本、
  冻结目录与技能文档全部残留；auto-plan-* 技能（3 副本）仍在教学已失效指令。
- **`with-file-history` 是死开关**：rustyline REPL 行历史 feature 的镜像，零 cfg 门、
  零构建效果（rustyline 经依赖项恒开），2026-02 起无任何真实用途。

## 2. 终态（Target State）

### 2.1 依赖方向

```
auto-ui 仓 ──► auto-lang 基础 crate（core/parser-api/ast/atom/val/lex/media/text/terminal/a2r-std/tool-*）
auto-lang 仓 ══✗══► auto-ui 任何包（零依赖，工作区层面环解除）
auto-man/auto-web ──► 同上基础 crate / 各自域
```

- `crates/auto-lang`（facade）：`default = []`；不再声明 `auto-ui-{compiler,runtime,dialect}`。
- UI 解析/生成管线（UiDialect 装配、aura 提取、design tokens、ui_gen、a2ui、
  `install_ui_host_capabilities`）整体迁 auto-ui 侧装配；生成长流程经子进程
  （沿用 `AUTO_UI_TOOL_BIN` 先例）。
- auto-lsp、UI examples、UI 集成测试、auto-cosmic/{demo,host-libcosmic} 迁 auto-ui 仓。
- auto-lang worktree 单仓即可构建（无需组内兄弟仓）。

### 2.2 feature 政策

- **测试集合不用 feature 门**：test-trans/test-book/test-cases/test-http-e2e/
  test-ace/test-case-batch/test-aavm 及其 cfg 门全部删除，测试体恒编译；
  选择交给 `cargo t` 固定组过滤 + `#[ignore]`（aavm 手动重放）+ 资源组/T3 条件原样。
- **余下 feature 仅"依赖可选"**：python（PyO3）、mem-profile（CountingAlloc）等——
  无脚本化切换、不产生循环分叉。零 cfg 门且零语义的死 feature（with-file-history、
  use-evaluator）直接删除。
- 被取代裁定记录：2026-09-22"ui-iced 进 default 统一全档"的前提（UI 未物理拆仓、
  档间切换整库冷重编）已消失，本设计取代之；统一对象改为 auto-ui 仓自身（计划 D）。

### 2.3 测试选择机制单一化原则

> **编译恒定，运行期选择。** 任何测试档差异不得引入新的 cargo feature 组合；
> 需要排除/包含时用固定组（test-groups.toml）、nextest 过滤表达式、`#[ignore]`
> 或资源组实现。违反即回归到多 profile。

### 2.4 单一 profile 断言（合入后的可验证不变量）

clean 后 `cargo build` 记录 deps 单元集 → `cargo t all` → **零新增 rlib**（同 hash 复用）。
单一 profile = 所有工作流共享同一编译单元集。

## 3. 四计划拆分与顺序

| 计划 | 仓 | 内容 | 顺序理由 |
|---|---|---|---|
| C | auto-lang | tm/tier 概念清退（别名/引擎/冻结目录/治理测试/文档/技能 3 副本）；周期批量回归改 `cargo t all` | 小而独立；先清地面避免后续计划触碰半死设施 |
| A | auto-lang | 去 UI 依赖反转（facade 管线迁移、auto-lsp/examples/测试/cosmic 迁出、api-gen 模板、删 UI feature 族与 with-file-history/use-evaluator） | 最大收益；独立于 C |
| B | auto-lang | 测试 feature 拍平（删 test-* 门 ~115 处、test-groups 去 --features、bins 恒编译） | 在 A 后：UI 测试已迁出，拍平面最小 |
| D | auto-ui | ui-iced 拍平为无条件依赖；承接 A 迁入件 | 依赖 A 的迁移落位 |

auto-man 仓自身的 rustyline 声明与 `core --no-default-features` 面不在本设计范围（另案）。

## 4. 风险与回退

- **aavm/http-e2e 测试体常编译**会拉长日常构建：实施时实测增量，**>2 分钟回报重议**
  （回退选项：仅保留 test-aavm 一个分叉）。
- api-gen 生成工程的 manifest 模板改动影响存量生成物：夹具（rust-workspace、
  modularity consumers）全量回归兜底。
- 技能用户级副本（`~/.zcode/skills`、`~/.agents/skills`）属用户资产：更新前在计划内
  列 diff 供过目。
- 体积预期：iced/wgpu/ash/naga 栈移出后主检出 deps 显著低于 3.6G 基线（量化在 B/D 合入后）。

## 5. 交接

- PLAN-896（固定组）为现行测试范式，本设计只做其 feature 面收口与前置概念清退；
  测试治理语义（ignored/平台/里程碑条件、重档纪律）原样保留。
- 计划 C 执行时同步重基底或关闭 drafting 中的 893（其验证面引用 cargo tm）。

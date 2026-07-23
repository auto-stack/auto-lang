# Auto-Lang Specs 体系规约

> 版本：v1（2026-07-23）
> 设计文档：[docs/design/plan-spec-hybrid-model.md](../design/plan-spec-hybrid-model.md)（背景、诊断、取舍对照）
> 本文档是**操作规约**：目录怎么组织、文档怎么写、流程怎么走。改动本规约需先改设计文档。

---

## 1. 概念

```
project   子项目 = 一个 Cargo crate / 一个 npm package / 一个顶层资源目录
module    模块   = project 内一个内聚功能单元（对应 src/ 下一个目录或一组强相关文件）
plan      一次开发任务的过程记录（docs/plans/NNN-slug.md），存"过程"
spec      本目录下的文档，存"现状与知识"，持续重写
```

原则：**plan 管过程，spec 管沉淀；只引用，不复制；索引脚本生成，不手维护。**

## 2. 目录结构

```
docs/specs/
├── README.md            # 本规约
├── INDEX.md             # 全局索引（scripts/spec-index.py 生成，勿手改）
├── _archive/            # 历史 spec 封存（只读）
└── <project>/
    ├── project.md       # 项目卡（必有）
    └── <module>/        # 微型 project 可无 module 层
        ├── overview.md      # 模块概述（必有）
        ├── architecture.md  # 架构图 + ADR 追加日志（有架构内容才建）
        ├── design/          # 主题设计文档（按需，slug 命名）
        └── plans.md         # 相关 plan 索引表（spec-sync 维护）
```

## 3. 文档类型与模板

### 3.1 project.md（项目卡）

```markdown
# <Project 名>

> **Status**: active | experimental | archived
> 路径：<代码路径>  | 技术栈：<...>

一句话定位。

## 目标与范围
（链接 docs/roadmap.md 相关节；写明"不做什么"）

## 模块架构
​```mermaid
graph LR
  A[module-a] --> B[module-b]
  click A "./module-a/" "module-a"
​```
（mermaid 节点用 click 链接到 module 目录，点图即可下钻）

## 模块清单
| 模块 | 职责 | 状态 |
```

### 3.2 overview.md（模块概述，≤100 行）

```markdown
# <module 名>

> **Status**: implemented | partial | planned | stale

## 职责
## 现状
## 关键入口        （文件路径:符号，如 crates/auto-lang/src/vm/engine.rs:VmEngine）
## 使用示例
## 已知坑
```

### 3.3 architecture.md（架构 + ADR 日志）

架构说明一张图；下方为 **ADR 追加日志**，只追加不改写：

```markdown
## ADR-03: 标题
- 日期 / 来源：plan-318
- 决策：……
- 备选：A（pros/cons）、B（pros/cons）
- 后果：正面 / 负面 / 缓解
- 状态：active | superseded-by ADR-05
```

### 3.4 design/<slug>.md（主题设计文档）

单主题单文件，模板基准：`docs/specs/_archive` 之外的范例是原
`http-server-spec.md` 风格——范围 / 原则 / 细节 / **显式非目标** / 附录。

### 3.5 plans.md（plan 索引）

```markdown
| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 318 | list-struct-id-corruption | ✅ | archive/ | List 结构体 ID 损坏根因与修复模式 |
```

## 4. 五环开发流程

```
1 brainstorm → 2 write-plan → 3 execute-plan → 4 review → 5 spec-sync
（前四环只写 plan，不写 spec；第五环一次性完成沉淀）
```

- **brainstorm**：先读本树相关 module 的 spec 作为上下文；结论冻结进 plan 的"已确认决策"节。
- **write-plan**：`scripts/new-plan.sh <slug>` 取号（中央 `.next-id`，禁止自行估算编号）；
  plan frontmatter 必须声明影响面 `affects: [<project>/<module>]`。
- **execute-plan / review**：沿用 superpowers 技能（worktree 隔离、分批执行、code-reviewer）。
- **spec-sync**（plan 合并的收尾门禁）：读 plan + diff + review，逐受影响 module 回写
  overview/architecture(ADR)/design，追加 plans.md 行，归档 plan，重生成 INDEX.md，
  在 plan 文末追加"spec-sync 回写记录"节。typo 级改动可只更新 plans.md 一行。

## 5. 编号与引用

- plan 编号：`docs/plans/.next-id` 中央取号（当前 371 起）。
- ADR：模块内局部编号，全局引用 `auto-lang/vm#adr-03`。
- spec → plan 引用写 `(plan-318)`；plan → spec 引用写相对路径。
- 禁止全局 G/A/D/P/S/V/X 式前缀（旧体系教训）。

## 6. 工具

| 工具 | 作用 |
|---|---|
| `scripts/new-plan.sh <slug>` | 原子取 plan 号并创建 plan 文件骨架 |
| `scripts/spec-index.py` | 扫描本树生成 INDEX.md |
| `scripts/spec-lint.py` | 健康检查：断链、编号冲突、stale、plans.md 与实际 drift |

## 7. 反模式（详见设计文档 §9）

手工中央 manifest / 按流程产物分文档类型 / spec 写入多 role 接力 / 并发自行取号 /
同构双索引 / spec 描述不存在的代码 / spec 与 plan 内容互相复制 / 归档双轨。

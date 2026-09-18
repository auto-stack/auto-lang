# bp-gate — 蓝图级双端 gate（PLAN-075 T-03 首证）

L2 Blueprint 层的 parity 门（auto-lang design 30 §2「每单元双端 gate」的
第一次落地）：对 `blueprints/` 包库的消费做**自动化构建渲染断言**——
vue 臂（构建+渲染+截图基线）× VM 臂（boot+state/snapshot 断言）。

## 跑法

```sh
cargo build -p auto                      # gate 用的 auto.exe（本检出 target/）
cd examples/bp-gate && pnpm install      # 一次性（playwright）
pnpm gate                                # 双臂；任一红 exit 1
pnpm gate --arm vue | --arm vm           # 单臂
pnpm gate --update-snapshots             # 刷截图基线（e2e/baselines/）
```

`AUTO_EXE` env 可覆盖 exe（默认 `<repo>/target/debug/auto.exe`）。fn 携带
单元（filetree 组合形态）要求编译器 ≥ PLAN-645；同文件模块 fn 通道要求
≥ PLAN-075（G-6）。

## 形态裁定（PLAN-075 T-00，详见 auto-down
`docs/plans/attachments/075-bp-extraction-record.md`）

- **R-B 沙箱构建**：`auto build` 在 `dep bps` 工程内物化 `deps/bps`
  junction（pac.rs materialize_local_dep；Plan 529 worktree 红线）——
  gate.mjs 把 `blueprints/` + `host/` 复制到仓库外临时沙箱再构建，
  退出路径先摘 `deps/` 链接再整树删除。**严禁在仓库内直接 build host/**。
- **G-7 补件**：gen 腿不落 `vite-env.d.ts`/`auto-sources.ts`（074-sink-mode
  G-7 环境缺口），gate.mjs 在 vue-tsc 前补最小 shim。
- **R-C VM 臂形态**：bp 子件子树对 MCP 快照不可见（F-1）——内容断言走
  root 投影字段（`sb_*`/`rl_count`/`ft_count`）+ root 单元标记行 needle；
  需要驱动行交互的消费方走内联 twin（filetree gotchas#1 right 形态，
  未来交互断言扩展时启用）。
- **单页同定 + clip**：宿主三单元固定高度（h-52=208px）堆叠、fixture
  三单元互斥词表——playwright 全页 needle 无歧义 + per-unit clip 截图
  （几何在 units.mjs GEOMETRY 单源）。

## 单元台账

`scripts/units.mjs`（配置驱动）：id/title/vue{needles,clip}/vm{state,
snapshot}。新单元 = host app.at 加单元区（标记行+互斥词表 fixture）→
units.mjs 登记（clip.y = 208×n）→ `--update-snapshots` 建基线 → `pnpm gate`。

首证三单元（design 30 §2 L2 门首批）：

| 单元 | bp | 形态 |
| --- | --- | --- |
| status_bar | `layout/status-bar`（PLAN-075 骨架） | fn-free |
| row_list | `data-display/row-list`（PLAN-075 骨架） | fn-free |
| filetree | `navigation/filetree`（组合形态） | 跨文件 fn 携带（645 转译） |

与 bps-gallery 的关系：gallery 是 vue-only 源浏览（live-render deferred，
README scope note 维持）；gate 是自动化门——两者互补，互不替代。

与 jade component-gallery 的关系（072 Q-1）：jade gallery gate jade 消费
侧（jade 特有件）；bp-gate gate bp 包侧（跨 app 通用件 canonical 家）。

# PLAN-706 r2 T-09 定谳报告：vm UI 脏标/确认弹层交互面回归

- 日期：2026-09-30；执行窗：PLAN-706 rev2 work（worktree `D:/autostack/.wt/lang-706/auto-lang`，branch `plan-706-dev` @ master 4dae03122 基）
- 断言面：消费者仓 jade-edit `tests/vm_matrix.mjs` merged 臂（AUTOUI_MCP_PORT 驱动真 UI，进程内直调）
- 家族侧最小驱动：`.wt/lang-706/tab-arc.mjs`（组目录，不入 git）——复刻 [7 tab] 弧：开两档 → typeWholeDoc 标脏 → 切走/切回（脏标 TabActivate 投影还原）→ 关闭确认两路（取消留/直接关闭弃改+磁盘零写入）

## 1. A/B 锚（同 harness 同弧绿红分明）

| 载体 | 提交 | 结果 | 证据 |
|---|---|---|---|
| pre（旧载体） | `c8f86ef92` | **GREEN** | 家族驱动全弧过（本窗实测）；消费者 `p25-merge-smoke.log` merged 16/16 ALL GREEN 在库 |
| post（新载体） | master tip `4dae03122`（code 同 `66c9cac19`+707 族） | **RED（exit=101 进程死亡）** | 家族驱动首跑即红：死亡点=「切回 Tasks（脏标投影还原）」；消费者 debug `p26-matrix-r1/r2.log` ×2 同点 |

## 2. bisect 定谳（`git bisect start 66c9cac19 c8f86ef92` + `bisect-run`）

- 步进测试=每提交 `cargo build -p auto` + 家族驱动弧（docs-only 非 merge 提交按改动面跳过 exit 125）。
- 判定序（绿→红边界）：

| 提交 | 内容 | 弧结果 |
|---|---|---|
| `c8f86ef92` | pre 锚 | GREEN |
| `7b8a3ccbc` | 706 r1 主提交（vue.rs gallery fixes） | GREEN（**消费者 grep 定谳「7b8a3ccbc 不含 checkbox/menubar 域」复证**） |
| `bb303e914` | plan047 T-04 引擎读臂拦截 | GREEN |
| `3a72d0f15` | plan047 T-05 memo 门三级判定 | GREEN |
| **`432068075`** | **plan047 T-06 computed 信号网**（`aura_view_builder.rs` +209 / `vm_bridge.rs` +88，纯增） | **RED（exit=101）** |
| `43f02f195`..master | plan047 T-07 及后 | RED |

- **定谳：first bad commit = `432068075`（plan047 T-06 computed 信号网）。** 705 HTTP 簇与 706 vue.rs 面均绿（消费者预期排除成立）。

## 3. 触发机制（panic 实证 + 机制初判）

panic 现场（`RUST_BACKTRACE=full`，家族驱动 load 复跑捕获，`app-death-*.log` 全文在案）：

```
[X9-PANIC] panicked at crates\auto-lang\src\vm\rc.rs:742:13:
[RC canary] use-after-free: heap object 4000555 was freed 0.0s ago
  rc_check_tombstone (vm/rc.rs:742)
  ← get_heap_object (vm/engine.rs:1445)
  ← VmBridge::vmref_to_vec (ui/vm_bridge.rs:1058)
  ← VmBridge::index_list_all (ui/vm_bridge.rs:1041)
  ← AuraViewBuilder::value_to_iter_vec (aura_view_builder.rs:763)
  ← resolve_for_iterable closure (aura_view_builder.rs:1186)   ← 视图重建 for 迭代面
  （axum/MCP 请求处理上下文）
```

**机制**：`432068075` 引入 `ComputedSignal{ cached: Value, deps }` 跨帧缓存**裸 Value**。
`Value::VmRef`/≥4M 整数约定形是**无 RC 份额的裸堆 id**——克隆入网零持有。脏档切换弧的
状态替换（TabActivate 投影还原/doc 交换）释放旧堆对象后，信号命中把悬垂 id 送回渲染
（`deps_unchanged` 对**已释放对象 id 的 dep 键**永不 bump → 版本恒等 → 伪命中）。三断裂面同根因：

| 面 | 载体 | 形态 |
|---|---|---|
| debug 死亡 | RC canary（`#[cfg(debug_assertions)]`） | 悬垂 id 命 tombstone → panic（exit 101） |
| release 弹层栈积 | 无 canary | 释放 id 被新对象复用 → 错读他物 → 确认弹层状态错乱滞留（fail-snap 双叠「有未保存的修改」） |
| release probe_mtime 挂起 | 同上 | 错读致 Init 弧 done 恒 false（release 特异=debug canary 先爆） |

**竞态敏感度**（为何消费者确定性 ×4 而本窗首日 1/9 红）：窗口=状态替换与信号命中/重填的
交错时机；CPU 繁忙（消费方构建窗）加宽窗口。家族驱动加 6 路 CPU load 后 ~1/1-1/2 复红，
卸载 8 连绿。

## 4. 修复（T-10，同窗落地）

`computed_signal_store` 准入守卫：`value_carries_heap_identity(&cached)`（`VmRef`/`ValueRef`/
≥4M 整数形直载+容器递归+不可证净复合变体保守拒收）→ 不入网退回每帧重算（pre-706 行为，
与空集/超预算退档同族；plan047 v1 既有边界语义，零新特性）。

- 家族单测（`vm_bridge.rs` tests）：`plan706_signal_store_skips_heap_identity_value`、
  `plan706_signal_store_hit_cycle_idempotent_and_fresh`——**红相取证**（临时 gate-off 环境开关）：
  pre-fix 行为下第一测红（「VmRef 载体必须被拒收」），fix 后双绿；开关已移除。
- 定向回归：plan047 24/24、plan046 38/38、memo 86/86、vm_bridge 54/54 全绿。
- 行为收口：修复 exe 家族驱动 load 复跑（见 §5 附录）+ 消费者 merged 矩阵（T-11 定义性验收）。

## 5. 附录：证据文件（组目录 `D:/autostack/.wt/lang-706/`，不入 git）

- `tab-arc.mjs`：最小弧驱动；`bisect-step.sh`：bisect 步进脚本
- `app-death-*.log`：panic 全量 backtrace（RC canary UAF @ rc.rs:742）
- `lrun-1.log`/`mrun-1.log`：截断版中间取证；`grun-*.log`：fix 后 load 复跑
- `p706-matrix-culprit.log`：culprit 提交全矩阵 16/16（ unloaded 未触雷——竞态窗口注记）

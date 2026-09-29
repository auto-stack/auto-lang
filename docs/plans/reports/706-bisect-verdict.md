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

## 6. T-10 续：第二断裂面与死键守卫（2026-09-30 补记）

值准入守卫落后全矩阵实机复跑暴露**同根因第二面**：debug 全矩阵在 [12 rename] 后
ECONNRESET 进程死亡、另跑在 [10] `首页 not found in region`（app 存活无 panic）——
UAF 只被值守卫断掉，**死键版本冻结伪命中**仍在：`path_versions` 条目随状态替换留存，
堆对象已从 `heap_objects` 摘除 → dep 键版本永不 bump → `deps_unchanged` 恒等 →
陈旧标量/陈旧视图回流（弹层滞留/树行缺失/条件错乱）。

修复= `AutoVM::heap_dep_key_alive`（engine.rs）+ `deps_unchanged` 死键按 miss：
- `< HEAP_ID_BASE` 合成键与未分配 id（id ≥ id_gen）按存活——纯版本比对语义保持
  （plan047 门测试依赖）；
- 已分配（≥ base）且不在 `heap_objects` = 已释放（id_gen 单调不复用；dying 宽限窗
  内对象仍在表）——零新增状态。
- 咽喉单点：信号网 hit 与 memo version_fast 同经 `deps_unchanged`。

家族单测 `plan706_deps_unchanged_dead_heap_key_misses`（死键 miss/活键恒等/合成键
存活三态）。定向回归：plan706 3、plan047 24、plan046 38、memo 86、vm_bridge 55 全绿。

## 7. T-11 收口证据（2026-09-30）

| 面 | 载体 | 结果 |
|---|---|---|
| 消费者 vm_matrix merged 16/16 | 修复 exe（debug） | **ALL GREEN**（`p706-matrix-fixed2-debug.log`） |
| 消费者 vm_matrix merged 16/16 | 修复 exe（release） | **ALL GREEN**（`p706-matrix-fixed2-release.log`）——[10b] 弹层滞留面消 |
| 家族驱动 [7 tab] 弧 ×5 CPU load | 修复 exe | 5/5 GREEN（双守卫后） |
| 家族 ui:: 域 1617 测 | 修复 exe | 1616/1617（1 红 `projector_counter_layout_and_hits` 为 **base 预存**——old-carrier c8f86ef92 同红复证） |
| gallery 实机抽查四页 | 修复 exe | **GREEN**——Sidebar/Menubar/Area Chart/Command 四路由真达（__current_route 断言）+ 渲染非空（74821/55750/36659/35335 字节异构内容；`gallery-spot.log`） |

### 7.1 probe_mtime release 挂起面：独立缺陷定谳（AC-11 后半臂）

- 修复 exe debug：probe_mtime **全案通过**（merged+split 五案，`p706-probe-mtime-dbg.log`）。
- 修复 exe release：`done` 恒 false（`p706-probe-mtime-rel.log`）。
- **old-carrier c8f86ef92 release：同形红**（`p706-probe-mtime-oldrel.log`）——pre-delta
  载体同挂 ⇒ **非 706 r2 回归，系独立预存缺陷**（merged 直调臂 back-bridge call 挂起，
  release 特异）。按 AC-11 预设臂「或定谳为独立缺陷另立记账」处置 → KNOWN-DEBT。

### 7.2 消费者 e2e（vue 轨 Playwright）：预存 flaky 定谳

- 载体 delta 内 e2e `快速打开` menubar 弹层点击偶发不稳（元素反复 detach → click 重试
  到超时）。样本统计：old-carrier release **2/4 pass**（run1 绿/连跑 1 绿 2 红）、
  new 载体 **1/4 pass**（debug/newrel/77d05aa3f-run1 红、77d05aa3f-run2 绿）——**双载体
  同 flaky，无载体区分度 ⇒ 预存 harness 稳定性问题，非 706 r2 回归**。
- 排除记录：gen/front/vue 为 09-22 陈旧产物（所有 e2e 跑同一前端字节）⇒ 前端非变量；
  vue 轨 bisect（c8f86ef92..66c9cac19，判据=全 spec）单样本噪声不可靠已弃用（
  77d05aa3f「first bad」为 1/2 样本侥幸，行为等价 diff+前端零再生成排除因果）。
- 处置 → KNOWN-DEBT（消费方 harness 加固面：force-click/动画等待），不在本 phase 修复。

### 7.3 gallery 侧栏连按失灵：预存 quirk 注记

- gallery VM 臂 sidebar 连续导航在 **双载体均偶发失灵**（old-carrier 2 连按后路由滞留、
  new 载体亦然；MCP press 间页面重挂载期 >10s 超时同现）——非 r2 回归。四页收口经
  press→fixture 双通道重试达成真路由。

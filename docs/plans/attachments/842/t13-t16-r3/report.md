# PLAN-842 r3 四项历史基面缺口修复验证报告（T-13..16）

冻结代码：lang `1e776d22e100fdaf5d6e37b0e9f872e49f853666`（未动）、
UI `bd228b0`（= e4aa3da + 1c4d864 + bd228b0，本报告新提交）、
Man `4cc912e`（= 010ef7e + 4cc912e，本报告新提交）、
down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（未动）。

## T-13 桌面双进程 smoke（AC-09 / V-07）

- **finding → invariant**：点击点硬编码 `(60,40)` 落在旧布局位置，首帧布局
  变更后按钮 quad 为 `(180,127.2,120,36)`，点击落空、帧 count 断言红 +
  nextest 报 child leak。不变量 = 冒烟必须点在**真实首帧**的 `+` 按钮上。
- **fix**：`crates/auto/tests/autodesk_client_smoke.rs`——首帧 DrawList 中
  定位 `+` 标签 Text，取包含该标签的最上层 Quad/QuadR，按 quad 中心 +
  真实窗口原点（`session.host.wm.wins[wid].rect`）计算宿主坐标点击。原
  双点击、帧递增、L2Detach、子进程 `exit=L2Detached`、宿主回收断言逐字
  未动；零产品行为改动。
- **evidence**：`t13-man-smoke.log`（最终提交态，1/1 pass，两次
  `[CALL_HANDLER]` 命中，0.36s）。与候选探针的 (256,161.2) 点击点一致。
- **boundary**：仅覆盖该冒烟协议形态（462 会话 + loopback）。

## T-14 Vue scaffold CSS golden（AC-10 / V-08）

- **finding → invariant**：PLAN-038 起生成器亮/暗各追加 4 个 status token，
  旧 55 对全等断言红。不变量 = 旧 55 对逐字保持 + 新 8 值精确锁定，缺项/
  漂移/意外额外/重复发射均不得通过。
- **fix**：`vue.rs` `index_css_values_match_p1_baseline` 重写——(1) 每键断言
  恰好 2 次且一亮一暗（以暗面 `background 222.2 47% 7%` 为界）；(2) 值逐
  字断言 success `142 71% 45%` / warning `45 93% 47%` / info `217 91% 60%`
  / error `0 84% 60%`（与 design_tokens registry 扩展色 rgb→hsl 同源）；
  (3) 移除 8 个已知对后与金样 55 对全等（意外额外键/缺项/重复即失配）。
  金样 fixture 文件未改写。
- **evidence**：`t14-css-golden.log`（1/1）；变异检查——期望值
  `142 71% 45%`→`142 71% 46%` 红、恢复绿（oracle 区分力）。
- **boundary**：status token 双面同值由 registry `scaffold_extended_tokens_dual_face`
  既有测试锚定，本测试锁 vue 生成面投影。

## T-15 merged API CRUD/route A/声明类型（AC-11 / V-09）

**有界决策**（依据 PLAN-648 T-03 裁定 → PLAN-681 route A → PLAN-710 D-7
普查裁定及 p710-census §10 边界注记 → 039 声明类型臂的时间线）：

1. 臂优先级维持现行：①db.at 吸收 ②route A（无 primary pub type、无
   db.at、转译成功；`[]str/[]int` 属标量面）③声明类型臂（返回为 back
   pub type）④viewer 媒体面 ⑤CRUD 脚手架。真体优先成立。
2. 无 db 标量契约模块（原测试 fixture 形态）由 route A 委派内嵌 api_impl
   真体；PLAN-648 的 `Vec<Value>` 全量列表签名对该形态**不再适用**。
3. CRUD 兜底适用条件 = db 在案但不覆盖该端点，或结构契约下返回非声明
   类型的端点。兜底内签名/返回按契约收口（D-7）：`[]str → Vec<String>`；
   GET 缺省补 `Vec<…> → Vec::new()`（修前落 `_` 分支发 `Value::Null`，
   与签名失配不可编译——真实潜在缺陷）。API_DATA 脚手架与 POST id 分配
   插入（PLAN-648 结构性保留）不变。
4. route A 转译失败不再静默：门控三分（Inapplicable / Active /
   TranspileFailed），失败在生成物留 `⚠ route A inactive: …` 标记后照常
   兜底（对齐 back 侧「transpile failure falls back to skeleton + ⚠ 告警」
   既有语义；不引入 back 侧也没有的硬失败）。设计门（primary type/db）
   不触发标记。

**fix**：`rust_ui.rs`——`MergedRouteAOutcome` 三分 + 生成端标记 +
`Vec<…>` 缺省；测试 4 项：absorption（原）、crud_fallback（fixture 改带
不覆盖端点的 db.at，断言 `Vec<String>` 契约签名 + 退役形态不再出现 +
route A 不适用且无标记）、route_a_wins（真体委派逐串断言）、
transpile_failure_leaves_marker（坏伴生 → 标记在案 + 兜底仍生成）。

**真实编译运行**（`t15-route-a/`、`t15-crud-fallback/`）：生成客户端码
（快照在案）组装独立 crate `cargo run --offline`——
- route A：`list=["alpha", "beta"]`、`create="created:"` = **真实业务体
  执行**（脚手架不可能给出：GET 空态/POST id 记录），exit 0。
- CRUD 兜底：GET 类型化空态；POST 返回 `{"id":102,"name":"x"}` 且 id
  递增（102→103）= API_NEXT_ID 分配插入实际观测，exit 0。
- 类型反例 = 编译期差异：`Vec<Value>`/`Value::Null` 旧形态在
  `Vec<String>` 调用点编不过（两个 crate 的 `let v: Vec<String> = …`
  harness 即持续反例）。
- 快照由 fmt 前构建导出；rust_ui.rs 的 fmt 改动仅白空且发射串为字面量
  不受影响，最终提交态形态由 `t15-merged-tests.log` 4/4 锁定。

## T-16 Vue 增量解析失败语义（AC-12 / V-10）

**有界决策**（依据 Plan 012 Batch B 统一语义 ← Plan 841 前置整树校验
89d0f44 的冲突裁断）：

- **受信准入失败**（错误串含 `UI domain validation failed` /
  `UI module validation failed`，如重复 widget/store/model 字段/message
  变体、坏模块导入）：前置整树校验在任何模式硬拒绝、cache/输出前——
  Plan 841 合同保持，整树校验**保留不删**。
- **语法解析失败**：普通模式交还逐文件统一语义（警告 + 跳过坏 store、
  正常 store 照常输出；严格模式硬失败），fresh（from_workspace）与增量
  （incremental_compile_changed）一致——两路径共用 validate_ui_tree。
- rust 后端路径的 `validate_ui_files` 行为不变（841 原样，未被本决策触及）。

**fix**：`util.rs` `validate_ui_tree`——逐文件错误分类，仅受信准入失败
或严格模式返回硬错误（格式 `Failed to compile <path>: …` 与逐文件处理
一致，保留 841 断言子串）；普通模式语法失败不在预扫拒绝（由逐文件
handle_compile_error 报警跳过）。`vue.rs` 测试扩展：fresh 非严格一致断
言、普通模式 domain 反例硬拒绝 + 零输出、严格模式 fresh/增量双拒绝。

**evidence**：`t16-parse-semantics.log`（1/1，含全部扩展断言）、
`t16-domain-integration.log`（Plan 841 集成 6/6——domain 族四后端硬拒
绝、模块导入、recipe、增量前置校验合同全部保持绿）。

## auto-ui-tools lib 套件对照（stash 归因）

最终提交态全套：191 pass / 5 fail（另有并行负载下偶发 env 族 flake）。
对 6 个红名用 `git stash` 在未修改基线（e4aa3da）复跑：
`ensure_workspace_lands_project_local`、`rust_workspace_resolution_order`
（并行 flake，隔离绿）、`review738_r5_lock_binding_state_machine`、
`review751_empty_lock_identity_matrix`、
`desktop_extra_app_roots_default_includes_galleries`、
`test_plan609_unresolved_dep_import_guard`（隔离绿）——4 个在基线同样红
（预存环境/布局依赖红），2 个为并行 env 竞争 flake（基线与本修复态隔离
均绿）。与本轮修复无关；merge 阶段 UI 全档按既有归因流程处理。

## merge-owned pending

裸 `cargo t`、tv/tt/tu 触面档、到期批量回归、跨仓 Man/UI 档、实际合入、
规范/ledger 沉淀与归档均未执行（work 阶段不跑整档）。源码/安装证据重绑
（§5.4：T-13..16 完成后按 T-03/07..10 既有关闭条件重新验证）待独立复审
通过后由 merge 统一执行。

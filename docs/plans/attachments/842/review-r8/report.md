# PLAN-842 R8 独立最终复审报告（final review）

- 日期：2026-10-27
- 审查主体：独立子代理 R8（本会话派发，与实施异上下文；不信任实施者总结，一切以代码、日志与可复跑命令为准）
- 计划：PLAN-842 r3（plan_revision 3），status execution_done 14/17，本次为 T-10 独立最终复审
- 冻结提交：lang `6f71646aca2b5d6c162ccdcd6c534f49761bc0ef`、UI `bd228b03bc50ebf73a1d3ce2d7b55193721f80ac`、Man `4cc912e9ffd96a2a3bef5470ecfec128bdd80601`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`
- **结论：pass（AC-09..12 全部 pass；D-842-G1 预检合规；SD-07/08 与实现一致；r11 重绑证据核对成立、复用论证裁定成立；零新 finding）**。merge-owned 门禁 pending（见文末）。

## 0. 复审基线核验

- 三树 HEAD/分支与任务给定完全一致（lang/UI/Man 均 `codex/plan-842-dev`，down detached `c39ac7f9`），`git status --porcelain` 三树全空（clean）。
- r3 提交范围核验：
  - UI `e4aa3da..bd228b0` = 两提交（`1c4d864` T-15、`bd228b0` T-16+T-14），仅 `rust_ui.rs`/`vue.rs`/`util.rs`（+296/−21）。
  - Man `010ef7e..4cc912e` = 一提交（T-13），仅 `crates/auto/tests/autodesk_client_smoke.rs`（+33/−1，纯测试文件）。
  - lang `1e776d22e..6f71646ac` = 一提交（SD-07/08），仅 `docs/specs/auto-man/project.md`、`docs/specs/auto-ui-compiler/project.md`（+17）。
- 历史 R7 pass 只覆盖旧提交（lang a8b4a5b/UI bd0e0f3/Man 9ac33ad），不覆盖 r3 新提交与 e4aa3da/010ef7e 预检诊断修复——本复审已全部覆盖。

## 1. D-842-G1 预检诊断修复（UI e4aa3da）—— 合规

- diff 核验（`git show e4aa3daf`，仅 +2/−1）：`delivery_preflight_producer` 中 `validate_development_dependencies` 的错误经 `.map_err(|err| format!("delivery: producer 预检失败: {err}"))` 包装。
  - 前缀恢复：本计划新增的依赖图预检原先透传 producer 错误、丢失既有「预检失败」前缀（旧测试断言 `contains("预检失败")` 失配——§9 2026-10-27 merge 记录在案）；修复后与同函数既有的 `delivery_crate_source` 包装（90eb7347 引入）前缀一致。
  - 错误内容保留（`{err}` 全文）；拒绝顺序未变（`?` 提前返回；受管 SDK `is_managed_sdk` 跳过逻辑未变；预检调用点位于触碰旧交付/staging 之前，rust_ui.rs:5404 及注释）。
- 重跑：`cargo test --locked -p auto-ui-tools --lib -- delivery_redelivery_failure_preserves_old delivery_preflight` → `delivery_redelivery_failure_preserves_old` **1/1 pass**（`delivery_preflight` 为函数名非测试名，按复审指示仅前者入选）。

## 2. AC-09 / V-07 / T-13（Man 真实双进程冒烟）—— pass

- diff 核验（`git show 4cc912e`，单 hunk +33/−1）：唯一行为变更是 `pointer_down(60.0, 40.0, …)` → `pointer_down(click.0, click.1, …)`；新增首帧定位逻辑，无断言删改。
- 断言结构逐字核对（当前文件全文 vs diff）：双点击循环 `for expect in 1..=2i64`、`assert!(seen, "帧 count 递增到 {expect}")`（帧 count 1/2）、`l2_detach()`、`stdout.contains("exit=L2Detached revision=3")`、`assert!(ph.session.apps.is_empty(), "L2Detached 后宿主回收")`、`assert!(ph.session.apps.contains_key(&AppId(1)), "462 AppSession 落地")`——全部逐字保留。
- 定位逻辑正确性：
  - `+` 标签（`DrawOp::Text { text: "+" }`）→ 包含该点的 quad（`Quad|QuadR` rect 包含判定，`next_back()` 取绘制序最后一个=最上层）→ quad 中心 + 窗口原点 `win_rect.x/y`。
  - `pointer_down` 宿主坐标语义经实现核实（UI 仓 host.rs:417）：入参为宿主坐标，内部 `wm.hit_test` 命中后 `x - rect.x / y - rect.y` 转 widget 本地——测试换算方向与实现精确一致；DrawList quad 坐标即窗内坐标，加原点回宿主坐标正确。
- 重跑：`cargo test --locked -p auto --features ui-extra --test autodesk_client_smoke autodesk_client_two_process_smoke -- --nocapture` → **1/1 pass**（日志可见两次 `[CALL_HANDLER] … __evt_onclick_1` 真实点击处理器调用 + `[host-dbg] server_end EOF` 子进程退出——双点击与退出语义真实可观察）。

## 3. AC-10 / V-08 / T-14（index.css golden）—— pass

- 测试核对（vue.rs:13185-13239 `plan593_index_css_golden_tests::index_css_values_match_p1_baseline`）：
  - (a) 旧 55 对金样全等保留：`stripped`（pairs 移除 4 status 键的全部命中后）与 `var_pairs(golden)` `assert_eq` 全等；金样 fixture 未被改写（`git diff e4aa3da..bd228b0 -- crates/auto-ui-tools/tests/fixtures/plan593_index_css.golden` 为空）。
  - (b) 8 新值精确：success `142 71% 45%`、warning `45 93% 47%`、info `217 91% 60%`、error `0 84% 60%` 逐字核对一致；亮/暗各一（`assert_eq!(hits.len(), 2)`——缺项 <2 或重复 >2 均红）；面内位置以暗面 `background == "222.2 47% 7%"` 界定（`hits[0] < dark_start`、`hits[1] > dark_start`）；意外额外键经 stripped 与金样全等被拒。
- 受控变异检查（R8 亲证）：期望 `142 71% 45%` → `142 71% 46%` 跑 **红**（FAILED，oracle 区分力成立）；恢复后跑 **绿**（1/1 pass）；`git status --porcelain` 确认 clean、无残留。
- 重跑：`cargo test --locked -p auto-ui-tools --lib -- index_css_values_match_p1_baseline` → **1/1 pass**。

## 4. AC-11 / V-09 / T-15（merged 客户端 route A/CRUD）—— pass

- diff 核验（`git show 1c4d864`，rust_ui.rs +195/−18）与当前代码逐项比对：
  - **臂优先级未变**：当前 `generate_merged_api_client` 端点臂序 = ①db 吸收（rust_ui.rs:1850）→ ②route A（:1862，条件从 `route_a.is_some()` 改为 `route_a_active.is_some()`——仅收紧为 Active 才进，顺序未动）→ ③声明类型臂（:1907）→ ④viewer 媒体面（:1946）→ ⑤CRUD 兜底（:1993）。
  - **route A 门控三分**：`MergedRouteAOutcome::{Inapplicable, Active, TranspileFailed}`；primary pub type / db.at 在案 = Inapplicable（原 `return None` 语义）。
  - **⚠ 标记仅 TranspileFailed 发**：生成分支为 `if let Active … else if let TranspileFailed(reason)`（:1817-1821）——Inapplicable 不进任何分支、零标记；测试双向锁定（`merged_api_client_route_a_transpile_failure_leaves_marker` 断言有标记+无半套真体+契约化兜底；`merged_api_client_crud_fallback_for_uncovered_endpoints` 断言 db 在案无标记）。
  - **GET 列表 `Vec<…>→Vec::new()`**：default_expr 补 `_ if ret_ty.starts_with("Vec<") => "Vec::new()"`（:2027），修复 `Value::Null` 占位与签名类型失配不可编译缺陷；POST id 分配插入保留。
  - **旧测试断言未弱化**：`merged_api_client_crud_fallback_for_uncovered_endpoints` 断言从 `Vec<Value>` 改为 `Vec<String>` 属契约收口（增强非弱化：另加 `!contains("fn list_items() -> Vec<Value>")`、Null 占位否定、`!contains("pub mod api_impl")`、`!contains("⚠ route A inactive")` 四重否定断言）；fixture 加 db.at（只含 helper `seed_note`、不覆盖端点）有依据——db 在案使 route A 设计性不适用，测试面才能落在 CRUD 兜底本身；PLAN-648 旧 `Vec<Value>` 形态按 PLAN-710 D-7 退役，依据已在测试注释记录。
- 重跑：`cargo test --locked -p auto-ui-tools --lib -- merged_api_client` → **4/4 pass**（route_a_wins_for_scalar_contract、route_a_transpile_failure_leaves_marker、db_absorption_wins_over_crud、crud_fallback_for_uncovered_endpoints）。
- 真实编译运行证据抽查（attachments/842/t13-t16-r3/）：
  - `t15-route-a/real-run.log`：`route_a OK: list=["alpha", "beta"] create="created:"`——route A 真体实际观测；`generated-client.rs` 快照与日志一致（`api_impl::list_items` 返回 `vec!["alpha","beta"]`，端点委派真体）；harness 断言真体与脚手架的区分性（非 id 记录序列化）。
  - `t15-crud-fallback/real-run.log`：`crud_fallback OK: id1=102 id2=103 r1={"id":102,"name":"x"}`——兜底 id 分配 102→103 与快照一致（API_NEXT_ID 起 100，GET 臂请求记录先占 101）；`Vec::new()` 空态在案。

## 5. AC-12 / V-10 / T-16（解析失败与受信准入双轨）—— pass

- diff 核验（`git show bd228b0` util.rs +15/−1、vue.rs 测试段）：
  - `validate_ui_tree`：失败时仅 `domain_admission_failed(&e) || strict_enabled()` 才硬拒（错误格式 `Failed to compile {path}: {e}` 保留）；普通模式语法失败不再在预扫拒——交还逐文件统一语义（Plan 012 Batch B）；**整树校验保留未删**（遍历仍在，domain/module 反例仍在此拦截）。
  - `domain_admission_failed` 为既有函数（util.rs:239，判定串 `UI domain validation failed` / `UI module validation failed`，与 841 合同错误串一致）；新逻辑与既有 `handle_compile_error`（vue.rs:7435，r3 未改）语义结构完全同构——预扫不再比逐文件更严格，也不吞准入失败。
  - 测试扩展断言齐全：fresh 一致（非严格 fresh load/generate 继续、健康 store 输出、坏 store 不输出）、非严格 domain 反例硬拒（重复 widget 声明 → `UI domain validation failed` 具名 + `gen/front/vue` 零输出）、严格双路径（incremental 既有 + fresh 新增 `Failed to compile` 断言）。
- 重跑：
  - `cargo test --locked -p auto-ui-tools --lib -- test_incremental_parse_failure_semantics` → **1/1 pass**。
  - `cargo test --locked -p auto-ui-tools --test ui_domain_validation` → **6/6 pass**（Plan 841 受信准入合同保绿：incremental_entries_validate_before_cache_or_output 等 6 项——受信准入未削弱）。

## 6. SD-07/08 规范增量核对（lang 6f71646ac）—— 一致

- SD-07（auto-man/project.md 新节「merged API 客户端臂语义」）：臂优先级①..⑤、`[]str/[]int` 属标量面（经 route A 测试证实 `[]str` 走 route A 委派）、GET/POST 契约收口、`Vec::new()` 缺省、POST id 分配保留、`Vec<Value>` 退役、⚠ 标记 + 设计门不触发——与第 4 节核验的实现行为逐项一致。
  - 「对齐 back 侧转译失败回落 skeleton + 告警的既有语义」经源证：lang 仓 `crates/auto-api-gen/src/api_gen.rs:1220` 确有既有 `⚠ api_impl.rs transpile failed (route A inactive)` 告警 + :1202 「transpile failure falls back to the skeleton unchanged」。
  - 与既有文本无冲突：auto-man/project.md:297 的「转译失败由装配预检拒绝」描述的是 api_gen **server 生成器**装配预检环节，SD-07 新节明确限定「merged 进程内 API 客户端（auto-ui-tools generate_merged_api_client）」——不同组件的行为描述，边界清晰。
- SD-08（auto-man/project.md 新节 + auto-ui-compiler/project.md 增补）：受信准入失败=domain/module 校验（错误串与 `domain_admission_failed` 判定串逐字一致）、任何模式、先于缓存/输出；语法解析失败交还逐文件 warn-and-skip（非严格）/硬失败（严格）、fresh/incremental 一致；「Rust 后端路径的前置文件校验行为不变」——r3 diff 确未触 Rust 后端前置校验（UI r3 hunk 全部在 merged client/测试/validate_ui_tree）。
  - 与 841 段（「准入错误不得被 warn-and-continue 吞成成功或半壳工程」）互补不冲突：SD-08 划清「准入失败硬拒」与「语法失败宽容」双轨且「双轨不得互相吞并」。
  - 两节均为现行行为规范描述，无执行日记内容。

## 7. r11 重绑证据核对与复用论证裁定 —— 成立

- 日志齐全（attachments/842/t13-t16-r3/r11-rebinding/ 15 份）：
  - **Full**：`r11-full-verify.log` = 6813 文件，sources= lang `6f71646ac` / UI `bd228b0` / Man `4cc912e` / down `c39ac7f9`（三仓 r3 提交绑定正确）。清单 SHA-256 **自行重算** `powershell (Get-FileHash …r11-final-full\install-manifest.json -Algorithm SHA256)` = `955FA96E3BF0349953FE6258D9E128901387CECC5176CE8651F095F3C293F0C8`，与 §9 记录**逐字一致**。
  - **Rust 交付**：fresh lock（cargo 解析完成）+ `cargo build --locked` Finished 1m16s + 资源校验 `[verify-delivery] PASS 1 资源校验一致（SHA-256=清单记录）` + 交付生成 5434 文件。
  - **Vue**：生成（17 文件）+ npm install（129 包）+ vite 生产构建（dist/index.css 10.62 kB）；生成 `src/assets/index.css` 中 `--(success|warning|info|error):` 恰 8 行且八值与 T-14 锁定值**逐字一致**（`--success` 亮/暗各 1 次）。
  - **Core**：`r11-core-verify.log` = 636 文件（lang+Man）；SDK locked check Finished；独立消费者 **3/3 pass**；安装 exe 实跑 `plan842-core-ok`（r11-core-vm.log）。
- 复用论证裁定（失败恢复/staging 保护按输入身份复用）：
  - Man `4cc912e` 仅测试文件（diff --stat 单文件）→ 安装器/交付生产码未变 ✓。
  - UI r3 不触交付保护码：`git diff e4aa3da..bd228b0 --stat` 仅 rust_ui.rs/vue.rs/util.rs；hunk 位置逐一核验——1c4d864 全部在 merged client（1030-2050）与测试（7265-7488），bd228b0 在 vue 测试（12334-12414、13196）与 util validate_ui_tree（216）——交付段（delivery_preflight/staging/swap，rust_ui.rs 4600-5400）**零改动** ✓。
  - 耐久反例 `delivery_redelivery_failure_preserves_old` 在当前代码重跑 1/1 绿（R8 亲证，见第 1 节）。
  - **裁定：复用论证成立**——重做的面（Full/Core 安装、Rust 交付、Vue、Core 消费）已由 r11 日志覆盖，复用的面（失败恢复）所依赖的生产码在 r3 未变。

## 8. 健康面

- `cargo check -p auto-ui-tools`：22 个 lib warning；逐一映射位置——rust_ui.rs:796/1439/1838/2132/3655/3666、vue.rs:1415/2559/7380/8048/10126/10190/12417/12423/12432/12942/12958/12977/12995、image_viewer.rs:149-151、util.rs 零——**无一落在 r3 新增 hunk 内**（1439/1838 为 server 生成器段与 merged 循环既有行；vue 12417+ 为 PLAN443 旧常量区 doc-comment warning，行号系 r3 插入推移）。判定：全部预存，无 r3 引入的新 warning。
- debug print/TODO 扫描：UI r3 diff（e4aa3da..bd228b0）与 Man r3 diff（010ef7e..4cc912e）新增行中 `println!/dbg!/eprintln!/TODO/FIXME` 均为 **0**；T-16 涉及的 `handle_compile_error` println 为既有函数语义（r3 未改该函数）。
- 变异检查后 `git status --porcelain` 空；复审全程未改任何仓代码。

## 9. 已知边界核销（4 项 lib 预存红 + 2 项并行 flake）

- 实施者记录（t13-t16-r3/report.md「stash 归因」节）：最终提交态全套 191 pass/5 fail；6 红名在基线（e4aa3da）stash 对照复跑——4 个基线同样红（预存环境/布局依赖红）、2 个并行 env 竞争 flake（隔离均绿）。
- R8 复核（当前树 clean 无可 stash，按指示做隔离定向抽查）：`review738_r5_lock_binding_state_machine`、`desktop_extra_app_roots_default_includes_galleries` 隔离定向跑 **2/2 pass（绿）**——与「环境/布局依赖、全套并行负载下偶发红」的归因特征吻合，非确定性代码缺陷；实施者 stash 基线对照记录在案。
- 裁定：归因**可接受**——这些红不属 AC-09..12 面（本复审已将 AC-09..12 面全部定向重验为绿），merge 阶段 UI 全档按既有归因流程处理。

## 10. Findings

无（F-842-R8-00：零新 finding）。

## 11. 限制与未覆盖面

- 本复审为定向核验（final 范围），**未运行**整档 `cargo t/tv/tt/tb/tu/taa/tf/ta`、未重建安装/交付（r11 日志为实施者产物，R8 做了 SHA 重算、内容抽查与生成物一致性核对而非全量重跑）、未跑 Vue 双端可视化走查。
- r11 日志中的构建/安装产物（`.wt/lang-842/r11-*`）按内容抽查核对，未逐字节复核全部 6813/636 文件。
- MPV 硬件视频、jet/ark v0.7 延后面等 R7 已披露的限制继续有效。
- **merge-owned pending**（T-11/T-12 前不得宣称交付完成）：冻结提交上的裸 `cargo t` + 触面档（VM/编译器 tv、trans tt、book tb、UI 生成 tu；UI/Man 跨仓适用档）、到期主检出单实例批量回归（tf）、并集影响复审、实际合入/固定版本/来源读回、canonical/受管 ledger 沉淀、归档与逐树 fresh wt-guard clean 清理。已知预存红/flake 与 plan707 类既存红须在实际门中归因。

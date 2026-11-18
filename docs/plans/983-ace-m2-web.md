---
plan_id: PLAN-983
status: executing
feature_name: Web服务嵌入与实际后端迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-19
plan_revision: 24
current_step: 14
total_steps: 20
depends_on: [PLAN-967, PLAN-968, PLAN-969, PLAN-974, PLAN-975, PLAN-978, PLAN-980, PLAN-982]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-web.md]
touched_goals: [GOAL-002]
---

# [PLAN-983] Web服务嵌入与实际后端迁移

## 0. 变更摘要

2026-11-19 r23（四仓门禁修复）：在现有 `B-CONSUMERS-V` 内设修复阶段，由 983 工程线负责 `plan988_byte_consumers` 的超时和 MSVC 尾进程两项排障及正式四仓复跑。工作范围、退出判据和复审归属见 §5。原任务 ID、进度、依赖和验收口径保持，988、989 的归档状态不变。本轮只更新计划。

2026-11-18 r22（**U0 收据接收 + 四仓验收执行**；用户裁定×3）：①**批量回归跳过**（用户："回归不做了，跳过"）——r21 fold 收据的 DUE 移交撤销；既有具名红（infra-fixed ace-facade 期望行）仍归 969 线。②**U0 以收据接收**（用户："983 可以接收这份 U0 收据"）：接收 980×982 联合验收收据（980 §9 2026-11-18 verify，982 round-188 代记）——原样 002-counter 真产品链 `auto run --render vue`（`AUTO_UI_ACE_VUE=1`）→ pnpm → vite v5.4.21 → 真浏览器 `Counter: 0→+→1→-→0`（`+,+`→2、零 page error）+ 磁盘 lane `App.vue`/官方 `a2x_rt.mjs`（sha256 `97065d43…`）；证据 man `4a7a28e`（`plan982_u0_vue_run_cli` 3/3）+ lang `8e94acfaf`；按收据自身边界"983 U0 消费同一产物树/同一链"，**983 不重跑**。#9 主树复跑（新语言基线）与 980/WEB 全批验收=他人收口项（用户具名），非 983 之门。③**四仓验收（V-B-CONSUMERS）执行中**：`web-four-repository.py` 前四次起跑失败均为主检出**他线在途状态**（lang 主检出 5 个 ace-core 半截 WIP=980 记录在案的 #9 项；auto-ui 主检出 theme/stylekit 5 文件 tracked WIP + `examples/*/deps/themes` **junction**→`packages/themes` 使 git 穿透枚举恒不净）——终以**全组树**运行（lang/web/down/man 组树=各自 master tip；ui=d188d695，与 master `01f85061` 的差=980 后续合并，具名偏差）。**环境处置（全部非破坏、可恢复）**：四仓主检出内 `.wt/rc989-old/*` 停驻副本（11-12，内嵌 .git，违反"材料不入仓"裁定）迁 `plan-artifacts/orphans/`；auto-ui 的 `NUL` 保留名垃圾已删；auto-ui 锁 churn 与 980 线 tracked WIP 各入具名 stash（WIP 已 pop 还原，锁 stash 门后 pop）；一次复原脚本误搬的混合副本整体隔离 `orphans/_p983-quarantine/`（字节保全，其中 `tests/plan058_t02_probe.rs` 等可按需取回）。四仓门**第五次起跑通过全部预检/布局校验、实跑至语言固定组末尾**：45/46 过、`plan988_byte_consumers::plan988_source_bytes_compile_run_and_preserve_rejected_targets` 在门的 120s 预算下超时被杀 ⇒ 门失败（外部腿未达）。**隔离复跑×2：测试层 46/46 全过**但组判决 FAIL——`vctip.exe`（MSVC 遥测，pin 校验有效）在 workload 退出后残留触发进程树策略（"never a workload exception"，reaper 清后 remaining=0 仍判死）。**判定：blocked（既有基础设施条件，非 983 触面）**——plan988 长 Atomic（60–115s，owner=988/989 线）+vctip 尾检（门 owner 侧策略/机器条件）+主检出他线在途（#9/980 各收口项）。详证 §9 r22。

2026-11-18 r21（**C1 收口补证**：文本路由接线上的真 CLI 复跑收据 + 并发提交顺序/一致性证据）：①用户核对指出 r20 只交了源码与断言、缺**绑定当前修订的真 CLI 复跑收据**——本轮在最终修订（auto-man `7dd8d76`）上以 `--nocapture` 复跑并逐字收据入档（§9 r21）：`plan983_c1_cli` **6/6（1.83s）**、`plan983_c0_cli` **1/1（0.48s）**，含 `✓ C1 concurrency receipt` 行。②并发用例强化（确定性断言 + 观测收据）：hammer 循环内每次 GET 后**直读配置文件并断言可解析且 `workspace_root` ∈ {canonical A, B}**（原子替换不变量——任意时刻磁盘上都是一笔完整提交）；POST 响应到达序记入收据（本轮观测 A→B，终态=后到者=最后提交；到达序为启发式证据不作断言——客户端线程调度理论上可换序，提交序由单 owner 顺序派发结构保证）。③man Cargo.lock 随 lang 组树 ff 补解析（ace-host 的 a2x-c/store 依赖）。C1 就此收口；U0/B-CONSUMERS 状态不变。

2026-11-18 r20（**改用 982 文本路由引擎（`5e4322a`）**；用户指令"采用 982 刚交付的文本路由"，r19 的整请求文本设计退役）：①**引擎对齐**——auto-web 分支重钉 master `5e4322a`（982 round 183：`#[api]` fn 形参恰为单个借用 `str` ⇒ **文本路由**——请求体作为运行时文本实参进 `execute_entry_text_platform`，每路由驻留模块 `d_<fn>` 入口编译一次；0 参路由保持文件通道）；本仓叠加 `combined_entry_dispatches` 计数（只计文本路由派发）。②**业务源改形**（auto-down `72b0970`）：`fn api_workspace_open(body str) int` = 文本路由（body=`{"root":…}` 直接解析，**不经请求暂存文件**；C1 全语义保留：JSON 请求、缺目录 400、索引与配置原子提交、两具名前缀）；`main()` 恢复 0 参文件通道自路由（GET + 旧引擎兼容的 POST 回退，共享同一 `handle_open`）。③**区分性证据按用户时序**（auto-web `3d328a3`/man `11b347b`）：c0 用例**POST 先行**（文本路由）后立即断言 `p983-req.txt` 不存在（此时无任何文件通道派发）、`combined==1==av`、`p983-resp.txt` 在场；随后 GET（文件通道）后收尾断言 `combined` 停在 1、`av≥4`、请求文件在场（分道并存实证）。CLI 收据语义分道：POST 服务断言 `combined≥1`，GET-only 运行断言 `combined=0`。④复跑：`plan983_c1_cli` **6/6（1.16s，五例+并发）**、`plan983_c0_cli` **1/1**、auto-web `http_runtime` **9/9（2.60s，含 982 自带的文本路由用例）**。**U0 备案**（同 r19）：复用 980 的 002‑counter 浏览器断言、启动改真 CLI；正式验收等 **980 WEB 正式交付与 982 Vue `auto run`**（产品链承担 TS 转译与 `a2x_rt` 接线）。进度保持 executing／14/20。

2026-11-18 r19（**C1 生产接线到 982 组合文本入口 + 并发提交合同落地**；上段先修正口径：r18 的 5/5 是文件通道上的业务行为验收；**r20 注记：本轮的"整请求 HTTP 文本进 `main(request str)`"设计已被 982 的按路由文本入口取代——分支重钉，收据见 §9 r20**）：①**请求腿退役文件通道**——`AvServiceEngine::dispatch`（auto-web `0029636`）改为 `load()` + `execute_entry_text_platform(&loaded, …, &[http 文本], None, backend)`，**原始 HTTP 文本（方法行+头+体）作为借用 `str` 实参**进 AV；`p983-req.txt` 暂存写入删除（常量一并退役），非 UTF‑8 请求体具名 400；响应仍走 `p983-resp.txt`（i32 返回值带不动响应字节，具名）。②**业务源入口改形**（auto-down `a0ff337`）：`fn main(request str) int` + `stage_str_utf8`（`len`/`char_at` 是 Unicode 标量域，逐标量重编 UTF‑8 字节，与宿主字节逐位等价）；member‑delivery fixture（av-runtime-service.at）同步迁移。③**区分性证据**（auto-man `58fc2b3`）：收据新增 `combined_entry_dispatches`（引擎计数）——C0 断言 `combined_entry_dispatches=2`、C1 断言 `combined==av≥1`；auto-web c0 用例直接断言 **`p983-req.txt` 从未创建** + `p983-resp.txt` 在场 + `combined==av≥3`。④**并发提交合同（r15"并发切换串行提交"）落地验证**：结构依据=单 owner 循环顺序派发（`!Send` 引擎，dispatch 同步完成）+ tmp/MoveFileEx 原子替换；新增永久用例 `c1_concurrent_switches_stay_consistent_and_last_commit_wins`（并发 POST A/B + 穿插 GET hammer：每次 GET 字节等于 canonical A/B 之一；终态 config=两笔提交之一；重启 GET=最后成功提交）。⑤复跑：`plan983_c1_cli` **6/6（0.83s）**、`plan983_c0_cli` **1/1**、auto-web `http_runtime` **8/8（2.93s）**。lang 组树 ff 至 master `0a167f611`（含 982 组合入口 `600d31527`）。**U0 备案（并行跟踪）**：复用 980 的 002‑counter 浏览器断言、启动改真 CLI；最终收据仍等 **980 WEB 正式交付与 982 Vue `auto run`**（须确认产品链承担 TS 转译与 `a2x_rt` 接线）。进度保持 executing／14/20。

2026-11-18 r18（**C1 业务行为在 AV 文件通道验证**）：①AV 源（auto-down `a634c09`）实现本轮预置用例覆盖的 r15 业务行为——**canonical `\\?\` 规范化**（`canonical_w16`）、**完整 Unicode 路径**（UTF‑8↔UTF‑16 全量转换含代理对，去掉 ASCII 门）、**目标索引重建**（递归枚举 `wiki/**/*.ad`，pages 行 `{path,title,frontmatter,mtime}`，同目录 `.tmp`+`file_replace_w16` 原子替换）、**原子配置提交**（`<config>.tmp`+replace；失败 ⇒ 500 `workspace.config-commit-failed:`）、索引失败 ⇒ 500 `workspace.index-rebuild-failed:` 保旧工作区/配置/旧索引。②man 永久回归（`259ef8b`）**去 ignore 5/5 绿（0.86s）**：索引失败/Unicode/canonical/config 提交失败四预置用例转正（用例1期望同步 canonical 基线）；`plan983_c0_cli` 1/1、auto-web `http_runtime` 8/8 保持绿。③**具名边界**：pages.mtime 恒 0 与 blocks/links/tags 恒 `[]`（AV 模块至多 16 平台操作的 verify 契约已满；链接图属后续端点切片）；`dir_close` 依赖宿主 ledger 每运行强制关句柄；并发串行提交（r15）未实现未验；U0 仍待 980 WEB 交付与 982 Vue auto run。④过程：组内 auto-ui sibling 由 PLAN‑058 停驻的 `c0035034` 前移至 master 修复提交 `d188d695`（ace‑bridge cfg 门；lock churn 副本存 `plan-artifacts/plan-983/auto-ui-lock-churn-c0035034.patch`）。

2026-11-18 r17（C1 回归补强 + 预置用例）：①r15 索引失败回归**先断言旧索引确实生成且含 `hello.ad`**（`pages[].path == "hello.ad"`），再注入目标索引失败；对 interim 源现在停在“旧索引未生成”，证据更精确。②新增三个**预置**（`#[ignore]`，目标行为）用例：**Unicode 工作区路径**（`工作区-测试` 的 POST/GET/重启同路径）、**Windows canonical 形态**（`\\?\` 前缀 + 反斜杠与 legacy 对齐）、**配置提交失败**（配置只读 ⇒ 500 且 error 以建议前缀 `workspace.config-commit-failed:` 开头，旧工作区/config 不变）。③`--ignored` 实跑 4/4 红（精确失败点：旧索引缺、canonical 形态不符、Unicode 400、config 前缀不符）；活跃用例 `POST→GET→重启` **1/1 绿**（0.81s）。C1 完整语义仍待 982 组合入口。

2026-11-18 r16（C1 首个可执行切片 + r15 回归固定）：①**配置单一来源落地**（auto-down `5b213b5`）：GET/POST 接 `JADE_GARDEN_CONFIG`（持久值优先、env 初始回退）；POST `/api/workspace/open` 在 AV 源内实现（400 不存在=legacy 同形且零副作用、wiki 创建、config 持久化、200 root/wiki_dir、非 ASCII 具名 400、无配置路径 500）。②**永久验收**（auto-man `36f6df6`/`f746c74`）：`POST→GET→重启→GET` 共享同一持久配置 **1/1 绿**（0.42s；env A 初始 → POST B → GET B → 重启 GET B）；r15 索引失败回归已写（非空 `jade-garden-index.json` 目录夹具，期望 500 `workspace.index-rebuild-failed:` + 旧工作区/配置/旧索引不变），当前 interim 源跑为**红**（POST 仍 200，无索引/原子提交）——`#[ignore]` 固定，`--ignored` 可复现，待 982 组合入口实现 C1 完整语义。③auto-web C0 用例随源更新（2 路由 + `platform.file.path`），`http_runtime` 8/8。**未做**：索引生成/原子提交/并发串行（r15 合同）、U0、四仓整体。

2026-11-18 r15（C1 失败语义定稿）：目标工作区完成 wiki 准备与索引重建后，才原子替换配置并发布当前工作区。旧后端真 HTTP 已复现索引失败返回 500 但配置与 GET 已切换。§5 冻结永久回归的输入与判据，C1 实现和新版复跑待 982 组合入口。

2026-11-18 r14（C0‑B 正式跑 + C1 基线冻结）：①**C0‑B 永久用例正式通过**——`01b-workspace-av-consumption.spec.ts`（含 `b830e07`）在 AutoDown 主检出的 Playwright/Vite 上（临时 overlay，不复制 node_modules、不在 worktree 建 symlink）对 **982 CLI 启动的 AV 后端**跑：`1 passed`（6.2s；首跑 11.5s），页面真实请求 `GET http://127.0.0.1:13100/api/workspace` → **200** + `{"root":"<fixture>","wiki_dir":"<fixture>\\wiki"}`，主视图断言（`#app` 挂载、主区提示、opener=0）全过；后端收据 `served=2 av_dispatches=2 legacy_dispatches=0 same_sdk=true`；退出码 0。②**C1 验收基线由现有 `open_workspace_impl` 实测冻结**（临时起 jade-garden 原后端）：不存在目录 ⇒ `400` + `{"error":"Directory does not exist: <原样路径>"}` 且**零副作用**；成功 ⇒ `200` + **canonical（`\\?\` 前缀、反斜杠）** 的 `root`/`wiki_dir`；副作用=`<root>/wiki` 创建（空 wiki ⇒ 索引 `pages: []`）、`<root>/jade-garden-index.json` 重写（pages/blocks 含 `*.ad`）、配置 `{"workspace_root":"<canonical>"}`；二次 open 幂等。基线含"配置先写、wiki/index 后建"的部分副作用顺序（index 失败可能 500 但配置已切换）——记为债务候选，交 982 实现组合入口时一并裁定。B‑CONSUMERS 仍未勾。

2026-11-18 r13（982 交接三件落主）：按 982 的 `--stack web` 交接（man `996694a`/`8729d0d64`），auto-web 的公开面已补齐并 **phase fold 到 auto-web master `5c484d9`**：a) `AvServiceEngine::new_with_grants(source, scratch_root, grants)`（含 `platform.env`，`56b3ac5`）；b) `AvServiceEngine::receipt() -> Arc<AvServiceReceipt>`（`av_dispatches()`/`legacy_dispatches()`，`228d775`）；c) 公开 `TransportConfig::from_service_config(&HttpServiceConfig)` 与 `from_service_config_with_policy(Arc<HttpServiceConfig>)`（`5c484d9`，含映射单测）。ready 用 `http_server::bound_addr_snapshot()`（serve_av 真实绑定；进程内单服务）——man 侧 `--server av` 已据此打印 `AUTO_SERVICE_READY`。**集成注意**：用 `*_with_policy` 时须 `ServiceRuntime::bind_shutdown(tx)`，否则 `/__auto/shutdown` 的 `request_shutdown()` 返回 false。man 的 C0 CLI 切片（`6e21bfa`，分支）已改用该公开映射。B-CONSUMERS 仍未勾；U0/C1 依赖不变。

2026-11-18 r12（C0 CLI + C0‑B 准备）：①**982/EXEC 的 C0 接线已交**（auto-man `6e21bfa`，分支 plan-983-dev）：`auto service --server av` 改为 **auto-web AV 服务宿主**（`serve_av`+`AvServiceEngine` 文件通道，`--api` 显式 AV 源），pre-C0 的 man 自带 TCP 宿主保留为显式 `--server av-direct`（17 例旧用例仍绿）；`ace-candidate` 编译红已解除（lang `ff55579c3`，`cargo check -p auto --no-default-features --features ace-candidate` 绿）。真 CLI 实测：`auto service --server av --api <jade-garden workspace_service.at>` → 真实 bind（port=0）→ `GET /api/workspace` 字节级 JSON → `/__auto/shutdown` 202 → 进程退出 0 → 端口重绑 → 收据 `av_dispatches=2 legacy_dispatches=0`（auto-man `crates/auto/tests/plan983_c0_cli.rs`，0.37s）。②**C0‑B 已备并直连验证**（auto-down `b830e07`）：e2e 增外部后端模式（`JADE_E2E_BACKEND_URL` 跳过 spawn/`open`），新增 `01b-workspace-av-consumption.spec.ts`；临时探针（plan-artifacts）用 982 CLI 启动 AV 后端 + Vite + Chromium 实测——页面发出真实 `/api/workspace` 请求、得 200/root、`#app` 挂载且 **opener 不出现**（未走拦截 stub），无 console error。③**外部红（非本切片）**：auto-ui master `c0035034` 默认特性下 `ace_av`/`ace_core`/`ace_a2x_ts` 未门控 → `cargo test -p auto`（默认特性）不可构建，`plan890_web_consumers` 默认档暂不可跑；归 980/981 F32 窗口配对。**未做**：C0‑B 永久 playwright 跑（worktree 无 node_modules；探针已证页面消费）、U0、四仓整体。B‑CONSUMERS 仍未勾。

2026-11-18 r11（范围增补：C0-A 落地 + U0 备例）：①**C0-A 已交**——消费者仓新增可执行 AV 业务源 `auto-down/jade-garden/back/auto/av/workspace_service.at`（`211b6d5`；`api.at` 的 `get_workspace()` 契约桩旁标注指向），auto-web 增 `AvServiceEngine::new_with_grants`（显式 `platform.file/mem/env`）并加真实 TCP 直连验收用例（`56b3ac5`）：`--test http_runtime` 8/8（新例 0.215s），字节级 200 JSON、null 形态、404 派发前拒绝、重复稳定、shutdown 端口重绑。②**纠正 C0 合同错误**：`wiki_dir` 按 Windows `PathBuf::join` 为 `<W>\wiki`（JSON `\\`），原 `<W>/wiki` 已修。③**U0 备例**（980/WEB UI 腿）：002-counter 浏览器 `Counter: 0→1`，oracle 先例=`p980_web_lane_browser_mount`（未运行）。982 CLI 接线后同一 AV 源再由 `auto service` 真 CLI 复验；C0-B 前端页与 U0 运行仍待。

2026-11-18 r10（范围增补）：①关闭 **B-GENERATED-R**——委派独立复核（`gen-reviewer`，与实现会话不同 agent）从 2026-11-17 `fd115ce` 旧双批收据出发，核对 `fd115ce → 3f8de15` 增量：生成道文件/依赖 crate 空 diff、生成服务 `main` 直接 `axum::serve` 不构造 `ServiceEngine`，故不触生成服务验收结论；AC-B-GENERATED 四项逐条 pass，复用同 SHA `http_generation` 3/3 并做两次定向复验（lane 单测 4/4 0.122s、参数例 9.702s、生成+坏源例 6.793s）。②**冻结 983/CONSUMERS 首例 C0**：jade-garden `GET /api/workspace`（无文本参数、两字段 JSON），固定字节级预期/失败面/资源清理与接口清单（983/AVSERVICE 已交付；982/EXEC `--server av` 需改走 auto-web `serve_av`；980/WEB 与 983/GENERATED 本首例不用）。本轮只做复核收据与首例定义，未写消费者代码、未运行新栈请求；B-CONSUMERS 仍未勾。

2026-11-18 r9（范围增补）：依用户指令「先接 `auto_web` 生产栈，由 983 主导；完整 JSON 等第一个真实消费者明确需要哪些字段后再做」，新增具名切片 **B-AVSERVICE-TEXT**：把 982 已验的受检文本入口（`fn main(query str, body str) int`）接到 auto_web 既有 TCP 生产栈——**同一份已编译 handler**收查询串 + UTF‑8 体，请求不重编译、不经文件；等字节长度 `中文`/`日文` 按内容分流，收据 `av_dispatches>0 / legacy_dispatches=0 / compile_count=1`，现有服务回归保持。**未做**：文本参数与平台 I/O 的同一次调用（未触发；触发时交 982 共享入口补齐）、完整 JSON/请求头（等首个真实消费者字段）。原19任务/AC/进度保留，总数 20、12 完成；本次只实施+验证这一切片并落规范增量，review 仍归 B-AVSERVICE-R/T-05。

2026-11-16 r7（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包仍完成现有Web服务/前后端实际消费者及RG证明；消费976/980所需TS/JS闭包，不将Godot或独立无消费的a2ts产品作为整包门，也不删Vue/Web现有业务。原19任务/进度保持。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r5当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-07 r3：细化为4批，§5–8为当前执行合同；仍仅规划、不施工。


迁Web owner线程/handler调用、动态与生成Rust服务、jade-garden及下游实际server入口，去除AutoVM私有类型耦合；保文件返回、上传、认证/权限、错误、shutdown。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-web/crates/auto-web/src/host.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-web/crates/auto-web/src/http_server/server_dispatch.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-web/crates/auto-web/src/http_transport.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：Web宿主端口/动态与生成服务/真实消费者。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-web/crates/auto-web/src/host.rs](../../../auto-web/crates/auto-web/src/host.rs)：public WebHttpServiceHost端口；读取SHA256前16 `a3be862079c790c6`

- [auto-web/crates/auto-web/src/http_server/server_dispatch.rs](../../../auto-web/crates/auto-web/src/http_server/server_dispatch.rs)：替Rc<AutoVM>与旧handler调用；读取SHA256前16 `44e4eeedac749dc4`

- [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs)：真实安装消费/端口关闭oracle；读取SHA256前16 `ff449536a73274af`

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

## 5. 详细设计

### CONFIG-ACE-B交接（2026-11-17）

[982/B-CONFIG](982-ace-m2-man-gen.md)现统一生产配置文档Schema/AV/owned SDK与Man接口，见[设计](../design/ace-config-document-evaluation.md)。本包原19任务/已交PORT和PROOF A面保持。AVSERVICE/GENERATED按原端口先行；CONSUMERS用到配置的工程须收IF-982-CONFIG-ACE＋982/EXEC＋980/WEB对应正式接口，不把同SDK的legacy配置owner冒新引擎。

CONSUMERS I/V/R负责每个真实配置入口/dep参数/env/锁、坏配置零写/修复下一读和实际服务结果；配置算法由982生产，本包只接真实Web/Man/UI/Down consumer，不重复parser/求值器。982 CONFIG不等本包或980/WEB完成才交能力；本包CONSUMERS也不因只缺某产品接口阻已就绪服务批。全部required最终齐才原T-05/06/07独立收口。

### M2-TARGET-SCOPE-20261116：本包当前边界

本包仍完成现有Web服务/前后端实际消费者及RG证明；消费976/980所需TS/JS闭包，不将Godot或独立无消费的a2ts产品作为整包门，也不删Vue/Web现有业务。原19任务/进度保持。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A=PORT/PROOF，PORT只等967/968/969/974，PROOF补975/978；不等982/EXEC、980整包或四仓CONSUMERS。B=AVSERVICE/GENERATED/CONSUMERS，保完整原功能及四仓收据。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PROOF-I, B-PROOF-V, B-PROOF-R, B-PORT-I, B-PORT-V, B-PORT-R |
| M2-B | B-AVSERVICE-I, B-AVSERVICE-V, B-AVSERVICE-R, B-GENERATED-I, B-GENERATED-V, B-GENERATED-R, B-CONSUMERS-I, B-CONSUMERS-V, B-CONSUMERS-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

### 四仓门禁修复（B-CONSUMERS-V 内置阶段，待执行）

以 §9 的 r22 阻塞收据为基线。四仓门禁在语言固定组阶段因 `plan988_byte_consumers` 达到 120 秒超时而中止，Web、UI、Man 外部腿尚未启动。隔离复跑的测试结果两次都是 46/46，进程审计仍因 `vctip.exe` 在工作负载退出时留存而失败。983 负责实施、取证和四仓复跑，测试治理维护方复核共享门禁或进程计量的改动。

1. **测试超时。** 优先按语义把 `crates/auto-generation-core/tests/plan988_byte_consumers.rs` 的长用例拆成多个独立测试，复用现有执行逻辑。对照原场景名逐项保全覆盖，保留 VM、生成 Rust 后真实编译运行、字节对照和非法输入保旧断言。若测量仍表明个别测试需要额外时间，在现有门禁配置内按具体测试设定有实测依据的预算。
2. **MSVC 子进程收尾。** 在同一冻结基线隔离复现，记录残留进程映像身份、父进程和退出时序，定位产生残留的编译或链接调用。把修复限制在相关调用或其受管进程树。按现行工作负载退出规则验收，编译准备阶段的辅助进程例外仅适用于准备阶段。
3. **恢复四仓验收。** 使用现有 983 组树和版本对齐的干净输入。先跑 `cargo t auto-generation-core`，核对原场景全部实际执行、断言全过、未触发超时、工作负载退出时受管进程归零。再按 §6 跑正式 `V-B-CONSUMERS`，核对语言固定组与 Web、UI、Man 外部腿均实际执行且完整收据通过。已签 U0 收据按原产物身份复用。

两项基础设施修复和四仓门禁均通过后，方可关闭本阶段并交 `B-CONSUMERS-R`。执行和复跑结果写入 §9，包含版本、场景对照、退出码、计时及进程审计。整包复审、合入和清理继续沿 T-05／06／07。

**814 路径依赖身份（2026-11-19 r24，用户指令在 983 新开 phase 修复；985/982b 归对应会话）**：PLAN-1034 `7db528b93` 把 `auto-tool-execution` 加成 auto-lang-core 直接路径依赖（非 optional ⇒ 进 SDK 编译闭包），`build.rs` 的 `IDENTITY_SOURCE_CRATES` 漏项 ⇒ 814 依赖覆盖守卫红。修复=清单补入该 crate（lang master `45afda45d`）；复跑 `plan814_build_input_boundary` **3/3**（依赖覆盖+两个 SDK 边界负控）与 `stdlib_assembly::manifest` 全模 **7/7**，既有守卫零改动。修后 master 剩余红=985×5+982b（各线会话在途）。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan局部批次可独立实施/验证/复核，不新增Plan状态，也不等于可单批归档/删树。本Plan唯一worktree保留至全部批次及最终review/merge完成。分支冻结SHA只供受控准备；下游最终验收须核已合入来源，批次review不冒正式D。

frontmatter是整包最终依赖并集，各批按下表最小接口前置启动。LNNN=auto-lang/PLAN-NNN；UI/AG/OS标仓身份。依赖收据必须包含repo/commit、Plan revision、接口/schema/能力域和实际consumer/oracle。只有计划名字不等接口就绪。

唯一协调组D:/autostack/.wt/lang-983/，各实际改动仓保一个兄弟tree；实施/review顺序复用，公共文件单写。外仓开工重核实际唯一树，不建junction/symlink。OS/App采用OS064规定的OS worktree/独立App clone，不套语言树规则。

| 批次 | 交付结果 | 最小前置 | owner |
|---|---|---|---|
| 983/PORT | Web中立执行与资源端口 | L967/L968/L969/L974 | auto-web |
| 983/AVSERVICE | AV真实HTTP服务 | 983/PORT; L975 | auto-web |
| 983/GENERATED | 生成Rust服务 | 983/PORT; L978/L975 | auto-web + auto-lang |
| 983/CONSUMERS | Web Man UI Down真实消费闭环 | 983/AVSERVICE; 983/GENERATED; 982/EXEC; 980/WEB | auto-web + auto-man + auto-ui + auto-down |
| 983/PROOF | RG-A02真实AV与生成服务 | L983/PORT, L974, L975, L978 | 原owner；RG-A具名真实验证 |

### 983/PORT：Web中立执行与资源端口
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/host.rs`；`D:/autostack/auto-web/crates/auto-web/src/http_transport.rs`；`D:/autostack/auto-web/crates/auto-web/src/http_server_handle.rs`。
I实现：把public host的Rc<AutoVM>/旧native签名替换为中立受检端口，保owner线程、取消、File/Stream所有权和shutdown。
V验收：独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/AVSERVICE：AV真实HTTP服务
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/http_server/server_dispatch.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_runtime.rs`。
I实现：动态路由、参数、认证/权限/中间件、上传/下载/stream与DB接ACE，保transport现有实现。
V验收：真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/GENERATED：生成Rust服务
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/api_gen.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_generation.rs`；`crates/auto-generation-core/src`。
I实现：新HIR A2R生成服务/共享载体，保route/store/schema/diagnostics与失败保旧；本批可与AVSERVICE在端口稳定后独立。
V验收：未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/CONSUMERS：Web Man UI Down真实消费闭环
改面（均核存在）：`scripts/web-four-repository.py`；`D:/autostack/auto-man/crates/auto/tests/plan890_web_consumers.rs`；`D:/autostack/auto-down/jade-garden`。
I实现：jade-garden与既有Web生产调用逐行接新SDK；每owner代码独立交付，消费相同source/tool版本，保旧四仓义务。
V验收：原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

#### 983/CONSUMERS 首例冻结（2026-11-18，C0：jade-garden `GET /api/workspace`）

**选择与理由。** 从 jade-garden（auto-down）现有生产调用取一条真实请求作首例：`GET /api/workspace`。它是应用启动/桌面 daemon 的既有调用（[front/src/lib/api.ts](../../../auto-down/jade-garden/front/src/lib/api.ts) `getWorkspace()`；e2e [01-workspace.spec.ts](../../../auto-down/jade-garden/front/e2e/01-workspace.spec.ts) 与 [global-setup.ts](../../../auto-down/jade-garden/front/e2e/global-setup.ts) 已对真实后端断言），**无查询串/请求体文本**（不触发 982 的「文本参数＋平台 I/O」组合缺口），响应只有两个可空字符串字段（root/wiki_dir）——正是「首个用例只读少量字段就先交那部分」的最小 JSON 面。

**真实请求与预期（字节级）。** 后端固定端口 `18181`（e2e `BACKEND_PORT`），fixture 工作区 `front/e2e/.runtime/workspace`（正斜杠形态 `WORKSPACE_ROOT`）。请求行 `GET /api/workspace HTTP/1.1`、`Host: 127.0.0.1:18181`、`Connection: close`、空体。
- 配置了工作区（`JADE_GARDEN_DEFAULT_WORKSPACE=<W>`）⇒ `200`、`content-type: application/json`、体逐字节 = `{"root":"<W>","wiki_dir":"<W>\\wiki"}`——**wiki_dir 用 Windows `MAIN_SEPARATOR`**：`PathBuf::from("<W>").join("wiki")` 实测为 `<W>\wiki`（`<W>` 以 `/` 或 `\` 结尾时才直接拼 `<W>wiki>`），JSON 中写作 `\\`；字段序 root→wiki_dir（serde_json struct 序），路径取 `to_string_lossy` 语义。
- 未配置（env 缺省且无配置文档）⇒ `200` + `{"root":null,"wiki_dir":null}`（与 01-workspace.spec.ts 的拦截 oracle 同形）。
- 工作区目录不存在 ⇒ 仍返回该路径与 `<W>/wiki`（原 `get_workspace_impl` 不校验存在性；校验只在 POST /open）。

**失败面。** 未声明路由（如 `GET /api/nope`）⇒ `404` 空体（AV 服务臂与 legacy 预处理同形；不回落 `d_main`、不回退 VM）；畸形请求 ⇒ 传输层 400/断开且不派发；AV 执行诊断 ⇒ `500` 具名 `ace.service/*`，不得静默 200。

**资源与清理。** 只读配置/env 与工作区路径：不写工作区、不建/改配置文件；每请求文件句柄关闭（AV 平台台账「每调用强制关句柄」）；连续 2+ 请求体字节相同且 `av_dispatches` 累加、`legacy_dispatches=0`；外部停止 ⇒ 排水、端口可重绑、进程退出 0。

**接口清单（实际用到 / 明确不用）。**

| 接口 | 状态 | 本首例用途 |
|---|---|---|
| 983/AVSERVICE：`auto_web::serve_av` + `AvServiceEngine`（文件通道；`#[api]` 路由、404/501、shutdown；2026-11-18 增 `new_with_grants` 显式 grants） | **已交付**（auto-web `3f8de15`；直连切片 `56b3ac5`，m2-web.md B 节） | 承载 `/api/workspace` 的 AV 程序与真实 TCP/配置/生命周期 |
| 982/EXEC：`auto service --server av`（**auto-web 宿主** + `--api` 显式 AV 源；`av-direct`=pre-C0 man 宿主保留） | **已交付**（auto-man `6e21bfa`，plan-983-dev；公开面 a/b/c 已落 auto-web master `5c484d9`；规范 m2-web.md「C0 公开面」=lang `2513903df`） | 真 CLI 启动同一 AV 源：真实 bind/就绪行/收据/控制面关停（`plan983_c0_cli`） |
| 982/CONFIG：服务配置 `--http-config-inline`（listen/profile/drain） | **已交付**（982/EXEC 面）；配置文档引擎本请求**不用**（jade-garden 后端无 pac.at；本轮配置入口=env+服务配置） | listen/profile/drain/退出；坏配置具名失败、旧工作区零写 |
| 980/WEB：Vue/TS 表达式后端 | **本首例不用**：jade-garden 前端由 a2ts 生成 + 手写 fetch，不经 auto-ui Vue 表达式面；其未闭项不阻塞 C0，四仓整体验收的 UI 腿须另立 case | — |
| 983/GENERATED：A2R 生成服务 | **本首例不用**：生成道无平台 I/O/配置读取；生成服务腿由既有 `http_generation` 用例与后续 case 覆盖 | — |

**C0-A 实现与直连验收（2026-11-18 已交）。** 可执行业务源=auto-down `jade-garden/back/auto/av/workspace_service.at`（提交 `211b6d5`；`api.at` 的 `get_workspace()` 契约桩旁已标注指向）：读 `JADE_GARDEN_DEFAULT_WORKSPACE`（`env_get_w16` UTF-16LE）→ UTF-8/JSON 转义（`"`、`\`、控制字符 < 0x20）→ 写响应文件；`wiki_dir` 按 Windows join 语义拼 `\wiki`；只读、每请求关句柄。宿主直连=`AvServiceEngine::new_with_grants`（`platform.file/mem/env`；默认 `new` 授权面不变，提交 `56b3ac5`）。真实 TCP 用例 `http_runtime.rs::c0_workspace_p983::jade_workspace_route_serves_exact_json_from_the_consumer_av_source`（0.215s）：200 字节级 JSON（含 `\\wiki`）、重复请求稳定、未配置 ⇒ `{"root":null,"wiki_dir":null}`、`/api/nope` ⇒ 404 空体（派发前）、shutdown 端口重绑。用例**直接读消费者仓源**（`AUTO_DOWN_ROOT` 或相对 `../../../auto-down`，缺失具名失败），不复制；`--test http_runtime` 8/8 绿（7.92s）。**982 CLI 接线到位后**同一源再由 `auto service` 真 CLI 复验（未做）。

**C0 CLI 复验（2026-11-18 已交，982/EXEC 接线）。** `auto service --server av --dir <project> --api <jade-garden workspace_service.at> --http-config-inline {"listen":{"port":0}}` 真 CLI：真实 bind（内核分配端口）、`AUTO_SERVICE_READY` 就绪行、`GET /api/workspace` 字节级 JSON（`root`/`wiki_dir` 与直连一致）、`POST /__auto/shutdown` ⇒ 202、进程退出 0、端口重绑、收据 `av_dispatches=2 legacy_dispatches=0 same_sdk=true`。实现：`--server av` = auto-web `serve_av` + `AvServiceEngine`（`--api` 显式源；`AUTO_AV_GRANTS` 默认 `platform.file,mem,env`）；pre-C0 的 man TCP 宿主保留为 `--server av-direct`（`plan982_av_http_service` 17/17 + `plan982_service_av_track` 2/2 仍绿）。用例 `auto-man crates/auto/tests/plan983_c0_cli.rs`（0.37s；`--no-default-features --features ace-candidate`）。`ace-candidate` 编译：`cargo check -p auto --no-default-features --features ace-candidate` 退出 0（UsePy 缺臂已由 lang `ff55579c3` 修复）。

**C0-B（前端页）已正式跑（2026-11-18）。** e2e 外部后端模式（`JADE_E2E_BACKEND_URL`）+ `01b-workspace-av-consumption.spec.ts`（auto-down `b830e07`）在 AutoDown 主检出的 Playwright/Vite 上运行（临时 overlay `front/e2e-983` + 临时 config，跑完即删；不复制 node_modules、不在 worktree 建 symlink）：**1 passed**（6.2s；首跑 11.5s）。正式收据：页面真实请求 `GET http://127.0.0.1:13100/api/workspace` → **200** + `{"root":"<fixture workspace>","wiki_dir":"<fixture>\\wiki"}`；主视图断言（`#app` 挂载、`Select a file…` 提示、opener=0）通过；后端（982 CLI `--server av --api <consumer source>`）收据 `served=2 av_dispatches=2 legacy_dispatches=0 same_sdk=true`；退出码 0；日志 `plan-artifacts/plan-983/c0b/playwright-01b.log`。managed 后端的 file-tree/opener 用例在外部模式下具名 skip，环境齐备时照旧覆盖。

**C1 验收基线（2026-11-18 实测冻结，交 982 实现组合入口）。** 用现有 jade-garden 后端（`open_workspace_impl`；临时 exe 副本 + 隔离 config）实测：
- 请求：`POST /api/workspace/open`，`content-type: application/json`，body `{"root":"<路径>"}`（单字符串字段；正/反斜杠输入均可）。
- **不存在目录**：`400`，body 逐字节 `{"error":"Directory does not exist: <请求原样路径>"}`；**零副作用**（不创建目录、不写 config/index、既有工作区不变）。
- **成功**：`200`，body `{"root":"<canonical>","wiki_dir":"<canonical>\\wiki"}`——canonical 是 Windows `canonicalize()` 形态（**`\\?\` 前缀 + 反斜杠**，JSON 里 `\\\\?\\`），与 C0（env 原样路径、未 canonicalize）**不同**；按原样钉住。
- **副作用**：`<root>/wiki` 不存在时创建（空 wiki ⇒ 索引 `"pages": []`）；`<root>/jade-garden-index.json` 重写（`pages[{path,title,frontmatter,mtime}]` + `blocks[...]`，覆盖 `wiki/**/*.ad`，实测含 `hello.ad`）；config 持久化 `{"workspace_root":"<canonical>"}`（现有实现写 exe 相邻 `jade-garden-config.json`；CLI/AV 宿主须显式 config 路径——建议 `JADE_GARDEN_CONFIG`，实现时冻结）。
- 二次 open 同路径幂等（200/同 body/index 重写）；切换到另一路径 ⇒ 新路径成为当前工作区，旧目录内容不被修改。
- **组合入口需求（982）**：一次调用同时要文本实参（root）+ 平台 I/O（exists/stat、`create_dir_all`、读写 config/index、遍历 wiki）+ JSON 出参。旧实现先保存配置再准备 wiki 和索引，失败行为以以下 C1 合同修正。

#### C1 工作区提交与失败验收

- **提交顺序**。先验证并规范化目标路径，再以目标目录为参数准备 wiki、生成并写入目标索引。准备成功后，用同目录临时文件原子替换工作区配置，再发布相互匹配的当前根路径与索引。并发切换须串行提交，GET 在切换期间读到一套一致的状态。
- **失败边界**。不存在的目标仍按 r14 返回 400 和原样路径，旧工作区、配置字节及旧索引不变，目标目录也不创建。目标索引重建失败时返回 500，JSON 的 `error` 字符串以 `workspace.index-rebuild-failed:` 开头，旧工作区、配置字节及旧索引保持有效。目标目录中已创建的 wiki 可以保留。配置替换失败时仍保留旧工作区与原配置，目标侧已完成的 wiki 与索引可以保留。
- **永久回归**。沿 C0 的真实 CLI HTTP 入口启动 C1，使用隔离的持久配置，先打开旧工作区并采集 GET、配置字节及旧索引字节。在另一可写目标目录内预建名为 `jade-garden-index.json` 的**非空目录**并放入标记文件，保持目标 wiki 尚未创建。POST 该目标，断言 500 和具名诊断，随后 GET 与重启后 GET 均返回旧工作区，配置和旧索引字节一致，标记文件保持原样。记录目标 wiki 已创建的副作用。使用同一入口验证不存在路径的 400 零副作用，并用合法目标验证 POST 后及重启后 GET 均返回新路径。
- **旧后端反例（2026-11-18，实测）**。隔离副本经本机真 HTTP，非空索引目录使旧实现返回 500（`Failed to build link index`，Windows OS error 5），配置字节和随后 GET 的根路径均变更，目标 wiki 已创建，标记文件保持原样。不存在路径返回 400，配置字节与 GET 均不变。此记录验证了故障夹具，新版永久回归待实现后运行。
- **r16 现状（2026-11-18）**。永久回归已落在 `auto-man crates/auto/tests/plan983_c1_cli.rs`：①活跃用例 `c1_open_get_restart_share_one_persisted_workspace_config`（**1/1 绿**，0.42s）：env A 初始回退 → POST B（200、`B/wiki` 建、config 写 B）→ GET B → 重启 GET B（持久值优先）；②`c1_index_failure_keeps_the_old_workspace_and_config`（`#[ignore]`，`--ignored` 复现）：非空索引目录夹具下**当前 interim 源返回 200**（应 500 `workspace.index-rebuild-failed:`），证明 r15 的索引准备/原子提交尚未实现。GET/POST 已共享 `JADE_GARDEN_CONFIG`（持久值优先、env 初始回退）；具名边界：POST root 仅 ASCII、无 canonicalize（不做 `\\?\`）、无索引写入——C1 完整实现时按 r15 收口。**r17 补强（2026-11-18）**：索引失败用例先断言旧索引确实生成且 `pages[].path == "hello.ad"`，再注入目标失败；新增 Unicode 路径 / Windows canonical 形态 / 配置提交失败三个预置用例（`#[ignore]`，`--ignored` 实跑 4/4 红，失败点分别为旧索引缺、形态不符、Unicode 具名 400、config 前缀不符）；活跃 `POST→GET→重启` 用例 1/1 绿（0.81s）。**r18（2026-11-18）**：C1 业务行为已在 AV 文件通道实现（auto-down `a634c09`）——canonical/Unicode/索引重建/原子提交/两具名前缀全部落地；四个预置用例**去 ignore 转正，`plan983_c1_cli` 5/5 绿（0.86s）**，`plan983_c0_cli` 1/1、auto-web `http_runtime` 8/8 保持绿。具名边界：mtime 恒 0、blocks/links/tags 恒 `[]`（16 平台操作 verify 封顶 + 链接图属后续切片）、并发串行提交未实现未验、`dir_close` 依赖宿主 ledger 运行终强制关句柄。**r19（2026-11-18）**：生产 HTTP 已接 **982 组合文本入口**——请求作为借用 `str` 实参（`p983-req.txt` 退役），业务行为基线（r18 的 5 用例）在组合入口上复跑全绿 + 新增并发一致性用例共 **6/6**；区分性证据=`combined_entry_dispatches` 计数与暂存文件缺席断言（见 §9 r19）。并发串行提交合同由单 owner 顺序派发 + 原子替换结构性满足并被永久用例固定。**r20（2026-11-18）**：POST 改走 **982 文本路由**（引擎 master `5e4322a`：body 作为借用 `str` 实参，不经请求暂存文件；GET 保持文件通道）——五例+并发在分道引擎上 **6/6 绿**，`combined_entry_dispatches` 计数与 POST 后/GET 前的暂存文件缺席断言构成区分性证据（见 §9 r20）。

**分期。** C0-A 后端线、C0 CLI 腿、C0-B 页面（正式 playwright）均已交。C1 基线已冻结（上段），**实现待 982 的组合入口**（文本 + 平台 I/O 同一次调用）；C1 之后为 U0 与四仓整体。

**U0（UI 腿，980/WEB 消费；2026-11-18 备例）。** 真实交互例=`auto-ui/examples/ui/002-counter`（[app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at)，初始 `Counter: 0`）：浏览器加载生成的 Vue 页面，点 `+` ⇒ `Counter: 1`，`-`/`Reset` ⇒ `Counter: 0`；DOM 文本逐字断言、无控制台错误。oracle 先例=auto-ui-runtime 浏览器挂载档 `p980_web_lane_browser_mount`（真 002 源 `.Inc` lane 体，41→42；`examples/ui/ace-web-lane/tests/test_lane_browser.mjs`，排窗 `AUTO_UI_ACE_WEB_E2E=1`）。接口：980/WEB——`ui_gen/vue` SFC/表达式与 lane handler 体（切片 #1–#6p 已交；**B-WEB-I/V/R 未勾**，余=真语料全量 spliced 切换 watch/setup/expose/ext_imports/style）；982/EXEC——产品形态需 `auto run`（Vue）真 CLI，候选腿当前对 run 家族具名拒绝、ACE 服务化未交（与 C0 同一 982 交接）；配置=`pac.at`（002 有）+ 应用窗口配置，实际字段随 CLI 腿冻结；983/AVSERVICE/GENERATED 不参与。失败/清理：坏源保旧产物、构建诊断具名；浏览器/服务子进程回收、端口释放、生成 tree 不手改。**现状**：只备例，未运行浏览器档；four-repo 的 UI 腿等 980/WEB 批勾选与 982 `auto run` 接口。**更新（2026-11-18，980 ④-A #5/#6）**：980 已交**产品出口** `auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane_product`（SFC 显式 `import … from "./a2x_rt.mjs"` + 官方 rt 随产物；`lang="ts"` 交既有 Vue 链）——002 未改写源经产品产物树真实挂载 `Counter: 0→1→0`（加/减/重置真点击、零 page error、零夹具注入，`plan980_u0_002_product_sfc` 双档绿，ui master `c035a458`/`5d7725d9`）；982 round-184 已落 `auto run --render vue` 分派（`AUTO_UI_ACE_VUE=1` 选择键，其具名 ignore 用例等 980 工具侧读键接线——合同=lane 缺席具名拒不静默落 legacy）。U0 产品验收剩余=工具侧接线后 983 沿同一份产物跑真链。**工具侧已接线（2026-11-18 ④-A #7，auto-ui master `4dd2654c`：`AUTO_UI_ACE_VUE=1` 双道走产品 lane+rt 落盘+缺席具名拒；工具按 `--features ace-vue` 构建）——983 的 U0 产品验收等 982 去 ignore 跑真链后沿同一份产物收口。**r22 闭合（2026-11-18，用户裁定接收）**：980×982 联合 U0 收据已由 983 接收（002-counter 真链 `auto run --render vue`→pnpm→vite→浏览器 Counter 0→1→0 + lane/rt 磁盘断言；man `4a7a28e` 3/3 + lang `8e94acfaf`）——983 沿「消费同一产物树/同一链」边界不重跑；#9 主树复跑与 980/WEB 全批验收归各线。U0 事项关闭。**双侧闭合确认（2026-11-18 用户）**：U0 已在 980/982 双侧闭合，联合收据落 `2c9c5f228`（auto-ui `70e9ec02` × auto-man `4a7a28e`，真 CLI→浏览器 3/3）；980 #9 主树复跑/AutoNotes ACE/完整 B-WEB-I/V/R 仍开放并行。

**给 982 的优先请求（EXEC 接口）。** ①`auto service --server av` 的宿主换成 auto-web `serve_av`（同 transport/config/shutdown/ready/receipt；`AUTO_AV_*` 保留为引擎选项）；②API 源路径可显式指定（默认发现 `src/back/api.at` 不变；jade-garden 用 `back/auto/api.at`），无 API 仍具名拒绝、不回退 vm；③暴露 AV 引擎的 grant/scratch（`platform.file`/`platform.mem`，jade-garden 首例还需 `platform.env`）与程序入口；④`--http-config-inline`/配置语义同 `serve_vm`，坏配置零写、失败具名。C/EXPORT 等按自身顺序并行。

**不声称。** C0-A 证「同一 AV 业务源 + 直连宿主」的字节级语义；C0 CLI 腿证同一源经 982 真 CLI 的可观察行为；C0-B 目前是「e2e 已备 + 临时探针实测页面消费真实响应」——**永久 playwright 跑、U0 运行、四仓整体验收**均未完成。另：auto-ui master `c0035034` 默认特性红（`ace_av`/`ace_core`/`ace_a2x_ts` 未门控）使 `cargo test -p auto`（默认特性）不可构建，`plan890_web_consumers` 默认档暂不可跑——外部（980/981 F32 窗口配对），非本切片引入。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。


A期生成入口实核：现有`http_generation.rs::cold_generated_rust_executes_and_missing_web_preserves_workspace`直接调用公开`auto_web::generate_api`后真实cargo编译/TCP验证，不依赖Man全目标CLI。本批沿该公开入口补新A2R接线与所需case，完整GENERATED与CONSUMERS仍B；不得为等待982/RUST或EXEC整包制造A→B边。

### A期PROOF批次（新增、未实施）

最小前置：L983/PORT, L974, L975, L978。实际改面：`D:/autostack/auto-web/crates/auto-web/tests/http_runtime.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_generation.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：在原TCP fixture给一份原工程接ACE AV服务和新A2R生成服务，读写隔离DB/JSON与真实响应；只接所需最小生产路径，完整AVSERVICE/GENERATED API面和四仓CONSUMERS仍B。
V：两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at](../../../auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at)（下列节选L4–L39；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
pub type CounterResult = { ok: bool, err: str, value: int }

#[api(method = "GET", path = "/value")]
pub fn value() int {
    let marker int = store.version_marker()
    if store.data_root() == "" {
        return 0 - marker
    }
    return marker
}

#[api(method = "POST", path = "/counter/add")]
pub fn counter_add(delta int) CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    let next int = store.add_counter(delta)
    return CounterResult { ok: true, err: "", value: next }
}

#[api(method = "POST", path = "/counter/double")]
pub fn counter_double() CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    let next int = store.double_counter()
    return CounterResult { ok: true, err: "", value: next }
}

#[api(method = "GET", path = "/counter/get")]
pub fn counter_get() CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    return CounterResult { ok: true, err: "", value: store.read_counter() }
}
```

本包所需独立结果：原fixture/API路由要经真实TCP返回独立预期；http_runtime的nonce正文web-package-runtime-890、names含照片.png及ints[-1,-4,-5]全部保；生成服务冷编运行同源。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Auto API路由/参数/鉴权源→公共ACE执行/typed host→既有transport→真实TCP/文件/DB/流→独立客户端。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct WebInvocation { route: RouteId, request: RequestContext, session: SessionId, cancellation: CancelToken }
struct WebHostPort { signatures: SignatureCatalog, owner_thread: ThreadOwner, grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-web/crates/auto-web/src/host.rs](../../../auto-web/crates/auto-web/src/host.rs) | public WebHttpServiceHost端口 |
| [auto-web/crates/auto-web/src/http_server/server_dispatch.rs](../../../auto-web/crates/auto-web/src/http_server/server_dispatch.rs) | 替Rc<AutoVM>与旧handler调用 |
| [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs) | 真实安装消费/端口关闭oracle |

**状态、错误与资源。** 错handle/路由参数/权限/畸形上传和断连精确拒；缺DB/环境保持缺证，不能用内存fakeHTTP或父exit0代子终态。 request/session/DB/file/stream各有owner，取消/晚完成不串请求；shutdown释放后台任务和端口，Core/UI闭包不反引入auto-web-contract。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at](../../../auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at) | 原Auto服务源/路由/状态 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `25dce22ebbb82249` | 拟ace.m2.p983.source01：serve-av-tcp/serve-a2r-tcp；负控/资源见V-DETAIL-02/03；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs) | 精确JSON/文件/权限/关闭 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `ff449536a73274af` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [auto-web/crates/auto-web/tests/http_generation.rs](../../../auto-web/crates/auto-web/tests/http_generation.rs) | 生成服务独立冷编/TCP | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `899b751e11a1db15` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at](../../crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at) | 客户端/传输公共语义，不能替服务端测试 | `http_client_sync.expected.rs`；SHA256前16 `3ab20dc82b46f669` | 拟ace.m2.p983.source04：serve-av-tcp/serve-a2r-tcp；负控/资源见V-DETAIL-02/03；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** PORT先交通用接口，AVSERVICE/GENERATED可按稳定端口并行，CONSUMERS保真实四仓产品；不反等Man全CLI再交公共Web能力。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-PORT-I`、`B-PORT-V`、`B-AVSERVICE-I`、`B-AVSERVICE-V`、`B-GENERATED-I`、`B-GENERATED-V`、`B-PROOF-I`、`B-PROOF-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量
| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CFG-HANDOFF | modify（拟，原owner） | `docs/specs/ace-migration/m2-web.md` | 原候选接收→实际配置新AV来源/宿主与guest边界、错误恢复/原平台，真实交付后才写已支持 | 982生产一次，本包只核实际消费者/安装退役阶段；A收据保留 | AC-CFG-HANDOFF、原AC与最终review |
| SD-01 | add | `docs/specs/ace-migration/m2-web.md` | 分批沉淀实际支持/来源/拒绝及未闭行 | 不把计划当现状 | AC-01..06及所有AC-B-* |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 959行关联本批新producer/consumer与收据 | 替换追踪 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-web.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-web.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-web.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

## 6. 测试设计

### V-CFG-HANDOFF（本轮新增接收控制，未执行）

沿[配置设计九行Case](../design/ace-config-document-evaluation.md#6-验证合同与旧用例映射)，在本包所用真实consumer/原测试入口验合法动态值及坏源/错SDK-context/零旧求值/失败保旧/修复下一次read；把route C legacy config冒ACE、只核same_sdk、缺source或仅用静态投影则拒绝。原V全部保留，不新造runner。实际未用配置的域不增加虚假依赖；原平台/ignore/输入规模保持。

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部命令为未来执行，本轮未运行。cwd前缀lang/ui/man/web/ag-examples分别为本Plan对应唯一tree。永久新fixture由I接到所列target；旧用例绿不能证明新ACE支持。参数尚待冻结的产品命令由I第一步读取现有runner/help与工程配置固定，之前不进入V，不声称未来runner当前已可用。

材料/外置target/生成安装副本只放D:/autostack/plan-artifacts/plan-983/下按仓/批次区分，普通target留树。AG仅受影响文件/case、窗口/GPU单实例；原ignore/平台/资源预算保留。发现/执行/排除、准备/运行/清理/总时与子终态分列，零选择/emit-only/未执行不能PASS。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-PORT | AC-B-PORT | B-PORT-V；web: cargo nextest run -p auto-web --lib；cargo nextest run -p auto-web --test http_runtime | 独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。 |
| V-B-AVSERVICE | AC-B-AVSERVICE | B-AVSERVICE-V；web: cargo nextest run -p auto-web --test http_runtime；lang: cargo t http（仅语言相关改动）；完整TCP断言沿现有runtime target | 真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。 |
| V-B-GENERATED | AC-B-GENERATED | B-GENERATED-V；web: cargo nextest run -p auto-web --test http_generation；lang: cargo t auto-generation-core web-delivery-controls | 未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。 |
| V-B-CONSUMERS | AC-B-CONSUMERS | B-CONSUMERS-V；lang: python scripts/web-four-repository.py --out <scope>/web-four；man: cargo test -p auto --test plan890_web_consumers；沿原四仓门核实际子项 | 原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。 |

最终T-05核全批收据适用性、遗漏、组合并集和原资源门；证据身份不变可复用，不反复全仓。每批独立复核不替整包final review，未闭in-scope阻execution_done。


### A期PROOF验证（本轮未运行）

| Case | AC | 实际/拟补入口 | 独立oracle |
|---|---|---|---|
| V-A-PROOF | AC-A-PROOF | web: cargo nextest run -p auto-web --test http_runtime --test http_generation；新增ACE具名case接原target并核实际子进程；与HTTP资源组原串行条件一致 | 两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 原fixture/API路由要经真实TCP返回独立预期；http_runtime的nonce正文web-package-runtime-890、names含照片.png及ints[-1,-4,-5]全部保；生成服务冷编运行同源。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 错handle/路由参数/权限/畸形上传和断连精确拒；缺DB/环境保持缺证，不能用内存fakeHTTP或父exit0代子终态。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | request/session/DB/file/stream各有owner，取消/晚完成不串请求；shutdown释放后台任务和端口，Core/UI闭包不反引入auto-web-contract。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

- [ ] **AC-CFG-HANDOFF / work**：§5本包实际配置consumer/阶段的正式来源和V-CFG-HANDOFF闭合，区分宿主AV与目标业务、原A与完整B；不以未交接口/上游状态或peer-equal代真实consumer与独立oracle。原工作任务负责，原final独立复审核验。

| ID | owner_stage | 标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/平台/输入oracle完整 | T-01及各I |
| AC-02 | work | 原整包能力经真实源码/工程运行，范围不减 | 全部I/V |
| AC-03 | work | 全consumer接线与权限/资源/恢复、无legacy fallback，跨仓身份准确 | 全部V及959行 |
| AC-04 | review | 批次及全包独立复核，原AC反例/health/Spec均核 | 各R及T-05 |
| AC-05 | merge | 正式来源/Spec/归档/清理完成 | T-06/07 |
| AC-06 | work | 原T-08补充义务分配至批次且全闭 | 所有AC-B-* |
| AC-B-PORT | work | Web中立执行与资源端口：独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。 | B-PORT-V/R |
| AC-B-AVSERVICE | work | AV真实HTTP服务：真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。 | B-AVSERVICE-V/R |
| AC-B-GENERATED | work | 生成Rust服务：未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。 | B-GENERATED-V/R |
| AC-B-CONSUMERS | work | Web Man UI Down真实消费闭环：原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。 | B-CONSUMERS-V/R |
| AC-A-PROOF | work | 两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共20任务（14 完成）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。
  [✅ 2026-11-17 T-01 冻结] 前置核对：A最小前置 L967/968/969/974/978 全部 ARCHIVED（archive/ 目录实测）；L975 亦 ARCHIVED。owner/AGENTS：auto-web `AGENTS.md` 已读（唯一 Plan worktree、无 symlink、纯合同无 Core/UI 依赖、wt-guard 清树）。基面：auto-lang master `e85096628`、auto-web master `0adebc0`（两仓 clean，主检出仅 docs WIP）。A能力冻结（B-PORT=公共 host 面中立受检端口；B-PROOF=RG-A02 两腿）：①AV 腿 runner=既有 `ace_av::console_host::compile`+`run_platform`（net974 同款，非第二 runner；`run_ace` E0 无 platform.net 道已核 ace_entry.rs/桥三车道）；②A2R 腿=沿公开 `auto_web::generate_api` 补显式新引擎道（ACE 前门→SemanticView→a2x-rust lower；无静默回退，域外具名拒；sqlite/fs/env/store 留旧道=B-GENERATED，具名不冒）；③PROOF fixture：`auto-web/tests/fixtures/ace-p983/`（AV 服务程序 + A2R 纯 handler 工程），永久用例接原 target `http_runtime.rs`/`http_generation.rs`；④依赖：auto-web 增 dev-dep `ace-av`/`ace-av-core`（根 workspace path 模式同 auto-lang-ace-host）；auto-lang 侧改面=auto-generation-core 新道 seam（组内兄弟树）。V 命令冻结=V-B-PORT（web: nextest -p auto-web --lib；--test http_runtime）与 V-A-PROOF（web: nextest -p auto-web --test http_runtime --test http_generation），串行条件沿原 HTTP 资源组；新增用例逐条计时（>10s 的编译绑定用例如实记录并按 2026-11-14 裁定在收据中显名）。

- [x] **B-PORT-I｜M2-A｜owner_stage: work｜AC-B-PORT/AC-02/03/06**：把public host的Rc<AutoVM>/旧native签名替换为中立受检端口，保owner线程、取消、File/Stream所有权和shutdown。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @93712e5（auto-web plan-983-dev）] 新增 `web_host_port`（WebHostedService/WebListenRequest/WebListenKind/WebRouteDecl/WebPortError：控制面冲突/空地址/owner 单次消费命名拒绝 + 静态 admit）；`host.rs` 公共 `listen_borrowed`/`serve_owned` 重定型为端口类型（core legacy trait 实现留内部桥经同一准入面）；`http_server_handle::serve_server_handle` 公共边中立化（legacy 臂 `serve_server_handle_legacy` crate 内私有）；`http_transport.rs` 冻结面文档化（签名本已引擎中立）。行为保持：owner 线程/取消/File-Stream/shutdown 机制未动。

- [x] **B-PORT-V｜M2-A｜owner_stage: work｜AC-B-PORT**：运行V-B-PORT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @93712e5] V-B-PORT 冻结命令实测（.wt/lang-983/auto-web，基面 auto-lang e85096628/auto-web 0adebc0）：`cargo nextest run -p auto-web --lib --no-fail-fast` = 184 例 183 绿/1 红/1 skip，17.76s；红 = `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，**基线 0adebc0 原样红（stash 复证），非本批引入，具名既有红**。`--test http_runtime` = 3/3 绿，5.94s（真实 TCP 消费正常调用+shutdown join+port 释放+上传 413 拒绝面）。新增端口单测 4 例（控制面冲突/空地址/单次消费/中立路由视图）各 ≤0.021s。`cargo tree -p auto-web-contract` = 仅 serde/serde_json，无 Core/UI 依赖。取消/晚完成不跨请求写入：plan730 cancel/timeout/lease 清理用例绿（lib 门内）。

- [x] **B-PORT-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。
  [✅ 2026-11-17 phase review pass @bdc48e8] 见 §9 phase 复核收据（review_scope=phase；F-R1 格式发现→work 修复 bdc48e8→复验）。

- [x] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：在原TCP fixture给一份原工程接ACE AV服务和新A2R生成服务，读写隔离DB/JSON与真实响应；只接所需最小生产路径，完整AVSERVICE/GENERATED API面和四仓CONSUMERS仍B。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。
  [✅ 2026-11-17 @35b9e47（auto-web plan-983-dev）] AV 腿：永久 fixture `tests/fixtures/ace-p983/av-service.at`（member-delivery/b API 面：/value、/counter/add|double|get + 自有 /__auto/shutdown；counter.txt 持久化经 `file_open_w16`（UTF-16LE staged，Win32 面 file_open_bytes 为 Linux-only 已核）+ file_read/write_some；单 owner 串行 accept 与 legacy 同步 server-handle 同构）；http_runtime.rs 新增 2 用例（`ace_av::console_host::compile`+`run_platform`，net974 同款 runner，ace-av/ace-av-core dev-dep）。A2R 腿：新增 `ace_a2r_lane`（ACE 前门 `compile_text_record_source`+`ac.runtime.text.v1`（cert_rt_902 实证配对）→SemanticView→a2x-rust lower_program→record-text 文件集；显式 `AUTO_WEB_A2R_ENGINE=ace`，子集外具名拒绝不静默回退）；api_gen.rs 早子集门（零参 int/str/bool、无 db.at/伴生）零写入拒 + ACE 道写 a2x 双模块 + api.rs 全 handler 委派 generated_ace + main.rs mod 声明；fixture 工程 `tests/fixtures/ace-p983/a2r-service/` + http_generation.rs 新增 1 用例。生产依赖新增 ace-core/ace-a2x-core/ace-a2x-rust（发射侧；解释器 ace-av 保持 dev-only）。

- [x] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 核子终态与身份，未执行不PASS。
  [✅ 2026-11-17 @35b9e47] V-A-PROOF 冻结命令实测：`cargo nextest run -p auto-web --test http_runtime --test http_generation` = **7/7 绿**（30.59s）。新增用例逐例计时（≤10s 门）：ace_lane 9.14s、ace_av_service 0.86s、ace_av_cancel 0.63s；lane 单测 3 例绿。AV 腿 oracle 实测：独立 HTTP bytes/状态（200/404/202 精确、CounterResult JSON 逐字节）、隔离 DB 独立读回（counter.txt="2147483647" 测试直读）、大整数 i32 全宽 2147483647 精确、断连（半请求+关）不触业务态、cancel（CancelFlag）→ Cancelled+端口重绑、shutdown 202+join+端口重绑。A2R 腿 oracle 实测：未手修生成物断言（generated_ace.rs/a2x_runtime.rs 在场+api.rs 委派）、坏源（带参端点）具名 zero-param 拒+旧 workspace manifest 字节不变、--offline 冷编译真实 cargo 子进程终态、真 TCP 独立 expected（890/"p983-ace-a2r-service"/2147483647）、202 shutdown join+端口释放。lib 回归 187 例 186 绿（唯一红=具名既有红 test_handler_body_r2_matrix_compile_oracle，基线同红）。既有 legacy 用例（cold_generated 14.24s、installed_host 等）保持绿——默认道行为不变。

- [x] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。
  [✅ 2026-11-17 phase review pass @bdc48e8] 见 §9 phase 复核收据；985/A 交接待 merge 阶段 fold 合入后生效（本勾选仅关 A 切片复核义务）。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [x] **B-AVSERVICE-I｜M2-B｜owner_stage: work｜AC-B-AVSERVICE/AC-02/03/06**：动态路由、参数、认证/权限/中间件、上传/下载/stream与DB接ACE，保transport现有实现。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @76c8001（auto-web plan-983-dev）] 新增 `av_service` 模块：`AvServiceEngine`（console 前门编译驻留 + Win32PlatformIo 与 `Rc<AutoVM>` 同律驻 owner 线程 + `#[api]` 路由抽取（剥属性行后交前门）+ 文件通道：每请求 staged 原始 HTTP 请求文件→`run_platform`（grants platform.file/mem；与 AV 平台账"每调用强制关句柄"纪律天然兼容）→响应文件读回）。http_server 增 `ServiceEngine` 枚举（Legacy 段驱动全量保留 / Av 同步完成永不 park——parked 机制自然旁路）与 `serve_av` 公共入口；server_dispatch 增引擎级派发 + AV 臂（CORS/404 与 legacy 逐字同形；**具名 501 边界**：已注册中间件链/上传路由——语言侧凭据消费通道缺位；限流=legacy 专有 VM launch settings 面）。**冻结设计依据**：AV 公开面无「具名函数+参数+平台IO」联合入口（入口参数仅标量无 Str、每调用 mint 新 ledger 强制关文件、session/snapshot !Send）——侦察报告已核 av-core av.rs/av.rs resident/session。永久 fixture `tests/fixtures/ace-p983/av-runtime-service.at`（member-delivery/b API 面 + counter.txt 持久化 + #[api] 声明）+ http_runtime 新用例。认证/中间件/上传/stream/SSE/`~T` 面：具名未执行（R 条款），非 NA。

- [x] **B-AVSERVICE-V｜M2-B｜owner_stage: work｜AC-B-AVSERVICE**：运行V-B-AVSERVICE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @76c8001] V-B-AVSERVICE 实测：`nextest -p auto-web --test http_runtime` = **6/6 绿**（7.22s；含 legacy installed-host 双例证 transport/legacy 臂零变化 + A 腿两例仍绿）。新用例 `av_service_engine_serves_axum_transport_over_real_tcp`（0.407s ≤10s 门）oracle 全实测：真 TCP 经既有 axum transport + AV 引擎执行；写 DB 再读（add7→get7→add3→10→double20→+100=120）；并发到达经 owner 串行不串上下文；断连（半请求+关）不触业务态；404 与 legacy 预处理同形（空体+请求 id 头）；隔离 DB 文件测试直读（counter.txt="120"）；外部 stop→排水→端口重绑+service 完结。lib 186/187（1 红=具名既有红）。lang `cargo t http`：lang 仓零改动，按 V 行"仅语言相关改动"条款免跑（记录）。

- [x] **B-AVSERVICE-TEXT｜M2-B（2026-11-18 用户裁定切片）｜owner_stage: work｜AC-B-AVSERVICE/AC-02/03/06**：把 982 的受检文本入口接到 **auto_web 生产栈**——同一份已编译 handler 直接收查询串 + UTF‑8 体（`fn main(query str, body str) int`），真实 TCP 路由；请求不重编译、不经文件；断言 `av_dispatches>0`、`legacy_dispatches=0`、产物指纹不变并跑现有服务回归。文本参数 + 平台 I/O 的组合调用与完整 JSON 不在本切片（见证据/§10）。
  [✅ 2026-11-18 @3f8de15（auto-web plan-983-dev；lang 组树 ff `850a49bf8` 零代码改动，规范提交 `b016ff9d3`）] 交付：①新 `av_text_service` 模块——`AvTextServiceEngine::new` 启动时**编译一次**（Text profile + `ac.runtime.text.v1` → `lower_checked_owned` → `verify_mir` → ABC encode/decode → `verify_abc`；`compile_count` 恒 1、ABC 指纹固定），请求级只调 `ace_av_core::av::execute_entry_text(&[query, body])`（982 受检入口；文本在本次执行 arena 内借用，非法 UTF‑8 在执行前具名拒）；查询串取原始 target 的 `?` 之后并按 `%XX`/`+` 解码（传输层给的是百分号编码 ASCII），体为原始 UTF‑8 字节；`int` 返回 ⇒ 200 + 十进制文本，Cancelled/Resource ⇒ 504，Trap/Error/Infra/Exit ⇒ 500 具名；`AvTextReceipt` 暴露 compile/av/legacy 计数与产物指纹。②接线：`http_server::serve_av_text` + `ServiceEngine::AvText`（transport/owner 线程原样、`legacy()`=None、无 legacy 路径），`server_dispatch` 文本臂——中间件/上传 501 与未命中 404 与文件通道 AV 臂逐字同形，miss **不派发**。③永久 fixture `tests/fixtures/ace-p983/av-text-service.at` + `http_runtime.rs` 新用例 `av_text_entry_reuses_one_compiled_handler_and_branches_by_content`（真实 TCP）。**实测**：等字节长度 `中文`/`日文`（各 6 UTF‑8 字节）按内容分流——`GET /text?<中文>`→3、`?<日文>`→4、`POST /text` 体 中文→1/日文→2、查询+体组合体优先→2；`/text?%FF`→500 `not valid UTF-8`（执行前拒）；`/missing`→404；收据 `av_dispatches=6 legacy_dispatches=0 compile_count=1 artifact_fingerprint=9db3cce95d63f38e entry=d_main`（6 次=5 成功+1 入口内受检拒绝；请求间指纹不变=同一编译产物）。**回归**：`--test http_runtime` 7/7 绿（7.48s；新例 0.60s ≤10s 门）、`--lib` 188 例 187 绿/1 红（21.39s，红=基线既有 `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，非本切片引入）、`--test http_generation` 3/3 绿（176.43s，默认道不变）。**未做（具名）**：①文本参数 + 平台 I/O 同一次调用——983 首消费者本轮只用文本，未触发；触发时按用户指令交 982 共享入口验证补齐（982 收据仅覆盖文本参数）；②完整 JSON/嵌套/浮点/字符串/请求头——按首个真实消费者字段推进；③中间件/上传/stream/SSE/`~T` 仍具名 501。**规范**：`m2-web.md` 增「B-AVSERVICE 文本入口」节（lang 树 `b016ff9d3`；已随 merge rebase 落主 `043542b2e`/`1af135c31`，见 §9 merge 收据）。

- [x] **B-AVSERVICE-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。**复核面含 2026-11-18 文本入口切片 B-AVSERVICE-TEXT**（`av_text_service`/`serve_av_text`/fixture/真实 TCP 用例与收据断言）。
  [✅ 2026-11-18 review pass @3f8de15] 见 §9 复核收据（review_scope=phase，含文本入口切片）。冻结基线复跑：http_runtime 7/7（新例 0.172s）、lib 187/188（1 红=基线既有 `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，文件不在 diff）、http_generation 3/3；`git diff --check` clean、新文件 0 warning/0 调试残留。F-R3（规范状态行陈旧任务计数）→ work 修复 `b952cc9a4` 后复验；无 in-contract 缺陷。落主：auto-web master `3f8de15`、auto-lang master `1af135c31`（见 §9 merge 收据）。B-GENERATED-R 与 T-05..07 仍独立执行。

- [x] **B-GENERATED-I｜M2-B｜owner_stage: work｜AC-B-GENERATED/AC-02/03/06**：新HIR A2R生成服务/共享载体，保route/store/schema/diagnostics与失败保旧；本批可与AVSERVICE在端口稳定后独立。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @6808511] lane 子集扩面：`int`/`str` 视图/`bool` **参数**（值传；a2x `l{idx}_` 局部名还原源参数名）与同族返回；owned/mut str 参数、record/list/浮点具名拒（后续扩面；`str+literal concat 产 owned Str 与 str 返回不匹配`=verify 具名拒已实证，恒等体为子集内形态）。api_gen：端点门放宽至参数类型 {int,str,bool}；`generate_ace_api_rs` 发射参数提取器（GET/DELETE=Query struct、POST/PUT=Json body，与 legacy 委派形态同源）+ 实参编组（int `as i32`/str `as_str`/bool）。保旧面：route（路由注册沿用端点驱动）、store/db（ACE 道仍具名拒 db.at/伴生——默认道 store 支持不变，member_delivery 由 V 证）、schema（types.rs/generation.json 字段不变）、diagnostics/失败保旧（早门零写入+receipt park+write_if_changed 原样；坏源负例改 scale(float) 具名拒+旧 workspace manifest 字节不变）。fixture 扩 echo/whisper/pick 参数端点。

- [x] **B-GENERATED-V｜M2-B｜owner_stage: work｜AC-B-GENERATED**：运行V-B-GENERATED及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @6808511] V-B-GENERATED 实测：`nextest -p auto-web --test http_generation` = **3/3 绿**（73.5s）。新例 `ace_lane_param_endpoints_execute_over_real_tcp`（6.45s ≤10s 门）：生成→参数 glue 断言（fn_echo(body["delta"]..)）→--offline 冷编译（真实 cargo 子进程终态）→真 TCP：echo(41)→82、whisper("p983")→"p983"（str 参数编组）、pick(true)→true（bool）→202 shutdown join+端口释放；原例（6.21s）保持含坏源负例（scale(float) 具名拒+保旧）。既有 `cold_generated`（legacy 道，60.7s）与 lane 单测 4/4 绿=默认道不变。lang 组（组树零改动非回归证据）：`cargo t auto-generation-core` PASS + `cargo t web-delivery-controls` PASS（冷建 3m16s，组树 @d43e7970+master 前移）。

- [x] **B-GENERATED-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。
  [✅ 2026-11-18 phase review pass @3f8de15（委派独立复核 `gen-reviewer`）] 从 2026-11-17 `fd115ce` 旧双批收据出发：`fd115ce→3f8de15` 八文件增量不含 `ace_a2r_lane.rs`/`api_gen.rs`/`http_generation.rs`/`auto-generation-core`，生成服务 `main` 直接 `axum::serve`（`api_gen.rs:5310`）不构造 `ServiceEngine` ⇒ 不触生成验收结论；AC-B-GENERATED 四项逐条 pass（冷编真 TCP/独立 expected/坏源保旧/子进程终态无自报）。证据：复用同 SHA `http_generation` 3/3（118.56s）＋定向复验 lane 单测 4/4（0.122s）、参数例 9.702s、生成+坏源例 6.793s；F-G1（低，非 AC 缺陷）首跑 19.46/22.53s>10s 系冷建+并发盘压，稳态 ≤10s，owner=B/CONSUMERS 预构建（r7 已名）。见 §9 复核收据。

- [ ] **B-CONSUMERS-I｜M2-B｜owner_stage: work｜AC-B-CONSUMERS/AC-02/03/06**：jade-garden与既有Web生产调用逐行接新SDK；每owner代码独立交付，消费相同source/tool版本，保旧四仓义务。 在§5改面接永久fixture，先冻结前置接口/命令。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。
  [C0-A 已交 2026-11-18 @211b6d5（auto-down）/ @56b3ac5（auto-web）] 见 §5「C0 首例冻结」：可执行 AV 业务源 `jade-garden/back/auto/av/workspace_service.at`（env→UTF-8/JSON→响应；Windows join `\wiki`；只读）+ `api.at` 指针；auto-web `AvServiceEngine::new_with_grants` 显式 grants；真实 TCP 直连用例（源读消费者仓、不复制）——`--test http_runtime` 8/8（7.92s；新例 0.215s），字节级 200 JSON / null 形态 / 404 派发前 / 重复稳定 / shutdown 端口重绑。
  [C0 CLI 已交 2026-11-18 @auto-man 6e21bfa / auto-web 228d775] 982/EXEC：`auto service --server av` 改走 auto-web `serve_av`+`AvServiceEngine`，`--api` 显式源，pre-C0 宿主保留 `av-direct`；`bind_control_plane_shutdown`+收据（av/legacy 计数）。真 CLI 用例 `plan983_c0_cli`（0.37s）：bind/ready/字节 JSON/shutdown 202/退出 0/端口重绑/`av_dispatches=2 legacy_dispatches=0`；`av-direct` 旧用例 17/17+2/2 仍绿；`ace-candidate` 编译绿（lang ff55579c3）。
  [C0-B 已备+探针 2026-11-18 @auto-down b830e07] e2e 外部后端模式（`JADE_E2E_BACKEND_URL`）+ `01b-workspace-av-consumption.spec.ts`；临时探针实测页面消费真实 `/api/workspace`（200/root、#app 挂载、opener 0、无 console error）。**仍未勾**：永久 playwright 跑（worktree 无 node_modules）、U0、四仓整体（默认特性 man 门被外部 auto-ui 红挡住）。
  [C0-B 正式跑 + C1 基线 2026-11-18] `01b` spec 在 AutoDown 主检出 Playwright/Vite 上 **1 passed**（6.2s/11.5s）：真实 `GET /api/workspace` → 200 + root/wiki_dir，主视图断言过；后端 `av_dispatches=2 legacy_dispatches=0`。C1 基线实测冻结（400 零副作用 / canonical `\\?\` / wiki+index+config / 幂等），实现待 982 组合入口。

- [ ] **B-CONSUMERS-V｜M2-B｜owner_stage: work｜AC-B-CONSUMERS**：运行V-B-CONSUMERS及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。
  - 当前先执行 §5「四仓门禁修复」，取得语言固定组和进程审计通过的收据后复跑完整四仓门禁。

- [ ] **B-CONSUMERS-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。


## 9. 复审记录

2026-11-19 stage: work | PLAN-983 r24（**四仓门禁修复复跑：语言侧全绿里程碑 + web 腿 9 红逐项归因（均非 983 触面）**）| 本轮新增修复：①`plan707_spike_cancel_gap` **确定性重设计**（lang master `20f1af3a0`）——窗口假设被 8s 仍红证伪后定位为 tokio abort/并发唤醒固有窄竞态（隔离 3 绿/门内 3 红）；改 drop 观测：future 内 DropGuard 置位，测试先等"取消已生效"（有界 20s 只容饥饿）再开 gate ⇒ post-gate poll 结构性不可能，宽限 8s→1s；oracle 严格更强（取消缺失=超时红，构造性）；隔离 1.1s 绿。②`plan724_cancel_storm` 许可归还窗 5s→20s（`06f7c9993`；真泄漏仍红/正常路径提前退出）。③ui 组树锁刷新（auto-ui 组树 plan-983-dev +1 提交：`cargo generate-lockfile --offline` 后提交，man bootstrap 的 `--locked` 构建过）。| **门复跑收据**（组树：lang=`20f1af3a0`、web=`3d328a3`、down=`72b0970`、man=`ac32b61`、ui=master+lock）：**语言固定组 6/6 全过**（组 0 3408 例 3407+1 修复后绿；plan988 拆分组 50.4s；receipt.json 记录六组 identities/sha）＋ **lang-web-e2e 腿过** ＋ **bootstrap-0/1 过**（auto-ui-tools `--locked` 构建成功）⇒ **门首次抵达 web 外部腿**。**web 腿 241/250，9 红逐项归因（预存，非 983 触面——983 的 auto-web 改动仅测试文件+计数器，零 Cargo/闭包变化）**：①`api_gen::tests::test_handler_body_r2_matrix_compile_oracle`＝**规范登记的既有基线红**（m2-web.md 边界节自 `0adebc0`）；②`member_delivery`×5＝PLAN-1006 冻结闭包布局表拒识 `ace-a2x-core -> experimental/ace/crates/a2x-core`（逐字："闭包源布局不识别"）——auto-web lib 的 ace 依赖使导出闭包含 ace crate 而布局表只认 `crates/{name}`；owner=auto-web delivery 线（1006/982，扩表或闭包排除为产品决定）；③`e2e_plan696_real_{015,017,023}`×3＝读 `../../../auto-lang/examples/ui/<nnn>`——**lang 主检出的未跟踪工具生成例**（git 只跟踪 examples/ui/README.md；干净组树无这些目录）；owner=ui/tooling 线（物化流程）。| 测试计时：门内语言 6 组（组 0 ≈158–190s）+lang-web-e2e+bootstrap（0.77s）；web 腿 566.8s/250 例。| **阶段判定**：语言侧门禁修复**全闭**（r23①②+r24 全部落地）；四仓门受阻于 web 腿 9 个预存红（1 登记+5 布局表+3 例物化，均外部 owner）。| next: delivery 线处置布局表、ui 线处置例物化（或门侧具名排除裁定）→ web 腿复跑 → ui/man 腿 → B-CONSUMERS-R
2026-11-19 stage: work | PLAN-983 r24（**814 路径依赖身份修复（新 phase）**）| 用户指令：814 在 983 新开 phase 修——1034 `7db528b93` 把 auto-tool-execution 加成直接路径依赖、build.rs `IDENTITY_SOURCE_CRATES` 漏了它；补入该 crate 并复跑 814 依赖覆盖与 SDK 边界负控、保留现有守卫；985/982b 用户到对应会话修。| 交付：lang master **`45afda45d`**（rebased 至当刻 master `f747a5bcd`=985 线 pre-landing 重钉之上；`build.rs` 清单 +4 行含归因注释）。| evidence：`plan814_build_input_boundary` **3/3 PASS**（`declared_path_deps_all_in_identity_sources` 恢复绿 + `assembly_contract_source_is_in_sdk_identity_without_backend_sources`/`ui_man_sources_never_enter_sdk_identity` 两负控保持）；`stdlib_assembly::manifest` 全模 **7/7 PASS**（无指纹连带）；rebased tip 复验 3/3；wt-guard clean 后 ff 落主。| **修后 master 红面**：985×5+982b 仍红（本会话实测：`p982b_node_identity…` left `kernel` right ``；`p985_real_matrix…` stale receipt 同 r23 逐字证据）——985 线已在途（`f747a5bcd` resync/pre-landing re-pin）、982b 归 981/982 会话。| 测试计时：plan814 模 0.025–0.065s、manifest 模 0.087s；新增用例 0。| 阶段判定：814 子项**闭**；四仓门 unblock 剩 985×5+982b（外部在途）。**r24 归因修正（2026-11-19，985 会话反馈+本会话组树实测）**：985×5 在干净树（组树==master `45afda45d`）**全绿**——钉值已随 `f747a5bcd` 重钉至 HEAD 冻结字节；主检出红=**971/982-B 会话在 `m2-scheduler.md` 上的未提交 WIP 叠加**（磁盘≠冻结收据，陈旧检测按设计报警；该 WIP 同时是四仓门主检出预检受阻源），消除条件在对方会话（提交后 985 常规重钉 / 撤走即绿），非 985 缺陷、r23"1067 未刷钉"表述据实修正（当时的提交态陈旧已被 985 重钉收口）。**四仓门语言组当前唯一剩余红=982b**（981/982 会话）。| next: 982b 落地后四仓门复跑 → B-CONSUMERS-R。| next: 985/982b 落地后四仓门复跑（plan988 拆分与 814 修复均已在 master）→ B-CONSUMERS-R
2026-11-19 stage: work | PLAN-983 r23（**四仓门禁修复阶段：第 1/2 项完成并落主；正式门第 3 项执行、受阻于他线新落主红**）| 用户指令：B-CONSUMERS-V 内置「四仓门禁修复」——①按语义拆 plan988 长用例保全场景；②定位并修复 vctip 残留；③组审计过后再跑正式四仓门，完整收据后交 B-CONSUMERS-R。| **①超时（完成，lang master `722dfdcd5`）**：`plan988_byte_consumers.rs` 拆为五个语义族 `#[test]`（`write_paths`15/`push_mutation`10/`replace_assignment`10/`typed_context`6/`json_values`9）+ **完整性守卫**（族并集==全部场景，漏挂/重复即红）；场景表与执行体**逐字保留**（50 场景名/源/期望/VM 拒绝位/VM+生成 Rust 双腿+字节对照+非法输入保旧/FIRST 保旧）；`PLAN988_BYTE_CASE` 重放钩子保留。实测：**50/50 场景逐名 PASS**（`--nocapture`）；组墙钟 106–118s→**50.4s**（族并行）、单族最大≈33s≪60s 警告线；**共享门配置零改动**（r22 预算 override 已 revert——`ff86c72bc`+`0090c61dc` 相互抵消，rebase 落主时只带拆分提交）；rebased tip（基 master `1fbc9b29e`）上 `cargo t auto-generation-core` 判决 **PASS**。**②vctip 残留（完成）**：根因=该测试 `rustc`→`link.exe`（MSVC 14.44）派生 `vctip.exe` 遥测上传器（独立生命周期，workload 退出后残留；r22 两次实录：pin 校验有效、Job reaper 清后 remaining=0 但 workload 规则"never a workload exception"）；修复=**机器级遥测 opt-out**（`vctip -disable` 官方开关，`-enable` 可逆；树内击杀被否：非直接子进程且 racy）；验收=组判决 **PASS**（测试+进程审计，受管进程 workload 退出归零）×多次复现。**③正式四仓门（执行，受阻）**：组树启动（lang=`722dfdcd5` 落主前树、web/down/man=master、ui=`d188d695`）；语言组 0（auto-lang+auto-lang-core，3407 例）**3400 过/7 红** ⇒ 门中止（plan988 组未达门内，但其独立判决已 PASS）。**7 红逐项归因（主检出复现 0/7 同红——master tip 既有，非组树/非 983 触面）**：`p985_*`×5=PLAN-1067 `b28399cc9` 更新 `m2-scheduler.md` 未刷 985 obligation-ledger L-29 行钉（stale receipt，逐字证据："sha256 mismatch: pinned 869aaf24… actual 5bdd5aea…"）；`p982b_node_identity…`=断言"separate id field stays empty"得 `kernel`——981 跨模块节点身份落地（`3047b3481` 系）改变预期；`plan814_build_input_boundary…declared_path_deps…`=现行态核对同族。**归属：1067/985 线（台账刷新）、981/982 线（节点身份预期）、814 线**——即 #9「新语言基线主树复跑」的分诊面（用户已具名为独立收口项）。| **环境记录**：r22 门跑时组 0 曾全绿——7 系昨夜后 981/982/1067 落主引入。| 测试计时：组=50.4s（拆分后）/组 0 门内 178.5s（3407 例）；新增用例=完整性守卫 1（<0.1s）。| **阶段判定：1/2 完成、3 受阻（他线红，具名）**——B-CONSUMERS-V 保持未勾；unblock=各 owner 修红（或 #9 批量分诊吸收）后复跑四仓门。| next: 跟踪 985/982b/814 三线修红 → 四仓门复跑 → B-CONSUMERS-R
2026-11-18 stage: work | PLAN-983 r22（**四仓验收（V-B-CONSUMERS）执行收据：blocked-基础设施**；U0 已按用户裁定以 980×982 收据接收、批量回归已按用户裁定跳过）| 运行：`python scripts/web-four-repository.py --out plan-artifacts/plan-983/web-four`，五次起跑——①②主检出预检拒（lang 主=980 记录在案的 #9 半截 WIP 五文件；web 主=rc989 停驻副本未跟踪）；③清理后 ui 主拒（junction 穿透）；④根集不一致（布局校验 a2r-std owner mismatch）；⑤**全组树**（lang=`3047b3481`=master tip、web=`3d328a3`、down=`72b0970`、man=`ac32b61`（皆=master）；**ui=`d188d695` 具名偏差**（master=`01f85061` 的差=980 后续合并；主检出被其 theme/stylekit tracked WIP+junction 阻））通过预检/布局并实跑。| **门结果**：语言固定组末尾 `auto-generation-core` 45/46、`plan988_byte_consumers` **120s 超时被杀**（nextest slow-timeout 60s 警告→120s 终止；门自有预算）⇒ `fixed language groups failed`，web/ui/man 外部腿未执行。**隔离复跑（`cargo t auto-generation-core` ×2，机器空闲）**：46/46 全过（1 slow），但两次均 `vctip.exe` workload 尾残留（exit=0、cleanup_complete=true、remaining_active=0、pin sha256 匹配 `compiler-auxiliaries.json`）⇒ 组判决 FAIL（策略：辅助进程豁免仅限 preparation，workload 永不豁免）。| **归因（具名，非 983 触面）**：①plan988 长原子 60–115s（真实 cargo/cl 子进程编译），门内负载下破 120s 预算——owner=988/989 线（AGENTS"特别长原子具名交 owner"）；②vctip 尾检=机器/工具链级时序（11-16 批回执曾"TIMEOUT 后 PASS"，同族）——门 owner 侧；③主检出在途=#9（lang 五文件）与 980（ui tracked WIP+junction）各线收口项。| **983 侧无代码改动**（本仓 lang 侧 r20 后仅 docs；门跑的是组树=各 master）。| **环境处置终态**：rc989 停驻族迁 `plan-artifacts/orphans/`（web 1.5M+四仓 6.6M，内嵌 .git 保全）；误搬混合副本隔离 `orphans/_p983-quarantine/`（118M，含 058 探针 `tests/plan058_t02_probe.rs` 可取回）；auto-ui tracked WIP stash→**已 pop 还原**、锁 churn stash→**已 pop 还原**；`NUL` 保留名垃圾已删；四主检出工作面恢复原状。| **unblock 动作（交各 owner）**：plan988 拆分/预算调整（988/989 线）或门预算复核（门 owner）；vctip 策略判定（门 owner：机器级禁用 vctip 或 workload 豁免辩论）；#9/980 主树收口后可主检出复跑全门。| 测试计时：门=语言组阶段（未完整）；隔离组 106s/115s（46 例）。| B-CONSUMERS 保持未勾（四仓门未绿）；next: 各 owner unblock 后复跑四仓门→B-CONSUMERS-R→T-05 final。
2026-11-18 stage: merge | PLAN-983 r21（**T-06 phase fold：C0/C1 + 文本路由交付落四仓主**；整包不归档，树保留待 U0）| outcome: pass（phase）| prepared: T-05 phase pass（§9 上条：新鲜重放 c1 6/6+c0 1/1 于 `7dd8d76`）+ **F-R1 规范增量已备**（lang 组树 `17af620d4`：m2-web.md 增「C1 工作区提交与失败语义」「文本路由消费形」两节 + 边界行更新；docs-only 后裔）| landed: ①**auto-web master `3d328a3`**（ff-only，=复审哈希，http_runtime 9/9 证据按同哈希复用）；②**auto-down master `72b0970`**（ff-only，=复审哈希；C0-A/B+C1 六提交）；③**auto-man master `ac32b61`**（**rebase 后 ff**：master 含 982 认领副本 `bf52911`，`6e21bfa` rebase 后判空被跳过（全量已上游）、余 7 提交 range-diff **全 `=` 等价**（旧→新：`36f6df6`→`e32de77`、`f746c74`→`bb6d6b0`、`867e913`→`52c3df6`、`259ef8b`→`1acb67d`、`58fc2b3`→`f48a832`、`11b347b`→`9e2d723`、`7dd8d76`→`ac32b61`）；冲突消解=三文件（main/cli_spec/cmd_service）全取 HEAD（master 演化版=超集：`--stack` 面）；**受影响验证在 rebased tip 复跑**：`plan983_c1_cli` **6/6（1.11s）**+`plan983_c0_cli` **1/1（0.46s）**；wt-guard clean 后合并；主检出脏 Cargo.lock（与分支携带版同内容 f0fbd36）还原后由合并带回同字节）；④**auto-lang master `286c9cb53`**（规范提交 rebased 自 `17af620d4` 基 `79a50097c`（PLAN-981 前移），docs-only 无冲突，ff-only）| canonical_verified: `docs/specs/ace-migration/m2-web.md`@lang master `286c9cb53`（+55/−2：C1 合同节+文本路由消费形节+CONSUMERS 边界行；landed 内容=reviewed delta）| **archived/cleaned=不适用**（phase fold：Plan 保持 `executing`，四仓 worktree/分支保留待 U0/四仓；`plan-artifacts/plan-983/` 维持在途 scope（parse-probe/auto-ui-lock-churn 副本等，registry 不变））| 环境观察：auto-web/auto-down 主检出各有他人 `?? .wt/` 未跟踪目录与 auto-web `M README.md`（非本计划，保留未动）；lang 主检出会话始起即有他线 docs WIP（未触碰）| **批量回归到期移交**：`.last-batch-regression.json` 回执 2026-11-16T10:04Z（>48h）且其后有 981/982 合并与本次 fold ⇒ **DUE**——按规程移交 `/auto-plan:regress` 主检出自单实例执行；既有具名红（infra-fixed ace-facade 期望行，owner 969 线）随批处理 | merge 会话测试：c1 6 例 1.11s + c0 1 例 0.46s（rebased tip 新鲜）；其余复用（同哈希或 docs-only）；新增用例 0 | next: U0（等 980 WEB 正式交付+982 Vue `auto run`）→ 四仓整体 → B-CONSUMERS 收口 → T-05 final/T-06 终态/T-07 清理；近项=批量回归（regress）
2026-11-18 stage: review | review_scope: phase（**T-05 首审：C0/C1 + 文本路由交付**；整包 final 被 U0/四仓在途挡住——具名开放，不判 final）| reviewer_identity: 实现会话内复审（局限声明：无独立 reviewer 可派；结论从工件重建——新鲜重放、分支 diff、逐字收据，非采信执行者总结）| plan_id: PLAN-983 | plan_revision: r21 | outcome: **phase pass**（整包保持 executing）| worktree: `.wt/lang-983/{auto-man,auto-down,auto-web,auto-lang}` | branch: plan-983-dev（lang=master 同步 0 提交）| reviewed_commits: auto-man **`7dd8d76`**（master..HEAD=8 提交：`6e21bfa` C0 CLI→`7dd8d76` r21）、auto-down **`72b0970`**（=6 提交：`211b6d5` C0-A→`72b0970`）、auto-web **`3d328a3`**（=1 提交，基=master `5e4322a`）、lang 组树 `0a167f611`（=master，含 982 组合入口/具名入口放行）| dependency_revisions: 982 文本路由引擎=auto-web master `5e4322a`；lang `84f2ad773`/`600d31527` 均在组树 | spec_inputs: `docs/specs/ace-migration/m2-web.md`（现含 A/B 批、文本入口、C0 公开面节）| acceptance_results: **C0/C1 相全部 pass**——①C0-A/CLI/B 三腿（r11–r14 收据，部分已随 r13 fold）；②C1 五例+并发：**复审会话新鲜重放**（非复用）`cargo test -p auto --test plan983_c1_cli --test plan983_c0_cli` ⇒ **6/6（1.09s）+1/1（0.85s）** 于 `7dd8d76`；逐字收据见 §9 r21（并发：窗口内配置恒可解析∈{A,B}、到达序 A→B、终态=后到者、重启=最后提交）；③文本路由区分性证据：auto-web `http_runtime` **9/9（2.60s）** 于 `3d328a3`（同一提交证据复用：POST 先行+暂存文件缺席+combined==1；GET 文件通道后请求文件在场）；④健康：av_service.rs 零 warning；man `http_service.rs` 的 2 处 warning（:110 unused imports、:1458 interrupt_sent）经 master..HEAD diff 核对为**既有**（分支 diff 只有 grants/transport 映射/receipt 打印）；改动文件 rustfmt clean；无调试打印；单例计时均≪10s。| **findings**: **F-R1（T-06 前置，阻塞合并不阻塞本 phase 判定）** canonical 规范增量未备——m2-web.md 无 C1 语义（canonical/Unicode/索引/原子提交/两具名前缀）与文本路由消费形（`#[api]` 单 `str` 参=文本路由/0 参=文件通道、combined 计数、分道证据）章节；须在 merge 准备期补写。**F-R2（合并冲突面，具名）** man master 已含 982 认领副本 `bf52911`（=本分支 `6e21bfa` 的 cherry-pick）——plan-983-dev 合并会在 serve_av/cli_spec 等处冲突；解法=rebase 到 master 后内容等价核对（982 侧可能已演化，逐块比对取语义并集）。**F-R3（具名边界确认，非新发现）** mtime 恒 0、blocks/links/tags 恒 `[]`、GET 走文件通道、并发到达序为观测证据、`dir_close` 由宿主 ledger 收——均已在源注释+§0/§9 登记。**F-R4（整包门，未闭）** U0（等 980 WEB 正式交付+982 Vue `auto run`）与四仓整体未做 ⇒ B-CONSUMERS-I/V/R 保持未勾、整包不得 final pass。| 测试计时表（复审会话）：c1 6 例 1.09s、c0 1 例 0.85s（新鲜）；http_runtime 9 例 2.60s（复用，同一提交 3d328a3）；新增用例 0。| pending_closeout_gates（merge-owned）: F-R1 规范增量、F-R2 冲突消解、三仓分支合入（man 8/down 6/web 1 提交）、冻结受影响组复跑（合并前）、U0/四仓（后续相）| next: T-06 phase fold（先备 F-R1 规范增量，再按仓合入+核消费者同版本；树保留待 U0）
2026-11-18 stage: work | PLAN-983 r21 | outcome: pass（**C1 收口：真 CLI 复跑收据（绑定修订）+ 并发提交顺序/一致性补证**；B-CONSUMERS 整批仍未勾，U0 待 980/982）| 用户指令：先收 C1——已有 5/5 收据对应旧文件通道，本轮只核对了源码、尚未见新接线的真 CLI 复跑收据；另补证两次并发切换的提交顺序与期间 GET 状态一致。| worktree/commits: auto-man plan-983-dev **`7dd8d76`**（并发用例强化 + Cargo.lock 随 lang 组树 ff 的解析更新；auto-web `3d328a3`/auto-down `72b0970`/lang `0a167f611` 未动）。| **真 CLI 复跑收据（逐字，`cargo test -p auto --test plan983_c1_cli -- --nocapture`，6/6，1.83s）**：`✓ C1 receipt: POST open → GET → restart → GET share one persisted config (env fallback C:/Users/zhaop/AppData/Local/Temp/p983c1bpSqiq/workspace-a, persisted C:/Users/zhaop/AppData/Local/Temp/p983c1bpSqiq/workspace-b)`；`✓ C1 index-failure receipt: 500 named, old workspace/config/index preserved; target wiki created = true`；`✓ C1 concurrency receipt: 2 consistent GETs (config always one complete commit) during concurrent A/B switches; observed POST arrival order A->B; final=\\?\C:\Users\zhaop\AppData\Local\Temp\p983c1xI2B3if\ws-b (root_a=C:/Users/zhaop/AppData/Local/Temp/p983c1xI2B3if/ws-a, root_b=C:/Users/zhaop/AppData/Local/Temp/p983c1xI2B3if/ws-b); restart keeps the last commit`。`plan983_c0_cli` **1/1（0.48s）**（GET-only ⇒ `combined_entry_dispatches=0`）。| 并发补证（新增确定性断言）：hammer 每次 GET 后直读配置文件——**可解析**且 `workspace_root` ∈ {canonical A, B}（预热已提交 ⇒ 文件恒存在；tmp+MoveFileEx 原子替换 ⇒ 任意时刻磁盘上是一笔完整提交，无 torn 状态）；到达序观测（记收据不断言：单 owner 顺序派发下响应写出序=提交序，但客户端线程调度理论上可换到达序）本轮 A→B 且 final=B（后到者=最后提交，自洽）。| 测试计时：c1 6 例 1.83s、c0 1 例 0.48s（单例≪10s）；新增用例 0（既有并发用例的断言强化）。| 未做：U0 实施、B-CONSUMERS 其余端点/四仓整体、mtime 真值。| next: U0 前置跟踪（980 WEB/982 auto run）；B-CONSUMERS → T-05/06/07
2026-11-18 stage: work | PLAN-983 r20 | outcome: pass（**改用 982 文本路由引擎，C1 五例+并发在真实产品入口分道复跑全绿**；B-CONSUMERS 整批仍未勾，U0 待 980/982）| 用户指令：980 的 002-counter 预验收与 F32 同窗已落主；983 采用 982 刚交付的文本路由（auto-web `5e4322a`）把 C1 五例跑在真实产品入口上——保留 JSON 请求、缺目录 400、索引与配置原子提交；POST 后 GET 前确认没创建请求暂存文件（避免把 GET 的文件通道算进去）；先收 C1 再补同一服务实例的并发 POST/GET 验收；B-CONSUMERS 保持未勾。| worktree/commits: auto-web `.wt/lang-983/auto-web` plan-983-dev **`3d328a3`**（分支自 `0029636` **重钉 master `5e4322a`**——r19 整请求文本设计退役，被 982 按路由文本入口取代；叠加 combined 计数 + c0 用例 POST 先行/缺席断言）；auto-down plan-983-dev **`72b0970`**（`api_workspace_open(body str)` 文本路由 + `main()` 文件通道恢复 + POST 文件通道回退保留）；auto-man plan-983-dev **`11b347b`**（shutdown 返回收据；POST 服务断言 combined≥1、GET-only 断言 combined=0）；auto-lang 组树不变（`0a167f611` 已含 982 具名入口放行 `84f2ad773`）。| 引擎分道（982 `5e4322a` 契约）：`#[api]` fn 形参恰为单个借用 `str` ⇒ 文本路由（请求体→`execute_entry_text_platform` 实参，每路由驻留 `d_<fn>` 模块，不写请求暂存文件）；0 参路由 ⇒ 文件通道（staged 请求 + `run_platform` 走 `d_main`）。| evidence: ①man `plan983_c1_cli` **6/6（1.16s）**——五例（含 Unicode/canonical/索引失败/config 只读）+ 并发一致性全部经真 CLI 在分道引擎上绿；每例的**首个（含 POST 的）服务**收据断言 `combined_entry_dispatches≥1`，重启（GET-only）服务断言 `combined=0`；②`plan983_c0_cli` **1/1**（GET-only ⇒ `combined_entry_dispatches=0` 断言）；③auto-web `http_runtime` **9/9（2.60s）**：c0 用例**POST 先行**——文本路由 200 canonical + **`av-service/p983-req.txt` 不存在**（无文件通道派发时点）+ `combined==1==av` + resp 文件在场；GET 三次后收尾 `combined` 停 1、`av≥4`、请求文件**在场**（分道并存）；982 自带 `c0_workspace_open_p982` 与 member-delivery（文件通道）同场绿。| 过程修正（一处实测）：收据断言初版挂在 shutdown() 内对所有服务生效——重启服务只做 GET（combined=0 正确）被误判红；改为 shutdown 返回收据、按"该服务是否做过 POST"分别断言。| 测试计时：c1 6 例 1.16s、c0 0.55s、http_runtime 9 例 2.60s（单例≪10s）；本会话新增用例 0（既有用例的断言重构+引擎切换复跑）。| U0 备案：980 已录 002-counter 真挂载（9/9 handler 走 ACE lane）；983 复用其浏览器断言、启动改真 CLI；正式验收等 980 WEB 正式交付 + 982 Vue `auto run`（产品链 TS 转译/a2x_rt 接线确认）——未开工。| 未做：U0 实施、B-CONSUMERS 其余端点/四仓整体、mtime 真值。| next: U0 前置跟踪（980 WEB/982 auto run）；B-CONSUMERS → T-05/06/07
2026-11-18 stage: work | PLAN-983 r19 | outcome: pass（**C1 生产接线到组合文本入口 + 并发合同用例**；B-CONSUMERS 整批仍未勾，U0 待 980/982）| 用户指令：①先接 C1——真 `auto service --server av` 把请求体作为借用 `str` 传入 `execute_entry_text_platform`，复跑 C1 五例/C0/http_runtime 并加断言证明组合入口确实被调、请求暂存文件从未创建；②验证并发提交（同实例并发切换 + 穿插 GET 一致、重启读最后成功提交）；③并行准备 U0（复用 980 浏览器断言、启动改真 CLI；最终收据等 980 WEB 正式交付与 982 Vue auto run，确认产品链承担 TS 转译与 a2x_rt 接线）。若只做一件=①。| worktree/commits: auto-web `.wt/lang-983/auto-web` plan-983-dev **`0029636`**（dispatch→`execute_entry_text_platform`、REQUEST_FILE 退役、combined 计数、fixture 迁移；+144/−61）；auto-down plan-983-dev **`a0ff337`**（`main(request str)`+`stage_str_utf8`；+48/−12）；auto-man plan-983-dev **`58fc2b3`**（收据打印 combined、C1/C0 断言、并发用例；+165/−3）；auto-lang plan-983-dev **ff 至 master `0a167f611`**（取得 `600d31527` 组合入口；零本地提交）。| 实现：dispatch 构造与旧 staged 同形的 HTTP 文本（方法行+Host+Content-Length+Connection+体），`from_utf8` 失败 ⇒ 具名 400（文本入口合同只收 UTF‑8）；`program.load()` 后调 `ace_av_core::av::execute_entry_text_platform(&loaded, limits, cancel, grants, &[text], None, backend)`；load 失败 500、AdmissionError 500（code+message）。业务源 `stage_str_utf8` 处理标量域（774）→UTF‑8 字节重编。| evidence: ①man `cargo test -p auto --test plan983_c1_cli` **6/6 绿 0.83s**（r18 五例在组合入口上行为不变 + 新并发用例：hammer GET 每次字节=canonical A/B 之一、POST 各自 200 同形、终态 config∈{A,B}、重启 GET=最后提交）；②`plan983_c0_cli` **1/1 0.36s**（断言 `combined_entry_dispatches=2`）；③auto-web `--test http_runtime` **8/8 2.93s**——c0 用例断言 `combined==av≥3`、**`av-service/p983-req.txt` 不存在**、`p983-resp.txt` 存在；member-delivery fixture（`main(request str)` 迁移后）DB 语义不变。| 并发结论：**现有单 owner 派发满足合同，无需额外同步**——网络线程单 rt 收连接入 mpsc，owner 循环顺序 `dispatch`（AV 臂同步完成无 await），配置提交=tmp+MoveFileEx 原子替换；永久用例固定该合同（夹具：Service 含 !Sync Receiver，并发腿用端口直连）。| 过程修正（两处实测）：①`stage_str_utf8` 首版参数声明 owned `Str`——借用 `request` 传不进（verify `source.type-mismatch: expected Str, found str`），改 `str` 后过；②member-delivery fixture 仍 0 参 ⇒ `Infra("entry expects 0 args, driver passed 1")`——所有 AvServiceEngine 源统一 `main(request str)` 形（`av-service.at` 是测试自驱动 A 腿不经引擎，不动）。| 影响面核对：`plan982_av_http_service`（17 例）走 `--server av-direct`（man 自持宿主）未触；`plan982_service_av_track`（`--server av` 无 `--api` ⇒ 派发前具名拒绝）未触；`plan982_exec_slice_c0` AV 腿为坏配置派发前失败未触——证据复用。| 测试计时：c1 6 例 0.83s、c0 0.36s、http_runtime 8 例 2.93s（新增并发用例含在内，单例≪10s）；本会话新增用例 1（并发合同）。| U0 备案：980 已录 002-counter 真挂载（9/9 handler 走 ACE lane）；983 将复用其浏览器断言、启动改真 CLI；最终收据等 980 WEB 正式交付 + 982 Vue `auto run`（产品链 TS 转译/a2x_rt 接线确认）——未开工。| 未做：U0 实施、B-CONSUMERS 其余端点/四仓整体、mtime 真值。| next: U0 前置跟踪（980 WEB/982 auto run）；B-CONSUMERS → T-05/06/07
2026-11-18 stage: work | PLAN-983 r18 | outcome: pass（**C1 文件通道行为验收 5/5 绿**；组合入口未接，B-CONSUMERS 整批仍未勾，U0 待 980/982）| 用户指令：982 的通用 AV 组合入口已落主，可以开始接完整 C1（U0 产品级验收仍等 980 WEB 交付与 982 Vue auto run）。| worktree/commits: auto-down `.wt/lang-983/auto-down` plan-983-dev **`a634c09`**（AV 源 +512/−70）；auto-man `.wt/lang-983/auto-man` plan-983-dev **`259ef8b`**（用例 +18/−16）；auto-web 无改动（复用既有公开面：`canonical_w16`/`dir_open_w16`/`dir_next_w16`/`file_replace_w16`/`file_mtime_ms` 等 intrinsic 与 `Win32PlatformIo` 全在组树 av-core `2513903df`，本切片零 lang/web 代码改动）。| 实现：POST open 顺序=r15 合同——①存在性（400 原样/零副作用）→②`canonical_w16` 规范化（`\\?\`）→③wiki 准备→④**索引重建**（递归 walk `wiki/**/*.ad`（深度>16 判失败防 junction 环）、pages 行 `{path(rel,`\`分隔),title(stem),frontmatter(头 4096B 内 `---` 块原文),mtime}`、`<root>\jade-garden-index.json.tmp`+`file_replace_w16` 原子替换、失败 ⇒ 500 `workspace.index-rebuild-failed:` 且尽力删 tmp）→⑤**配置原子提交**（`<config>.tmp`+replace、失败 ⇒ 500 `workspace.config-commit-failed:`）→⑥200 canonical root/wiki_dir。Unicode：`utf8_to_units` 全量转换（1‑4 字节+代理对）替换 ASCII 门。| evidence: ①man `cargo test -p auto --test plan983_c1_cli` **5/5 绿 0.86s**（索引失败：500 具名+旧工作区/配置/旧索引字节不变+标记文件原样+重启 GET 保旧；canonical：与 `std::fs::canonicalize` 字节相等+config 持久+重启保持；Unicode：`工作区-测试` POST/GET/重启同 canonical 路径+wiki 创建+config 持久；config 只读：500 具名前缀+配置字节不变+重启保旧；活跃 POST→GET→重启 1/1）；②`plan983_c0_cli` **1/1 0.35s**（GET/env 回退语义不变）；③auto-web `--test http_runtime` **8/8 3.18s**（含消费源直连 C0 用例）。| 具名边界（源内注释+此处登记）：pages.**mtime 恒 0**（AV 模块至多声明 **16 个平台操作**的 verify 契约已满：mtime lo/hi 双查询放不下）、blocks/links/tags 恒 `[]`（链接图/块解析属后续端点切片）、`dir_close` 不显式调（宿主 ledger 每运行强制关全部句柄）、并发串行提交（r15“GET 在切换期间读到一致状态”）**未实现未验**（单请求串行派发下无暴露）。| 过程（三处实测修正）：①`to` 是关键字（r16 已知坑复发，parse 探针 `plan-artifacts/plan-983/parse-probe` 定位）；②16 平台操作封顶（verify `host.duplicate`）——裁 mtime/dir_close 换 delete 清理；③**canonical 是 verbatim 路径不归一化 `/`**——wiki 拼接分隔符从 `/` 改 `\` 后全绿（首版 5/5 全挂在 "workspace open failed"）。| 环境处置：组内 auto-ui sibling 被 PLAN‑058 停在 `c0035034`（E0433 红）→ 前移至 master 修复 **`d188d695`**（"ace_package_bridge/ace_package_lane 补 ace-bridge cfg 门…双绿"）；其脏 Cargo.lock（解析 churn）副本存 `plan-artifacts/plan-983/auto-ui-lock-churn-c0035034.patch` 后还原再 checkout；PLAN‑058 会话恢复时 `git checkout c0035034` 即回其停驻态。| 测试计时：c1 5 例共 0.86s（单例≪10s）；c0 0.35s；http_runtime 8 例 3.18s；本会话新增用例 0（全部为既有预置转正）。| 未做：983 生产 HTTP 接用 `execute_entry_text_platform`、U0（待 980 WEB + 982 `auto run`）、B-CONSUMERS 其余端点/四仓整体、并发串行提交、mtime 真值。| next: C1 组合入口接线并复跑本套用例；并行跟踪 U0 前置（980/982）
2026-11-18 stage: work | PLAN-983 r17 | outcome: pass（**C1 回归补强 + 三个预置用例**；C1 完整语义仍待 982）| 用户指令：补强现有用例（先断言旧索引生成且含 hello.ad，再触发目标索引失败），并先写好 Unicode 路径 / Windows canonical 路径 / 配置提交失败的测试。| commits: auto-man `867e913`。| evidence: 活跃 `c1_open_get_restart_share_one_persisted_workspace_config` **1/1 绿（0.81s）**；`--ignored` 4/4 红（精确失败点）：①`c1_index_failure_...` 停在“old index generated by open”（旧索引未生成，尚无 hello.ad）；②`c1_windows_canonical_paths_match_legacy` 实际返回原样路径、期望 `\\?\` canonical；③`c1_unicode_workspace_path_roundtrips` POST 返回具名 400（ASCII-only）；④`c1_config_commit_failure_...` 返回 500 但无 `workspace.config-commit-failed:` 前缀。| 契约补记：配置提交失败的 error 前缀为本切面**建议**（`workspace.config-commit-failed:`），待 982 组合入口冻结；Unicode/canonical 的目标行为按 r14/legacy 对齐。| 未做：C1 索引生成/原子提交/Unicode/canonical/config 失败语义实现（982 组合入口）、U0、四仓整体。| next: 982 组合入口 → 去 ignore 复跑 4 用例；U0 待 980/982；B-CONSUMERS → T-05/06/07

2026-11-18 stage: work | PLAN-983 r16 | outcome: pass（**C1 首个可执行切片 + r15 永久回归固定**；C1 完整语义仍待 982）| 用户指令：把 C1 基线写成永久验收（同一 `POST→GET→重启→GET` 用例），GET/POST 接同一显式工作区配置，冻结 `JADE_GARDEN_CONFIG`（持久值优先、env 初始回退）。| commits: auto-down `5b213b5`（AV 源：config 优先级 + POST open）、auto-man `36f6df6`（永久验收 POST→GET→重启；grants 含 `platform.file.path`）/`f746c74`（r15 索引失败回归，ignored）、auto-web `13027c3`（C0 用例随源：2 路由 + path grant）。| evidence: ①`plan983_c1_cli` 活跃用例 **1/1 0.42s**：env A → POST B（200、`B/wiki` 建、config 写 B）→ GET B → 重启 GET B；②`plan983_c0_cli` **1/1 0.22s**；③auto-web `--test http_runtime` **8/8**（8.11s）；④r15 回归（`--ignored`）**红**：非空索引目录夹具 POST 返回 **200**（应 500 `workspace.index-rebuild-failed:`），证据=test panic 输出；已 `#[ignore]` 固定待 982 组合入口。| 合同（本切面）：`JADE_GARDEN_CONFIG` 持久值优先、`JADE_GARDEN_DEFAULT_WORKSPACE` 仅初始回退；POST 400 不存在（legacy 同形、零副作用）/非 ASCII 具名 400/无 config 路径 500；响应 root 原样（无 canonicalize，与 legacy `\\?\` 差异具名）；索引未生成（r15 待实现）。| 过程：AV 源两轮编译修复（`to` 为关键字、`Str` 参数搬移、误删函数复原）；man 提交曾带 rustfmt 噪声已 amend 清除。| 未做：r15 提交顺序/原子替换/索引生成/并发串行（982 组合入口）、U0、四仓整体。| next: 982 组合入口实现 C1 r15 语义 → 去 ignore 复跑；U0 待 980/982；B-CONSUMERS → T-05/06/07

2026-11-18 stage: work | PLAN-983 r14 | outcome: pass（**C0‑B 正式跑 + C1 基线冻结**；B-CONSUMERS 仍未勾）| 用户指令：跑 `01b-workspace-av-consumption.spec.ts`（用含 `b830e07` 的源码、复用 AutoDown 主检出依赖/CI），拿页面真实请求/响应/主视图的正式收据；随后按 `open_workspace_impl` 写 C1 验收基线交 982。| C0‑B 运行形态：临时 overlay（`front/e2e-983` + `playwright.983.config.ts`，跑完即删）+ **982 CLI AV 后端**（`auto.exe --server av --api <workspace_service.at>`，`JADE_GARDEN_DEFAULT_WORKSPACE=<fixture>`）；未复制 node_modules、未在 worktree 建 symlink。| evidence（C0‑B）：Playwright **1 passed**（6.2s；首跑 11.5s）；页面真实请求 `GET http://127.0.0.1:13100/api/workspace` → **200**，body `{"root":"D:/autostack/auto-down/jade-garden/front/e2e/.runtime/workspace","wiki_dir":"…\\wiki"}`；主视图断言（`#app` 挂载/主区提示/opener=0）通过；后端收据 `served=2 av_dispatches=2 legacy_dispatches=0 same_sdk=true`；退出码 0；日志 `plan-artifacts/plan-983/c0b/playwright-01b.log`。| evidence（C1 基线，临时 exe 副本 + 隔离 config 实测）：不存在目录 ⇒ `400` + `{"error":"Directory does not exist: <原样路径>"}`，**零副作用**（目录未建、既有工作区不变）；成功 ⇒ `200` + `{"root":"\\\\?\\D:\\…\\ws","wiki_dir":"\\\\?\\D:\\…\\ws\\wiki"}`（canonical `\\?\` 形态）；`<root>/wiki` 创建（wiki 缺失时；空 wiki ⇒ 索引 `pages: []`）；`<root>/jade-garden-index.json` 重写（`pages[{path,title,frontmatter,mtime}]`+`blocks[...]`，实测含 `hello.ad`）；config `{"workspace_root":"<canonical>"}`；二次 open 幂等。| 债务候选：现有实现先 `save_config` 后建 wiki/index（index 失败 ⇒ 500 但配置已切换）——982 实现组合入口时裁定。| 未做：C1 实现（待 982 组合入口）、U0、四仓整体。| next: 982 组合入口 + C1 实现；U0 待 980/WEB + 982 `auto run`；B-CONSUMERS → T-05/06/07

2026-11-18 stage: merge（phase fold：**982 `--stack web` 交接三件公开面**）| PLAN-983 r13 | outcome: pass | delivery_commit: **auto-web master `5c484d9`**（ff-only；含 `56b3ac5` 公开 grants、`228d775` receipt+control-plane shutdown、`5c484d9` 公开 config→transport 映射+单测）；canonical: **lang master `2513903df`**（`m2-web.md` 新增「C0 公开面（供 982 `--stack web`）」节）| 交接件（982 可直接消费）：a) `AvServiceEngine::new_with_grants(source: &str, scratch_root: &Path, grants: Vec<String>) -> Result<Self, String>`（默认 `new` 仍 file+mem）；b) `AvServiceEngine::receipt() -> Arc<AvServiceReceipt>`，`AvServiceReceipt::av_dispatches()/legacy_dispatches()`（路由 miss 不计；legacy 结构常量 0）；c) `TransportConfig::from_service_config(&HttpServiceConfig)` 与 `from_service_config_with_policy(Arc<HttpServiceConfig>)`（后者装配 `ServiceRuntime`）；ready=`http_server::bound_addr_snapshot() -> Option<String>`（serve 真实绑定，进程内单服务）。**集成注意**：`*_with_policy` 须 `ServiceRuntime::bind_shutdown(tx)`（否则 `/__auto/shutdown` 不停）；无 policy 用 `bind_control_plane_shutdown(tx)`。| evidence: `--test http_runtime` **8/8**（7.55s）；`--lib` **189 例 188 绿/1 红/1 skip**（红=基线既有 `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`；新映射单测 `service_config_maps_into_transport_budget_and_policy` 1/1）；`cargo fmt --check` clean；man 的 `serve_av` 改用公开映射后 `plan983_c0_cli` **1/1**（0.15s，含 `bind_shutdown`）。| 过程：lang master 前移导致首次 ff 失败，规范提交 rebase 后 ff（无内容漂移）。| 未做：man `6e21bfa` 仍在 plan-983-dev（待 982 认领/合入；man master 已有 982 `--stack`，注意合并冲突面 cli_spec/cmd_service/http_service）；C0-B 永久跑/U0/四仓不变。| next: 982 `--stack web` 接线（消费 master `5c484d9`）；C0-B 永久跑；C1 交 982；U0 待 980/982

2026-11-18 stage: work | PLAN-983 r12 | outcome: pass（**C0 CLI 腿 + C0‑B 准备/探针**；B-CONSUMERS 仍未勾）| 用户指令：982 优先修通 `ace-candidate` 并完成 `serve_av` 接线（真 CLI 验 JSON/AV 派发/关停/端口重绑）；983 并行准备 C0‑B（前端断言消费真实 AV 后端）；U0 等依赖齐备。| worktrees/commits: auto-web `.wt/lang-983/auto-web` `228d775`（服务收据 + `bind_control_plane_shutdown`）、auto-man `.wt/lang-983/auto-man` `6e21bfa`（`--api` 显式源；`--server av`=auto-web `serve_av`+`AvServiceEngine`；pre-C0 宿主保留 `av-direct`；`plan983_c0_cli`）、auto-down `.wt/lang-983/auto-down` `b830e07`（e2e 外部后端模式 + `01b-workspace-av-consumption.spec.ts`）；lang 组树 `ff55579c3`（UsePy 修复）、auto-ui 组树 `c0035034`（只读依赖）。| evidence: ①`cargo check -p auto --no-default-features --features ace-candidate` 退出 **0**（UsePy 缺臂红已由 lang `ff55579c3` 解除）；②`cargo test -p auto --no-default-features --features ace-candidate --test plan983_c0_cli` = **1/1**（0.15s；首跑 0.37s）：真实 bind（port=0→14977/6650 等）、`AUTO_SERVICE_READY`、`GET /api/workspace` 字节级 JSON、`POST /__auto/shutdown` 202、进程退出 0、端口重绑、收据 `av_dispatches=2 legacy_dispatches=0 same_sdk=true`；③`--test plan982_av_http_service --test plan982_service_av_track`（`av-direct` + ace-candidate）= **17/17 + 2/2**（旧 man 宿主行为保持）；④`cargo fmt --check`（auto/auto-man-runtime）绿；⑤C0‑B 临时探针（`D:/autostack/plan-artifacts/plan-983/c0b/run-c0b.mjs`）：982 CLI AV 后端（14977/10205）+ Vite（13101）+ Chromium → 页面真实 `/api/workspace` 200/root、`#app` 挂载、opener 0、console error 0 ⇒ `C0B_RECEIPT ok:true`。| 过程记录：man 提交曾因 `cargo fmt` 带入 10 个无关文件的格式噪声，已 `git checkout HEAD^` 复原并 amend（现 `6e21bfa` diff=8 文件 +404/−13，无噪声）。| 外部红（具名，非本切片）：auto-ui master `c0035034` 默认特性下 `ace_av`/`ace_core`/`ace_a2x_ts` 未门控 → `cargo test -p auto`（默认特性）不可构建、`plan890_web_consumers` 默认档暂不可跑；归 980/981 F32 窗口配对（UsePy 红本身已闭）。| 未做：C0‑B 永久 playwright 跑（组树无 node_modules，避免 worktree 内建 symlink；e2e 改动已入库待有依赖检出复跑）、U0 运行、四仓整体验收、`av-direct` 去留（待 982 裁定）。| next: C0‑B 永久跑 + U0 前置（980/WEB 勾选、982 `auto run` 接口）+ B-CONSUMERS → T-05/06/07

2026-11-18 stage: work | PLAN-983 r11 | outcome: pass（**C0-A 切片：AV 业务源 + 直连验收**；B-CONSUMERS 仍未勾）| 用户指令：现在就做 C0-A 的 AV 业务源和直连验收；`api.at` 的 `get_workspace()` 契约桩需可执行实现；982 接线后再跑真 CLI + C0-B；另备一例消费 980/WEB。| worktrees: auto-down `D:/autostack/.wt/lang-983/auto-down`（`211b6d5`）、auto-web `D:/autostack/.wt/lang-983/auto-web`（`56b3ac5`）；auto-man 组树已建（`60321f9`，未改）待 CLI 腿 | 交付：①`jade-garden/back/auto/av/workspace_service.at`——读 `JADE_GARDEN_DEFAULT_WORKSPACE`（`env_get_w16`）→ UTF-8/JSON 转义 → 响应；`wiki_dir` 按 Windows join 拼 `\wiki`；只读、每请求关句柄；`api.at` 契约桩旁标注指向。②auto-web `AvServiceEngine::new_with_grants`（显式 `platform.file/mem/env`；默认 `new` 不变）。③真实 TCP 直连用例（auto-web `http_runtime.rs::c0_workspace_p983`；源读消费者仓，`AUTO_DOWN_ROOT` 或相对 `../../../auto-down`，缺失具名失败）。| evidence: `--test http_runtime` **8/8 绿 7.92s**（新例 0.215s）；字节级 200 JSON（含 `\\wiki`）、重复稳定、未配置 ⇒ `{"root":null,"wiki_dir":null}`、`/api/nope` ⇒ 404 空体（派发前）、shutdown 端口重绑；`cargo fmt --check` clean；回执行 `✓ C0-A workspace receipt: exact JSON ...; null form; 404 pre-dispatch`。| 合同纠正：C0 的 `wiki_dir` 预期由 `<W>/wiki` 更正为 `<W>\wiki`（Rust `PathBuf::join` Windows 实测；JSON `\\`）——原文本错误已修（r11 同次）。| U0 备例：002-counter 浏览器 `Counter: 0→1`（含 `-`/`Reset` 回 0），oracle 先例 `p980_web_lane_browser_mount`（41→42，`test_lane_browser.mjs`）；**未运行**，前置=980/WEB 批勾选 + 982 `auto run` 接口。| 未做：982 CLI 腿、C0-B 前端页、U0 运行、四仓整体验收分母不变。| next: 982 `auto service --server av` 走 auto-web `serve_av` → 同源 CLI 复验；C0-B；U0 待前置

2026-11-18 stage: review | review_scope: phase（B-GENERATED-R；整包仍 executing，CONSUMERS/T-05..07 未动）| reviewer_identity: 委派独立复核（gen-reviewer；与实现会话不同 agent、同机；旧收据延续+同 SHA 复用已声明）| plan_id: PLAN-983 | plan_revision: r9 | outcome: pass | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: 3f8de15（auto-web，=master tip；复核前后 clean）| base_commit: fd115ce | dependency_revisions: lang 组树 1af135c31（auto-generation-core/auto-lang-core/ace-{core,a2x-core,a2x-rust,av,av-core,lowering} 对 850a49bf8 空 diff；套件跑于 850a49bf8，代码等同）| spec_inputs: docs/specs/ace-migration/m2-web.md（B-GENERATED 节、文本入口节）| delta_impact: fd115ce→3f8de15 八文件 +547/−1，无 ace_a2r_lane.rs/api_gen.rs/http_generation.rs/auto-generation-core/lane 依赖 crate；生成服务 main 走 axum::serve（api_gen.rs:5310），不构造 ServiceEngine，故不影响本批验收 | acceptance_results: AC-B-GENERATED pass——①未手改生成物冷编真 TCP（双模块在场+api.rs 委派；cargo build --offline 子进程 success；spawn exe 真 TCP 200/202/端口重绑）②独立 expected 890/"p983-ace-a2r-service"/2147483647/82/"p983"/true ③坏源 scale(float) 具名拒含端点名+member manifest 字节不变 ④真实 cargo 子进程终态与产物退出码断言，无自报布尔 | findings: F-G1（低，非 AC 缺陷）新 ACE 例首跑 19.46/22.53s>10s（冷 cargo+新 ace-lowering 首建+并发盘压），稳态复验 6.793/9.702s≤10s；owner=B/CONSUMERS 预构建（r7 已记）；无 in-contract 缺陷；探针误用 `-E test(/.../)` 选 0 例已剔除（非产品）| evidence: lane 单测 4/4 0.122s；参数例 PASS 9.702s；生成+坏源例 PASS 6.793s；复用 3f8de15 套件 3/3 118.562s（19.464/22.529/76.562）、lib 188 例 187 绿 63.662s（唯一红=基线既有且文件不在 diff）| pending_closeout_gates（merge-owned）: B-GENERATED 已随 fd115ce phase fold 落主、文本切片 3f8de15 已落主；T-06/T-07 待 CONSUMERS | next: work/merge 按 owner（本批无修复项；F-G1 归 B/CONSUMERS 预构建）

2026-11-18 stage: plan（首例冻结，非实施）| PLAN-983 r10 | outcome: pass（合同冻结；B-CONSUMERS 仍未勾）| 用户指令：先收 B-GENERATED-R，同时为 B-CONSUMERS 选第一个真实业务用例；982 优先交首例实际需要的 EXEC 接口。本轮交付：①B-GENERATED-R 复核 pass（见上一条）；②§5「983/CONSUMERS 首例冻结」——C0 = jade-garden `GET /api/workspace`：无文本参数、两字段 JSON（root/wiki_dir），固定字节级预期（含 `null` 失败形态与 Windows JSON 转义）、404/500 失败面、只读+句柄/端口清理；接口清单=983/AVSERVICE 已交付、982/EXEC `--server av` 部分交付（需改走 auto-web `serve_av`、API 路径可指定、grant/scratch 暴露、配置/lifecycle 同 `serve_vm`）、982/CONFIG 服务配置已交付/配置文档引擎本请求不用、980/WEB 与 983/GENERATED 本首例明确不用。③分期：C0-A 后端线 → C0-B 前端页 → C1 `POST /api/workspace/open`（将触发文本+平台 I/O 组合，交 982）。**未做**：未写 jade-garden 适配/AV 程序、未运行新栈请求、未改四仓整体验收分母。| evidence: 事实取自 jade-garden `back/server/src/workspace.rs`/`state.rs`、`front/src/lib/api.ts`、`front/e2e/01-workspace.spec.ts`/`global-setup.ts`/`runtime.ts`（端口 18181、WORKSPACE_ROOT 正斜杠）、982 EXEC receipts（rounds 149–166）与 man `http_service.rs` 实测读取；接口状态以 auto-web `3f8de15`/man `1d26b50`/lang `1af135c31` 为据。| next: work（C0-A 实现随 982 EXEC 接口到位分段接；本 Plan 不动 982 代码）

2026-11-18 stage: merge | PLAN-983 r9 | outcome: pass（**phase fold：AV 文本入口切片合入**；整包不归档，status 保持 executing）| delivery_commit: auto-web master `3f8de15`（=reviewed commit；ff-only，tip 相等已核；主检出 README WIP 属他 session 保留）| canonical: auto-lang master `1af135c31`（reviewed 规范 `b952cc9a4` rebase 至 `bf006d2e7` 后 ff；两提交=`043542b2e` 文本入口节 + `1af135c31` 状态/边界行修复；rebase 内容无漂移已核 tip 含该节）| checkpoints: prepared=review pass（下一行）；landed=两仓 ff-only tip 相等；canonical_verified=`m2-web.md` 文本入口节 + status/boundary 已在 master；**archived/cleaned=不适用**（CONSUMERS/T-05..07 在途；worktree/分支/lang-983 组保留）| **口径澄清**：用户称 T-06；按计划 T-06 为整包终态合入（前置含 CONSUMERS），本次只执行本切片的 phase fold 落主，**T-06/T-07 复选框保持未勾** | 消费登记：文本入口生产面=`http_server::serve_av_text`+`ServiceEngine::AvText`+`av_text_service::AvTextServiceEngine`（编译一次；`fn main(query str, body str) int`；收据 compile/av/legacy 计数+ABC 指纹）；source/接口/边界以 m2-web.md 文本入口节与 auto-web `3f8de15` 为准；982 文本参数契约（m2-man-gen.md）+ 本仓宿主接线=组合消费面 | merge 会话测试：复用 review 会话同 SHA 门禁（`3f8de15`：http_runtime 7/7 9.31s、lib 187/188 63.66s、http_generation 3/3 118.56s；准备=0 热构建）；本 merge 会话无新增用例 | next: B-CONSUMERS（前置 983/AVSERVICE+GENERATED 已落、982/EXEC、980/WEB）

2026-11-18 stage: review | review_scope: phase（B-AVSERVICE 批复核，含 2026-11-18 文本入口切片 B-AVSERVICE-TEXT）| reviewer_identity/limitations: 实现会话内复核（同一上下文；结论自冻结提交的 diff/断言/门禁复跑重建，不采信执行者摘要；独立复审窗口不可用已声明）| plan_id: PLAN-983 | plan_revision: r9 | outcome: pass（phase；整包仍 executing）| worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: auto-web `3f8de15` | base_commit: `fd115ce` | dependency_revisions: auto-lang 组树 `850a49bf8`（自 `19281566f` FF；零代码改动；982 文本入口/`execute_entry_text` 已含）；规范 reviewed `b952cc9a4` → 落主 `1af135c31` | spec_inputs: docs/specs/ace-migration/m2-web.md（本切片节；ace-migration/m2-man-gen.md 982 文本入口契约）| acceptance_results: AC-B-AVSERVICE pass（真实 TCP 7/7；文件通道臂维持 DB 写读/并发 owner 串行/断连/404 同形/端口释放；文本臂：同一编译产物收查询串+体、等字节长度 中文/日文 按内容分流、`av_dispatches=6 legacy_dispatches=0 compile_count=1`）；AC-02 pass（既有用例未改且 lib 187/188、generation 3/3）；AC-03 pass（文本臂无 `Rc<AutoVM>`、无 legacy 回退；未命中 404 在派发前拒）；AC-06 pass（AVSERVICE 批 T-08 面：中间件/上传 501 具名；认证/权限/stream/SSE/`~T` 按 R 条款记录）；AC-04 pass（本批）；AC-05/T-06..07 merge-owned 未闭 | findings: F-R3（已修）规范 status/boundary 行携带陈旧任务计数（13/19）→ work `b952cc9a4` 改为交付切片表述并去掉易腐计数，复验通过；无 in-contract 缺陷。观察（非阻塞/debt 候选）：①两 AV 臂 501/404 预处理为重复代码（未来边界改动需双写）②`failure_reply` detail 转义 `"`/换行但未转义 `\`（与文件通道臂同形；Infra 含 Windows 路径时错误体 JSON 可能不合法，当前消息域未见）③`legacy_dispatches=0` 为结构常量（无 legacy 句柄，非可增长计数器）——配套证据=无 AutoVM 构造、404 派发前拒、响应体由 AV 内容分支产出④路由 `fn_name` 未与入口交叉校验（单入口臂设计）| evidence: 复核会话实测（冻结 `3f8de15`；他 session 构建并发已观察并记录，均在其他 worktree）：`--test http_runtime` 7/7 9.31s（新例 0.172s）；`--lib` 188 例 187 绿/1 红/1 skip 63.66s（红=`api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，该文件不在 diff，r7 已在基线 `0adebc0` stash 复证）；`--test http_generation` 3/3 118.56s；`git diff --check` clean；新文件 0 TODO/dbg/println/unwrap、0 warning；fmt clean | pending_closeout_gates（merge-owned）: 本切片 phase fold（已落：auto-web `3f8de15` / auto-lang `1af135c31`）；T-06 final 与 T-07 清理待 CONSUMERS | next: merge（phase fold，已完成）→ B-CONSUMERS

2026-11-18 stage: work | PLAN-983 r9 | outcome: pass（**B-AVSERVICE-TEXT 切片 work handoff，非整包 execution_done**）| worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | base_commit: auto-web fd115ce | code_commit: auto-web `3f8de15`；lang 组树 ff `850a49bf8`（零代码改动）+ 规范 `b016ff9d3` | task_ids: B-AVSERVICE-TEXT | evidence: 见 §8 任务证据行与 m2-web.md 文本入口节；命令/实测：`--test http_runtime` 7/7（7.48s）、新例 0.60s、`--lib` 188（187 绿/1 基线既有红/1 skip，21.39s）、`--test http_generation` 3/3（176.43s）；收据 `av_dispatches=6 legacy_dispatches=0 compile_count=1 fingerprint=9db3cce95d63f38e` | 用户指令执行口径：983 主导接生产栈（已交）；982 共享入口缺口本轮未触发（文本+平台 I/O 组合调用未出现，未派 982）；完整 JSON 等真实消费者字段（未做）| blockers: 无 | next_responsible_role: review（B-AVSERVICE-R 复核本切片）；982 交接按触发条件。

2026-11-17 维护（B fold 后）：按用户磁盘偏好再次清理组内构建中间产物——auto-web/target（17GB）+ auto-lang/target（3.1GB）删除（他 session cargo nextest auto-ui 与本组无交集已核）；组余纯检出；两树 git 不变（auto-web @fd115ce=master、lang 树待 CONSUMERS 开工 resync master 22ff37dbb）。**CONSUMERS 开工注意：双树冷重建。**

2026-11-17 stage: merge | PLAN-983 r7 | outcome: **pass（phase fold：M2-B 双批 AVSERVICE+GENERATED 合入；整包不归档，CONSUMERS/T-05..07 在途）** | delivery_commit: auto-web master `fd115ce`（=reviewed commit；rebase 无漂移；ff-only 无 merge commit，tip 已核相等；主检出 README WIP 属他 session 保留）| canonical: m2-web.md 增"B 批已验接口"节+Status/边界更新；replacement-scope-and-evidence.md 增 B 批增量挂接——随本 docs 提交落 auto-lang master | checkpoints: prepared=双批 phase review pass fd115ce；landed=auto-web master tip==fd115ce（wt-guard clean）；canonical_verified=本提交；**archived/cleaned=不适用**（CONSUMERS 在途；worktree/分支/组保留；plan-artifacts scope 保留）| 供他计划消费登记：消费面=AVSERVICE（`http_server::serve_av`+`AvServiceEngine`+`ServiceEngine` 枚举+av-runtime-service.at fixture 形态）与 GENERATED（`AUTO_WEB_A2R_ENGINE=ace` 参数子集+Query/Json 提取器 glue）；source/接口/边界以 m2-web.md B 节+本仓 SHA 为准 | merge 会话测试：复用 worktree 同 SHA 门禁证据（fd115ce：http_runtime 6/6、http_generation 3/3、lang 两组 PASS）；主检出身份相同未重跑；本 merge 会话无新增用例 | next: CONSUMERS 批（前置：983/AVSERVICE+983/GENERATED 已落、982/EXEC、980/WEB 归档面沿 §5 阶段表核）

2026-11-17 stage: review | review_scope: phase（M2-B 双批：B-AVSERVICE-R/B-GENERATED-R；整包仍 executing，CONSUMERS/T-05..07 未动）| reviewer_identity: 实现会话内复核（局限声明；结论自工件重建：双批 diff、门禁复跑、断言核读）| plan_id: PLAN-983 | plan_revision: r7（r8 为他 session 规划交接，与本实施收据并存）| outcome: **pass（phase，双批）** | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: fd115ce（=6808511 B-GENERATED + 76c8001 B-AVSERVICE + 复核修复；基 bdc48e8=A 落地点）| base_commit: bdc48e8 | dependency_revisions: auto-lang 组树零改动（lang 组门禁作非回归证据跑于组树）；前置 983/PORT（A 切片 master）+ L975/L978 ARCHIVED | acceptance_results: AC-B-AVSERVICE pass（V 6/6 + 复核复跑 av_service 0.915s；oracle 断言逐条核读：DB 写读/并发经 owner 串行不串/断连不触态/404 同形/独立 DB 直读/shutdown 端口释放；**具名边界**：中间件链/上传路由=501（语言侧凭据通道缺位）、限流=legacy 专有、上传失败清理面随 501 无 AV 面；认证/权限/stream/SSE/~T 未执行=R 条款记录非 NA）；AC-B-GENERATED pass（V 3/3 + 复跑；参数编组 int/str/bool 独立 expected、坏源 scale(float) 具名拒+保旧、真实 cargo 子进程终态断言；store 保旧=cold_generated legacy 例绿；str concat owned-Str 边界具名）| findings: F-R2（已修）av_service 未用绑定 warning→fd115ce→0 warning+用例绿；观察：①两新生成例计时波动 6.21–11.8s/6.45–9.66s（冷 cargo 启动+并发盘压；稳态≈6s；>10s 离群已记录，B/CONSUMERS 考虑预构建阶段）②ace-av/ace-av-core 转 auto-web 正式依赖=构建图扩大（AVSERVICE 引擎为生产面）③AV 臂 ASCII scratch/数据根约束沿 A 腿 | evidence: 复核会话实测——av_service 0.915s、ace_lane 双例复跑（22.4s 含冷重建波动；首测 6.21/6.45s 见 V 行）；批内文件 0 warning、0 调试打印；diff 复核=seam 仅加引擎枚举臂，legacy 路径未动 | 测试计时表（复核会话）：3 例 22.4s；准备=0（复用热构建）| pending_closeout_gates（merge-owned）：①双批 fold 合入 auto-web master ②m2-web.md 增 B 段 + SD-02 挂接 ③树保留待 CONSUMERS | next: merge（phase fold 双批）

### 2026-11-17 CONFIG-ACE-B规划交接（r8；非实施验收）

stage: new/revision | outcome: pass（规划合同预检） | 本轮用户仅授权补承接/设计/接口/验收及排程；未work/建树/运行测试/探针/构建/合入/归档/派agent。GAP-982-CONFIG-ACE由982原B-CONFIG两仓端到端承接，能力本身保持未实现；本包按§5–8现任务生产或接收，不新增Plan/任务，不改已验A收据。下一步须未来实施指令，982先B-CONFIG-I全链前探/接口与原输入冻结，再V→独立R→正式phase交接；其他owner按所用正式来源接。

取材和静态核验：7计划123原任务ID/顺序/勾选与状态/进度保留，语义revision各增1；仅982补已交967/979直接基础依赖。9个配置判别Case/4个能力AC、typed接口/值语义/预算/生命周期/原用例映射与Spec候选已列；24节点30边阶段图无环，新增本地链接可达，18份源码/Spec/归档/计数器身份不变，目标文档git diff --check通过。旧无owner快照/route C/执行回执作历史保留，不把规划PASS当真实能力或独立产品review。M2A-08 r29/95c7fbad8冻结不变。

2026-11-17 维护：用户指令清理构建中间产物（等 985/A、B 组 parked）。`.wt/lang-983/auto-web/target`（20GB，末次写入 09:06=本切片最后测试；他 session 09:48 `cargo t ace-*` 与本树无交集已核）删除；plan-artifacts/plan-983 内 2 个调试日志删除（scope 目录保留在途）。组余 526MB 纯检出。两树 git 状态不变（auto-web @bdc48e8=master、auto-lang @d43e7970f）。**B 开工注意：冷重建 target（约一次全量 debug 编译）。**

2026-11-17 stage: goal | plan_id: PLAN-983 | plan_revision: r7 | round: 6 | from: merge | to: done（**目标范围=983/A（M2-A 切片）**） | status: executing（B 组在途，7/19） | outcome: pass | worktree: D:/autostack/.wt/lang-983/auto-web（保留）+ auto-lang（resync d43e7970f） | commit: auto-web master `bdc48e8` / auto-lang master `d43e7970f` | next: done（/A 目标）；B 组另启（M2A-08 收据=上行 merge 收据）

2026-11-17 stage: merge | PLAN-983 r7 | outcome: **pass（phase fold：M2-A 切片合入；整包不归档，status 保持 executing 供 B 组）** | delivery_commit: auto-web master `bdc48e8`（=reviewed commit；rebase 无漂移=master 基 0adebc0 未动；ff-only 落地，无 merge commit，已核 tip 相等） | canonical: auto-lang `docs/specs/ace-migration/m2-web.md`（新建，A 切片接口+边界）+ `replacement-scope-and-evidence.md`（PLAN-983 增量挂接，沿 980/976 同款 SD-02 格式）——随本 docs 提交落 auto-lang master | checkpoints: prepared=review pass bdc48e8+冻结 delta；landed=auto-web master tip==bdc48e8（README WIP 属他 session 保留）；canonical_verified=两文件落库本提交；**archived=不适用**（phase fold；B 组 12 任务在途，A交接规约"同树保留/resync 至 B 完成"）；**cleaned=不适用**（worktree/分支/lang-983 组保留；plan-artifacts scope 保留在途；.retention 登记表带 BOM 损坏=PLAN-1036 已登记既有债，fail-closed 只记录，本 fold 无删除/持有变更故无需写） | 985/A 消费登记：A 切片正式收据=本行+m2-web.md；可消费面=中立端口类型、AV 服务腿 fixture/runner 形态、A2R 道显式选择与子集界；source/SDK/接口/实际结果以 m2-web.md"依赖身份"节与上表 SHA 为准 | merge 会话测试：复用 worktree 同 SHA 门禁证据（bdc48e8：两腿 3/3 10.55s、lib 186/187 20.05s、单测 7/7；主检出身份相同未重跑，准备=0）；本 merge 会话无新增用例 | next: B 组开工（M2A-08 收据=本行；B 前置沿 §5 阶段表）

2026-11-17 stage: review | review_scope: phase（M2-A 切片：B-PORT-R/B-PROOF-R；整包仍 executing，B 组 12 任务未动）| reviewer_identity: 实现会话内复核（局限已声明；结论自工件重建：两仓 diff、门禁复跑、断言逐条核读，非采信执行者摘要）| plan_id: PLAN-983 | plan_revision: r7 | outcome: **pass（phase）** | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: bdc48e8（=35b9e47+格式修复；基 93712e5→0adebc0） | base_commit: 0adebc0 | dependency_revisions: auto-lang 组树 e85096628 零改动（纯依赖钉基，已核 clean）；前置 967/968/969/974/975/978 ARCHIVED | spec_inputs: docs/specs/ace-migration/{m2-host,m2-facade,m2-stdlib-network,m2-stdlib-data,m2-build}.md（取材已核）| acceptance_results: AC-01 pass（T-01 冻结）；AC-B-PORT pass（公共面中立性=inspection 无 AutoVM 泄漏；行为保持=http_runtime 3/3+shutdown/upload 负例绿+内部机制 diff 未动；contract 纯度=cargo tree 仅 serde/serde_json）；AC-A-PROOF pass（V-A-PROOF 7/7+复核重跑 3/3；oracle 断言逐条核读：独立 bytes/DB 独立读回/大整数/断连/cancel/端口释放/坏源保旧/未手修）；AC-PHASE（A 部分）pass（A 阶段 required 5 任务全闭）| findings: F-R1（已修）本批文件 rustfmt 漂移 26 处→work 同树修复提交 bdc48e8→7 单测+3 集成+lib 186/187 复验绿；具名观察（非阻塞，B 域）：①core trait `listen_blocking` 签名仍为 core legacy trait 所定（桥已文档化，B/AVSERVICE 替换）②A2R 道零参 int/str/bool 子集（B/GENERATED 扩面）③AV fixture ASCII 数据根约束（B/AVSERVICE/973 域）④ace_lane 用例在具名 target 内显式 cargo --offline 子进程（计划 r7 明文合同"新增ACE具名case接原target并核实际子进程"；≤10s；与 2026-11-14 纪律的 build-lane 张力已记录，B/GENERATED 考虑预构建阶段）| evidence: 复核会话实测——单测 7/7（0.154s）；两腿 3/3（重跑 10.55s，含 ace_lane 5.89s）；lib 186/187（20.05s，1 红=test_handler_body_r2_matrix_compile_oracle **基线 0adebc0 原样红**，stash 复证，具名既有非本批）；fmt：本批文件 clean（余 http_server.rs:119 基线既有漂移，不在触面）；新文件零 warning、零调试打印 | 测试计时表（复核会话）：单测 7 例 ≤0.032s；集成 3 例 5.892/0.620/0.874s（首测 9.14/0.63/0.86）；lib 并集 20.05s；准备=复用 work 会话已热 target（未重编译全量）| pending_closeout_gates（merge-owned，phase fold）: ①auto-web master 合入 A 切片（bdc48e8）②规范沉淀 docs/specs/ace-migration/m2-web.md A 切片段（SD-01/SD-DETAIL/SD-PHASE-A/SD-TARGET-SCOPE 关联）+ replacement-scope-and-evidence.md 959 行关联（SD-02）③worktree/分支保留至 B（同树 resync 规约）④985/A 消费登记 | next: merge（phase fold）

2026-11-17 stage: work | PLAN-983 r7 | outcome: pass（**M2-A 切片 work 完成 handoff，非整包 execution_done**）| worktree: D:/autostack/.wt/lang-983/auto-web @plan-983-dev | code_commit: 93712e5（B-PORT）+ 35b9e47（B-PROOF） | base: auto-web 0adebc0 / auto-lang e85096628（lang 树零改动，纯依赖钉基） | dependency_revisions: 前置 967/968/969/974/978/975 全 ARCHIVED | task_ids: T-01、B-PORT-I、B-PORT-V、B-PROOF-I、B-PROOF-V | evidence: 见 §8 各任务证据行；V-B-PORT（lib 183/184 绿+1 具名既有红基线同红、http_runtime 3/3、contract 纯 serde/serde_json）、V-A-PROOF（7/7 绿 30.59s，新增 3 例 ≤10s 门：9.14/0.63/0.86s）、lane 单测 3/3 | session 测试表：lib 并集 23.78s（准备=冷编译已热，执行 23.78s，复用=既有证据未重跑）、V-A-PROOF 30.59s、http_runtime 门 5.94s（B-PORT 时点）；新增用例 4 例计时见 B-PROOF-V 行 | blockers: 无 | 本切片 open 项：①既有红 test_handler_body_r2_matrix_compile_oracle（基线 0adebc0 同红，非本切片引入，owner=待查既有）②A2R 道子集界（零参 int/str/bool、无 store/db/伴生）= B/GENERATED 扩面，具名不冒 ③AV 腿 ASCII 数据根约束（测试保证；非 ASCII 路径 staged 未支持，B/AVSERVICE 或 973 域）| next_responsible_role: review（A 切片独立复核 B-PORT-R/B-PROOF-R）

2026-11-17 stage: work | PLAN-983 r7 | goal 授权（/auto-plan-goal 完成计划983/A）开工 M2-A 切片：T-01 冻结完成（见 §8 T-01 证据行）；工作树组 `D:/autostack/.wt/lang-983/{auto-lang,auto-web}`，branch `plan-983-dev`（两仓），材料 scope `D:/autostack/plan-artifacts/plan-983/`。本记录非 review。

2026-11-16 stage: new/revision | PLAN-983 r7 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-12 stage:new / PLAN-983 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原19任务/全部AC与V/依赖/阶段/预算和drafting 0/19；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部19个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/19；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化4批，0/16。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

## 10. 待澄清事项

**2026-11-18 C1 交接（982 组合入口）**。982 在同一次调用里提供**文本实参 + 平台 I/O + JSON 出参**，C1 按 §5「C1 工作区提交与失败验收」先准备目标 wiki 与索引、后提交配置和当前状态。983 承接真 CLI 回归，含索引失败保旧工作区与配置、成功后 GET 及重启持久化。完整 C1 仍待实现和联合验收。**r16（2026-11-18）**：983 侧已交首个可执行切片（配置优先级 + POST）与永久回归——活跃用例绿；r15 索引失败用例 `#[ignore]` 固定、当前实现跑为红（证据见 §9）。**r18 业务行为验收（2026-11-18）**：C1 文件通道 5/5 绿（见 §0/§9 r18）。**r19 复核（2026-11-18）**：982 组合入口 `600d31527` 已交付，983 接线仍开放。当前 `auto-web/crates/auto-web/src/av_service.rs` 写入 `p983-req.txt` 再调用 `run_platform`，Jade Garden 的无参 `main()` 读回请求文件。收口要求为生产 HTTP 将请求体作为借用 `str` 交给 `execute_entry_text_platform`，保留 C1 受检平台 I/O 和响应行为。复跑现有五例以及 C0/HTTP 回归，并给出组合入口确实执行且请求 staging 文件从未创建的区分性证据。r15 并发串行提交单独验证。**r19 闭合（2026-11-18）**：全部落地——组合入口接线（auto-web `0029636`）、源入口 `main(request str)`（auto-down `a0ff337`）、`combined_entry_dispatches` 收据 + `p983-req.txt` 缺席断言（auto-man `58fc2b3`/http_runtime）、并发一致性永久用例（6/6）；见 §0/§9 r19。

**2026-11-18 982/EXEC 优先项（首例 C0 依赖）——已交**：按用户裁定「尽快解锁 983 ⇒ 982 优先交首例实际需要的 EXEC 接口」，auto-man `6e21bfa` 已把 `auto service --server av` 接到 auto-web `serve_av`（`--api` 显式源、grants 可配、ready/收据/控制面关停），pre-C0 宿主保留为 `--server av-direct`；真 CLI 用例 `plan983_c0_cli` 绿。982 `--stack web` 所需的**三件公开面已落 auto-web master `5c484d9`**（`new_with_grants`/`receipt`/`TransportConfig::from_service_config[_with_policy]`；ready=`bound_addr_snapshot()`），规范见 `m2-web.md`「C0 公开面」（lang `2513903df`）；集成注意 `*_with_policy` 须 `bind_shutdown`。**剩余外部项**：auto-ui master 默认特性红（`ace_av`/`ace_core`/`ace_a2x_ts` 未门控，`c0035034`）挡住 `cargo test -p auto`（默认特性）与 `plan890_web_consumers` 默认档——归 980/981 F32 窗口配对，非本切片引入。C/EXPORT 按自身顺序并行。

**2026-11-18 用户指令（983 先接 auto_web 生产栈）**：本轮已交文本入口切片 B-AVSERVICE-TEXT。两项未触发/未做，按用户口径保留为下一步条件：①「文本参数 + 平台 I/O 同一次调用」——983 首个消费者本轮只用文本；当真实消费者需要组合调用时，把**具体需求**交 982 验证并补齐（982 收据仅覆盖文本参数，组合调用需单独验证）。②完整 JSON（嵌套/浮点/请求头）——等 983 选定真实消费者、明确所需字段后再做，先交实际用到的少量字段。983/CONSUMERS 四仓整体验收仍按计划核对 980/WEB、982/EXEC 与实际用到的配置接口。

**2026-11-17 用户指令（B 期开工授权）**：用户明确指示先行实施 B-AVSERVICE 与 B-GENERATED 两批并合入主分支供其他计划使用——本指令解除此前"等 985/A 完成再继续 B 组"的等待（CONSUMERS 与 T-05..07 仍按原门保留）。B 前置实测：983/PORT 已随 A 切片落 auto-web master `bdc48e8`；L975/L978 ARCHIVED ✓。

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。








---
plan_id: PLAN-1123
status: archived
feature_name: "AV 标准库 HTTPS 与代理提供者"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-23
plan_revision: 1
current_step: 3
total_steps: 5
depends_on: [PLAN-967, PLAN-974, PLAN-1064, PLAN-1096, PLAN-1114]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-network.md, docs/specs/stdlib/design/av-rust-provider-coverage.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [stdlib, auto-abi, auto-av, ace-m2-build]
---

# [PLAN-1123] AV 标准库 HTTPS 与代理提供者

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

通过 Windows AV 普通工程入口，为已有 HTTP 标准库交付显式 HTTPS/TLS provider 和代理连接。公共行为覆盖直接 HTTPS、经代理转发 HTTP 和经 CONNECT 的 HTTPS，以及验证失败、超时和关闭。

## 2. 架构方案

普通 Auto 请求 → 标准库来源选择 → 已核授权及身份的传输 provider → 真实网络对端。沿 974 的显式选择规则，继续复用公共 HTTP 请求/响应合同。T-01 用普通项目比较 OS 与受检外库接缝，锁定一条可交付的 Windows 来源、部署依赖、错误映射与资源责任。

## 3. 技术栈

`stdlib/ace/std/{net,url}.at`、`stdlib/ace/avrs/net.rs.at`、ACE ABI/M2Build、受控本地 TLS 测试服务和代理。

拟实施树为 `D:/autostack/.wt/lang-1123/auto-lang`，分支 `plan-1123-dev`。

## 4. 需求分析与背景调查

[974 出口](../specs/ace-migration/m2-stdlib-network.md)已验 HTTP 方法族、SSE、基本重定向和 WebSocket 帧层。[1114 B-04](../specs/stdlib/design/av-rust-provider-coverage.md)保留 TLS/proxy 差额；[旧 HTTP 客户端](../../crates/auto-lang-core/src/vm/ffi/stdlib/http_client.rs)有实际 TLS 传输。AutoScape [1119](1119-autoweb-https-readonly-navigation.md)按其宿主网络合同交付，Windows AV 能力完成后可按真实需求消费。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

**执行授权（2026-11-23，auto-plan-goal 范式）**：用户 `/goal 完成计划1123` = work + 独立复审 + 合同内有界修复 + merge 的整链授权（目标/验收/阈值/兼容承诺/仓库与动作范围/预算不变）。实施树 `D:/autostack/.wt/lang-1123/auto-lang`（分支 `plan-1123-dev`，基面 master `dca52e196`；同组 `auto-down` 兄弟 detached `456c86a` 只读消费）；材料区 `D:/autostack/plan-artifacts/plan-1123/`（已登记 `.retention/policy.json` scopes，owner=PLAN-1123，in-flight）。主检出预检：代码路径零 WIP，仅他方会话文档簿记改动与误建空目录 `.wt/lang-1095/`（他人所有，仅上报不处置）。

## 5. 详细设计

T-01 冻结一条显式 Windows provider、直连和代理配置来源、受检信任来源、失败映射和响应/连接所有权。证书与主机名校验必须实际生效。跨请求不得借用前次授权、代理配置或连接；受控服务/代理只用本机回环和局部测试信任。既有 plain HTTP 不要求新 provider。复用已支持的重定向、状态和流消费；代理认证、TLS 客户端证书及更广旧 HTTP API 按旧消费者证据另列，不从测试输出推定已交付。Windows 交付证据交 982/EXEC；Linux 实跑交 1063。

### T-01 冻结记录（2026-11-23，worktree `plan-1123-dev`）

可行性探针（`plan-artifacts/plan-1123/t01-probe`，以交付源 `tls.rs.at` 实跑 6/6 绿）后冻结：

- **D-1123-T01-1 传输来源 = 受检外库**：钉版 `rustls =0.23.46`（`default-features=false features=ring,std,tls12`——默认加密后端 aws-lc-rs 不在离线 registry）+ `rustls-native-certs =0.8.4`，经 `// auto-ffi dep` 注解（新增内联键，两词形字节不变）。离线闭包 36 crate 可解析（锁文件实证）。OS 接缝（SChannel）未选：私有测试信任需原始 SSPI 自建链引擎、仅 Windows、无 Linux 臂。
- **D-1123-T01-2 独立选层 `tls`**（`stdlib/ace/avrs/tls.{at,rs.at}`），不并入 `net.rs.at`：并入会使每个 plain HTTP 消费者携带 TLS 依赖并走 cargo 慢径，违背"既有 plain HTTP 不要求新 provider"。选择=整单元或 `str`+`tls`（`AVRS_TLS_SELECTION`）。
- **D-1123-T01-3 分工与配置来源**：provider 只搬运字节（连接/CONNECT 隧道/受检握手/写/读到流尾/关闭）；HTTP 请求构造与响应解析留在调用方 Auto 源，公共请求/响应合同不变。配置只来自**本次**请求的 spec（`host`/`port`/`trust file:…|system`/`via`/`mode`/`timeout`），无隐式信任来源、无跨调用状态；套接字在调用返回前关闭，应用只拥有响应（owned 句柄，关闭/旧代次在被调方执行前拒）。
- **D-1123-T01-4 失败表**：0；-1 spec、-2 resolve、-3 connect、-4 timeout、-5 proxy.rejected、-6 proxy.closed、-7 proxy.bad、-8 trust 来源、-9 untrusted、-10 hostname、-11 证书其它、-12 握手、-13 write、-14 read、-15 截断（字节保留）、-16 超限（字节保留）。信任与主机名两项分别具名（rustls `UnknownIssuer`/`NotValidForName*`），且发生在写任何请求字节之前。
- **D-1123-T01-5 预算关系与 Windows 事实**：AV 执行默认墙钟 2000 ms（`MAX_AV_WALL`），一次交换=一次 foreign 调用，故默认请求期限取 1500 ms（保证默认配置下具名 `net.timeout` 可观察）；更长 `timeout` 需宿主调高 `ExecLimits.wall`。被拒 SYN 由 Windows 栈重试约 2 s：期限不足时被拒端口表现为 -4。名字解析出的不可达首地址只消耗在试地址均分的期限份额。
- **D-1123-T01-6 日常道不编译**：依赖闭包由显式 `prepare-avrs-tls` 步骤（测试治理工具 `avrs-tls-provider`，为 `ace-abi`/`ace-m2-build` 准备并转发 `ACE_AVRS_TLS_PROVIDER_ROOT`）构建一次；日常用例只校验并打开预备闭包（`ace-abi/provider-closure/1`），缺失/陈旧/损坏=`prepared.missing/stale/corrupt`。工程道以 `RustProjectInputs.prepared` 采用（不符=`m2build.rust-prepared`）。生成的 cargo 单元带空 `[workspace]`（闭包位于工作区树下），并在 dev 档内对**依赖闭包**取 `opt-level = "s"`（单元本体不优化；非 release）：未优化 DLL 3.1–3.4 MiB 无法经 M2Build 发布（见 KNOWN-DEBT `D-1123-01`），优化后 1,824,256 B=1.74 MiB（`str`+`tls`）/1,934,336 B=1.84 MiB（整单元）。
- **公共接口选择的消费者影响**（§10 所询）：`tls` 四个公开名经 `use.rs avrs` 调用；Auto 侧 HTTP 胶水的唯一作者=`experimental/ace/crates/abi/tests/fixtures/tls1123/https_app.at`（两条道同一字节执行）；`stdlib/ace/std/net.at` 零改动（它引用 platform.net 宿主导入，且被 `net_974` 逐字内联）。`spec` 为 Auto 保留字，胶水形参命名 `route`。

**交 982/EXEC 与 1063 的输入**：
- 入口：普通 AV 工程 `use.rs avrs` + `tls_exchange/tls_result/tls_read_byte/tls_close`；胶水=`https_app.at`（请求构造与 `status*10000+body_len` 解析）；provider 闭包=`cargo run -p ace-abi --bin prepare-avrs-tls` 产出的 `experimental/ace/target/debug/avrs-tls`，工程声明 `RustProjectInputs { source, cargo_lock: <闭包锁>, prepared: Some(<闭包目录>) }`。
- 调用参数：spec 头块六键（见 D-1123-T01-3）；执行墙钟须大于请求期限。
- 部署依赖（provider DLL 真实 PE 导入，SDK 打包自动记为 `system-lib`）：advapi32、api-ms-win-core-synch-l1-2-0、api-ms-win-crt-heap-l1-1-0、api-ms-win-crt-runtime-l1-1-0、api-ms-win-crt-string-l1-1-0、bcrypt、bcryptprimitives、crypt32、kernel32、ntdll、vcruntime140、ws2_32；`trust system` 读取当前用户 ROOT 证书库，经 Crypt32（已是 12 个静态导入之一），单元不引用 SSPI 接口；12 个库中 vcruntime140.dll 属 VC++ 运行库而非 OS 组件，目标机须已安装。
- 982 待验证范围：`03_image_scraper.at` 整支迁移（http/file sys 装配）与经 `trust system` 的真实互联网 HTTPS 取页（本计划测试只用回环，未访问外网）。
- 1063 待验证范围：同一源码臂在 Linux 实跑 `avrs_tls_1123` 8 例与 `tls_project_1123` 2 例（Windows 记 NOT_RUN）、Linux registry 上的离线闭包解析、`rustls-native-certs` 的 Linux 原生根库路径。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-network.md`](../specs/ace-migration/m2-stdlib-network.md) | 增加 Windows AV HTTPS/代理来源、公开错误及适用范围 | AC-01/02 |
| SD-02 | modify | [`docs/specs/stdlib/design/av-rust-provider-coverage.md`](../specs/stdlib/design/av-rust-provider-coverage.md) | 更新 TLS/proxy 边界，并标明真实来源与显式选择条件 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 普通 AV 工程向受控服务直接发 HTTPS 请求 | 实际对端见证请求方法和路径，独立检查响应状态及完整字节；换成不受信 CA 或错误主机名，具名失败，无应用响应。 |
| V-02 | AC-01/02 · T-03 | HTTP 代理、HTTPS CONNECT；代理拒绝和关闭 | 代理实际收到转发及 CONNECT，TLS 对端收到真实请求。代理拒绝或超时可区分，资源释放后下一请求仍可执行。 |
| V-03 | AC-02 · T-03 | 去掉 provider 或授权；仓外干净包；plain HTTP 近邻 | 缺失/撤权具名拒绝，预期无对端请求。仓外按发布闭包直连实跑，plain HTTP 按原验收继续通过。 |

冻结验证命令为 `cargo t ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [x] AC-01 · owner_stage=work。直接 HTTPS 与代理 HTTP/HTTPS 的 Windows 公开入口真实执行，V-01/02 的正负对端观察成立。
  - 证据：ace-abi `avrs_tls_1123` 8 例（真回环 rustls 服务与 HTTP 代理：V-01 直连+验证反例、V-02 CONNECT/明文转发/拒绝/关闭/停滞超时、生命周期、撤权、缺层、预备闭包源绑定）与 `ace-m2-build` `tls_project_1123` 2 例（V-03 仓外发布闭包）；逐例计时见 §9。
- [x] AC-02 · owner_stage=work。锁定 provider、配置和诊断合同，V-02/03 的关闭、失败、可重建与无额外授权等边界可重放。向 982/1063 交明确输入。
  - 证据：合同与错误表=`stdlib/ace/avrs/tls.at`；关闭/失败/可重建/无额外授权可重放（`handle_lifecycle…`、`refused_connect…`、`timeout_refused…`、`v03_revoked_grants…`、`v03_provider_missing…`、`p1123_prepared_closure…`）；982/1063 输入见 §5 T-01 冻结记录。
- [x] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
  - 证据：三轮复审（全量核验+38 探针 → 修复轮受影响项 → 文档项复核）全过；F-1123-R1/R2 全部闭合，无新发现（§9 末条收据）。
- [x] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。
  - 证据：冻结并集 run-jxty4khg 零新红（§9 末条收据：13/28/2 红逐名=已知清单，1 例负载瞬时红已孤立与整组复验归因）；SD-01/SD-02 以 reviewed 哈希落主 `4a2054466`；主检冒烟 8/8。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [x] T-01 核定 Windows 传输来源与公开合同
  - 证据：§5“T-01 冻结记录”（D-1123-T01-1..6）：探针以交付源实跑 6/6 绿，离线锁 36 crate 可解析，来源/选择/配置/错误表/预算关系/部署依赖已定。
  - owner_stage=work。以现有 HTTP 普通工程跑本地 TLS 可行性探针，核 974/967 兼容和可重建依赖。固定选择键、配置来源及错误表。
- [x] T-02 交直接 HTTPS 与验证反例
  - 修复轮闭合（`6792336f3`）：F-1123-R1-02 rustfmt=0 hunk；F-1123-R1-03 `port 0`→-1；F-1123-R1-04 call-bytes 说明；-7/-14 AV 道断言；`avrs_tls_1123` 8/8（≤5.84 s/例）。
  - 复审重开（F-1123-R1）：F-1123-R1-02 `tls.rs.at` 未过 rustfmt；顺带 F-1123-R1-03（`port 0`）、F-1123-R1-04（call-bytes 说明）、补 -7/-14 的 AV 道用例。此前证据（下行）保留为历史。
  - 证据：`d694d45f7`：`stdlib/ace/avrs/tls.{at,rs.at}`、ace-abi 预备闭包与 `dep` 内联键、`avrs_tls_1123` 8 例全绿（§9 计时表）。
  - owner_stage=work。依赖 T-01。实现公开请求路径与生命周期控制，执行 V-01。
- [x] T-03 交代理与工程分发证据
  - 修复轮 2 闭合（`b72bf1d9a`）：F-1123-R2-01 SD-01 尾句按实验清单改写（内容检查）；F-1123-R2-02 取整更正。
  - 再复审重开（F-1123-R2）：SD-01 一句“导入表之外无其它运行依赖”被模块差实验推翻（getaddrinfo 侧 7 个 OS 库、trust system 另 MSASN1、回溯路径 dbghelp）；顺带 F-1123-R2-02 取整。此前证据（下行）保留为历史。
  - 修复轮闭合（`6792336f3`）：F-1123-R1-01 规范与 §5 改按实测事实；F-1123-R1-08 尺寸更正并撤回未实测的余量数字；B-12 补共享生成器备注；`ace-m2-build` 定向 12/12。
  - 复审重开（F-1123-R1）：F-1123-R1-01 SD-01/SD-02 的 Crypt32/SSPI 表述不实；顺带 F-1123-R1-08（DLL 尺寸）与 B-12 一句备注。此前证据（下行）保留为历史。
  - 证据：`d694d45f7`：`prepare-avrs-tls` 与测试治理工具登记（`cargo t ace-m2-build` 47 执行 PASS）、m2-build `RustProjectInputs.prepared`、`tls_project_1123` 2 例；982/1063 输入见 §5；债务 `D-1123-01` 已登记。
  - owner_stage=work。依赖 T-02。运行 V-02/03，提供 982 和 1063 的源、调用参数、provider 及待验证范围。
- [x] T-04 独立复审
  - 证据：T-04=三轮复审链：R1 needs_fix→修复 6792336f3→R2 needs_fix（一句文档）→修复 b72bf1d9a→R3 pass（final）；绑定 reviewed_commit b72bf1d9a。
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [x] T-05 冻结验证与归档
  - 证据：rebase（range-diff 补丁等价）→ 冻结门 → ff-only 落主 → canonical 哈希核验 → 归档/清理（§9 末条 merge 收据，completion_kind=delivered）。
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1123 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 1 | from: goal | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: dca52e196（base） | next: work

2026-11-23，stage=work，PLAN-1123 r1（/auto-plan:work，/auto-plan-goal 整链授权）。**work 交接（readiness）**：

- work-owned AC-01/02 全过（§7/§8 逐项证据）；无未闭 in-contract 发现，无未决合同决定。
- 本会话实跑的测试模块/范围（机器同时有他会话 `-j4/-j2` auto-ui-runtime 作业，本会话构建并发限 `CARGO_BUILD_JOBS=4`；统一并集未跑）：

| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| `nextest -p ace-abi --test conformance -E "avrs_tls_1123 \| avrs_stdlib_1114"`（定向） | 20 执行 / 20 PASS / 0 排除 / 0 NOT_RUN | 增量构建，未单列 | 未单列 | 25.9 s | 未单列 | 32 s |
| `nextest -p ace-m2-build --test tls_project_1123 --test avrs_project_1114 --test sdk_pack`（定向） | 12 / 12 PASS / 0 / 0 | 增量构建，未单列 | 未单列 | 6.7 s | 未单列 | 10 s |
| `cargo t ace-m2-build`（固定组，经真实测试治理入口） | 47 / 47 PASS / 0 排除 / 0 not_run，gate_passed | 工具 `avrs-tls-provider` 构建 38.6 s（冷）+ 预热 0.39 s（两闭包校验通过，`ACE_AVRS_TLS_PROVIDER_ROOT` 已转发） | 19.0 s | 25.5 s | 未单列 | 44.5 s |
| 定稿复跑 `nextest … -E "avrs_tls_1123"`（含格式化与重定向断言） | 8 / 8 PASS / 0 / 0 | 增量，未单列 | 未单列 | 9.6 s | 未单列 | 未单列 |

- 新例行为与实测（dev 档；nextest 每例墙钟，含夹具起停）：

| 新例 | 行为与 oracle | 实测秒 | 结果 / ≤10 s |
|---|---|---|---|
| `avrs_tls_1123::v01_direct_https_peer_observes_request_status_and_bytes` | 直连 HTTPS：对端记录 GET/POST 请求行、头与体字节；Auto 侧独立解析 status/体长并逐字节比对（20000141、20100071）；DNS 名与 IP、PEM 与 DER 信任源；单跳重定向 302→`/next`=200（50200062） | 1.058 | PASS / 是 |
| `avrs_tls_1123::v01_verification_failures_are_named_and_send_nothing` | 错主机名 -10、不受信签发者 -9、系统根库 -9、无信任源 -1、不可读信任文件 -8；三个对端零请求；随后受信请求通过（无粘滞状态） | 0.336 | PASS / 是 |
| `avrs_tls_1123::v02_proxy_connect_plain_forward_and_verdicts` | CONNECT 隧道（代理记录 `CONNECT` 行，TLS 对端收真实请求）；明文经代理逐字节转发（绝对形式）；拒绝 -5 / 关闭 -6 / 停滞 -4（≥380 ms）；失败后健康请求仍通 | 0.768 | PASS / 是 |
| `avrs_tls_1123::refused_connect_is_timeout_then_connect_by_the_deadline` | 默认期限下被拒端口=-4；`timeout 4000` + 墙钟 6 s=-3；同请求默认墙钟=执行器 `ExecutionWall` | 5.880 | PASS / 是 |
| `avrs_tls_1123::timeout_refused_and_truncated_close_are_named` | 静默对端握手超时 -4（≥280 ms）；无 close_notify 截断保留字节（观测 20000141）；同单元明文回环近邻=9 | 0.550 | PASS / 是 |
| `avrs_tls_1123::handle_lifecycle_closed_and_stale_are_rejected_before_the_callee` | 关闭/双关闭/旧代次句柄=`foreign.handle`（被调方未执行）；旧会话仍答其合法句柄 | 0.443 | PASS / 是 |
| `avrs_tls_1123::v03_revoked_grants_reject_before_any_dispatch` | 去 foreign 授权=具名能力拒、dispatch=0、对端零请求 | 0.195 | PASS / 是 |
| `avrs_tls_1123::v03_provider_missing_and_prepared_closure_is_source_bound` | 无 `tls` 层的 provider=`source.undefined` 点名交付合同中的 tls op（零构建）；`prepared.missing/stale/corrupt` 三反例，不建目录不编译 | 0.380 | PASS / 是 |
| `tls_project_1123::p1123_tls_project_builds_runs_and_sdk_relocates_from_the_prepared_closure` | 普通工程 build→run→SDK 打包（12 个系统库 `system-lib` 依赖）→重定位→删开发缓存→自迁出 SDK 运行同码 40000273；对端 4 请求 / 2 CONNECT；工程道零 provider 编译 | 1.433 | PASS / 是 |
| `tls_project_1123::p1123_prepared_closure_must_match_the_project_exactly` | 另一锁 / 缺失 / 另一源三反例=`m2build.rust-prepared`，不编译 | 0.588 | PASS / 是 |

- 复用的未变证据：plain HTTP 原验收沿用 `net_974`（`ace-av`）与 `p1114_avrs_project…`，在 merge 冻结并集中执行；本会话同单元明文近邻已实跑（上表）。
- 未执行/排除：新增面无排除、无 NOT_RUN；Linux 腿=2026-11-14 裁定排除，归 1063。未在运行期演练、仅由代码路径承载的结果：-16 超限（64 KiB 请求 / 1 MiB 响应）、-2 resolve、-7 proxy.bad、-11 证书其它失败、-13 write、-14 read（复审可据合同复核或要求补例）。测试治理 Python 集（`scripts/tests` 六文件读 `toolchains.json`/`test-groups.toml`）：445 过 / 30 败 / 14 错，失败集合与 master 逐 ID 相同（44 个，均非本计划引入）。**统一冻结并集 `cargo t ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity` 待 merge 门执行**（owner=merge，T-05）。
- 代码/依赖/配置身份：worktree `D:/autostack/.wt/lang-1123/auto-lang` @ `plan-1123-dev` `d694d45f7`（base master `dca52e196`）；依赖 967/974/1064/1096/1114 全 archived；`experimental/ace/Cargo.lock` +3 条 dev 依赖边（`--locked` 通过）；测试治理 `toolchains.json` +1 工具、`test-groups.toml` 2 组 `tools` 增项。
- 债务候选（已登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md`）：`D-1123-01` M2Build 发布收据十六进制嵌入全部产物 vs 783 暂存预算 8 MiB——本计划未改任何预算，以 dev 档内依赖闭包尺寸优化 + `str`+`tls` 选择交付，整单元余量仅 ~0.26 MiB。
- 环境观察：他会话在本会话期间持续有 cargo 作业（非统一门，未等待）；PATH 上无 `dumpbin/objdump`，PE 导入以脚本解析。

stage: work | plan_id: PLAN-1123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | base_commit: dca52e196 | code_commit: d694d45f7 | dependency_revisions: 967/974/1064/1096/1114 archived | task_ids: T-01,T-02,T-03 | evidence: 本节两张表与 §8 逐任务证据 | blockers: 无 | next_responsible_role: review（T-04）

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: d694d45f7 | next: review

2026-11-23，stage=review（final），PLAN-1123 r1。**outcome: needs_fix**（两项 minor 修复合并前必改；无 blocking/major 代码缺陷）。
- reviewer：独立复审子代理上下文（与实施上下文分离；在同一实施树上只读+定向测试；探针只写 `plan-artifacts/plan-1123/review-probe/`）。复审主持方逐项对工件复核了其结论。
- reviewed_commit `d694d45f7`（base `dca52e196`=master，未漂移；worktree 干净，复核后仍干净）。spec_inputs：SD-01 `m2-stdlib-network.md`@HEAD sha256 `9d1b2923053bf7cc…e6cf1bb`（master `62f23dabb60f19f1…b729948d`）；SD-02 `av-rust-provider-coverage.md`@HEAD `3598a3e547a94850…c6e5075`（master `6adb85a705318821…93380f48`）。依赖 967/974/1064/1096/1114 archived。无旧账本写入。

**acceptance_results**（AC → 独立重放/检查的证据）：

| AC / 条款 | 结果 | 证据 |
|---|---|---|
| AC-01 直接 HTTPS | pass | 重放 `v01_direct_https…`：对端记录 GET/POST 请求行、头与体；Auto 侧 20000141/20100071；302→`/next`=20000141+… 的单跳重定向两请求均到达对端 |
| AC-01 代理 HTTP / CONNECT HTTPS | pass | 重放 `v02_proxy…`：`CONNECT` 行逐字、绝对形式明文转发、-5/-6/-4（≥380 ms）、失败后健康请求恢复 |
| AC-01 验证失败 / 超时 / 关闭 | pass | -9/-10 时对端零请求；-4；-15 保留字节 |
| AC-02 校验实际生效 | pass | `tls.rs.at` 无 `dangerous()`/自定义校验器/不安全来源；空根库 -8；系统信任仅经显式 `trust system`。探针：过期叶 -11、未生效叶 -11、`trust insecure` -1 且零请求 |
| AC-02 无跨请求借用 | pass | 文件内无 static/OnceLock/线程局部/缓存；`transport` 各返回路径均释放套接字；探针 A/B 交错交换游标独立 |
| AC-02 响应/连接所有权 | pass | AV 侧 `foreign_exec.rs` 句柄表+代次/类校验先于原函数调用；重放 `handle_lifecycle…` |
| AC-02 日常道不编译 | pass | `verify_provider_closure` 只读/哈希/import；m2-build `adopt_prepared` 先于任何构建；缺失/陈旧/损坏分支具名且不建目录 |
| AC-02 墙钟关系 | pass | `MAX_AV_WALL`=2000 ms；`refused_connect…` 第三腿重放=执行器 `ExecutionWall` |
| AC-03（owner_stage=review） | 本轮执行；**needs_fix** | 见 findings F-1123-R1-01/02 |

**findings**：
- **F-1123-R1-01**（minor，文档/收尾；T-03、SD-01 部署依赖行、SD-02 B-13）：两处写“`trust system` 在运行期动态装载 SSPI/Crypt32、不在导入表内”，但 `crypt32.dll` 是 12 个静态导入之一（本计划自己的测试即断言之）。主持方复核事实：单元 DLL 按名引用 `CertOpenStore`/`CertEnumCertificatesInStore`/`CertFreeCertificateContext`/`CertCloseStore`（Crypt32 证书库），**不引用任何 SSPI 接口**（`AcquireCredentialsHandleW`/`InitializeSecurityContextW` 计数 0）；复审代理建议的替换措辞（“SSPI 运行期装载”）同样无据，修复按核实事实改写为“读取当前用户 ROOT 证书库，经 Crypt32 静态导入，已在 12 库之列”。
- **F-1123-R1-02**（minor，健康门；T-02）：`stdlib/ace/avrs/tls.rs.at` 未过 `rustfmt --check`（主持方复核 3 处 hunk；同级 net/str/json 为 0）。格式化改变 provider 摘要，须重新预备闭包。
- F-1123-R1-03（note，合同澄清；T-02）：`port 0` 被解析接受（仅 `via` 拒 0，`tls.rs.at:74/149`），合同面写 `port <1..65535>`；观测 -3。
- F-1123-R1-04（note，合同澄清；T-02）：AV `MAX_CALL_BYTES = 65_536`（`foreign_exec.rs:27`）先约束整个 spec，-16 的请求腿经 AV 前门不可达；合同面应说明。
- F-1123-R1-05（note）：-13 net.write 未在运行期观察到（回环写缓冲即成功），路径经代码检查。
- F-1123-R1-06（note，既有）：1114 `stage()` 用例仍在测试内构建 provider（既有做法，本计划缩小了该面）。
- F-1123-R1-07（note）：校验路径内 `rustc --version` 子进程（身份绑定，不编译；与既有车道同做法）。
- F-1123-R1-08（note）：DLL 尺寸实测 1,824,256 B=1.74 MiB（str+tls）、1,934,336 B=1.85 MiB（整单元），此前记 1.73/1.84。
- 备注（非缺陷）：`opt-level = "s"` 的生成器为 967 各 cargo 单元共享（serde_json 单元同样受益），此前缓存的未优化 DLL 在输入不变时会被复用；宜在 B-12 补一句。

**裁定：-16/-2/-7/-11/-13/-14 的运行期用例**——合同（§6/§7）未逐码枚举为必需，不构成阻塞。探针实测（直接 `include!` 交付源，回环）：-2（不存在的主机/代理）、-7（垃圾/无空格状态行）、-11（过期与未生效叶）、-14（RST 中断，0 字节保留）、-16（64 KiB 恰好通过、+1 拒且零对端字节；1 MiB 恰好通过、+1 拒且保留恰 1 MiB）均与合同一致；-13 未能触发。修复轮补 -7、-14 的 AV 道永久用例（回环、确定性）；-16/-11/-13 以本条的探针输入/观测为可重放证据；-2 依赖真实 DNS，不入日常。其它探针（合同未言明、行为合理）：大写主机名通过（RFC 6125 不区分大小写）、尾点主机名通过、未加方括号的 `::1` 可解析、CRLF 空行分隔=-1（合同面规定 `\n\n`）、双 CA 的 PEM 捆绑两根均受信、代理 CONNECT 200 后关闭=-12、407=-5。

**测试纪律**：新增 10 例复审重放逐例墙钟全部 ≤10 s，无可达隐式构建（含失败分支），profile=dev，工具预热为显式准备阶段。

| 新例（复审重放） | 实测秒 |
|---|---|
| avrs_tls_1123::v01_direct_https_peer_observes_request_status_and_bytes | 1.049 |
| avrs_tls_1123::v01_verification_failures_are_named_and_send_nothing | 0.333 |
| avrs_tls_1123::v02_proxy_connect_plain_forward_and_verdicts | 1.069 |
| avrs_tls_1123::refused_connect_is_timeout_then_connect_by_the_deadline | 6.073 |
| avrs_tls_1123::timeout_refused_and_truncated_close_are_named | 0.552 |
| avrs_tls_1123::handle_lifecycle_closed_and_stale_are_rejected_before_the_callee | 0.706 |
| avrs_tls_1123::v03_revoked_grants_reject_before_any_dispatch | 0.268 |
| avrs_tls_1123::v03_provider_missing_and_prepared_closure_is_source_bound | 0.558 |
| tls_project_1123::p1123_tls_project_builds_runs_and_sdk_relocates_from_the_prepared_closure | 1.448 |
| tls_project_1123::p1123_prepared_closure_must_match_the_project_exactly | 1.162 |

| 本轮复审实跑 | 结果 | 测试 | 总墙钟 |
|---|---|---|---|
| `nextest -p ace-abi --test conformance -E "avrs_tls_1123 \| avrs_stdlib_1114"` | 20/20 PASS | 26.1 s | 未单列 |
| `nextest -p ace-m2-build --test tls_project_1123 --test avrs_project_1114 --test sdk_pack` | 12/12 PASS | 7.4 s | 未单列 |
| 复审代理探针集（Rust 级，scratch：spec_negatives/net_negatives/boundaries/trust_and_names/interleave） | 15+8+4+10+1 全绿 | 未单列 | 未单列 |

（复审时机器无竞争 cargo 作业；预备步骤未需运行；发现/清理阶段未单列。work 阶段未变证据不重跑：`cargo t ace-m2-build` 47/47，主持方已对 `results.json` 原始记录核对 47 ok / 0 排除 / 0 not_run。）健康检查：本计划新增/改动文件零编译器警告、零调试打印；唯一格式缺口即 F-1123-R1-02。

pending_closeout_gates：统一冻结并集（merge 门）。next: work（有界修复 F-1123-R1-01/02，顺带 03/04/08；修复循环 1/3）→ 受影响项再复审。

stage: review | review_scope: final | reviewer: 独立复审子代理上下文（同树只读+测试；主持方复核） | plan_id: PLAN-1123 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | reviewed_commit: d694d45f7 | base_commit: dca52e196 | dependency_revisions: 967/974/1064/1096/1114 archived | spec_inputs: SD-01/SD-02 @d694d45f7（哈希见上） | acceptance_results: AC-01 pass、AC-02 pass、AC-03 needs_fix | pending_closeout_gates: 冻结并集（merge） | findings: F-1123-R1-01（minor）、F-1123-R1-02（minor）、F-1123-R1-03..08（note） | evidence: 本节 | next: work

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: d694d45f7 | next: work

2026-11-23，stage=work（有界修复，修复循环 1/3），PLAN-1123 r1。修复提交 `6792336f3`（在 `d694d45f7` 之上；base 仍为 `dca52e196`）。**outcome: pass**（work-owned AC-01/02 复核全过；无未闭 in-contract 发现）。

**逐发现闭合行**（finding → invariant → 受影响路径/状态 → 修复与永久用例 → 当前结果 → 余量）：

| 发现 | 不变量 | 受影响路径 | 修复 / 永久用例 | 当前结果 | 余量 |
|---|---|---|---|---|---|
| F-1123-R1-01 规范表述不实 | 规范只陈述经核实的行为 | SD-01 部署依赖行、SD-02 B-13、本计划 §5 交接输入 | 按实测改写：`trust system` 经 Crypt32 读取当前用户 ROOT 证书库，Crypt32 是 12 个静态导入之一，单元不引用 SSPI 接口（DLL 按名引用 `CertOpenStore`/`CertEnumCertificatesInStore`/`CertFreeCertificateContext`/`CertCloseStore`；`AcquireCredentialsHandleW`/`InitializeSecurityContextW` 计数 0）；并注明 vcruntime140.dll 属 VC++ 运行库而非 OS 组件 | 两份规范与 §5 中“动态装载”字样=0（内容检查） | 无 |
| F-1123-R1-02 格式 | 本计划新增源过 rustfmt | `stdlib/ace/avrs/tls.rs.at` | `rustfmt --edition 2021`（纯格式：+4/−10 行，无语义改动，无 `dyn `） | `rustfmt --check`=0 hunk；本计划四个 Rust 文件均 0 | 无 |
| F-1123-R1-03 `port 0` | 代码与合同面一致（`port <1..65535>`） | `parse_spec` 的 `port` 臂（`via` 已拒 0，目标端口未拒） | 解析时拒绝→`-1`；永久断言 `send(0, &file, "") == -1`（`v01_verification_failures…`） | 实测 -1（修前复现代理探针=-3） | 无 |
| F-1123-R1-04 call-bytes | 合同面说明执行器在先的预算 | `tls.at`、SD-01 预算关系行 | 补 `MAX_CALL_BYTES`=65536 作用于整个 spec，-16 请求腿经 AV 前门不可达 | 内容检查 | 无 |
| 复审裁定补例 -7 / -14 | 诊断结果可经 AV 道重放 | 代理回以非 HTTP 状态行；对端在响应中途复位 | `ProxyBehavior::Garbage`→`-7`（`v02_proxy…`）；`CloseStyle::Reset`（逐字节完成握手、回应部分响应、请求始终未读即关闭→栈复位）→`-14` 且对端零请求记录（`timeout_refused_and_truncated…`） | 实测 -7、-14（两例内断言，随例全绿） | 无 |
| F-1123-R1-05 `-13` 写失败 | — | `tls.rs.at` 写路径 | 未补例：回环写缓冲即成功，无确定性触发 | 仍为代码检查 | 合同未要求运行期用例（复审裁定） |
| F-1123-R1-06（既有）/ R1-07（`rustc --version` 身份绑定） | — | — | 不修改：既有做法/既有车道同形 | 记录 | 无 |
| F-1123-R1-08 尺寸 | 数字为实测 | SD-02 B-12、§5、债务 `D-1123-01` | 更正为 1,824,256 B（1.74 MiB）/1,934,336 B（1.85 MiB）；撤回无实测支撑的“整单元余量 ~0.26 MiB”，改为预算算式（收据≈2×十六进制(DLL+ABC)，再计暂存两份，余量随 ABC 体积变化） | 内容检查 | 如需精确余量须以整单元实测发布一次 |

同因面已扫：`via` 的端口 0 早已拒绝；`timeout` 上下界既有；-15/-16 的描述同步写准（复位不保留字节；响应超限保留前 1 MiB）。规范 B-12 另补一句：单元生成器为 967 各 cargo 单元共享，serde_json 单元等同样在 dev 档内取依赖尺寸优化，既有缓存的未优化 DLL 在输入不变时继续复用。

**本修复轮实跑**（他会话同时有 `cargo t ace-source ace-av ase-m0 ace-python --prepare-only` 准备作业与 nextest archive，其 target 另置；本会话 `CARGO_BUILD_JOBS=4`）：

| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| `prepare-avrs-tls`（显式准备，两闭包重建） | exit 0 | 6.7 s | — | — | — | 6.9 s |
| `nextest -p ace-abi --test conformance -E "avrs_tls_1123"`（定稿复跑） | 8 执行 / 8 PASS / 0 排除 | 增量，未单列 | 未单列 | 9.8 s | 未单列 | 12 s |
| `nextest -p ace-m2-build --test tls_project_1123 --test avrs_project_1114 --test sdk_pack`（定稿复跑） | 12 / 12 PASS / 0 | 增量，未单列 | 未单列 | 6.5 s | 未单列 | 9 s |
| `cargo check --locked -p ace-abi -p ace-m2-build --tests --bins` | exit 0；本计划文件诊断=0（与警告位置集合交集为空） | 1.8 s | — | — | — | — |

- 新例行为与实测（dev 档；nextest 每例墙钟）：**无新增用例**，三个既有用例各增断言（`port 0`→-1；垃圾代理→-7；复位对端→-14）。

| 用例 | 本轮新增断言的行为 | 实测秒 | ≤10 s |
|---|---|---|---|
| avrs_tls_1123::v01_direct_https_peer_observes_request_status_and_bytes | （未改） | 1.030 | 是 |
| avrs_tls_1123::v01_verification_failures_are_named_and_send_nothing | `port 0` 具名 spec 失败，零发送 | 0.351 | 是 |
| avrs_tls_1123::v02_proxy_connect_plain_forward_and_verdicts | 代理回非 HTTP 状态行=-7，对端未达 | 0.815 | 是 |
| avrs_tls_1123::refused_connect_is_timeout_then_connect_by_the_deadline | （未改） | 5.836 | 是 |
| avrs_tls_1123::timeout_refused_and_truncated_close_are_named | 对端响应中途复位=-14，对端零请求记录 | 0.743 | 是 |
| avrs_tls_1123::handle_lifecycle_closed_and_stale_are_rejected_before_the_callee | （未改） | 0.453 | 是 |
| avrs_tls_1123::v03_revoked_grants_reject_before_any_dispatch | （未改） | 0.191 | 是 |
| avrs_tls_1123::v03_provider_missing_and_prepared_closure_is_source_bound | （未改；闭包因源变更重建后仍校验通过） | 0.373 | 是 |
| tls_project_1123::p1123_tls_project_builds_runs_and_sdk_relocates_from_the_prepared_closure | （未改） | 1.409 | 是 |
| tls_project_1123::p1123_prepared_closure_must_match_the_project_exactly | （未改） | 0.469 | 是 |

- 复用的未变证据：`avrs_stdlib_1114` 12 例（不装配 `tls` 层，其输入自复审轮未变）沿用复审轮重放 20/20 中对应部分；`cargo t ace-m2-build` 47/47（第 1 轮）因闭包重建而不复用其工具预热计时，归 merge 冻结并集重跑。
- 统一冻结并集 `cargo t ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity` 仍待 merge 门。**master 自 base 起已前进**（`bafc880f1`/`eb3315247`/`674eda6d7`，PLAN-1127 落主），merge 时需 rebase 并做补丁等价核对。
- 代码身份：`plan-1123-dev` @ `6792336f3`（累计对 base 32 文件 +3084/−35）；worktree 干净。

stage: work | plan_id: PLAN-1123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | base_commit: dca52e196 | code_commit: 6792336f3 | dependency_revisions: 967/974/1064/1096/1114 archived | task_ids: T-02,T-03（复审重开项已闭） | evidence: 本节闭合行与计时表 | blockers: 无 | next_responsible_role: review（受影响项再复审）

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: 6792336f3 | next: review

2026-11-23，stage=review（final 再复审，修复循环 1），PLAN-1123 r1。**outcome: needs_fix**（仅一处 minor 文档断言；无代码/测试/接线/时序/纪律问题）。
- reviewer：独立复审子代理上下文（与第 2 轮同一复审上下文续用，含其探针；同树只读+测试）。主持方对关键事实复核：`LoadLibraryA` 确在 DLL 导入名表（我此前已独立确认）；模块差实验日志复核属实。
- reviewed_commit `6792336f3`（parent `d694d45f7`，base `dca52e196`；worktree 干净；master 已前进 6 提交，与本分支 3 文件重叠，归 merge 处理）。spec_inputs：SD-01@HEAD `7d84960f6e260463…`、SD-02@HEAD `7ccba2227fff53e2…`。复审期间有他会话 cargo 准备作业（并发限 4，无干扰）；预备步骤未需运行。

**逐项闭合核验**（复审代理独立复核）：
| 前发现 | 闭合 | 证据 |
|---|---|---|
| F-1123-R1-01 Crypt32/SSPI 表述 | **部分——正句其余全部证实（crypt32 静态导入=当前用户 ROOT 枚举；零 SSPI 引用），新加的"导入表之外无其它运行依赖"被推翻** | 见 F-1123-R2-01 |
| F-1123-R1-02 rustfmt | 闭合 | tls.rs.at 与本计划全部 Rust 文件 0 hunk；修复 diff=预期 `port` hunk+三处纯格式 |
| F-1123-R1-03 `port 0` | 闭合 | 探针 12 断言：0/00/+0/-0/-1/65536/80abc/空→-1；65535 可解析；`via` 端口 0→-1；AV 道永久断言绿 |
| F-1123-R1-04 call-bytes | 闭合 | `MAX_CALL_BYTES=65_536`（foreign_exec.rs:27），dispatch_owned 内先于 provider 调用检查（:1435-1441），`foreign.call-bytes` 为执行器 Infra 结果 |
| F-1123-R1-05 -13 | 不要求 | 闭合行如实记录为仅代码检查 |
| F-1123-R1-06/07 | 记录 | 不变 |
| F-1123-R1-08 尺寸 | 闭合 | 字节数 1,824,256/1,934,336 复测一致；未实测的余量数字已撤回（四舍五入见 F-1123-R2-02） |
| B-12 共享生成器/缓存复用句 | 闭合且属实 | `build_cargo_provider_with_lock` 单一生成器；m2-build `prepare` 按 cdylib.sha256+存在性复用；BuildIdentity 键于摘要+rustc+锁 |

**新发现**：
- **F-1123-R2-01**（minor，文档过度断言；SD-01 部署行尾句；影响 AC-04、T-03）：“导入表之外无其它运行依赖”被实证推翻——① kernel32 导入含 `LoadLibraryA`/`GetProcAddress`/`GetModuleHandleExW`，`LoadLibraryA` 调用方为 rust std 回溯支持（dbghelp 路径）；② 模块差实验：`file:` 路径按需映射 mswsock/DNSAPI/DSPARSE/IPHLPAPI/NSI/rasadhlp/fwpuclnt（getaddrinfo），`trust system` 另映射 MSASN1；③ 两条交付路径均未映射 dbghelp。所列附加模块全为操作系统组件——交付结论（12 个 `system-lib` 为静态要求、无需第三方随包）成立，仅全称否定句错误。修复：按复审代理给出的、有实验支撑的句子替换。
- F-1123-R2-02（note，取整）：1,934,336 B=1.8449 MiB，写"1.85 MiB"处应为 1.84（SD-02 B-12、债务 `D-1123-01`、计划闭合行）。
- F-1123-R2-03（note，仅记录）：`CloseStyle::Reset` 复位用例的确定性——8/8 稳定（0.71–1.23 s）；理论上极端调度下客户请求晚于 150 ms 睡眠到达会把 -14 变为 -15（同为合同值），同进程线程风险极低，不要求动作。

**重放**（复审代理）：`ace-abi`（avrs_tls_1123+avrs_stdlib_1114）20/20，24.7 s；`ace-m2-build` 12/12，6.5 s；复位用例连跑 8 次全绿；探针 `r2_repair.rs` 全绿。新增用例仍为 0（三断言在既有用例内），逐例 ≤ 5.83 s。健康：本计划文件零警告、零调试打印；规范与债务文本为规则/事实形。

acceptance_results：AC-01/02 维持 pass（证据不变且已复验）；AC-03 本轮=needs_fix（F-1123-R2-01）。
pending_closeout_gates：统一冻结并集（merge）。
next: work（有界修复循环 2/3：替换 SD-01 一句 + 取整更正；纯文档修复以内容检查核验）→ 快速再复审。

stage: review | review_scope: final（再复审） | reviewer: 独立复审子代理上下文（第 2 轮上下文续用；主持方复核） | plan_id: PLAN-1123 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | reviewed_commit: 6792336f3 | base_commit: dca52e196 | dependency_revisions: 967/974/1064/1096/1114 archived | spec_inputs: SD-01@6792336f3 7d84960f…、SD-02@6792336f3 7ccba222… | acceptance_results: AC-01 pass、AC-02 pass、AC-03 needs_fix | pending_closeout_gates: 冻结并集（merge） | findings: F-1123-R2-01（minor）、F-1123-R2-02（note）、F-1123-R2-03（note） | evidence: 本节与 review-probe/logs/r2-* | next: work

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 4 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: 6792336f3 | next: work

2026-11-23，stage=work（有界修复循环 2/3），PLAN-1123 r1。修复提交 `b72bf1d9a`（`6792336f3` 之上；base 仍 `dca52e196`）。**outcome: pass**。纯文档修复，按 work 技能以内容检查核验。

**逐发现闭合行**：
| 发现 | 不变量 | 受影响路径 | 修复 / 核验 | 当前结果 | 余量 |
|---|---|---|---|---|---|
| F-1123-R2-01 “导入表之外无其它运行依赖”被模块差实验推翻 | 规范只陈述经核实的行为 | SD-01 部署行尾句 | 按复审代理给出的、有实验支撑的句子替换：列明运行期按需映射的 OS 辅助库（getaddrinfo 侧 mswsock/DNSAPI/NSI/rasadhlp/fwpuclnt/DSPARSE/IPHLPAPI，`trust system` 另 MSASN1，panic 回溯另 dbghelp），均无需随 SDK 声明或分发 | 内容检查：旧句在全部活性文档=0；新句在位（依赖复审实验日志 `r2-modules2.log` 的实测清单） | 无 |
| F-1123-R2-02 取整 | 数字与实测一致 | SD-02 B-12、§5、债务 `D-1123-01` | 1,934,336 B→1.84 MiB（=1.8449） | 内容检查：1.85 在活性文档=0（历史 §9 收据行按“不改写收据”保留） | 无 |
| F-1123-R2-03 复位用例残余风险 | — | `CloseStyle::Reset` | 不动作（复审 note：8/8 稳定，极端调度下或落 -15，同为合同值） | 已记录 | 无 |

**本轮实跑**：无产品测试（纯文档修复；核验=内容/差异检查，如上）。新增用例=0；上一轮全部测试证据（abi 8/8、m2-build 12/12、复位 8/8、探针）输入未变，复用。统一冻结并集仍待 merge 门。

stage: work | plan_id: PLAN-1123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | base_commit: dca52e196 | code_commit: b72bf1d9a | dependency_revisions: 967/974/1064/1096/1114 archived | task_ids: T-03（再复审重开项已闭） | evidence: 本节闭合行（内容检查） | blockers: 无 | next_responsible_role: review（受影响项快速再复审）

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: b72bf1d9a | next: review

2026-11-23，stage=review（final 再复审，修复循环 2），PLAN-1123 r1。**outcome: pass（final）**。
- reviewer：独立复审子代理上下文（第 2/4 轮同一上下文续用；同树只读）。reviewed_commit `b72bf1d9a`（parent `6792336f3`，base `dca52e196`；worktree 干净；diff=两份规范各 1 行，纯文档）。
- F-1123-R2-01 闭合：SD-01 尾句已换为实证清单（getaddrinfo 侧 7 库、trust system 另 MSASN1、回溯路径 dbghelp，均无需随 SDK 声明）；所列库与其实验逐一相符，无未观察项。F-1123-R2-02 闭合：两规范无 "1.85 MiB"。两规范 PLAN-1123 节全文复读：无其它过度断言（其余条款均映射其第 1/2 轮已验证事实）。新发现：无。
- 复用证据：本轮纯文档，未跑产品测试（核验=内容检查）；前两轮全部重放与探针证据（abi 20/20、m2-build 12/12、复位 8/8、模块差实验）绑定输入未变。新增用例=0。
- AC-03 完成（三轮复审：首轮全量核验+38 探针、修复轮受影响项复核、本轮文档项复核）。pending_closeout_gates：统一冻结并集（merge）。
- spec_inputs（冻结的规范增量身份）：SD-01 `m2-stdlib-network.md`@b72bf1d9a sha256 起于 `8f3ab18df27ef88d`；SD-02 `av-rust-provider-coverage.md`@b72bf1d9a sha256 起于 `66f833884eb7cf4c`。
- status: **reviewed**；next: merge（T-05）。

stage: review | review_scope: final（再复审 2） | reviewer: 独立复审子代理上下文（主持方复核） | plan_id: PLAN-1123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1123/auto-lang | branch: plan-1123-dev | reviewed_commit: b72bf1d9a | base_commit: dca52e196 | dependency_revisions: 967/974/1064/1096/1114 archived | spec_inputs: SD-01/SD-02 @b72bf1d9a（哈希见上） | acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass | pending_closeout_gates: 冻结并集（merge） | findings: 无（F-1123-R1/R2 全闭） | evidence: 本节 | next: merge

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 6 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1123/auto-lang | commit: b72bf1d9a | next: merge

2026-11-23，stage=merge，PLAN-1123 r1。**prepared**：reviewed 基线 `b72bf1d9a`（final pass，修复循环 2）；master 漂移至 `a3e0c4586`（PLAN-1112/1125/1126/1127 落主，与本分支 3 文件文本相邻）。`plan-1123-dev` rebase 到 master：唯一冲突=`test-groups.toml` ace-abi 组 `tools` 行（master 加 `av-providers`、本计划加 `avrs-tls-provider`，合并两侧）；`git range-diff dca52e196..b72bf1d9a HEAD~3..HEAD`：提交 2/3 补丁全等（`6792336f3=3e95e7f55`、`b72bf1d9a=4a2054466`），提交 1（`d694d45f7!f0ea2da9f`）差异仅上述 tools 行与 `rust_provider.rs` 一处上下文行合并（无语义差）；两份规范哈希在重定基尖端=reviewed 值；stdlib 源零变化（预备闭包仍有效）；`cargo check --locked` 干净。**交付候选=重定基尖端 `4a2054466`**。

**冻结门禁收据（基线 `4a2054466`，run-jxty4khg，墙钟 206.5 s）**：
- 准备：首跑一工具 rustc 进程瞬时失败（重试即过）；一次收据含陈旧工具哈希（多轮 prepare 累积，清 `target/test-groups` 后单次 prepare+verify 通过）。
- `ace-stdlib` PASS（2 执行）；`ace-m2-build` PASS（47 执行，含 `tls_project_1123` 2 例）。
- `ace-av` 1629 执行 13 红——逐名=已知清单（av_method_modes、av_method_owned×5、av_owned_first_case、av_owned_recipe、av_test_batch::c_data_profile、component_event_sessions_1106::v02b、resource_915×3、layout::tests），与 PLAN-1121 收据的 master 红名单一致（D-1055-BASE-01/D-1088-LAYOUT-01 归属）。
- `ace-abi` 350 执行 29 红——28 个 `source_foreign` 族=已知预存（1114 已对主检逐族核对；本日组内重跑=恰 28）；另 1 例 `binding::abi_worker_concurrent_gc_same_store` 首跑红：主检 master 孤立重跑 PASS（2.97 s）、本树孤立 PASS（2.80 s）、本树整组重跑 PASS（322 ok/28 已知红）→ 判定为全并集负载下的瞬时红（并发 GC 用例），环境归因，保持显式记录不充绿。
- `case-av-language-parity` 549 PASS / 161 SKIP（AV-ONLY-1078..1096 裁定）/ 200 NOT_RUN（未选目标）/ 2 红=`av.parity.1093.explicit-c-library`、`c-header-spelling-rejected`＝已知 1093 C 闭包债务（PLAN-1125 记录在案）。
- **判定：零新增红（known-reds-only + 1 例负载瞬时红已归因复验），门禁对 1123 通过**；预存红保持显式红。

**landed**：wt-guard clean（无 reparse point）→ 主检 `git merge --ff-only plan-1123-dev` → master==交付提交 `4a20544663fa3c2ee2f4aa85d1d9661a68179a29`（无 merge commit；历史线性）。

**canonical_verified**：两 canonical 路径在落主提交哈希核验：`docs/specs/ace-migration/m2-stdlib-network.md` sha256 `8f3ab18df27ef88d…`（=reviewed 冻结值）、`docs/specs/stdlib/design/av-rust-provider-coverage.md` `66f833884eb7cf4c…`（=reviewed 冻结值）；无 specs.json/索引写入（PLAN-106）。主检冒烟：prepare-avrs-tls 两闭包重建（15.7 s）+ `avrs_tls_1123` 8/8 全绿（10.3 s，逐例 0.24–5.84 s）。



批量回归到期检查：1123%5=3≠0；回执读取见下一条收据。

stage: merge | plan_id: PLAN-1123 | plan_revision: r1 | outcome: pass (delivered) | worktree: (removed after guard) | commit: 4a2054466 | next: done

stage: goal | plan_id: PLAN-1123 | plan_revision: r1 | round: 7 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 4a2054466 | next: done

## 10. 待澄清事项

T-01 先核可交付的 Windows 来源和真实 ABI 形。方案需新公共接口时在执行合同中记明实际选择及消费者影响。

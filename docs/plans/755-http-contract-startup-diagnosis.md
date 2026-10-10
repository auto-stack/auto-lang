---
plan_id: PLAN-755
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: http-contract-startup-diagnosis
author: [zcode-agent]
created_at: 2026-10-10
updated_at: 2026-10-10
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []   # SD-01 为 modify 既有文档规则增补，非整体替换
new_spec_components: [docs/specs/stdlib/design/assembly-manifest.md]  # 新增「公共面契约覆盖完整性」节（PLAN-755）
touched_goals: [GOAL-003, GOAL-016]  # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [stdlib, auto-vm]     # 受影响的 specs 路径
current_step: 7
total_steps: 7
---

# [PLAN-755] http-contract-startup-diagnosis

## 0. 变更摘要

PLAN-738 引入的装配严格门在公共 HTTP native 面上缺独立适配契约：`NativeInterface::production()`
尾部的契约声明名单未覆盖 upload 家族（`upload_receive/commit/reject`）、Response 构造族（含
`response_redirect`）等大量 `stdlib/auto/http.at` 公共面 callee，引用这些 native 的程序在编译期即返回
`STDASSEMBLY.SIGNATURE_UNVERIFIED`（实证 native #9937 / #3108，启动前 0.1s 级快返）。而 plan730/plan326
两族 e2e 夹具以 `let _ = crate::run(&code)` 丢弃启动错误，把本应快速失败的契约错误藏成 nextest 120s
TERMINATING（751 收据：14 timeout + 1 项 109.988s fail）。

本计划三件事：(a) 先把两族夹具的启动诊断**有界化**，并分解 120s 等待的实际位置（P751-R2-02 未接通的
因果链）；(b) 按"真实 producer 契约"口径**补齐** HTTP 公共面全部选中 callee 的独立契约，处置
http_stream 声明名/注册名错位；(c) 复验真实服务启动与 wire 行为，跑完整串行 `cargo th` 对账，收口
P751-D1。**严格门保持不放宽；738/751 保持归档；空 lock 修复已闭，不再触碰。**

## 1. 目标

1. **启动诊断有界化**：test-http-e2e 两族夹具不再以 `let _ =` 丢弃 `crate::run` 结果——Err/panic/readiness
   有界上报，测试失败信息呈现真实启动原因；产出诊断工件，回答"120s 等待实际花在哪"。
2. **契约补齐**：production() 契约声明覆盖 `stdlib/auto/http.at` 公共面全部被选中 callee（真实签名口径，
   过 `signature_matches`），修正 http_stream 别名错位，保证声明次序（注册后最后落位、CFFI merge 后存活）。
3. **真实链路复验**：两个原反例程序（plan730 上传、plan326 重定向）经真实 `crate::run` 启动成功并按原
   wire 断言通过；完整串行 th 全档对账，P751-D1 挂死族清零或逐项有界分诊。
4. **规范沉淀**：装配契约规范补"公共面契约覆盖完整性"规则，堵住同类缺口再入。

### 非目标

- 不放宽严格门：不改 `verify_core_reference`/`verify_assembly_references`/`CORE_MODULES` 判定逻辑，不跳过
  核心校验，不把已有能力改判 Unsupported，不修改成功断言来清绿。
- 不重开 PLAN-738/751（保持 archived）；空 lock（P738-R11-01）已由 751 修复并复验，不触碰。
- 不修 P751-D2（auto-man 日常档盲区三红）——独立债务，另行清偿。
- 不修 `back_proxy_tests::http_e2e_back_proxy_real_routes_corpora_data_face` 在册基线红——仅确认不恶化。
- 不新增 HTTP 能力、不改 `stdlib/auto/http.at` 公共声明面（除非坐实公共声明本身与真实 ABI 矛盾，见 §10）。

## 2. 架构方案

修复点在**契约声明侧**，不在门侧。拒绝链（全部保持原样）：

```
用户源码引用 http native
  → codegen 发射期挂账 native id，核心 id（CORE_MODULES 首段 http / http_stream. 前缀）强制走
    NativeInterface::production()（vm/codegen.rs:12616-12669，get_or_insert @12644-12646）
  → verify_core_reference（vm/native.rs:476-489，按 id 缓存证明）
  → verify_native_reference（stdlib_assembly/reference.rs:101-205）：
      公共名解析选中该 id ✓ → shim 已绑定（PROVIDER_CLAIM_NO_CALLEE 检查 @181-183）✓
      → contract(id) == None ⇒ reference.rs:184-185 报 SIGNATURE_UNVERIFIED   ← 缺口在此
      → 有契约则 signature_matches @186-187 ⇒ ReferenceProof(SignatureChecked)
```

契约机制语义（补齐必须遵守）：

- 契约不在数据文件，而是 Rust 源码内与 shim 注册同址声明：`declare_contract_identity` /
  `declare_method_contract_identity`（vm/native.rs:417-470），集中在 production() **尾部**
  （native.rs:270-341，注释 264-269"显式身份必须最后落位"）。
- **`register_static` 撤销同 id 契约**（native.rs:355-357）——无契约重注册会抹掉已声明契约；`AutoVM::new`
  在 production() 之后 merge CFFI（vm/engine.rs:747-756），merge 路径的重注册不得落在契约声明之后
  （锚点 19973b089 已修一例，补齐后须复核该语义不被新声明破坏、新声明不被 merge 撤销）。

诊断侧：夹具 `start_server` 改为**通道化捕获**——spawn 线程内 `crate::run` 的 `Result`/panic 载荷经
channel 回传并有界 join，ready 轮询失败时测试失败信息携带真实启动原因；120s 分解用一次性计时观测
（connect 重试耗尽 / read 挂起 / 端口占用者三个候选形态逐一冻结），诊断跑在**契约未修基面**上。

## 3. 技术栈

- Rust（crates/auto-lang）：`vm/native.rs` 契约声明、`vm/ffi/stdlib.rs` 注册面（仅别名错位坐实才动）、
  `stdlib_assembly/reference.rs` 校验语义（只读参照）。
- 测试：feature `test-http-e2e`（真 TCP、串行档 `cargo th`，`.cargo/config.toml:109`）；scoped 单项用
  直接 nextest 单滤串（见 §6 注意）；探针复用 `docs/plans/reports/751-review-r2-http-probe.py`。
- 门禁档：worktree 内裸 `cargo t`（日常档）+ `cargo tv`（VM/编译器触面）；零 taa 触发（不改
  auto/lib、test/vm/aavm2、parity、aavm2 测试基建）。
- Worktree：`D:/autostack/.wt/lang-755/auto-lang`（branch `plan-755-dev`，Plan 529 分组平铺布局）。

## 4. 需求分析与背景调查

### 授权记录

- 2026-10-10 用户以 PLAN-751 R2 复查结论（[751-review-r2.md](reports/751-review-r2.md)，收尾提交
  `de147e098`）指示按建议**另立 HTTP 契约与启动诊断专项计划**：补齐契约、定位超时、再完成全档验收；
  738/751 保持归档。
- 授权范围：auto-lang 主仓，HTTP 公共 native 契约补齐 + e2e 夹具启动诊断 + th 复验；未给预算与自动
  续跑限额。执行前按 AGENTS.md L1 第 2 步需本计划确认。

### 证据链（file:line 均经 2026-10-10 只读调查核实）

- **拒绝路径**：`vm/codegen.rs:12616-12669`（verify_assembly_references，核心 id 判定 12633-12649）→
  `vm/native.rs:141-344`（production()，契约声明 270-341）→ `native.rs:476-489`（verify_core_reference）→
  `stdlib_assembly/reference.rs:101-205`（契约缺失判定 184-185）。
- **现有契约名单**（native.rs:270-341）：http.get/get_stream/request/transfer_download/file_response/
  transfer_wait/transfer_error/transfer_cancel/upload_metadata/upload_error、json.is_valid/encode/decode、
  方法面 stream_next/stream_is_done。
- **缺口盘点**（stdlib.rs 注册点 × http.at 公共声明 × 现有契约三方对账，初盘）：
  - upload 家族：`upload_receive/commit/reject` 无契约（stdlib.rs:9681-9720；http.at:388-410；
    **#9937 已实证**）；
  - Response 构造族：response/response_status/response_header/response_text/response_html/response_bytes/
    redirect 无契约（stdlib.rs:9506-9515；http.at:96-123；**#3108 已实证**）；
  - 客户端：post/put/delete/get_msg 无契约（get 已有）；RequestBuilder 方法面无契约（stdlib.rs:9554-9603）；
  - transfer：next_progress 无契约；SSE 族（stdlib.rs:9745-9751）、Response 方法（9756-9765）、
    sync/杂项（9781-9786）无契约；
  - http_stream 族：get_stream/post_stream/close/iter 无契约，且已有两条声明用 `auto.http.*` 名而注册用
    `auto.http_stream.*`（native.rs:306-317 vs stdlib.rs:9768-9774）——**别名错位风险**。
- **夹具吞错点**：`tests/plan730_http_upload_tests.rs:254-271`（start_server，`let _ = crate::run` @261）、
  341-405（raw_request_with_body：60×100ms connect + read_timeout 60s；read_response）；`vm/ffi/http_server.rs:1766-1785`
  （start_server，`let _ =` @1773）、1625-1646（http_get：50×100ms connect + read_timeout **5s** @1636）。
- **收据基线**：[751-th-full-receipt.md](reports/751-th-full-receipt.md)——102 selected / 86 pass /
  2 fail / 14 timeout（总 1962.969s）。16 项未过：14 timeout = plan730 族 13 项 + plan326
  `e2e_a_redirect_302_with_location`；2 fail = back_proxy 在册基线红（0.138s）+ plan730
  `http_e2e_plan730_client730_server_interop`（109.988s，契约嫌疑）。
- **历史锚点**：`d82eb02eb`（2026-10-09，PLAN-738 R2）引入 verify_core_reference/reference.rs（+323 行），
  `git log -S verify_core_reference` 首提交；`19973b089`（2026-10-10）修 merge 撤销语义。R2 已纠正旧归因
  （60s×2 read timeout 仅为候选；http_server.rs last-touch 不足定因）——本计划以 T-02 实测接通，不预设结论。
- ** specs 现状**：契约机制规范在 [stdlib/design/assembly-manifest.md](../specs/stdlib/design/assembly-manifest.md)
  （PLAN-738 沉淀），已定"producer 签名核对（含适配契约）后才通过、缺契约=SIGNATURE_UNVERIFIED 拒绝"，
  **未规定 production() 契约声明面必须覆盖公共层全部选中 callee**——缺口即 SD-01。

## 5. 详细设计

### 5.1 契约盘点口径（T-01 产物）

盘点表字段：`native 名 | 注册点(stdlib.rs:line) | http.at 公共声明行 | 现契约(有/无+声明点) |
gate 可达性 | 处置`。gate 可达 = 该 id 会被 `stdlib/auto/http.at` 某条 pub fn/method 的公共名解析选中
（reference.rs:162-170 语义）。**补齐范围 = 全部 gate 可达 callee**（不止最小两件）；不可达项在表中
登记但不强制补契约。初盘家族见 §4；终表以代码为准，不以上表为限。

### 5.2 契约补齐规则（T-03）

1. 每条契约的 parameters/returns/receiver/is_static/parameter_modes/error_shape 对齐**真实 shim ABI**
   （producer 记声明点 file:line，error_shape 记 wire 适配，参照 native.rs:431-446 既有写法），
   必须过 reference.rs:186-187 的 `signature_matches`。
2. 声明位置保持 production() 尾部"注册/覆盖之后最后落位"（native.rs:264-269 不变量）；补齐后复核
   `AutoVM::new` 的 CFFI merge（engine.rs:747-756）不撤销新契约（`register_static → contracts.remove`
   语义，native.rs:355-357）；若现有守卫测试未覆盖新契约面则扩列。
3. http_stream 别名错位：核实 `declare_contract_identity("auto.http.stream_*")` 是否实际绑定到
   `auto.http_stream.*` 注册 id；坐实错位则以注册名为准修正（或按 reference.rs:119-132 的
   `http_stream→http` 模块映射规范命名），并补一条防回退断言。
4. 发现 `.at` 公共声明与真实 ABI 无法调和时：**默认改契约侧**；改 `.at` 面属公共面变更，升 §10 待澄清。

### 5.3 夹具有界化设计（T-02）

- `start_server` 返回句柄含：run 结果通道（`Result` Err 文本 / panic 载荷）、ready 观测、有界 join。
- 测试失败路径优先呈现启动原因（如 `server failed to start: STDASSEMBLY.SIGNATURE_UNVERIFIED: ...`），
  而非让客户端等待吞掉。
- 120s 分解：在契约未修基面上，用计时观测冻结原等待形态——connect 重试耗尽（约 5-11s，应快速失败）、
  read 挂起（60s/5s 档）、端口被其他监听者占据（connect 成功但非本服务）。三种候选逐一排除或坐实，
  写入 `docs/plans/reports/755-startup-diagnosis.md`；结论必须能同时解释"repo-root 有界 run 0.10s 快返
  Err"与"测试进程 120.046s TERMINATING"两种已实测形态（进程 CWD/入口初始化差异显式冻结）。
- 夹具改动全部限定在 `#[cfg(feature = "test-http-e2e")]` 测试模块内，不动生产代码。

### 5.4 复验设计（T-04/T-05）

- 原反例复演：复用 751-R2 探针（compile 臂 + 真实 `crate::run` 臂），期望从 SIGNATURE_UNVERIFIED 转为
  启动成功。
- scoped e2e：两项原反例 + plan730 族抽样（multipart/wire_caps 等代表项），期望真实 bind + wire 断言生效。
- 全档 th：与 751 收据 16 项未过名单逐名对账（14 timeout 清零、interop 转绿或有界分诊、back_proxy 不
  恶化、86 绿不回归）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/stdlib/design/assembly-manifest.md | before：公共 native 需 producer 签名核对（含适配契约）后才通过，缺契约以 SIGNATURE_UNVERIFIED 拒绝；**未规定契约声明面的覆盖完整性**。after：stdlib 公共层（.at pub 面）可达的每个核心 native 必须在 production() 最终身份声明中有独立适配契约；新增公共 native 的改动必须同步补契约，验收物含"契约覆盖清单对账表" | HTTP 公共面缺口即本计划根因；规则化防同类缺口再入 | AC-02、AC-05、AC-07 |

## 6. 测试设计

- **探针层**：`docs/plans/reports/751-review-r2-http-probe.py` 复演（compile + actual `crate::run` 双臂），
  T-03 前红（SIGNATURE_UNVERIFIED）/后绿（启动成功）。
- **夹具层**（T-02，契约未修基面）：快速失败 + 失败信息含真实原因；计时分解 120s。
- **scoped e2e**：⚠️ 不用 `cargo th <name>` 追加滤串——th 别名自带 `http_e2e` 滤串，nextest 多滤串为
  **OR 并集**会扩成全族；用直接单滤串：
  `cargo nextest run -p auto-lang --lib --features test-http-e2e http_e2e_plan730_vm_plain_endpoint_untouched`
  （及 `e2e_a_redirect_302_with_location` 等逐项）。
- **全档**：低负载时段单实例 `cargo th`（751 基线 ~33min），收据对账。
- **门禁档**：worktree 裸 `cargo t`（日常档，预存红族按在册清单豁免）+ `cargo tv`（VM 触面）。
- **门保持审计**：复审逐 hunk 确认 diff 不含门逻辑/断言弱改（AC-03）。

## 7. 验收标准

- **AC-01 启动诊断有界化**：两族夹具（plan730 `start_server`、http_server.rs `http_e2e` `start_server`/
  `http_get`）不再丢弃 `crate::run` 结果，失败信息呈现真实启动原因；`docs/plans/reports/755-startup-diagnosis.md`
  回答 120s 等待实际位置，且同时解释 0.10s 快返与 120.046s TERMINATING 两种已实测形态（或证伪并给出
  实际形态）。验证：契约未修基面上两项 scoped e2e 实跑 + 诊断工件复审。
- **AC-02 契约补齐**：盘点表全列 http 公共面 callee；gate 可达集 100% 有契约且逐条过 `signature_matches`；
  http_stream 别名错位核实并处置；新契约经 CFFI merge 后存活。验证：盘点表对源核证 + 探针复演 + 守卫
  测试扩列。
- **AC-03 严格门保持**：diff 不触及 `verify_core_reference`/`verify_assembly_references`/`CORE_MODULES`
  放宽，无 Unsupported 改判，无断言弱改。验证：复审逐 hunk 审计记录。
- **AC-04 真实链路复验**：plan730 上传与 plan326 重定向两个原反例经真实 `crate::run` 启动成功，wire
  断言（plain endpoint 上传链路、302+Location）通过。验证：两项 scoped e2e PASS。
- **AC-05 全档 th 对账**：`cargo th` 单实例完整收据：14 项 timeout 清零；`http_e2e_plan730_client730_server_interop`
  转绿或给出有界分诊证据与后续处置；back_proxy 维持在册基线红不恶化；原 86 绿零回归。验证：
  `docs/plans/reports/755-th-full-receipt.md` 逐名对账表。
- **AC-06 复审门禁**：worktree 裸 `cargo t` + `cargo tv` 通过（预存红族在册豁免）；零新增编译警告、
  格式干净、无调试残留。验证：门禁输出留档。
- **AC-07 规范落账**：SD-01 写入 assembly-manifest.md；frontmatter spec-impact 字段（docs/specs/ 前缀）
  定稿；KNOWN-DEBT P751-D1 闭合注记（复审时定稿）。验证：merge 技能对账。

## 8. 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-01 契约缺口盘点表**（分析工件，无代码改动；依赖：无；AC-02）
  - [x] [✅ 已完成] 走查测试自动产出缺口清单：41 项缺契约 + 5 项 inventory 契约漂移（ok 族返回
    int vs 公共面 Response）；13 项 resolved-but-unbound 知情登记（CALL_SPEC 分派名异 id，另案）。
    表格落 [reports/755-contract-inventory.md](reports/755-contract-inventory.md)；别名错位疑点解除
    （auto.http.stream_next 与 auto.http_stream.stream_next 经 catalog 同 id 2242）。
- **T-02 夹具启动诊断有界化 + 120s 分解**（仅测试代码；依赖：无，**必须先于 T-03 提交**；AC-01）
  - [x] [✅ 已完成] worktree 提交 `327069006`（三助手：plan730 start_server、http_server
    start_server、start_example_api_server）。契约未修基面验证：两原反例由 120s TERMINATING 转
    **2.15s FAIL 且信息含 SIGNATURE_UNVERIFIED（#9937/#3108）**。120s 分解实测：run 32ms 快返被吞
    → 拒绝连接 ~2.15s/次 × ready-poll 50 次（107.6s）+ 客户端 60 次重试爬行 → nextest 120s 击杀；
    read 相位从未到达，"60s×2 read timeout"证伪。工件
    [reports/755-startup-diagnosis.md](reports/755-startup-diagnosis.md)；T-08 不触发（无第二缺陷）。
- **T-03 契约补齐**（依赖：T-01、T-02；AC-02/AC-03）
  - [x] [✅ 已完成] worktree 提交 `00e7f712c`：production() 尾部 +46 条声明（41 新增 + 5 漂移
    覆盖）+ `declare_generic_method_contract_identity`（RequestBuilder.json[T]）。走查守卫
    `plan755_http_public_face_contract_coverage`（覆盖+signature_matches+空转守卫）与
    `plan755_http_contracts_survive_cffi_merge`（AutoVM::new 同款 merge 存活）两测 PASS；门逻辑
    零触碰（diff 仅声明侧与测试）。
- **T-04 真实链路与 wire 复验（scoped）**（依赖：T-03；AC-04）
  - [x] [✅ 已完成] 两原反例经真实 `crate::run` 启动成功并过 wire 断言：
    `http_e2e_plan730_vm_plain_endpoint_untouched` PASS 0.588s（200 + body 730730）、
    `e2e_a_redirect_302_with_location` PASS 0.589s（302 + Location:/new + follow 到达）。plan730
    族全量转 T-05 全档对账覆盖。
- **T-05 完整串行 th 全档对账**（依赖：T-04；AC-05）
  - [x] [✅ 已完成] 串行单实例 `--jobs 1 --no-fail-fast`：**102 run / 101 pass / 1 fail / 0
    timeout，186.390s**（751 基线 86/2/14、1962.969s）。对账：14 timeout 全清（13 plan730 + 1
    redirect）、interop 转绿、back_proxy 维持在册基线红同形未恶化、86 绿零回归。附加观测：并行态
    `vm_quoted_boundary_wire` 曾 flake（隔离与串行均 PASS，serial 纪律自证）。收据
    [reports/755-th-full-receipt.md](reports/755-th-full-receipt.md)。
- **T-06 复审门禁 + 健康检查**（依赖：T-05；AC-03/AC-06）
  - [x] [✅ 已完成] 日常档 `cargo t --no-fail-fast`：5227 run / 5215 pass / 12 fail / 105s——12 红
    全部预存归档：musk_p053_4（在册 P733-R2 ③，主检出基线同红实证）、plan606/schema_drift×2
    （在册）、ash/projector/p748×2/iced×2（主检出基线同红实证 6/7）、plan502（双端隔离 PASS=负载
    flake）、docs_gen kitchen_sink（非本计划触面，schema 族在册）。`cargo tv` 162/162 PASS。
    触碰文件零新增编译警告（强制重编核验）、fmt --check 干净。AC-03 门保持：diff 仅契约声明/测试/
    规范文档，verify_core_reference/verify_assembly_references/CORE_MODULES 零触碰。
- **T-07 规范增量与簿记**（依赖：T-06；AC-07）
  - [x] [✅ 已完成] SD-01「公共面契约覆盖完整性」节落 assembly-manifest.md（worktree 提交
    `4ce645516`，随 merge 发布）；frontmatter spec-impact 定稿；KNOWN-DEBT P751-D1 闭合候选注记
    已写（复审定稿）；T-08 未触发（诊断证实无第二缺陷）。
- **T-08（条件）第二缺陷处置**（依赖：T-02 结论）
  - [x] [✅ 不触发] 755-startup-diagnosis.md 实证契约错误完全解释超时（放大器=本机 ~2.15s/次拒绝
    连接成本），无第二缺陷；任务关闭。

## 9. 复审记录

- 2026-10-10 draft handoff（/auto-plan:new）：stage `new`；PLAN-755 rev 1；outcome `pass`（立项授权在案，
  范围=契约补齐+启动诊断+th 复验）；next `work`（待用户确认本计划后进入执行，worktree
  `D:/autostack/.wt/lang-755/auto-lang`）。
- 2026-10-10 work handoff（/auto-plan:work）：stage `work`；plan_id PLAN-755；plan_revision 1；
  outcome `pass`；code_commits（branch `plan-755-dev`，base `d6e4819ba`）=
  `327069006`（T-02 夹具诊断有界化）/ `00e7f712c`（T-01/T-03 契约补齐+走查守卫）/
  `4ce645516`（T-07 SD-01 规范节，随 merge 发布）；依赖 auto-down detached@`895f8d0`。
  task_ids T-01..T-07 全完成、T-08 不触发。evidence：reports/{755-startup-diagnosis,
  755-contract-inventory,755-th-full-receipt}.md；两原反例 scoped PASS（0.59s）；串行 th
  102/101 pass/0 timeout（186.4s）；日常档 12 红全预存归档（基线同红/在册/隔离过三类实证）；
  tv 162/162；零新增警告+fmt 干净。blockers 无。next `review`（/auto-plan:review，独立复审
  AC-01..AC-07 与 SD-01 规范增量，复审门已含本档证据）。
- 2026-10-10 review（/auto-plan:review）：stage `review`；plan_id PLAN-755；plan_revision 1；
  **outcome `pass`**（→ `reviewed`，next `merge`）。
  - reviewed_commit `4ce645516`（branch `plan-755-dev`：`327069006`/`00e7f712c`/`4ce645516`，
    worktree clean）；base `d6e4819ba`；依赖 auto-down detached@`895f8d0`（组内兄弟）。
    规范输入：assembly-manifest.md@`4ce645516` blob `383d2dab`。
  - **独立性声明**：本复审在实施会话内进行——结论从工件重建并关键复现，不采信执行者叙述：
    ①AC-01 未修基面形态经 git 重建的抛弃式 worktree@`327069006` 独立复现（两原反例 ~2.1s 有界
    FAIL，panic 含 `SIGNATURE_UNVERIFIED #9937`），复验后 wt-guard clean 移除；②AC-04/AC-02 守卫与
    原反例 HEAD 复跑 2/2+2/2 PASS；③AC-05 串行全档 th 二次独立采样 **102 run/101 pass/1 fail/
    0 timeout（200.6s）**与首轮（186.4s）一致，唯一红= back_proxy 在册基线红同形。
  - acceptance_results：AC-01 pass / AC-02 pass / AC-03 pass / AC-04 pass / AC-05 pass / AC-06 pass /
    AC-07 pass。
    - AC-03 结构审计：`stdlib_assembly/` 零触碰；全 diff 无 verify_core_reference/
      verify_assembly_references/CORE_MODULES/Unsupported 改动（`verify_core_reference` 仅现于新测试
      注释）；夹具 diff 仅移除 5 行吞错代码，零断言删改；`stdlib.rs` 与 `stdlib/auto/http.at` 均未
      触碰（计划内条件路径未触发）。
    - AC-06 红名单核算：12 红全预存——在册（musk_p053_4=P733-R2③、plan606、schema×2、
      kitchen_sink 族）+ 主检出基线同红实证 6 项 + 双端隔离 PASS 负载 flake 1 项（plan502）。
  - findings（均非阻塞）：O-1 门禁 `cargo t` 以 `--no-fail-fast` 执行（nextest 默认首红即取消，
    全量核算必需；测试集无缩减）；O-2 merge 存活守卫采样 3 id 非全 46（机制级覆盖，walk 补全量
    面；改进候选非缺口）；O-3 13 项 resolved-but-unbound 已在盘点报告 §3 登记（既有 shim/id 分派
    缺口，755 边界外，如后续被发射路径触达走 PROVIDER_CLAIM_NO_CALLEE 分诊）；O-4 并行态
    vm_quoted_boundary_wire flake 已在收据登记（serial 纪律内不复发）。
  - 规范增量审定：SD-01（modify assembly-manifest.md，新增「公共面契约覆盖完整性」节）描述当前
    行为与持久决策（覆盖规则/守卫锚点/unbound 边界），路径与 AC 关联正确；frontmatter 定稿
    supersedes=[]（增补非替换）、new=[docs/specs/stdlib/design/assembly-manifest.md]、
    touched_goals=[GOAL-003, GOAL-016]。发布随 merge。
  - evidence：reports/{755-startup-diagnosis,755-contract-inventory,755-th-full-receipt}.md（含命令
    与逐名对账）；本记录内嵌复现命令/结果摘录（R3 抛弃式 worktree 复现、th 双采样）；KNOWN-DEBT
    P751-D1 闭合候选注记（merge 时定稿销案）。

## 10. 待澄清事项

1. ~~**120s 分解结论的分支**（T-02/T-08）~~ **已销案**：实测分解闭合（契约错误被吞 × ~2.15s/次拒绝
   连接爬行 → 120s 击杀），无第二缺陷，T-08 未触发。
2. **`.at` 公共声明与真实 ABI 矛盾**（T-03 规则 4）：本轮未触发（46 条声明全部 signature_matches
   通过，无 .at 面改动）；规则保留供后续。
3. ~~**interop fail（109.988s）归因**~~ **已销案**：与契约缺口同因——服务真实启动后互操作链 PASS
   （T-05 对账表 #2）。
4. **全档 th 时段**：已按单实例串行执行完毕（186.4s）；后续批量回归若触 HTTP 面沿用 serial 纪律。

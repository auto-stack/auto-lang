---
plan_id: PLAN-669
status: execution_done               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-server-api-arg-binding
author: [zcode]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 2
current_step: 6
total_steps: 6

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/stdlib/design/http-server.md]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/stdlib/design/http-server.md]
---

# [PLAN-669] AutoVM HTTP server `#[api]` 实参按名绑定修复

## 0. 变更摘要

`auto run --server vm`（AutoVM HTTP server）的 legacy `#[api]` 实参装配违反
spec 已定约定（按名绑定）：**GET query 参数被序列化为单个 JSON 集合串塞给第
1 位置实参；POST/PUT body 不解析 JSON、原始串直塞第 1 实参**。带参 handler
在此路径上从未正确工作过——上游无感只因 013/015 等示例 `api: "rust"` 走 a2r
轨（参数绑定在生成代码里，是对的），且既有 e2e 从未 POST 过带参端点。

修复：镜像仓内两个既有先例（back_proxy Plan 658 按名绑定、axum_adapter
Plan 442 `record_param_sig` codegen 侧信道），把 handler 形参签名（名+类型）
经 codegen 侧信道发布到服务进程，四条实参装配点收敛到共享按名绑定器：
**路径段 → body JSON 字段 → query 参数**（按形参名、按声明序），缺参 400，
trailing 未绑定形参保留 cookies/auth 元数据 opt-in，无签名时回退现行位序
（兼容）。缺陷由 auto-edit 项目勘定报告钉到源码级（2026-09-20 转交），
本计划起草前已在本仓独立复核全部定位（见 §4）。

## 1. 目标

修复后，VM-server 路径的 `#[api]` 带参 handler 按 spec §4.1 约定接收实参：

- `create_note(title str, body str)` 经 `POST /api/notes` + JSON body →
  两个字段分别按名入参（不再是原始 body 串塞 `title`）。
- `search_notes(query str)` 经 `GET /api/notes/search?q=...` → query 值入参。
- 声明 `int` 的形参从路径段/query 字符串精确转换（兑现 Plan 326 Phase 5
  注释里的 long-term TODO，替代 `parse::<i32>()` 试探）。
- 缺参返回 400 + 指名 error JSON（back_proxy 对齐，先例
  `http_e2e_back_proxy_missing_param_is_400`）。
- 既有无参 fn、`:param` 路径段、元数据 opt-in（token auth）、SSE、multipart、
  form-urlencoded、a2r 轨、`__axum:` 适配轨全部不回退。

**非目标（Non-goals）**：

- **不改** axum_adapter `__axum:` 合成路由的 extractor 编组（自成体系，正确）。
- **不改** a2r/api_gen 轨（参数绑定在生成代码里，本就正确）。
- **不做** `Request` 对象注入（spec §4.2，spec 自列 P1 未实施项，另立项）。
- **不做** F-R1（a2r server 模板 `use api::Db` 硬编码 + 契约 fn `// TODO`
  空体）与 F-W3（`SrcChanged` 事件空串载荷派发 int 形参）——不同子系统，
  建议另立计划（见 §10-1）。
- **不做** `openapi.generate()` 的逐参文档增强（拿到签名后可做的后续项）。
- `#[api]` 形参带默认值的缺失场景按缺参 400 处理（仓内无此用法，边缘记录）。

**成功样貌**：auto-edit 六端点（`exists/read_text/tree/write_text/env_str/...`）
在 vm-server 轨零改动复活（跨仓复跑属 merge 后协作项，见 §8 T-06）；
`cargo th` 20 e2e + `cargo t` 日常档全绿（环境红基线另计，§10-2）。

## 2. 架构方案

三件套：**签名侧信道 → 共享绑定器 → 四点收敛**。

1. **签名侧信道 `API_PARAM_SIGS`**（进程级全局，生命周期模型同
   `HTTP_ROUTES`，stdlib.rs:3481）：`HashMap<fn_name, Vec<ApiParamSig>>`，
   `ApiParamSig { name: String, ty: String }`（ty = `Type` Display 串）。
   codegen 在 `api_routes.push(...)` 处（vm/codegen.rs:1398-1404，`fn_decl.params`
   名+类型现成）同步发布；清除纪律镜像 `record_param_sig`（axum_adapter.rs:231
   注释："run pipeline resets the table up front"）+ `clear_http_routes()`
   （stdlib.rs:3504，e2e 隔离）双点。**不动 `(method, path, fn_name)` 三元组
   类型**——loader/asm/openapi/back_proxy/a2r_tests 全部消费方零波及。
2. **共享绑定器 `bind_api_args_by_name`**（http_server.rs 内新增）：
   按声明序逐参绑定，优先级 **路径段（按名）→ body JSON 字段 → query 参数**；
   字符串源按声明类型转换（int/float/bool 精确 parse，其余按 str）；
   body 值经 `json_to_vm_value`（stdlib.rs:2860，自然类型）编组；
   字符串一律走 `push_str_arg` 咽喉（Plan 510 G1-1 rc 纪律）；
   恰好一个 trailing 未绑定形参 → 塞现行元数据 JSON（Plan 317 Phase 11
   opt-in 语义保留）；其余未绑定 → 400。**无签名条目 → 按现行位序装配，
   字节级语义不变**（legacy 生产者兼容）。
3. **四装配点收敛**（见 §4 现勘表）：活的 async 路径与 in-language native
   接绑定器；两条死循环机械收敛防将来复活分叉。

## 3. 技术栈

- Rust（既有 crate，无新依赖）：`std::sync::Mutex` 全局表、`serde_json::Value`
  body 解析、既有 `json_to_vm_value`/`push_str_arg`/`match_route`。
- 测试：nextest 分档（`cargo t`/`th`/`tf`），e2e 模块 `mod http_e2e`
  （http_server.rs:923，真 TCP、固定唯一端口、nextest 每测试一进程并行）。
- 不触及 aavm/trans/book → 零 `taa/tt/tb` 触发；不改 schema/文档生成器 →
  零 `docs_gen` 触发。

## 4. 需求分析与背景调查

### 4.0 授权记录

- 本会话授权范围：**起草本计划**（用户指令：按 auto-edit 勘定报告定位问题
  并立修复计划）。drafting → executing 的 flip 待用户确认（§10 流程）。
- 执行期允许动作（按 AGENTS.md 标准流程）：`D:/autostack/.wt/lang-669/auto-lang`
  worktree 内改码+测试；master 上仅计划簿记。无预算/自动续作限制指令。

### 4.1 缺陷勘定（auto-edit 报告 + 本仓独立复核）

auto-edit 报告的源码级定位**全部复核成立**，且本会话补齐三处报告未覆盖事实：

| # | 事实 | 源码锚点 |
|---|---|---|
| F1 | async 版（生产路径）query 集合串单参 | http_server.rs:2870-2878（`build_handler_args` 内，"Plan 346: Push query params as a JSON object string if no body"） |
| F2 | async 版 body 原始串直塞第 1 实参 | http_server.rs:2887-2902（form-urlencoded 转 JSON 集合串后同样整串单参） |
| F3 | stdnet 版 query 完全未消费 | http_server.rs:2082-2099（`serve_blocking_stdnet`：仅路径段+原始 body） |
| F4 | **装配点实为四处而非两处**（报告补齐①）：另含 stdlib.rs `run_http_server_blocking`（:4355，同 F3 形态）与 `shim_http_server_listen`（:4485，注册为 native `auto.http.server_listen`，stdlib.rs:8163，同 F3 形态） | stdlib.rs:4355/4485 |
| F5 | 路由表三元组无参名，long-term TODO 在注释里 | stdlib.rs:3481（`HTTP_ROUTES`）；http_server.rs:2087-2088（"codegen should record per-param types in api_routes"，Plan 326 Phase 5） |
| F6 | **spec 已明文按名绑定**（报告补齐②）：实现违反自家规范，非"约定缺失" | docs/specs/stdlib/design/http-server.md:150-157（"路径参数按名字匹配…POST/PUT body: JSON 对象的字段按名字匹配函数参数"）、:187（Request 注入约定） |
| F7 | **仓内零消费方依赖现行 query 集合串约定**（报告补齐③）：http_e2e 20 测无一带 query 串请求；`e2e_notes_crud` 声明了 `create_note(title str, body str)`（:1927）但断言只打 GET——带参 POST 从未被测试消费 | http_server.rs:1901-1952 |
| F8 | 活/死路径：`auto run --server vm` → execute_autovm auto-serve（lib.rs:1720-1766）→ 专属线程 → `serve_async` → `handle_connection_async` → legacy 分支 = `build_handler_args`（**生产+e2e 全走这条**）；`serve_blocking_stdnet` 与 `run_http_server_blocking` 全仓无调用方（死码，pub 保留） | lib.rs:1761；http_server.rs:2002；stdlib.rs:4355 |
| F9 | 元数据 opt-in（cookies/auth JSON）按声明形参数量推断，Plan 317 Phase 11 修复过一次同类错位 | http_server.rs:2904-2920（`get_fn_n_args`，engine.rs:2244） |
| F10 | axum_adapter `__axum:` 轨自成编组体系（`push_extractor_args`），**不在本缺陷面** | http_server.rs:2587-2643；axum_adapter.rs |

### 4.2 可复用先例（零发明量）

- **按名绑定全流程**：back_proxy `handle_request`（back_proxy.rs:1070-1101）——
  绑定优先级 path → body 字段 → query、缺参 400、body 解析失败对单参形态
  原始串透传的宽容度、`json_to_vm_value` 编组、fresh task 调用清算。逐条可抄。
- **codegen 侧信道**：axum_adapter `record_param_sig`（axum_adapter.rs:109；
  发布点 vm/codegen.rs:1455，`fn_decl.params` 的 Type Display 串已在此发布）
  ——本计划只是把**名字**也带上、且仅对 `#[api]` fn 发布。
- **400 缺参 e2e 先例**：`http_e2e_back_proxy_missing_param_is_400`
  （tests/back_proxy_tests.rs:197）。

### 4.3 上游为何一直没炸

013-todo/015-notes 均 `api: "rust"`（pac.at:8/:11）→ split 演示走 a2r rust
引擎；VM-server 路径日常只被无参 fn（`list_todos`/`ws_root`）消费。
auto-edit 全部症状（exists/read_text/tree/env_str 拿集合串当路径 →
`unwrap_or_default` 静默空返）由此统一解释。

### 4.4 多会话风险

PLAN-668（executing 中）红清单不含 http/e2e 面——无文件与测试基线冲突。
本计划不改 `docs/plans/668-*`。

## 5. 详细设计

### 5.1 数据通路与重置纪律

```rust
// http_server.rs（新）
pub struct ApiParamSig { pub name: String, pub ty: String }
static API_PARAM_SIGS: std::sync::Mutex<HashMap<String, Vec<ApiParamSig>>> = ...;
pub fn register_api_param_sig(fn_name: &str, sigs: Vec<ApiParamSig>);
pub fn api_param_sig(fn_name: &str) -> Option<Vec<ApiParamSig>>; // clone
```

- 发布：vm/codegen.rs `api_routes.push(...)` 处（L1398-1404）同点发布
  `fn_decl.params` 的 `(name, ty.to_string())`。dep-module Codegen 实例同样
  经过该路径（`record_param_sig` 注释同款论证，codegen.rs:1449-1454）。
- 清除：两处——`clear_http_routes()`（stdlib.rs:3504，e2e 隔离点，连带清）
  与 run pipeline 编译前重置点（对齐 axum PARAM_SIGS 的重置调用方；
  worktree 内以 `record_param_sig` 现有 reset 调用方为准逐一挂接）。

### 5.2 绑定器语义（`bind_api_args_by_name`）

输入：`route_match`（path_params 已按名、query_params 已解码——match_route
http_server.rs:46-105 现成）、body 三形态（原始串 / form 转 JSON 串 /
multipart JSON 串，按现行分支预处理后传入）、content_type、fn 签名 sigs。

1. **签名缺失**（`api_param_sig` 无条目）：走现行位序装配原逻辑，行为不变。
2. **按声明序逐参绑定**，每参依次尝试：
   a. `route_match.path_params` 按名命中 → 字符串源，按声明 ty 转换；
   b. body JSON 对象按名命中 → `serde_json::Value` 经 `json_to_vm_value`；
   c. `query_params` 按名命中 → 字符串源，按声明 ty 转换。
3. **字符串源类型转换**：`int` → 精确 parse（对齐 `json_to_vm_value` 的整数
   表示，替代现行 `parse::<i32>()` 试探，兑现 F5 的 TODO）；`float`/`bool`
   同理精确 parse；parse 失败 → 400（类型错误也指名报）；其余 ty 按 str。
4. **trailing 元数据 opt-in**：所有可绑定参数绑定后，若恰好剩余**一个**
   trailing 声明形参未绑定 → 塞现行元数据 JSON（`{"cookies":...,"auth":...}`
   格式不变，Plan 317 Phase 11 语义）。多于一个未绑定 → 其余按缺参 400。
   既有 `e2e_b1_realworld_token_auth`（023-realworld 真源）守护此行为。
5. **缺参 → 400**：`{"error":"missing param `name` for <METHOD> <path>"}`
   （back_proxy 措辞对齐）。async 臂沿用现有 404/500 同款响应构造。
6. **raw-body 单参宽容**（back_proxy.rs:1070-1071 对齐）：声明恰 1 个
   str 形参、body 非空且 JSON 对象解析失败 → 原始 body 串直塞（保
   text/plain 消费者）。
7. **字符串入池纪律**：一切字符串实参走 `push_str_arg`（Plan 510 G1-1 咽喉，
   http_server.rs:1998）。

### 5.3 四装配点收敛表

| 装配点 | 状态 | 动作 |
|---|---|---|
| `build_handler_args`（http_server.rs:2845，async 生产+e2e） | 活 | legacy 分支整体改走绑定器（`__axum:` 分支不动） |
| `shim_http_server_listen`（stdlib.rs:4485，native `auto.http.server_listen`） | 活 | 同上收敛 |
| `serve_blocking_stdnet`（http_server.rs:2002） | 死（无调用方） | 机械收敛（防复活分叉） |
| `run_http_server_blocking`（stdlib.rs:4355） | 死（无调用方） | 机械收敛（同上） |

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/stdlib/design/http-server.md §4.1 | before：注入规则仅列路径段与 POST/PUT body 按名（VM 模式未实现）；after：补 **query 参数按名**为第三注入源 + 缺参 400 语义（back_proxy Plan 658 对齐） | spec 补齐 VM 模式实际交付的完整注入语义；query 先例已在线上（back_proxy） | AC-01..AC-04 |
| SD-02 | add | docs/specs/stdlib/design/http-server.md 新增"VM 模式实现状态"小节 | before：无 VM 模式装配实现记载；after：记 API_PARAM_SIGS 侧信道、绑定优先级（path→body→query）、trailing 元数据 opt-in、无签名位序回退、raw-body 单参宽容 | 装配约定首次落档，防四处装配点再分叉 | AC-05/AC-06 |

## 6. 测试设计

- **单元测试**（绑定器矩阵，随 T-02）：三源按名命中、声明序、int/bool 精确
  转换与类型错 400、trailing 元数据、多缺参 400、无签名回退位序、
  raw-body 单参宽容、form-urlencoded 字段名命中。
- **e2e**（`mod http_e2e`，固定唯一端口，遵守 e2e_ports_unique 守卫
  http_server.rs:1051——新测试选未占用端口）：
  - `http_e2e_api_post_body_by_name`：POST JSON body 两字段分别入参
    （013 create_todo 形状，断言字段值非 JSON 字面量）；
  - `http_e2e_api_query_by_name`：GET `?query=...` 过滤命中（015
    search_notes 形状）；
  - `http_e2e_api_typed_query_int`：声明 `int` 的 query 形参收到数值；
  - `http_e2e_api_missing_param_400`：缺字段 → 400 + 指名 error；
  - `http_e2e_api_raw_body_single_param`：非 JSON body → 原始串入参；
  - **扩 `e2e_notes_crud`**：补真实 POST `create_note(title, body)` 断言
    （F7 的声明未测面转正）。
- **回归面**：既有 20 e2e（multipart B6、token auth B1、SSE、302、rate-limit、
  struct/int-path 返回……）全绿；a2r 轨与 `__axum:` 轨不触碰由 diff 审。
- **档位**：`cargo check -p auto-lang` → `cargo t`（日常）→ `cargo th`
  （http 档，本机 ≥3 环境红基线见 §10-2）→ 合入前 `cargo tf` 全量。
  改动不触 aavm（零 `taa`）。

## 7. 验收标准

- **AC-01**（POST body 按名）：vm-server POST JSON body 的字段按形参名分别
  入参——`http_e2e_api_post_body_by_name` 断言 `title`/`body` 字段值等于
  请求体对应字段（非原始串/JSON 字面量）；`e2e_notes_crud` 扩展断言同证。
- **AC-02**（GET query 按名）：`?query=Build` 命中声明 `query str` 形参并
  过滤——`http_e2e_api_query_by_name` 断言仅命中项返回。
- **AC-03**（类型精确转换）：声明 `int` 形参从路径段/query 字符串收到数值
  （既有 `e2e_int_path_param_handler` + 新 `http_e2e_api_typed_query_int`）；
  非数值入 int 形参 → 400。
- **AC-04**（缺参 400）：缺 body 字段/query 参数 → HTTP 400 + body 含
  `missing param` 与形参名（`http_e2e_api_missing_param_400`）。
- **AC-05**（回归不退）：既有 20 个 `http_e2e` 对照改动前基线全绿（环境红
  另计）；`cargo t` 日常档全绿；合入前 `cargo tf` 全绿；a2r 轨/`__axum:` 轨
  diff 零触及。
- **AC-06**（legacy 回退）：无签名条目时位序装配字节级不变（单元测试直证
  绑定器回退臂）。
- **AC-07**（raw-body 单参宽容）：非 JSON body 对单 str 形参 → 原始串入参
  （`http_e2e_api_raw_body_single_param`）。

## 8. 执行步骤

- [x] **T-01** codegen 发布签名 + 注册表与重置纪律
  涉及：`vm/codegen.rs`（api_routes.push 处）、`vm/ffi/http_server.rs`
  （`API_PARAM_SIGS`+register/getter）、`vm/ffi/stdlib.rs`
  （`clear_http_routes` 连清）+ run pipeline 重置点挂接。
  验证：`cargo check -p auto-lang`；单元测试直证发布-查询-清除闭环。
  → AC-06 前置。**新路径**：注册表与发布点。
  [✅ 已完成] worktree 43fd4eed8：check 0 错；
  `plan669_api_param_sigs_roundtrip` PASS；重置挂点=lib.rs 四处
  axum_adapter::reset 同位 + clear_http_routes 连清。
- [x] **T-02** 共享绑定器 `bind_api_args_by_name`（§5.2 语义全量）
  涉及：`vm/ffi/http_server.rs`。
  验证：绑定器单元矩阵（§6 列举项）绿。
  → AC-01..AC-04/AC-06/AC-07 逻辑源。
  [✅ 已完成] worktree 02fabdce3：单测矩阵 10/10（后随 rev2 语义扩展
  增至 13/13：whole-body 可解析对象容忍 + 元数据按名识别负例）。
- [x] **T-03** async 装配点接线 + e2e 新battery + notes_crud 扩展
  涉及：`build_handler_args` legacy 分支、`mod http_e2e` 新 5 测 + 扩展。
  验证：`cargo t http_e2e`（滤串）全绿；`auto run --server vm` 手探
  013 形状一次。
  → AC-01..AC-05/AC-07。依赖 T-01/T-02。
  [✅ 已完成] worktree a6a309a12：e2e 32/32（含 5 新测 + notes_crud POST
  断言）。**执行中三项证据驱动修订（rev 2）**：①入口分片根修——async
  请求头单次 read() 遇 TCP 分片产生空 400（实证：master 上 nextest 跑
  th 档 30+ 红、cargo test 串行同测可过=时序型；即 plan 568 T7 在案
  "环境红"真身），改读至 \r\n\r\n（64KB 帽 431）后 master 侧 28 项预存
  红全数转绿；②元数据 opt-in 增按名识别（meta/metadata/req/request，
  否则 POST 缺末字段被误喂 cookies/auth，AC-04 语义靠此成立）；
  ③whole-body 单参容忍扩至可解析对象（b6 `upload(form str)` 夹具依赖）。
  b1/b6 e2e 夹具中 `arg str` 整 body 单参工作区现代化为 typed 形参
  （原为围绕旧缺陷约定的等价改写）。notes_crud 曾加"创建后列表可见"
  断言后撤（夹具不回写数组，断言超程序语义）。
- [x] **T-04** native 与死码装配点收敛
  涉及：`shim_http_server_listen`、`serve_blocking_stdnet`、
  `run_http_server_blocking`。
  验证：`cargo check -p auto-lang` + `cargo th`（native 路径无独立语料，
  随档回归；死码收敛以编译+审读为证）。
  → AC-05 面完整性。依赖 T-02。
  [✅ 已完成] worktree 1a8b0f82b：三站点统一 `bind_api_args_or_legacy`
  （同步助手+绑定失败 400/500 早返）；stdlib 两循环路由换共享
  match_route（原生 listen 路径 query 从未解析的附带修复）；
  find_route 死码移除；e2e 32/32 维持。
- [x] **T-05** 规范增量落档 + ledger 元数据
  涉及：`docs/specs/stdlib/design/http-server.md`（SD-01/SD-02）、本文件
  frontmatter（`new_spec_components` 定稿）。
  验证：SD 表逐行对照 diff。
  依赖 T-03（语义定稿后再落墨）。
  [✅ 已完成] worktree 83e1b68c3：§4.1 注入规则五条（query 第三源/缺参
  400/单参 whole-body/元数据按名 opt-in/精确类型转换）+ §4.1.1 VM 模式
  装配实现状态小节 + Status 行修订注记。
- [x] **T-06** 全量门禁 + 健康检查 + 跨仓交接
  验证：`cargo t` → `cargo th`（对照基线）→ `cargo tf`；零新警告/无散落
  debug print/格式整洁；随后通知 auto-edit 会话复跑六端点探针（PLAN-003
  B1 闭环 + r2/T-03 重验，跨仓协作项非本仓 AC）。
  [✅ 已完成] **基线对照法**（§10-2 口径）：`cargo t` 分支 36 红 ⊆
  master 37 红（唯一差集=分支多治愈 external_config_poll_hot_apply_loopsafe
  一项），零新增回归；36 红族谱全数对上 668 在册清单（musk p053/p054、
  ui::layout 环境、零散集成红）。`cargo th` 32/32 全绿（曾两轮
  e2e_concurrent_sse 负载 flake：与后台基线跑竞争 CPU 时 40s 超时挂，
  单跑 0.9s 过、空载复跑 1.77s 全绿——测试注释在案的历史 flaky，非本
  改动面）。`cargo tf` 3680/3682，2 红=master 在册 ui_gen 双测（668
  R 清单内）。健康：改动文件零新警告；rustfmt 我新增区间零新分叉
  （全仓分叉在案禁整跑）。auto-edit 复跑探针=merge 后跨仓通知
  （§10-4）。

## 9. 复审记录

- 2026-09-20（draft handoff）：`stage: new`，PLAN-669 rev 1。
  `outcome: pass`（起草完成，源码级定位全部独立复核成立，任务/AC/SD 齐）。
  `next: work`——待用户确认 flip `executing` 后建
  `D:/autostack/.wt/lang-669/auto-lang` worktree 开工。
- 2026-09-20（用户确认执行）：用户指令"解决方案确定、计划写好，直接
  auto-plan-work 实施修复"——§4.0 授权补记：执行获准，§10-1 按 rev 1 建议
  （F-R1/F-W3 排除出执行面、另立计划）默认生效。status → executing。
- 2026-09-20（work handoff）：`stage: work`，PLAN-669 rev 2，status →
  execution_done。`outcome: pass`。`code_commit`: worktree plan-669-dev
  43fd4eed8→83e1b68c3（T-01..T-05 五提交，base 5d5090e14）。
  `task_ids`: T-01..T-06 全勾（证据见 §8）。`evidence`: e2e 32/32
  （5 新测+notes_crud POST 转正）、单测 13/13、cargo t 基线对照零新增
  回归（36⊆37，多治愈 1）、cargo tf 3680/3682（2 红=master 在册）、
  th 档曾现 concurrent_sse 负载 flake（空载全绿，历史在案）。rev 2 增量
  （T-03 三项证据驱动修订，§8 T-03 条目在案）：入口分片根修（plan 568
  T7 "环境红"真身，master 侧 28 预存红转绿的附带收获）、元数据按名
  opt-in、whole-body 容忍扩展；b1/b6 夹具现代化。`blockers`: 无。
  `next: review`（/auto-plan:review 独立复审 → merge；merge 后通知
  auto-edit 复跑六端点探针收 PLAN-003 B1/r2/T-03，§10-4）。

## 10. 待澄清事项

1. **F-R1 / F-W3 并案与否**（auto-edit 报告建议"可一并登记"）：本计划已将
   二者排除出执行面（F-R1=a2r server 生成模板 `use api::Db` 硬编码+契约 fn
   空体；F-W3=上游 `code_editor` oninput 事件载荷空串派发 int 形参——均非
   HTTP 装配面）。**建议各另立 L1 计划**（子系统与验证面不同）；若用户裁定
   并案，本计划升 rev 扩面。
2. **`cargo th` 本机环境红基线**：资源表在案 ≥3 环境相关红（Plan 568 T7
   归因）。执行首步先跑改动前基线存档，AC-05 以"对照基线"口径判绿。
3. **`#[api]` 形参默认值边缘**：缺参一律 400（含带默认值形参；仓内无用法）。
   若用户要求默认值参与绑定需 VM 调用约定支持，另立项。
4. **auto-edit 复跑闭环方式**：merge 后由本计划执行会话通知 auto-edit 侧
   复跑 split 探针（PLAN-003 B1/r2/T-03 收口在彼仓完成）。

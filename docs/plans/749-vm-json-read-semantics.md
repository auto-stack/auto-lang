---
plan_id: PLAN-749
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM 解释器对 JSON 产物/深链的读语义退化 + 视图模板 fn 调用返空根修（坑①家族第三型，auto-musk PLAN-100 上游缺口）
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/vm/architecture.md
  - docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md
new_spec_components: []
touched_goals:
  - GOAL-007

affects: [auto-lang/vm, auto-lang/ui]
current_step: 0
total_steps: 7
plan_revision: 1
---

# PLAN-749 — VM 解释器 JSON 产物/深链读语义退化 + 视图模板 fn 调用返空根修

## 变更摘要

auto-musk PLAN-100 E2E 沙箱（2026-10-09，relay-plan-gate-and-vm-runbox）在 VM 轨
聊天暴露三型读语义病灶，均指向本仓 VM 值系统/读路径（auto_val NaN-box 表示 +
interpreter GET_FIELD + 视图构建求值）。musk 侧已用"预计算浅字段"战术绕过并落地
为双端契约，本计划在引擎层根修：

1. **信封布尔读不稳定（症状1）**：`Http.get_msg` 完成后 handler 内
   `let r = JSON.parse(payload)`，`r.ok` 读出 **-2147483648**（NaN-box TAG_BOOL
   位型被按 int 解释/打印）；`r.ok == true` 间歇工作——同一 payload 有时进成功
   臂有时进失败臂（SessionsLoaded/DetailLoaded/PollBackfill 三消费 handler 全部
   间歇失效）。print 对 bool 的显示同样垃圾。
2. **handler 上下文深链字段读退化（症状2）**：
   `resp.session.messages[1].blocks[0].tool_name` 四层链全 MISS（探针 print
   实锤），而 `resp.session.messages.length`（浅层/数组长度）工作。块对象存在、
   字段读 None → 前端 `??` 兜底全空。
3. **视图模板内 fn 调用静默返空（症状3，坑①家族第三型）**：chat_message.at 视图
   里 `if extractRunId(.block.tc) != ""` 恒 false（fn 在视图上下文返回空），同一
   fn 在 computed 上下文工作正常。先例：forge_store.at:1773 / chat_message.at:339
   坑①注记、PLAN-081 r5 定罪、KNOWN-DEBT P733-R2 登记的 fn 调用语义同族残腿
   （台账明确留有"后续计划指针"——即本计划）。

修复后 musk 的预计算浅字段绕过层**仍须工作**（向后兼容，双端契约不破坏）；
chat_message.at 的 fn 调用形态可回迁（可选，非验收必需）。

## 目标

1. **T-01 先行钉死病灶**（用户 2026-10-10 要点）：三上下文（handler 栈 / computed
   求值 / 视图模板表达式）最小复现单测先行——修复前红、修复后绿，再动实现；
   复现不稳定时优先怀疑 NaN-box bool 打标与 GET_FIELD 对 boxed obj 的解包路径。
2. 症状1：三上下文对 JSON.parse 产物顶层 bool 读值一致，`==` 比较确定工作，
   print 显示 `true`/`false`（不再位型 int 垃圾）。
3. 症状2：≥4 层深链字段读在任意上下文返回真实值或干净的 None（不返回位型
   垃圾；`??` 兜底行为可预期）。
4. 症状3：视图模板内 fn 调用（条件位/真值位）返回值与 computed 上下文一致，
   不再静默返空；与 P733-R2 预存红族做同根甄别（同根则收敛并在 §9 说明，
   异根则登记边界）。
5. musk PLAN-100 E2E 复跑稳定（AC-5）；musk 预计算绕过层向后兼容（AC-7）。
6. 既有回归零新增红（tv + 裸 `cargo t` + 受影响面；预存红零增减，与 PLAN-748
   基线口径一致——P733-R2 六条为零增减基线，若 T-04 同根收敛则改判并说明）。

### 非目标

- **不修 PLAN-748 的三个断点**（模板条件成员访问挂载面 / 续体静态名字段写
  Terminated / 动态 style 提取）——748 范围冻结，见 §4 边界判定。症状3 的修复
  面 = `eval_condition_with_inner` 的 **CALL 表达式通道**（LHS 比较位与裸真值
  位），不含 748① 的 binding 挂载面（`w.is_current` 走 `resolve_binding_path`
  臂，两通道已证异根——见 §4 已取证事实）。
- 不改 auto-musk 消费侧（其绕过层已落地并成为双端契约；根修后仍应工作）。
- 不动 park/resume/Stakes/RC 弹栈协议（与 748 同款红线；本计划读路径为主，
  唯一疑似交叠 = 748② 的 SET_FIELD Err 若与症状2 物化异常同链，则归 749 定音
  报告登记、748 侧按其"允许 T-00 后修订"机制联动）。
- 不处理 auto-lang 主检出他人 WIP（媒体引擎 8 文件，含 aura_view_builder.rs
  ——T-04 会改同文件；worktree 基面恒钉 master 已提交代码，rebase 时点与
  WIP owner 协调，同 748 待澄清#1）。

## 架构方案

```text
T-00 worktree lang-749（基 master 已提交代码）+ 干净构建 + 定音探针
  ├─ ① 症状1：JSON.parse 产物顶层 bool 三上下文读/print/== 矩阵
  │    （bool 位型十六进制 dump + 各上下文解码路径标注）
  ├─ ② 症状2：≥4 层深链逐层单变量探针（哪一层开始 MISS、接收者 tag 是什么）
  └─ ③ 症状3：视图条件 CALL 形态（LHS 比较 / 裸真值）vs computed 对照
  判定产物：定音报告（每症状根因层 文件:符号 + 修复路线 + 回归面）
T-01 三上下文最小复现单测族 plan749_read_semantics_tests（红相先行，钉死三病灶）
T-02 症状1 修复：bool 编码/解码/打印/比较一致性收口（NaN-box TAG_BOOL 双轨点）
T-03 症状2 修复：深链读语义（GET_FIELD 接收者分派兜底臂 + JSON 产物物化/分派一致性）
T-04 症状3 修复：视图条件求值器 CALL 通道（委托 resolve_expr_to_value 的 Call 臂）
  + P733-R2 同根甄别
T-05 musk PLAN-100 E2E 跨仓复跑对拍（绕过层兼容性 + 载入链稳定性）
T-06 全量门 + 提交 + 交接（含 748 边界联动注记与 KNOWN-DEBT 更新）
```

## 需求分析与背景调查

### 授权

用户 2026-10-10 转交 auto-os 侧发现的依赖问题（auto-musk PLAN-100 E2E 定音的
三型读语义病灶，根因在本仓引擎），并裁定分流：**新建 PLAN-749 单独根修，不并入
PLAN-748**（748 三断点范围冻结；判定依据见下"与 PLAN-748 的边界判定"）。本阶段
只起草；work 授权待用户确认计划后另行给出。

### 已取证事实（本会话 2026-10-10 master 代码实读，非转述）

**引擎侧锚点**：

| 症状 | 嫌疑面（文件:行，master @ 49f17c536） | 实读发现 |
|---|---|---|
| 1 | `crates/auto-val/src/nano_value.rs`（NaN-box 表示） | TAG_BOOL=0x0003_…（:15），encode_bool :66 / decode_bool :141 / is_bool :172；测试有 bool roundtrip |
| 1 | `vm/engine.rs:6810` GET_FIELD 字段值压栈臂 | **bool 压栈双轨**：Node 臂压裸 `push_i32(1/0)`（:6917 一带）；ObjectData 臂与 GenericInstanceData 臂已按 Plan 402 §13.10 压 `encode_bool`（含注释："否则 obj.bool_field==false 比较 i32(0) vs TAG_BOOL → 混 tag → EQ false"——**该失效模式与 musk 观测同形**） |
| 1/2 | `vm/engine.rs:6810-6852` GET_FIELD 接收者分派 | 只有 is_i32/is_object/is_list 三 tag 有正臂；**其余 tag 落 `decode_i32(nv) as u64` 兜底（:6850）**，注释自认"非 i32 接收者的 decode 兜底 id 可能含位型碰撞"（:6867-6869）——bool/f64/null 接收者 → 垃圾 id → 堆 MISS |
| 1/2 | `vm/ffi/stdlib.rs:2967`（占位 `Json.parse` 原样透传字符串）与 `:2981 shim_json_parse_vm`（vm 感知版，对象→`__json_object` GenericInstanceData、数组→ListData\<Value\>） | **命名双轨**：占位版注册名 `"Json.parse"`，vm 版经 `engine.rs:766 resolve_qualified("auto.json.parse")` 绑定覆盖；musk 调用形态 `JSON.parse`（大写）最终命中哪条、大小写敏感性如何，T-00 ①定音（占位版透传 = os-config 现场"body.provider.kind==0 看似可用最坏形态"的前科，:2974-2975） |
| 2 | `stdlib.rs:2996 json_doc_heap_id`（JSON 文档堆对象判定） | i32 直收 `v>0` 当堆 id（:2999-3001）——与 GET_FIELD 兜底臂同款"裸 i32 当 id"惯例，负 i32/其他 tag 直接 None |
| 2 | Http.get_msg 信封入队路径（stdlib Http natives） | handler 侧 payload 到达形态（串/堆 id）未取证，T-00 ②定音 |
| 3 | `ui/aura_view_builder.rs:12833 eval_condition_with_inner` | **条件求值器无 CALL 通道**：比较臂 LHS 通道集合 = `.len()` 特例 / `resolve_binding_path` / `.` 开头 state 引用 / 裸名 read_state+eval_computed 兜底——`extractRunId(.block.tc)` 不匹配任何臂 → 裸名臂 `read_state("tc)")`/`eval_computed` miss → **恒 false**（:12960-13055 实读确认）；裸真值位同缺口 |
| 3 | `ui/aura_view_builder.rs:12542 resolve_expr_to_value` 的 `Expr::Call` 臂 | computed 上下文工作路径：裸 fn 名 → 实参递归求值 → `bridge.call_vm_fn`（PLAN-051 C3）；症状3 修复即把条件求值器的 CALL 形态**委托到此臂** |

**坑①家族谱系**：PLAN-733 修了 computed 侧三腿（fn 调用语义契约化）；残腿登记
于 KNOWN-DEBT-AND-RISKS.md:221 P733-R2（"fn 调用语义同族深层腿……后续计划指针：
以 p053_1 两测为入口单变量收敛"）+ `docs/specs/auto-lang/vm/design/vm-fn-call-
semantics.md` 已知边界③。本次症状3 是同族第三型（视图模板上下文），**P733-R2
指针由本计划承接**。

**musk 侧锚点（存在性已验证）**：`src/front/forge_store.at`（SessionsLoaded/
DetailLoaded/PollBackfill 信封消费 + :1773 坑① computed 注记）、
`src/front/forge_helpers.at`（messageBlocks 拍平臂 + extractRunId）、
`src/front/chat_message.at:339`（坑③视图 fn 返空注记 + 预计算字段绕过）、
`docs/plans/archived/100-relay-plan-gate-and-vm-runbox.md` §9（E2E 沙箱
vm4–vm13 观测摘要）。

### 与 PLAN-748 的边界判定（本次分流决策依据）

1. **层不相交（决定性）**：症状1/2 的嫌疑面是值表示层（auto-val nano_value）
   + native 物化层（stdlib json/http）+ 解释器读臂（GET_FIELD）——748 三断点
   （条件求值挂载面 / 续体 SET_FIELD 写 / style 静态提取）不碰这些层。
2. **症状3 与 748① 确证异根**：748① `w.is_current` 走 `resolve_binding_path`
   臂（binding 挂载面断裂，P733-R2 子件 override 嫌疑）；症状3 走裸名兜底臂
   （求值器无 CALL 通道）——同一函数 `eval_condition_with_inner` 内**不同臂**，
   修复面不重叠。
3. **验收宇宙不同**：748 权威验收 = musk 101 双 variant probe 回退；749 权威
   验收 = musk 100 E2E 沙箱载入链稳定性 + r.ok 真值。合并会使 748 变成跨两层、
   双消费计划的巨型计划，违反"中等粒度独立可执行计划"分解原则（733→748 先例
   也是每簇一计划）。
4. **候选同根链路（须甄别，非合并理由）**：748① 的 bool 条件观测可能被症状1
   的位型读污染；748② 的 SET_FIELD Err 可能与症状2 的物化/解包异常同链。故
   **建议 749 先行、748 的 T-00 定音在 749 合入后重取基线**（748 已有"允许
   T-00 后修订（revision 递增）"机制）；若用户裁定 748 先行，其 T-00 须显式
   甄别值层污染（748 待澄清#6 已注记）。
5. **串行执行**：两计划同碰 `engine.rs`/`aura_view_builder.rs`，禁并行 worktree
   各自改同文件后互毁合并。

### 风险与约束

- **回归面最大档**：值系统层（NaN-box 打标/解码）改动波及**所有 .at 应用**——
  门禁按 AGENTS Category B 全套（`cargo check -p auto-lang` 快检 / scoped
  `cargo t` / 复审裸 `cargo t` + `cargo tv`），修复纪律 = 收口双轨点而非改
  tag 位型（位型是 ABI，`decode_i32` 兜底臂的既有消费者依赖它——Plan 419/402
  注释链为证；改位型须全仓清点，默认不走）。
- **主检出 WIP**：master 工作区 8 文件媒体引擎 WIP 含 `aura_view_builder.rs`
  （T-04 同文件）——他人 WIP 不归本计划，worktree 钉 master 已提交代码，
  收尾 rebase 前与 owner 协调（向用户上报，不静默包含/丢弃）。
- **musk 观测基面**：PLAN-100 全部实机证据来自 musk 沙箱当日的 auto.exe 构建，
  T-00 干净基面重取是硬门槛（748 同款纪律），不得直接引用 100 的观测定罪。
- **禁静默兜底补丁**（748 AC-07 同款纪律）：三修复点均须在单测中观测到真值
  行为差异；不许加"catch 后返默认值"式塌缩。

## 详细设计

### T-00 定音探针（判定产物：定音报告，每症状根因层 + 修复路线 + 回归面）

1. **干净基面**：`git worktree add D:/autostack/.wt/lang-749/auto-lang -b
   plan-749-dev master`（或 new-wt-group.sh）→ cargo 构建 debug auto.exe，
   固定 `AUTO_EXE`。
2. **症状1 矩阵**：最小 .at 语料 `let r = JSON.parse("{\"ok\":true,...}")`，
   三上下文（handler 栈内 / computed 体内 / 视图模板表达式）× {读 r.ok 直接
   print、r.ok == true、r.ok == 1、真值位 if r.ok}；bool 位型以十六进制 dump
   （`AUTO_DEBUG_GETFIELD=1` + 探针打印 raw nv）。标注每上下文的解码路径
   （哪个压栈臂、哪个 print/比较臂）。含 `JSON.parse`/`Json.parse`/
   `json.parse` 命名敏感性核验（占位透传 vs vm 物化哪条命中）。
3. **症状2 逐层探针**：`resp.session.messages[1].blocks[0].tool_name` 拆成
   逐层单变量（resp.session / +.messages / +[1] / +.blocks / +[0] / +.tool_name），
   每层 print 接收者 tag（is_object/is_i32/is_list/其他）——定位从哪层开始
   MISS、接收者落 GET_FIELD 哪个分派臂（正臂 or :6850 兜底）。Http.get_msg
   信封真实 payload 到达形态一并取证。
4. **症状3 对照**：同一 fn（extractRunId 形态最小化）在 computed 体内 vs 视图
   条件位（`if f(.x) != ""`）vs 视图裸真值位（`if f(.x)`）三通道取值对照。
5. **判定纪律**：每症状输出"根因层（文件:符号）+ 修复路线（A/B 案）+ 回归面"；
   证据不足时扩探针不猜；与 748①/②的候选同根链路显式甄别（结论写进定音
   报告并同步 748 待澄清#4）。

### T-01 三上下文最小复现单测族（红相先行）

`plan749_read_semantics_tests`（挂 `crates/auto-lang/src/tests/`，镜像 plan733
矩阵风格）：
- 症状1 组：JSON.parse 产物顶层 bool 三上下文读值一致 + print 显示 + `==`
  两种比较（bool 字面量 / int 1）；
- 症状2 组：≥4 层深链逐层读（对象.对象.数组[索引].对象.字段）三上下文真值或
  干净 None；
- 症状3 组：视图条件 CALL（LHS 比较/裸真值）vs computed 上下文同值。
修复前全红（绑定 T-00 定音的干净基面形态）、修复后全绿；红相证据留 §9。

### T-02 症状1 修复：bool 编码/解码/打印/比较一致性

按定音落点收口双轨点（候选面，以 T-00 为准）：
- GET_FIELD Node 臂裸 `push_i32(1/0)` → `encode_bool`（对齐 Plan 402 ObjectData/
  GenericInstanceData 臂口径）；全仓清点其余 bool 压栈裸 i32 点（SET_ELEM/
  GET_ELEM/native 返回通道/print 与 to_string 解码臂）；
- print/显示路径对 TAG_BOOL 的解码（display/int 双态消费点逐个核验）；
- EQ/NEQ 混 tag 臂（Plan 402 注释所述 `i32(0) vs TAG_BOOL` fallthrough false
  形态）是否需要 bool↔int 规范化（以 musk 观测的间歇性定方向——间歇 = 不同
  臂不同打标，收口后应确定工作）。

### T-03 症状2 修复：深链读语义

按定音落点（候选面，以 T-00 为准）：
- GET_FIELD 接收者分派 `else` 兜底臂（engine.rs:6850）：非 i32/object/list
  tag 接收者从"decode_i32 位型碰撞静默垃圾 id"改为响亮 Err 或干净 None push
  （语义择一由定音报告定，兼容性以裸 `cargo t` + tv 全量对拍兜底）；
- JSON 产物物化链一致性：`json_to_vm_value`（Plan 446 D1）产物经深链访问时
  中间层值 tag 与 GET_FIELD/GET_ELEM 分派的匹配性；`json_doc_heap_id` 负 i32/
  混 tag 行为对齐。

### T-04 症状3 修复：视图条件求值器 CALL 通道 + P733-R2 甄别

- `eval_condition_with_inner` 补 CALL 通道：LHS 比较 位与裸真值位对 CALL
  形态委托 `resolve_expr_to_value`（其 :12542 `Expr::Call` 臂含 bridge.call_vm_fn
  与 t()/style recipe 通道），保持既有臂序（`.len()`/binding/state 优先，
  CALL 兜底）不打扰既有语义；
- 判定来源：字符串式条件先用既有 `parse_dot_path_to_expr` 家族解析失败后，
  尝试整段 parse 为 Expr 再分派（与 parser 保留原文的形态对齐，748 §4 断点①
  链路 parser.rs:16390 条件逐字保留为证）；
- **P733-R2 同根甄别**：跑 `musk_vm_track_p053_*` 预存红六条——若本修复使其
  中 computed 链式 helper 腿转绿则同根收敛（改判基线 + KNOWN-DEBT 条目更新 +
  §9 说明）；异根则维持零增减并登记边界。

### T-05 musk PLAN-100 E2E 跨仓复跑

musk 仓起 E2E 沙箱（独立 musk 后端 + 种子 chats.json + `GET
/api/chats/session/{id}/page` ~50KB 拍平块 JSON），锁 `AUTO_EXE` 指向 749
worktree 构建：
1. **绕过层兼容（AC-7）**：musk 现状代码（预计算浅字段版）在根修构建上
   probe 全绿——双端契约不破坏；
2. **病灶面验证（AC-5）**：forge_store.at 观测点 `[084-probe]`/`[SSE]` print
   显示 `ok=true`；SessionsLoaded/DetailLoaded/PollBackfill 载入链连续
   ≥5 次 boot 无间歇失效；
3. （可选，非验收）chat_message.at fn 调用形态回迁验证。

### T-06 全量门与收尾

`cargo check -p auto-lang` + 受影响 scoped `cargo t`（vm/ui）+ 复审门禁裸
`cargo t` + `cargo tv`（VM 改动随身档）——预存红零增减（P733-R2 六条口径）；
KNOWN-DEBT-AND-RISKS.md P733-R2 条目更新（同根收敛或边界登记）；**748 边界
联动注记**（若 T-00 甄别出 748①/② 同根链路，748 修订其 T-00/T-02 范围，
revision 递增）；提交 + §9 记录 + 交接 review。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/architecture.md | before：JSON.parse 产物读语义无契约成文（bool 打标/深链读/三上下文一致性未规定）；after：成文"JSON 产物读语义契约"——bool 字段读值三上下文一致为 TAG_BOOL、深链读真值或干净 None、接收者 tag 分派表与兜底语义 | 症状1/2 是契约缺位导致的实现双轨 | AC-01/02/06 |
| SD-02 | modify | docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md | before：已知边界③登记 fn 调用语义残腿（P733-R2 指针）；after：视图条件位 CALL 通道语义并入契约（委托 resolve_expr_to_value Call 臂），坑①家族第三型关闭，边界③按 T-04 甄别结果收敛或改写 | 症状3 根修 + 债务台账销账 | AC-03/04 |

## 测试设计

| 验证 | 命令/方法 | 期望 |
|---|---|---|
| V01 定音探针 | T-00 探针套件（干净基面 + AUTO_DEBUG_GETFIELD + 位型 dump） | 三症状根因层判定报告；干净基面复现成功（或证伪并改判——musk 构建污染情形须报告） |
| V02 红相先行 | `cargo t plan749_read_semantics`（T-01 单测族） | 修复前全红（形态绑定 T-00 定音）、修复后全绿 |
| V03 全量回归 | 裸 `cargo t` + `cargo tv` + scoped `cargo t vm`/`cargo t ui` | 零新增红；P733-R2 六条零增减（同根收敛则改判并说明） |
| V04 跨仓 E2E | musk PLAN-100 沙箱（AUTO_EXE 锁 749 构建，≥5 次 boot） | 绕过层 probe 全绿；载入链零间歇失效；`ok=true` 显示 |
| V05 daily 对拍 | daily 回归 vs master 基线逐名 | 零增减 |

## 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | 三上下文（handler/computed/视图）对 JSON.parse 产物顶层 bool 读值一致，print 显示 `true`/`false`（无位型 int 垃圾），`== true` 确定工作 | V02/V03 |
| AC-02 | ≥4 层深链字段读任意上下文返回真实值或干净 None（无位型垃圾；`??` 兜底可预期） | V02/V03 |
| AC-03 | 视图模板内 fn 调用（条件位/真值位）返回值与 computed 上下文一致，不静默返空 | V02/V03 |
| AC-04 | 三型最小复现单测钉死（plan749_read_semantics_tests，修复前红/修复后绿证据留档） | V02 |
| AC-05 | musk PLAN-100 E2E：SessionsLoaded/DetailLoaded 载入链连续 ≥5 次 boot 无间歇失效，`r.ok` print 显示 `true` | V04 |
| AC-06 | 全量零新增红；预存红零增减（P733-R2 六条口径；同根收敛则改判记录在案） | V03/V05 |
| AC-07 | musk 预计算浅字段绕过层在根修构建上 probe 全绿（双端契约向后兼容） | V04 |
| AC-08 | 禁静默兜底补丁：三修复点均可在单测中观测到真值行为差异；兜底臂改动为响亮 Err 或干净 None，非默认值塌缩 | review 绑定代码+测试 |

## 执行步骤

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证 | AC |
|---|---|---|---|---|
| [ ] T-00 | 无 | worktree lang-749（plan-749-dev，基 master 已提交代码）；干净构建 AUTO_EXE；三症状定音探针 + 判定报告（含 748 同根甄别） | V01 | — |
| [ ] T-01 | T-00 | `plan749_read_semantics_tests` 三上下文最小复现单测族（红相先行） | V02 | 04 |
| [ ] T-02 | T-01 症状1 组红 | bool 编码/打印/比较一致性收口（Node 臂等双轨点 + print/EQ 解码臂） | V02/V03 | 01 |
| [ ] T-03 | T-01 症状2 组红 | GET_FIELD 接收者兜底臂语义化 + JSON 物化链一致性 | V02/V03 | 02 |
| [ ] T-04 | T-01 症状3 组红 | 条件求值器 CALL 通道（委托 :12542 Call 臂）+ P733-R2 同根甄别 | V02/V03 | 03,06 |
| [ ] T-05 | T-02～T-04 | musk PLAN-100 E2E 跨仓复跑（AUTO_EXE 锁 749 构建；绕过层兼容 + 载入链稳定） | V04 | 05,07 |
| [ ] T-06 | T-05 | 裸 `cargo t` + `cargo tv` + KNOWN-DEBT P733-R2 更新 + 748 边界联动注记 + 提交 + §9 交接 | V03/V05 | 06,08 |

## 复审记录

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-749
- plan_revision: 1
- outcome: pass
- next: work（待用户确认计划后授权；主检出 WIP 8 文件的 owner 协调为 work
  前置面，同 748 待澄清#1；与 748 的执行顺序建议 749 先行——见待澄清#2）
- changed_tasks: T-00～T-06
- changed_acceptance: AC-01～AC-08
- evidence: 本节§需求分析（引擎侧七锚点 file:line 实读 + musk 侧锚点存在性
  验证 + KNOWN-DEBT P733-R2 台账承接 + 733→748 分流判定五条依据）
- review_authority: 独立 auto-plan:review 复验定音报告与修复证据。

## 待澄清事项

1. **主检出 WIP owner 协调**（owner=用户/WIP 所属会话）：master 8 文件媒体
   引擎 WIP 含 `aura_view_builder.rs`（T-04 同文件）——work 前确认归属与
   rebase 策略（同 748 待澄清#1，两计划共用此协调点）。
2. **与 PLAN-748 的执行顺序**（owner=用户）：建议 749 先行（值系统层是 748
   观测的下游真值层；748①bool 条件/②SET_FIELD Err 有候选同根链路，先修值层
   可能让 748 T-00 定音更干净甚至收缩范围）；若用户裁定 748 先行（其已就绪
   待确认，Phase 2 jade 四件可先行且不受本顺序影响），则 748 T-00 须显式
   甄别值层污染（748 待澄清#6 已注记），两计划严格串行（同文件冲突）。
3. **musk E2E 复跑归属**（owner=T-05，需 musk 仓操作授权）：PLAN-100 沙箱
   复跑在 musk 仓进行，属跨仓验证动作——work 授权时一并确认。
4. **兜底臂语义择一**（owner=T-00 定音报告）：GET_FIELD 非常规 tag 接收者
   改"响亮 Err"还是"干净 None push"，以兼容面（裸 `cargo t`/tv 对拍）定，
   不预设结论。

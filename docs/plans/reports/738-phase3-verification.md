# PLAN-738 Phase 3 / T-14 收据：最终提交门禁、真实验收与独立 review 交接

- 最终 code commit：worktree `plan-738-dev@b3a4d660e`（clean），基线 `2c1b4a763`（R2），Phase 3 链：`c1579ed71`（T-10）→ `84bec29ef`（T-11）→ `b6cfc6df1`（T-12）→ `1a983091f`（T-14a lock 首物化语义）→ `b3a4d660e`（T-14b 服务链收口）。改动面 8 文件 +2224/−284。
- 依赖：auto-down 兄弟 `895f8d0f` 只读未动；生成 workspace 依赖=auto-lang workspace path-dep（收据 workspace_lock 身份在案）。

## 1. 门禁总表（全部在最终树或 auto-lang 字节等同树上执行）

| 门禁 | 命令 | 结果 | 绑定 |
|---|---|---|---|
| 三 crate check | `cargo check -p auto-lang/-p auto-man/-p auto` | 零 error | b3a4d660e（及前树，见注） |
| 裸 t 全档 | nextest t 别名 --no-fail-fast | 5183 测试：**16 红全分诊**（见 §2） | b6cfc6df1（auto-lang 面与最终提交字节等同——1a983091f/b3a4d660e 仅改 auto-man，见 §4） |
| 语料档 | `cargo tv` | **162/162** | 同上 |
| 转译档 | `cargo tt`（--no-fail-fast） | **13 红全分诊**（§2） | 同上 |
| HTTP 档 | `cargo th -j 1`（--no-fail-fast） | 102：85 绿/3 红/14 超时——逐名分诊 §3 | b6cfc6df1 |
| plan738 族 | `cargo t plan738` | **69/69** | b3a4d660e |
| plan724 族 | `cargo t plan724` | **5/5** | b3a4d660e |
| CLI 档 | `cargo test -p auto --bin auto stdlib -- --test-threads=1` | **10/10** | b3a4d660e |
| auto-man api_gen | `cargo test -p auto-man --lib api_gen -- --test-threads=1` | **43/43**（1 ignored=⑤腿按需） | b3a4d660e |
| 新鲜度族 | `cargo test -p auto-man --lib freshness -- --test-threads=1` | 2/2 + shell_pack 环境红（§2） | b3a4d660e |
| **⑤腿 Rust 实编 witness** | `--features test-trans rust_host_provider_real_compile_witness -- --ignored` | **1/1**（真实 rustc） | 最终树 |
| **C stdio 真实 MSVC witness** | `c_stdio_provider_bindings_compile_and_read_real_file -- --ignored` | **1/1** | 最终树 |
| **正式生成服务完整链** | `--features test-http-e2e http_e2e_plan738 -- --ignored --test-threads=1` | **1/1**（38.6s：生成→收据 schema4+workspace_lock→实编→serve→ready 指纹=consumer_fingerprint→业务 42→仅改 stdlib→陈旧→再生→重建复验） | b3a4d660e |
| 健康扫描 | 触面 diff dbg!/DBG 残留=0；rustfmt --check=FMT-CLEAN | 通过 | b3a4d660e |

## 2. 逐名红分诊（裸 t 16 / tt 13 / th 3+14 / freshness 1——零未解释新增确定性红）

**裸 t（16）**=全部属 R2 已档基线：musk p053/p054 ×6、desktop_protocol、iced renderer ×2、ash_leak、schema_drift ×2、docs_gen（13 master 预存，R2 逐名实证在案）+ plan502、plan707_client、plan606_029（flake 族；plan484_024/clipboard 本轮绿）。
**tt（13）**=同基线族子集（过滤非基线红=0）。
**th（3 红+14 超时）**：
- back_proxy corpora data face：R2 收据在档基线红；
- plan730 interop（FAIL）与 vm_multipart（TIMEOUT）：**基线复现实证**（临时 worktree @`2c1b4a763` 同命令同症状 109s/120s——plan730 固定端口上传族今晚环境性红；R2 今晨收据为绿，环境因素未定机但归因基线成立，留独立 review 复核）；
- e2e_sse_chain：串行全档中失败，**双树隔离复跑绿**（负载级联类）；
- 其余 13 超时：interop 失败后 plan730 族级联（与上述同族）+plan326 redirect。
**freshness**：`test_shell_pack_lib_freshness` 环境预存红（T-11 基线树 stash 实证同红——本地 auto-os/shell 检出与入库 shell-pack 物不一致）。

## 3. T-14 执行中发现并修复的问题（新提交重跑受影响门禁）

1. `1a983091f`：lock_freshness 首次物化语义——生成时未建 lock（收据 absent）→ 首次构建后 lock 出现是**已记录 manifest 依赖的确定性物化**，不是依赖漂移；原语义会令每个新 workspace 首启误判陈旧（plan730 interop 隔离复现定位）。真值表更新（absent→实值=新鲜）。
2. `b3a4d660e`：T-12 补丁的 api_gen 收据 `workspace_lock` 写入因当时脚本断言中断被遗漏（rust_ui 门读字段而 api_gen 未写）——服务链实测暴露，补位；新鲜度门 current 侧 lock 缺失归一化为 "absent"（表示层不伪造陈旧）；ready 烘焙指纹与收据字段对齐（consumer_fingerprint）；服务测试 schema 4 断言。受影响门禁全部复跑绿（§1 b3a4d660e 行）。

## 4. 收据绑定说明

裸 t/tv/tt/th 跑在 `b6cfc6df1`；`1a983091f`/`b3a4d660e` 仅改 auto-man 两文件（`git diff --name-only` 实证），auto-lang 测试面字节等同，t/tv/tt（均 `-p auto-lang`）收据据此外推有效；th 含生成服务面的部分由最终提交的服务链 1/1 与 freshness 2/2 复跑覆盖。

## 5. 范围声明

- 本档证明：Phase 3 修复面（strict 门/引用闭包/双重身份/lock 收据）零新增确定性红，三目标能力以 witness/CLI/服务链实证。
- 不称全库语义 parity（D3b 边界不变）；P738-D1/D2 与第四绑定面维持登记债务；legacy 别名/接收者分型面边界见 [738-phase3-reference-audit.md](738-phase3-reference-audit.md)。
- R2 收据保留历史；本档为 `b3a4d660e` 的执行证据，不外推后续提交。

## 6. 交接

- 全部执行验收（AC-01..08 执行面、T-09..T-14）闭合 → 计划置 `execution_done`。
- 下一步：**未参与实现的独立 review agent** 按 `/auto-plan:review` 复核最终提交 `b3a4d660e`：全部 AC、实际 callee/manifest、`2c1b4a763..b3a4d660e` 完整 diff（T-09 分类 + T-10..14 语义面）、SD 正文、th 环境红基线归因。执行 agent 不自行宣告独立复审 pass。
- 当前授权止于修复/验收/复审准备：不合入、不归档、不删 worktree。

## 7. R5 修复轮收据（2026-10-10，最终提交 `68398d2d6`）

R5 补充复审（[738-review-r5.md](738-review-r5.md)）三项必修闭合；§1-6 的 R3/R4 收据保留历史（其 pass 不覆盖 R5 反例）。

| 门禁 | 结果 |
|---|---|
| 三 crate check | 零 error（auto-man `--tests` 的 plan734 fixture 6 错=R4-01 在案 master 预存） |
| `cargo t plan738` | 72/72（含 R5 新 5 测：receiver 收集/漂移拒绝/用户隔离 + 一次绑定状态机 + 三目标真实三角） |
| `cargo t plan724` / CLI stdlib / api_gen | 5/5、10/10、44/44（含 generation_consumer_identity 真实生成腿） |
| freshness 族 | 双真值表（严格比较语义）+ `review738_r5_lock_binding_state_machine` 1/1；shell_pack 环境红在案 |
| tt 全档 --no-fail-fast | 非基线红=0（R5-02 初版曾致 6 语料红——receiver 形态守卫+扁平 helper 切片修正后归零；基线红同 §2 名单） |
| 裸 t 全档 --no-fail-fast | 5186 测：5170 绿/16 红全基线（13 master 预存+plan502/plan707_client/plan606 flake 族）；非基线红=0 |
| `cargo tv` | 162/162 |
| 服务完整链 | 1/1@64.66s（生成→收据→实编→serve→ready→业务→stdlib-only 失效→再生→重建，lock 一次绑定语义下） |
| CLI 级 R5 反例复验 | 原始 exit0+`JsonValue.len` proof；len int→str 漂移 exit1+`SIGNATURE_DRIFT`（真实 auto.exe） |

修复链：`bc0b95cfd`（三项主体）→ `796d279a4`（真值表对齐状态机）→ `68398d2d6`（守卫+切片，tt 归零）。R5 findings→AC 映射重开项（T-11..T-14）全部重闭；SD-04/05 重锚、SD-06/07 维持；acceptance 计数勘误清零。

## 8. R6 修复轮收据（2026-10-10，最终提交见 git log）

R6 复审（[738-review-r6.md](738-review-r6.md)）P738-R6-01 必修闭合：receiver 方法面泛化到全部公共方法（shape③ 全方法 + 模块面落穿 + 未接管发射臂 case④ 诚实拒绝 + json 值实参 Value 面分派修正 + Vec<T>→[]T 序列面）；双拼写漂移矩阵（keys/as_int/is_null × receiver/模块限定）落为正式测试 `receiver_method_drift_matrix_both_spellings`——keys/as_int 基线零违规+proof/漂移拒绝，is_null 双拼写基线即拒（公共 int vs producer bool 真漂移，同 is_valid 族诚实面）。R6-02：bind 写失败保守判陈旧。R6-03：plan498_bar_group_emphasis 入 flake 名册（隔离复跑绿）。SD-04/reference-audit #11 按实际行为重锚。受影响门禁见下表（本节提交时附齐）。

## 9. R7 修复轮收据（2026-10-10，最终提交 `3713337d9`）

R7 复审（[738-review-r7.md](738-review-r7.md)）P738-R7-01/02 必修闭合：
- **R7-01**：`json.has_key` 发射臂双闭括号 bug 修复（产物编译错在案）；全分母 12 方法 × 双拼写矩阵落为正式测试（`receiver_method_drift_matrix_both_spellings`）——绿面（as_string/as_number/as_int/as_array/keys/len/get/get_at/has_key 的可写拼写）基线零违规+proof+漂移拒绝；真漂移面（is_null/as_bool 公共 int vs producer bool）双拼写基线即拒；parser 不支持拼写（get/get_at/has_key receiver 带参）语言层拒绝；无臂模块拼写（type/as_number/as_array）case④ 诚实拒绝。
- **R7-02**：case④ 增 local 变量/模块排除（`let json="abc"` shadow 不误拒）。
- 新适配契约：BoolToIntIf（has_key）/len ScalarCast（宽度归一）；observe 剥 Paren 壳；AST 实参序列化（shape③ 构造不再依赖缺陷发射文本）；Null 哨兵族扩展（get/get_at 与 parse 同族）；守卫跳过改诚实拒绝（公共面存在而形态不可核对=UNVERIFIED）。
- 门禁（3713337d9）：plan738 73/73、plan724 5/5、CLI 10/10、api_gen 44/44、tv 162/162、tt 非基线红=0、freshness 双真值表+lock 状态机（shell_pack 环境红在案）、服务完整链 1/1@109.5s。

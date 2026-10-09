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

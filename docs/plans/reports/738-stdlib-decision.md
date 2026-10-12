# PLAN-738 T-01 决策报告：stdlib 装配调用图与六核心 provider 冻结

- 日期：2026-10-08
- 状态：T-01 完成（worktree `D:/autostack/.wt/lang-738/auto-lang`，分支 `plan-738-dev`）
- 基线：master `c3ccd32c3`（worktree 基面）；探针提交 `ac41c2d1e`
- 执行序说明：计划 §4.1 原授权以「736 独立 review pass + 合入」为执行前置；2026-10-08
  用户明确指令「计划738 实施它」，据此进入 executing。736 尚在 executing 3/8
  （worktree `700e916bc` 未合入），其最终合同未冻结——因此 **T-06 中消费 736
  最终生成/ready 合同的接线保持 gated**，其余任务（T-01..T-05、T-07 的大部分）
  与 736 无生产耦合，按计划继续。T-01 对 736 基线的要求以**只读勘验其 worktree
  现状**替代（见 §7），736 合入后须复核的增量在 §8 列明。

## 1. 装配调用图（实勘冻结，全部经真实入口验证或静态锚定）

### E1 四个装配面（互不共享装配计划）

| # | 入口 | 模块装载 | 层选择 | 实现来源 |
|---|---|---|---|---|
| ① | VM 执行管线 `execute_autovm`/`test_code`/`run_with_path`/`debug_file`/`create_vm_from_source`（lib.rs:1319/1779/5128/5357 一族） | `session.resolve_uses(code)` → `load_module`（compile.rs:1322） | 公共 `.at` + **硬编码 `.vm.at`**（compile.rs:1614 `context_ext` 常量，注释声称"根据编译引擎类型选择"但无分支） | 合并源 → TypeStore + bytecode；`#[vm]` fn 由全局 native 注册表按名提供 |
| ② | VM persistent `AutovmReplSession::run`（autovm_persistent.rs:253 `load_and_register_module`） | stdlib-only 查找（`CARGO_MANIFEST_DIR/../../stdlib/auto`）；用户模块必然 `Module not found` | 公共 `.at` only——context 路径为 `<stdlib>/auto/<module_path>.vm.at`，`module_path` 未剥离 `auto/` 前缀 → **`stdlib/auto/auto/net.vm.at` 双前缀恒不存在，静默跳过**（:292-293） | 全局注册表（ffi/stdlib.rs 手工面 + `register_vm_declarations` 磁盘扫描面）；公共层 `#[vm]`（io.at read_line、sse.at parse_sse）可注册短别名，`.vm.at` 层符号一律不可见 |
| ③ | Rust 转译 `trans_rust_with_session`（lib.rs:5960） | **零模块装载**——`compile_source` 不解析 use；发射前独立重解析入口文件（`CompileDest::TransRust`） | 不选层；`.rs.at` 生产不消费（仅 `a2r_std_signature_parity.rs` 文本级 name/arity 对拍镜像） | 发射期名称表：`use auto.X` → `a2r_std::X`（trans/rust.rs:17900/17928/17945 **三处重复硬编码 26 名单**） |
| ④ | C 转译 `trans_c_with_session`（lib.rs:5888）＋C stdlib 生成 `cmd_a2c_stdlib`（crates/auto/src/cmd_a2c_stdlib.rs） | 转译入口零模块装载；stdlib 侧为**逐文件批生成**（`.at`/`.c.at` 各自独立单元，跳过 `.vm.at`，无 manifest 无跨文件关联） | `.c.at` 仅被 cmd_a2c_stdlib 消费（io.c.at→io.c.c/io.c.h） | 用户程序 `use auto.X` → `#include "X.h"`（trans/c.rs:1324-1327 `libs.insert`）——依赖预生成头文件在 C 构建期存在 |

探针证据：P1（①的层选择）、P3（③④零装载：合成四层模块经两个 trans 入口后
session 均无该模块；Rust 发射对表外名称落 `crate::proto` 无 provider 校验）、
P5（②的 context 缺失）。测试族 `crates/auto-lang/src/tests/plan738_assembly_probe_tests.rs`。

### E2 ModuleCache（仅参与面①，三重缺陷）

- 键 = 模块**名**（无 target/env/root/session 维度）——跨 target 串台一旦命中即发生。
- `content_hash` = **合并源**（公共+`\n`+context）hash；`is_valid()` 重读 `file_path`
  （=公共文件）单独重算 → 含 `.vm.at` 层的模块**永不命中**（P2 实证）；无 context
  层的模块缓存机制正常（对照组）。
- `interface_hash` 恒 0（TODO 未实现）。

### E3 native 名 surface（三机制并存，六核心无固定 ID 目录）

1. `for_each_native!` 目录（native_catalog.rs）：固定 ID，只覆盖 native.rs 内
   `self.register()` 的内建面（print/str/hashmap/list/assert/editor 等）；注释明示
   stdlib.rs 手工 shim **不在目录内**。
2. `register_vm_declarations()`（native_registry.rs:374）：init 时**扫描 CWD 相对
   `stdlib/auto` 下 `*.vm.*.at`**（排序后），解析 `#[vm]` 声明注册名 → 动态 ID
   （计数器自 100 起）。**VM native 名 surface 的来源 = 磁盘扫描**，排序保证同
   进程稳定，但 ID 依赖文件集合/扫描根。
3. ffi/stdlib.rs `register_shim_by_name`（~54 个手工 shim，net/process/option 等）+
   `#[rust_fn]` inventory 面：按名从注册表查 ID 后绑 shim，**名缺失即 panic**。

**来源身份裂缝（G5 的实锤）**：模块装载走 `find_std_lib()`（util.rs:8），其项目根
分支拼出 `<root>/stdlib/stdlib/auto`（三级上溯+重复 stdlib）恒不命中 → cargo 构建
下实际解析 `~/.auto/libs/stdlib/auto`（本机是指向仓库的 symlink；无该 link 的机器
行为完全不同，最终回落 CWD 相对 `stdlib/auto`）；而 native 名扫描走 **CWD 相对
`stdlib/auto`**。两个根不同源：CWD ≠ 仓根时，装载的层与注册的 shim 名可来自不同
stdlib 身份。manifest 必须记录实际 stdlib 来源身份（本计划 §5.1 已有该字段）。

### E4 公共↔VM 层名称/签名漂移（六核心普遍存在）

- net：公共 `TcpListener.accept(self) TcpStream?` vs VM 层/native `tcp_listener_accept(listener) TcpStream`（方法式↔自由函数式、`?`↔裸返回）；公共 14 符号（含 2 type）vs VM 层 14 自由 fn，非同名子集。
- http：公共 79 pub fn + 10 type（Server/Request/Response/RequestBuilder/FileResponse/FileTransfer/HTTPStream/UploadReceipt/UploadRequest/UploadSession）vs VM 层 68 个 `http_*`/`server_*`/`request_*`/`response_*` 前缀化 fn（`get`→`http_get`、`listen`→`server_listen`）——靠 `TYPE_CANONICAL_MAP` + `resolve_qualified` 归一缝合，逐符号绑定关系 T-04 冻结。
- json：公共 20（encode/parse/…）vs VM 19（部分同名、部分 `json_*` 前缀）。
- io：公共 type File 内联 7 方法 + 顶层 say/read_line；VM 层 ext File 8 方法；C 层 ext File 8 方法（三方名称一致但属不同物理层）。
- 混合属性 legacy：io.at 顶层 `#[vm] pub fn read_line`、sse.at 顶层 `#[vm] pub fn parse_sse`——公共声明带 VM 属性，按计划 §5.3 记录为可追踪 legacy 事实。

## 2. 六核心分母（parser 精确清点，P6 冻结）

13 层全部计入（含 2 个 parse 破损层）；分母=13，parse-OK=11，parse-fail=2。

| 模块 | 层 | 顶层 fn | pub | #[vm] | type | ext 块/方法 |
|---|---|---|---|---|---|---|
| io | io.at | 2 | 2 | 1 | 1（File，内联 7 方法未计） | 0 |
| io | io.vm.at | 1(say) | 0 | 8 | 0 | 1/8 |
| io | io.c.at | 0 | 0 | 0 | 0 | 1/8 |
| net | net.at | 14 | 14 | 0 | 2 | 0 |
| net | net.vm.at | 14 | 0 | 14 | 0 | 0 |
| async | async.at | — | — | — | — | **parse 失败** |
| async | async.vm.at | 10 | 0 | 10 | 0 | 0 |
| http | http.at | 79 | 79 | 0 | 10 | 0 |
| http | http.vm.at | 68 | 0 | 68 | 0 | 0 |
| json | json.at | 20 | 20 | 0 | 1 | 0 |
| json | json.vm.at | 19 | 0 | 19 | 0 | 0 |
| json | json.rs.at | — | — | — | — | **parse 失败** |
| sse | sse.at | 1 | 1 | 1 | 0 | 0 |

注：type 块内联方法（如 io.at File 的 7 方法）不计入本轮顶层分母；T-02 正式
inventory 须遍历 `TypeDecl.methods`，分母只增不减。（T-02 已落实：loader 遍历
TypeDecl.methods 与 ext methods，归一身份 `Owner.name`。）

### §2b 全库 isolated-parse 基线（T-02 实勘，2026-10-08）

T-02 全库扫描（115 个 `.at` 层：74 公共 + 23 vm + 13 rs + 5 c）用**单文件独立
parse**（fresh TypeStore，生产同款构造）实勘：**39/115 解析失败**。三类构成：

1. **语法破损（真不可解析，生产同败）**：`str.at`（`fn replace(from str, with str)`
   —— `with` 被当关键字，参数名非法）；`list.at`/`async.at`（`type X[T]` 泛型声明
   无 parser 支持）；`iter/*` 10 件（spec 上下文语法）；`may.at` 等。
   **佐证**：`stdlib_tests.rs` 的 `use auto.str` 族测试全部
   `#[ignore = "requires stdlib import support (use auto.str)"]`——stdlib 导入面
   破损是在案已知事实。
2. **跨模块类型依赖（isolated-parse 假阳性候选）**：`app.at`（`Parent type 'Widget'
   not found`）等——生产路径经 resolve_uses 递归装载依赖后可解析。T-03 真实装配
   inventory 接线后按实际装配结果重分类。
3. **镜像语法漂移**：`json.rs.at`/`file.rs.at`/`image.rs.at`/`storage.rs.at`/
   `inline.rs.at`/`redis.rs.at`/`sqlite.rs.at`/`str.rs.at` 等 .rs.at 族——与
   parity-harness 文本消费的定位一致（不用语言 parser）。

**结构性结论（强化计划核心命题）**：VM 生态实际工作的 stdlib 面 = native 注册面
（`.vm.at` 磁盘扫描 + 目录固定 ID + stdlib.rs 手工 shim），**不是公共 `.at` 声明**；
公共声明层大面积不可装载而生态照常工作，正说明「声明与实现缝合」当前不存在门。

### §2c sse 修正（T-02 实勘）

§3 原「sse vm=supported（parse_sse）」**修正为 unsupported**：`parse_sse` 虽有
`#[vm]` 声明，但**无任何 native 注册/绑定**——注册扫描只读 `.vm.*` 层（sse 无
该层），stdlib.rs 手工面亦无 `auto.sse.*`（grep 零命中）。Response 上的 sse_* 流
方法在 http.vm.at 另有绑定。这是「声明存在但 callee 缺失」的实册首例。

### §2d native 名 surface 的 CWD 依赖（T-02 实勘补充 E3）

`register_vm_declarations` 扫描 **CWD 相对 `stdlib/auto`**；测试/工具进程 CWD 非
仓根时注册面为空（探针 P5 的注册名实际来自 VM 模块 init 与 NATIVE_ID_MAP 懒注册
面，非磁盘扫描）。生产 `auto` 二进制从非仓根 CWD 启动时 native 名 surface 的实际
构成（扫描空 → 手工 shim 按名查 ID 的 panic 面）**须在 T-04 冻结**；manifest 的
stdlib 来源身份字段必须覆盖 CWD 维度。

**parse 破损冻结（E5）**：
- `async.at`：`type Sender[T]` / `type Receiver[T]`（offset 692/797）——当前 parser
  的 type_decl 不消费 `[` 泛型参数语法，全 stdlib 仅此文件使用该写法。即
  **`use auto.async` 在面① today 即解析失败**；async 的可用执行面实际是 VM 内建
  task/chan natives（async.vm.at 10 个 `chan_*`/`spawn`/`sleep` 等）+ a2r-std task.rs。
- `json.rs.at`：镜像文件与当前 parser 语法漂移，parse 失败——与"`.rs.at` 仅 parity
  测试文本消费"一致（该 harness 用自带解析，不经语言 parser）。
- 738 不改语言语法（计划 §3 约束）；两者按 §5.1 记 diagnostic，修复属后续计划决策。

## 3. provider 目录要点（T-02 schema 的输入）

| 模块 | VM provider | Rust provider | C provider |
|---|---|---|---|
| io | ext File 8 方法（native 按名） | **无**（表路由 `a2r_std::io` 不存在——P4 同族证据） | io.c.at + 预生成 io.c/io.h/io.c.c/io.c.h |
| net | net.vm.at 14 #[vm]（ffi/stdlib.rs 手工 tcp shim 族，名称=VM 层名） | **无**（`a2r_std::net` 不存在，发射仍路由——P4 实证） | 无 |
| async | async.vm.at 10 #[vm]（公共层 parse 破损） | a2r-std `task.rs`（TaskRef actor 词汇，与 VM chan natives 非同一 provider，计划 §5.2：不强行视为同一调度者） | 无 |
| http | http.vm.at 68 #[vm]（server/client/stream/transfer/upload 族） | a2r-std `http.rs`=**客户端**（networking-stdlib.md 明示非 VM server 后端）；生成 Axum 服务=auto-man 独立装配 | 无（C 无 HTTP provider——Unsupported 而非手写） |
| json | json.vm.at 19 #[vm] | a2r-std `json.rs`（json.rs.at 镜像 parse 破损） | 无 json.c.at（gap 在案） |
| sse | **unsupported**（§2c：parse_sse 声明无 callee） | a2r-std `sse.rs`（仅解析无网络 IO） | 无 |

## 4. 关键设计裁决建议（供 T-02/T-03 实施，不改变计划验收面）

1. **装配计划引入点**：面①在 `resolve_uses → load_module` 处替换 context_ext 硬编码
   为 AssemblyContext（target/environment）驱动的层选择——`get_file_extensions`
   的 dest→后缀映射（parser.rs:7075，现为 dead_code）可作为选择语义的种子，但
   装配所有权在 compile.rs（backend-assembly.md 规约：compile.rs 是装载事实源）。
2. **面③④的 manifest**：Rust/C 入口当前零装载，T-03 不必为其强造层装载——manifest
   如实记录 provider kind（rust=name-table→a2r-std 映射 + 真实模块存在性校验；
   c=header-include + 预生成层身份），`.rs.at`/`.c.at` 记 candidate。"无法闭合真实
   target 接线则 needs_replan"的红线评估：面①接线可闭合（单一装载点）；面③④按
   上述 provider-kind 建模即可闭合，无需改造发射器架构。
3. **缓存修复方向**（T-05）：key 增 target/env/root 维度；valid 校验改为重读**全部
   selected 层**（公共+选定 context）联合 hash——顺手消灭 E2 的"双层恒 miss"。
4. **parse 破损**：async.at/json.rs.at 冻结于探针 P6；T-02 inventory 对其产出
   diagnostic（不升验证等级），不阻 Strict 门（六核心 strict 覆盖以"可解析符号"为分母，
   破损层如实报 partial）。

## 5. 与 734/736 的接口事实

- **734（已合入）**：`generation.json`（FNV-1a source_hashes + scaffold 标记）与
  `backend_generation_is_fresh`（rust_ui.rs）是两轨共用的在产新鲜度门——T-06 的
  assembly fingerprint 挂接点。
- **736（未合入，worktree 700e916bc，T-03 done）**：已有 `auto service` CLI、
  `HttpServiceConfig::effective_config_hash`（FNV-1a 64 canonical JSON）、
  `AUTO_HTTP_SERVICE_JSON` 环境合同、`AUTO_SERVICE_READY {bound,profile,config_hash}`
  就绪回执、`serve_async_with` VM seam、连接许可/观测计数器。**T-05 的 ready 身份
  与 734-generation gate 尚未实现**；738 T-06 消费其最终形态，保持 gated。

## 6. 门禁与后续

- 本轮验证：`cargo check -p auto-lang` 绿；`cargo t plan738` 6/6 绿（P3 24s/P4 48s
  为 trans 入口冷进程开销，T-02 正式族若复用同型断言须控制进程级成本）。
- T-02 依赖本报告冻结的：四装配面边界（§1）、六核心分母（§2）、provider 目录要点
  （§3）、native 三机制（§3/E3）。
- 736 合入后复核项（§8 清单）：ready 回执字段、generation gate 实现、config hash
  语义——如与 §5 描述漂移，T-06 按最终代码重锚（不改 AC）。

## 7. T-01 前置项的替代满足记录

| 计划要求 | 实际满足方式 |
|---|---|
| 「736 review pass+merge 后执行」 | 用户 2026-10-08 指令「实施它」进入 executing；736 消费面（T-06）保持 gated；其余任务与 736 无生产耦合（§5 分离论证） |
| 「读最终 config/ready/生成与 canonical 增量」 | 只读勘验 736 worktree @700e916bc（§5），标注非最终；736 合入后按 §8 复核 |
| 「最小公共+三目标/host mapped 真入口原型」 | 探针 P1-P6（真实 CompileSession 管线/两 trans 入口/persistent/a2r-std 对照），全部经真实入口 |
| 「无法闭合真实 target 接线则 needs_replan」 | 评估：可闭合（§4.2），不触发 needs_replan |

## 8. 736 合入后的复核清单（T-06 前置）

1. `AUTO_SERVICE_READY` 回执最终字段集（worktree 现状：bound/profile/config_hash）。
2. ready=734-generation 校验的实现位置与语义（736 T-05 承诺）。
3. `HttpServiceConfig::effective_config_hash` 与 738 assembly fingerprint 的边界
   （736 计划文本：service 配置 hash ≠ assembly 内容指纹——738 SD-05 保持）。
4. `auto service` CLI 与 738 `auto stdlib inspect` 的命令面共存（clap 子命令无冲突）。

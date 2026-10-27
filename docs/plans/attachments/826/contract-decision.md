# PLAN-826 T-01 紧凑 ABC 合同决策

状态：T-01 格式与验证合同冻结；T-10 仅实现本文的 core-i32 独立模块子集。基面 `master@e417247a5ca51aa3ba305215be52b9d543071a19`，唯一实施树 `D:/autostack/.wt/lang-826/auto-lang`，分支 `plan-826-dev`。

## 1. 已交付输入、owner 与写面

- D798 已归档交付，D828 已归档交付（merge 收据分别见两份 archive plan）；本轮消费当前 master 已合入源码，不读取其历史 worktree。
- 真实公开接口：`auto_av_core::mir::{MirModule, MirFunction, MirInstr, MirType, MirConst, Terminator}`、`auto_av_core::mir_verify::verify_mir`、`auto_av_core::contract` 的 profile/预算常量，以及 `abc::AbcDecodeError` 的公开分类。
- `MirModule` 明确是 UNCHECKED；`verify_mir` 是 `CheckedMir` 唯一构造入口。compact decoder 只返回 `DecodedCompactAbc { version, profile, module }`，不返回 `CheckedMir`/`LoadedModule`，也不提供执行入口。
- 私有生产源只有 `experimental/ac-av-probe/core/src/compact_abc.rs`。测试壳只有 `experimental/ac-av-probe/parallel/826/{Cargo.toml,Cargo.lock,src/lib.rs}`；壳重导出已交付 `contract/mir/mir_verify` 并以 `#[path]` 编译同一源。不得复制 IR/verifier/VM，不改 `core/lib.rs`、父 workspace members 或公共装配。
- 本阶段 owner：826 work；私有写面自 2026-10-26 合同登记后接受。公共 opcode/tag/version 注册、loader/cache/AV/AutoDown 写面未申请、未接受；T-02 起仍等 D825/D828 和核心窗口。825 的私有帧表示不是本 codec 输入。

## 2. 候选比较与选择

两候选均只改变 operand 编码，opcode/sub-op 均为一个固定字节；以下 body 不含模块头、section 目录、函数/块表和 debug。

| operand 值域 | canonical ULEB128/u32（每 operand） | 窄 u8 + `0xff` Wide/u32（每 operand） |
|---|---:|---:|
| 0..=127 | 1 B | 1 B |
| 128..=254 | 2 B | 1 B |
| 255 | 2 B | 5 B |
| 256..=16383 | 2 B | 5 B |
| 16384..=2097151 | 3 B | 5 B |
| 2097152..=268435455 | 4 B | 5 B |
| 268435456..=u32::MAX | 5 B | 5 B |

- 小索引 0..=127：`BIN = opcode + sub-op + dst/lhs/rhs = 5 B`，`COPY = opcode + dst/src = 3 B`，`RETURN = opcode + value = 2 B`，两方案均达标。
- ULEB 在 128..254 仅比窄方案多 1 B/operand，在其余大值域不劣于 Wide；Wide 的固定扩展路径没有总体密度优势。选择 **canonical ULEB128/u32**，不引入 Wide opcode。
- 最短编码是 AV 自己的 canonical 政策，不冒称 WASM 的要求。解码最多 5 字节；第 5 字节只能含低 4 位且 continuation=0；冗余零组、第 5 字节高位或 continuation、截断均拒绝。
- 所有 section offset/length/body length 保持显式 little-endian `u32`；索引、计数、entry、source position 用上述 ULEB。i32 常量保留标记后的 4 字节精确位型，避免负数 zigzag 新语义；bool 只能 0/1。

T-10 固定反例：operand `0/1/127/128/255/256/16383/16384/65535/65536/u32::MAX` 的期望字节分别按公式手算；非法 `0x80 0x00`、六字节序列、`ff ff ff ff 1f`、尾部截断必须受控拒绝。可编码数值域与可准入 ref 分开：65535/65536 可 round-trip 为整数，但越实际表长的 slot/type/const/block/function ref 由真实 `verify_mir` 拒绝。

## 3. 新 wire 身份与分区

- magic `AVAC`，version `1`，profile id `1` = `av.core-i32/1`；逻辑 schema `auto.abc.av-compact-core-i32/1`。与旧 `AVAB/2`、`AVAD/1|2` 不互相解释；旧 AVAB 输入在 compact decoder 以 magic 拒绝，compact 在旧 decoder 同样以 magic 拒绝。
- producer/query key 的正式登记归 T-02/T-05；集成时必须同时纳入 magic/version/profile/schema/producer/query key，旧 cache 只能确定性 miss/rebuild 或走旧 decoder，不能将 AVAC 填成 `DecodedAbc` 绕入口。
- 固定头 12 B：magic[4]、version:u16le、profile:u16le、section_count:u16le、reserved:u16le=0。
- 目录项 9 B：id:u8、offset:u32le、length:u32le。目录后 payload 必须连续覆盖到 EOF；offset+length checked，缺口、重叠、越界、尾随均拒绝。section 顺序按 offset，id 顺序不作为语义。
- required：`0x01 MODULE`（kind、entry、capabilities）、`0x02 TYPES`、`0x03 CONSTS`、`0x04 IMPORTS`、`0x05 FUNCS`；均必须且只能出现一次。未知 `<0x80` 拒绝。
- 可选 debug：`0x7f SOURCE`，最多一次，含每函数 snapshot 和 span 表；无该 section 时 snapshot=0/spans=[]。已知 `0x80..=0xff` 预留给可选 debug 扩展，本版本对未知 id 仅按已校验 length 跳过，绝不据此改变语义。所有 section 总次数仍受 4 MiB header 可表达界限。
- `FUNCS` 是 count + 每函数 `body_len:u32le + body`。body 只能精确消费其 slice；body 内为 sig、slots、entry block、blocks/instrs/terminator。函数名与 slot 名属 debug，本版本不携带；函数目录序号才是函数索引。owner/module identity 沿 D798 信封责任，decoder 重建 compact schema + 空 module owner，并为所有函数赋同一 owner；正式 adapter 重盖章后必须 verify。

## 4. core-i32 操作表

只接受 `I32/Bool/Fn` 类型、`I32/Bool` 常量、core imports/capabilities，以及：

| tag | 内容 |
|---|---|
| 0x01 CONST | dst, const-index |
| 0x02 COPY | dst, src |
| 0x03 BIN | sub-op 1..=11, dst, lhs, rhs |
| 0x04 UN | sub-op 1..=2, dst, operand |
| 0x05 CALL | callee-kind 0=Func/1=Import, callee-index, dst, argc, args... |
| 0x06 TRACE | event 1=MarkA/2=MarkB |
| 0x10 JUMP | target |
| 0x11 BRANCH | cond, then, else |
| 0x12 RETURN | value |

类型 tag：0=I32、1=Bool、2=Fn；Fn 为 argc、param type indices、result。常量 tag：0=I32 + 4B 位型、1=Bool + 1B。slot kind：0=Param、1=Local、2=Temp。import 为 UTF-8 symbol、effect 0/1、sig。所有其他 type/const/opcode/sub-op/kind/profile 均 unsupported/format 拒绝；data/owned 不静默按 scalar 解码。

不实现短立即数、slot 复用、指令融合或常量池重写；现有 const table 保留。短立即数若未来采用需新 wire version 和独立语义/密度反控，本计划不伪称已实现。

## 5. 预算与失败分类

- 直接复用已交付常量：输入 4 MiB；module 的 types+consts+imports+funcs ≤65536；每函数 slots+blocks+instrs ≤65536；decode work ≤2,000,000 steps/5s；decoded structural estimate ≤16 MiB。AV 的 1,000,000 steps/1024 frames/2s/8 MiB slots 和执行预算不因 wire 改变。
- 分配前检查 count 对应的 Vec/String/span 估计及累计 decoded budget；实际 `Vec::with_capacity` 仍以小上限起步，count 不能直接导致巨额预分配。decode 后 `verify_mir` 再核真实 `heap_estimate`、类型/CFG/init/signature/capability。
- offset/length/count 用 checked arithmetic。format（截断、重叠、非 canonical、未知 required/opcode）与 resource（input/table/work/wall/decode-memory）分态；semantic invalid/unsupported 由 verifier 保持其原分类。
- 32 MiB 受管分区、128 MiB worker 增量、loaded/raw/compressed/峰值/P95 只能在 T-02+ 的真实父侧测量，本独立壳不声称通过。

## 6. 固定测试与独立 oracle

命令：`cargo test --manifest-path experimental/ac-av-probe/parallel/826/Cargo.toml`，必须 selected>0 且无 ignored。测试内永久样例即 `av_wire_826_*`。

| 计划 case | T-10 子集与断言 | 后续边界 |
|---|---|---|
| V-01 | 完整真实 MIR（direct call、import trace、短路 CFG）encode/decode/verify，对照各表/指令/kind/entry/capability | 42/trace/LoadedProgram/AV/store/new-worker 待 T-02/05 |
| V-02 | 手工字节：`BIN [03,01,dst,lhs,rhs]`、`COPY [02,dst,src]`、`RETURN [12,value]`；0/1/127 各 ≤5/3/2B；raw 总长=头+目录+sections | T-03 生产接入复核 |
| V-03 | 上述整数域手算 oracle；非法 ULEB；真实 verifier 拒绝越界 ref，表族超额为 resource | 完整可准入大表与组合由 T-03/04 补 |
| V-04 | 实际 bytes 变异：截断、尾随、重复/缺 section、offset 越界/重叠、未知 required、未知 optional 合法跳过、未知 opcode | loader/dispatch=0 待 T-02/04 |
| V-05 | 有/无 SOURCE 的语义一致；非法类型/CFG 仍被真实 verifier 拒绝；decoder 不返回凭证 | strip 后执行与来源重绑定待后续 |
| V-07 | 超输入/表/decode 预算在分配前拒绝；合法对照通过 | 父侧峰值、cancel/fuel 后续 |
| V-11 | AVAB/AVAD/未知 profile 拒绝，core kind 不由函数数猜测 | data/owned 既有路由和正式版本矩阵后续 |
| V-13 | 同一 `compact_abc.rs` 被独立壳路径编译；round-trip/目录/边界/损坏/预算全有实际断言 | 生产 adapter 同源 hash、cache/AV/restore 待 T-02/03/05 |

固定语义 corpus：单函数 42；双函数 direct-call；带 lazy 分支的三块 CFG；带 mark_a/mark_b imports 和 capability 的 trace 模块；unit-import 模块；带 SOURCE 与 strip 两组。T-10 的 corpus identity 由测试源码和最终 commit 绑定；T-06 必须另冻结 825 共同负载、大量小函数/短常量、大索引、有/无 debug 的原始数据文件身份，不能用本小 corpus 替代正式 bench。

## 7. 后续 V/AC/SD 与测量清单

- T-02：V-01，完整 42/trace、新版路由、verify→LoadedProgram→AV、persist/new-worker；first scope。
- T-03/04：V-02/03/04/05/07 与 V-11 的 codec/security 全矩阵；targeted scope。
- T-05：V-06/07/09 和 V-11 的 cache/current grants/data-owned compat；compat scope。
- T-06：V-08/09/10 与 V-11 的计量/父侧反控；bench 与 acceptance 汇总 V-01..13，0 例、skip、缺必需子例均非 PASS。
- T-09：V-12 四子例，828 基线 273575B/115 函数至少 30 修订、新能力局部更新、依赖/授权变化、强制 whole 退化反控。
- AC-01..06 由上述工作阶段分别关闭；AC-10 的模块轴本轮只关闭 T-10 子集，完整 V-13 需生产同源与恢复证据；AC-07/08 仍 review/merge-owned。SD-01..07 均保持 proposed，不在本轮发布 canonical/ledger。

正式测量固定四入口：AV 参考/新执行路径、固定 Ignition 的 V8 JS、V8 Wasm baseline、V8 Wasm optimized。每组独立源码/二进制身份、等价结果、raw/code/metadata/debug、同算法同参数 compressed、loaded、encode/decode/verify/load、cold/warm、编辑 P95 和真实峰值；工具版本/tier/flags 必须实测证明，缺失即 gap，不填 0/skip 冒充通过。工具准备与首次构建另计时，不新增安装授权。WASM/V8 对标执行及 AutoDown 反控不属于 T-10。

## 8. T-01 决策闭环

格式、支持子集、源码路径/owner、公开 API、预算、测试 oracle、集成门均有唯一归属；当前无待裁定问题。T-10 只能实现本合同；若最小接缝不能编译或公开接口不足，记录具体符号/输入反例并回 new，不复制 WIP 或降低验证/资源门。

## 9. T-10 定向验证记录

在唯一实施树运行 `cargo test --manifest-path experimental/ac-av-probe/parallel/826/Cargo.toml`：**9 passed / 0 failed / 0 ignored**；`cargo fmt --manifest-path experimental/ac-av-probe/parallel/826/Cargo.toml -- --check` 与 `git diff --check` 均通过，编译无警告。未运行整档 `cargo t/tv/tt/tb/tu/taa`。

- 候选边界 corpus 的 operand 总长：ULEB **25 B**，窄+Wide **39 B**；最小单函数 42 模块的 ULEB raw 总长 **89 B**，分解为固定头/目录 57 B、MODULE 3 B、TYPES 5 B、CONSTS 6 B、IMPORTS 1 B、FUNCS 17 B。小索引 BIN/COPY/RETURN 分别精确为 5/3/2 B。
- direct-call + 算术模块、三块 CFG + 两个 trace event + capability、unit 的 `fn:` import、SOURCE 与 strip 两组均经真实 decode/encode 往返及 `verify_mir`；大索引 256 的 slot/type 不截断。
- 65535/65536/u32::MAX 使用手算字节 oracle；非最短/溢出/截断 ULEB 拒绝。slot 65536 和非法控制目标能 decode 为未授信 MIR，但真实 verifier 分别以 `mir.slot-ref`/`mir.jump-target` 拒绝。
- 截断、总输入尾随、section 重复/缺失/重叠/未知 required、函数 section 与 body 尾随、未知 opcode 均受控拒绝；未知 optional debug 按长度跳过且合法模块仍 verify。
- 超 4 MiB 输入、65537 module entries 和小 wire 大结构展开均在对应分配前以 resource 拒绝。该记录只覆盖 V-13 的 T-10 模块子集，不声称 V-01 全路径、V-08 测量、生产 adapter/cache/restore 或整 Plan AC-10 完成。

## 10. T-02 集成刷新记录（2026-10-28）

D825 已交付合入（master `9eb9ebca3`，LoadedProgram/loaded_program.rs 可用），D798/D828 此前已归档。
本树同步 master（merge commit `9c7c71a1b`，无冲突），核心/注册窗口空闲（825 随合并释放，843 触面在
crates/auto-lang-core 重构域、与本格式无交集），本计划按 §5 并行合同接受核心窗口并登记：

- **基面**：plan-826-dev `9c7c71a1b`（= e417247a5 基础上 T-10 提交 8190551fd + master 同步）。
- **接口刷新（PLAN-823 r8 env 合同）**：`MirFunction::env`/`MirImport::env: Option<Vec<TypeIdx>>`
  为 T-10 写作基面后合入。compact 编码器对携带 env 布局的函数/导入以
  `compact.encode.profile` 拒绝（标量子集外，与旧 AVAB 编码器对 core profile 的同型拒绝一致）；
  解码侧 `env: None`。同源壳 9/9 复跑绿。
- **正式路由（核心窗口）**：`core/src/compact_abc.rs` 注册入 `core/lib.rs`（`pub mod` + re-export）；
  `abc_verify::{verify_compact_abc, verify_compact_abc_bytes}` 为 AVAC/1 的唯一 LoadedModule 准入入口
  （owner 重盖章 + verify_mir_with_deadline + executable-closure 凭证，共享 `executable_closure` 助手；
  profile 1 = 同一 `av.core-i32/1` 语义，仅 wire 世代不同）。
- **producer/query key 登记**：A09 键新增 `abc-codec` 维度取值 `auto.abc.av-compact-core-i32/1`
  （`abc_unit_key_with_codec`）；manifest 新增 `abc_codec` 字段（serde 默认 = 旧 AVAB schema，旧
  manifest 恢复行为不变；不入 fingerprint——codec 差异已由 payload_fp 区分）。两世代键空间不相交，
  跨世代为确定性 miss/rebuild（O(1) 世代闸门按策略判 magic+version）。
- **driver/worker**：`probe/src/wire.rs` WireCodec 策略（LegacyAbc 默认/CompactAbc opt-in
  `set_wire_compact`）；driver 的 A09 发射/命中/闭包准入与 restore 的单元/闭包解码准入均按策略路由；
  compact wire 不携带函数名（合同 §3），单元名自 A09 信封/manifest link 重绑定
  （`rebind_unit_name`；legacy 保留 payload 名 vs 声明名校验）。worker binary 接受
  `ac-av/compact-run-v1`。
- **正式 recipe**：`AV_COMPACT_RECIPE = "ac-av/compact-run-v1"` 注册于
  `auto-lang-core/test_cases/ac_av.rs`（RecipeFamily::Compact，核心版本轴 = EXPECTED_AV_CORE_VERSION，
  工具身份轴 `av-wire-codec=auto.abc.av-compact-core-i32/1`）与 `auto-test-case/contract.rs`
  （REGISTERED_PROBE_RECIPES + 步骤词汇/av binding）。父侧 `judge_compact_runtime` 以
  `abc_wire_compact_units`/`abc_wire_compact_closure`/loaded 计数器判真（AV826_JUDGE_SABOTAGE 反控）。
- **组内兄弟树**：按组解析序补 `D:/autostack/.wt/lang-826/{auto-ui@bd228b0, auto-down@c3464d0}`
  只读固定版本（与 825/843 组同约定；根 workspace check 需要）。
- **首例验证**：`powershell -NoProfile -File scripts/verify-av-826.ps1 -Scope first` 退出 0：
  同源壳 9/9、probe 首例 5/5（往返/跨代隔离/cache-hit+新worker恢复/legacy共存+双键空间/unit不派发）、
  父侧 corpus 1/1 + 敏感性反控 1/1。回归：av_runtime_825_first_case 4/4、cache_tier 20/20、
  825 corpus runner 1/1。其余 scope（targeted/compat/bench/baseline/acceptance）显式未实现。

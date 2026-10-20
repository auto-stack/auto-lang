# PLAN-797 T-01 合同冻结件（contract-decision.md）

2026-10-16，work 阶段 T-01 冻结。本文件是计划 §8 T-01 的交付物：候选
profile 原型、语义 ref 快照身份、资源策略、工具链身份与性能配方的**首轮
冻结**。v0.1 仍是候选标签——按计划 §0，冻结态只在独立 review + merge 门禁
通过后登记，本文件不构成"ABI v0.1 已发布"。

证据基线：worktree `D:/autostack/.wt/lang-797/auto-lang`，分支
`plan-797-dev`，基 `5dcf4e3c2d5266cfa4ebf1444d6d7e154305d3c4`（master）。
T-01 测试实跑全绿（5 单元 + 8 集成 = 13/13），证据见 §7。

## 1. 域、target 与符号方案（冻结）

- **ABI 域**：`autoabi`（独立版本域，0.1 与 0.2 视为不同兼容域，禁止
  semver 自动推断兼容；RFC §3.1）。
- **候选 profile 标签**：`autoabi.win-x64.v0.1-candidate`。
- **首平台**：`x86_64-pc-windows-msvc`，Windows x64 调用约定
  （Cranelift `WindowsFastcall` / MSVC x64 默认）。
- **符号方案**：公开 AutoABI 符号 = C 标识符安全的 `<name>`（v0.1 刻意
  不引入 C 标识符规则之外的 mangling），逻辑路径→物理符号的映射由
  descriptor（T-02）承载并指纹化。probe 内部符号沿用 `p797_*` 前缀。
- **版本五元组分开存在**（RFC §3.2）：ABI 域、平台 target/profile、接口
  revision（descriptor 指纹）、runtime 服务表版本、语义 schema 版本。
  未知版本/错 target/错 packing/错符号/缺服务能力在**调用前**拒绝，
  native 被调次数为 0（T-02 validate 落地）。

## 2. 物理签名形态（冻结原型；T-02 补 descriptor 全量字段）

两种形态，descriptor 逐函数显式声明，不靠调用者猜：

- **pure 形态**：平台调用约定直传直返。参数/返回限于
  `{i32,u32,i64,u64,f64,bool(u8 仅 0/1),char(u32 合法标量值)}`；标量单返回。
  纯函数不强制 context（RFC §7.2）。
- **fallible 形态**：`i32 status f(params..., out*)`——显式 context 指针为
  首参（可 null 当且仅当 descriptor 声明 pure-ish），显式 out 指针尾随。
  status `0` = 成功；负值为受控失败码（码表自 `-1` 起分配，T-02 随
  descriptor 冻结；`-1..-16` 保留 probe 域）。**成功才提交 out；失败清理
  本次部分分配且 caller 旧输出不变**（T-04 落地验证）。
- **聚合/借用/拥有资源不按值跨界**：record 走显式 `const view / mut view /
  caller-owned out` 指针协议；buffer 走 `{i64 ptr, u64 len}` 描述对（字节
  长度，文本另标 UTF-8，`len=0` 允许 null ptr，非零长度要求有效对齐缓冲）；
  资源对象走 opaque handle（含可校验身份/代际位，位布局 T-02 冻结）。
- **callback（冻结支持面）**：仅同步 per-call、显式 ctx、不保存、不跨线
  程/代际；provider 不得在调用返回后再次进入。保存/异步/跨代际能力在
  v0.1 **不支持且受控拒绝**，拒绝不是 loader 能力承诺（T-04 落地）。
- **不支持面（v0.1 明确拒绝）**：按值聚合传参/返回、variadic、bitfield、
  任意 union、C++ ABI、异常/panic/unwind 穿界、跨线程 callback 保存、
  热重载/DLL 卸载事务、跨域自动转发。

## 3. 语义 ref 快照（临时 adapter，D-796 退役）

796 尚未合入（T-01 时点仍 drafting、无 schema 产物），本计划以**797 本地
只读快照**承载 fixtures 所需语义引用：

- 格式：`autoabi-semref-snapshot/0`，JSON，定义于
  `experimental/auto-abi-probe/src/semref.rs`（含 RecMix record 与
  p797_mix 函数签名两类 fixture 引用）。
- 内容指纹（blake3，64 hex）：
  `15f81fd02a19ff6b9b03b4f64d43d04d721fd524f3419b7cf3ae0f66926d32fa`
  ——进入 layout 缓存 key（sem-ref 分量）与后续对象缓存 key。
- **退役义务**：D-796 后 T-06 必须切换到 796 真实 semantic contract 解
  码/校验，删除本地重复模型，重跑全部三腿并留收据；不允许双权威模型共存
  （计划 §5.6 / parallel-contract）。

## 4. 类型布局规则（冻结；T-02 conformance 对照 C sizeof/offsetof）

- 声明序、自然对齐：字段 offset 向上取整到字段对齐；record 对齐 = 最大
  字段对齐；size 向上取整到 record 对齐。padding 不承载数据、不可比、不
  可序列化。
- 标量：bool=1B（仅 0/1，非法值受控拒绝）、u8=1B、i32/u32/char=4B
  （char 为 u32 合法 Unicode 标量值）、i64/u64/f64=8B。
- **nominal 身份**：不同名义类型即使布局相同不自动互换；canonical 形式
  含 record 名与字段序（`rec RecMix { bool flag; i32 count; ... }`）。
- 布局计算全程 checked 算术；越界/溢出是受控拒绝，不 wrap。
- 尺寸/对齐的独立 oracle 是 C 侧 `sizeof`/`_Alignof`/`offsetof`（T-02 的
  独立审定 header + C 编译产物），**不**由本侧同一生成器自证。

## 5. 工具链身份（本机冻结值 + 发现规则）

发现规则（代码 `src/ctool.rs`，不做任何自动下载/安装）：

1. env `AUTOABI_MSVC_CL`（必须存在，否则受控错误）；
2. vswhere 固定路径 `-latest -products * -requires
   Microsoft.VisualStudio.Component.VC.Tools.x86.x64` → 最高
   `VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe`；
3. 都没有 → **infra 阻断**（`abi.ctool-missing`），相关腿止步，禁止以
   Rust repr(C) 伪通过。

本机冻结身份（T-01 实测）：

| 项 | 值 |
|---|---|
| cl.exe | `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\cl.exe` |
| banner | 用于 x64 的 Microsoft (R) C/C++ 优化编译器 19.44.35222 版 |
| cl.exe blake3 | `443ac31500a4c6b9594db749c3d7a313ea6d86abcdd1e2710a4c9044dad2e62b` |
| C 编译 flags | `-c -nologo -GS- -Zl -W4 -WX -O2`（freestanding：无 CRT 初始化、无栈 cookie、无默认库指令、警告即错误） |
| C 编译 deadline | 60s（contained executor） |
| 链接 | ac-core `find_rust_lld`（rust-lld，flavor link）+ `find_sdk_um_dir`（本机 SDK 10.0.26100.0，um/x64 kernel32.lib） |
| `_fltused` | 含 FP 字面量的 C 翻译单元自带定义（每链至多一个 C 对象，实测不冲突） |
| 指纹串 | `cl:<path>|ver:<banner>|hash:<blake3>|flags:<flags>`（后续对象缓存 key 的 toolchain 分量） |

## 6. 资源与预算策略（T-01 冻结字段；T-05 全量实施）

- 每 probe 请求：build ≤60s、run ≤5s、total ≤70s（阶段和输出保留，
  不截断成功、不加预算换绿）。
- 输出：stdout/stderr 各 ≤1MiB；IPC 元数据 ≤4MiB；单实际产物 ≤64MiB、
  每请求合计 ≤128MiB。
- 受管 cache：热点数据 ≤8MiB；布局/查询暂存+索引另 ≤8MiB（783 store
  staging 预算本就 8MiB，一致）。
- 布局深链 ≥4096 字段与坏图走全路径资源/拒绝控制；超预算 =
  **resource 状态**，与 ABI 不兼容拒绝分开报告。
- 执行缺口声明：master 的 `run_deadlined_split` 只保证
  deadline/contained/全量收集，**不**强制输出字节上限——T-01 由探针
  harness 在运行后按 ≤1MiB 验收（超限即失败，见 `src/run.rs`）；T-05
  按本表全量实施并复核缺口。
- native 子进程内存另测，不冒称 8MiB cache 预算覆盖 C 编译器。

## 7. T-01 最小三腿探针与首个 layout 缓存（实测证据）

入口：`experimental/auto-abi-probe/tests/t01_feasibility.rs` +
crate 单测；命令 `cargo test`（workspace 目录）；结果 **13/13 绿**
（2026-10-16 实跑）。

| 测试 | 腿/内容 | 结果 |
|---|---|---|
| `t01_oracle_matches_hand_reference` | Rust oracle 与手算基准 2000001876 一致；输入置换可区分 | ok |
| `t01_leg_a_autoabi_to_autoabi` | 两个独立 ObjectModule 发射的 caller/callee obj → 真 PE → exit 0 | ok |
| `t01_leg_a_negative_expected_control` | 错误期望 → exit 1（比较可区分，非空洞通过） | ok |
| `t01_leg_b_autoabi_calls_c` | Cranelift caller + cl.exe C callee obj → exit 0 | ok |
| `t01_leg_c_c_calls_autoabi` | cl.exe C caller（自带 entry+oracle）+ Cranelift callee → exit 0 | ok |
| `t01_layout_cache_first_persist` | 783 store ABI-L0 层：fresh miss → put → 重开实例 → hit 且 payload 全等；key 确定性 | ok |
| `t01_ctool_fingerprint_freezes` | cl 指纹解析+进程内稳定 | ok |
| `t01_semref_snapshot_identity` / `snapshot_wellformed` / `snapshot_hash_stable` | 快照良构+hash 稳定 | ok |
| `rec_mix_layout_matches_hand_derivation` / `canonical_record_is_nominal` | 布局手推 oracle（off 0/4/8/16,size 24,align 8）+ nominal/字段序区分 | ok |

探针函数 `p797_mix(i32,i32,i32,i32,i64,f64,i32,f64)->i64`（8 参数跨
RCX/RDX/R8/R9+栈+XMM0/XMM1；每个参数独占权重 2/1/3/7/11/13/1000/17，
任何参数错位必改结果）。三份**相互独立**实现：Rust oracle
（`src/decl.rs`）、Cranelift body（`src/emit.rs`）、手写 C
（`cside/{callee_mix.c,caller_main.c}`）；期望值由独立 Rust oracle 烘焙进
PE，leg C 由 C 侧独立 oracle 当场计算。

**声明边界**：发射源是本计划受限声明 IR（`src/decl.rs`），不经 Auto 源码
parser；三腿是 probe 证据，不宣称 Auto 源码编译器已支持任何 ABI 类型。

## 8. 缓存首例约定（T-01 生效；T-05 扩展）

- 复用 783 `ManagedStore`，probe 自有层 `ABI-L0`（target-layout，
  schema `auto797/abi-l0-layout/v1`），LayerSpec 常驻本 crate——
  **不改** 783/787 的 LAYERS 注册或 GC。
- key 分量：`sem-ref`（快照指纹）+ `record`（canonical 形式，含名义）+
  `target` + `abi-profile`。任一输入变化即新 key（真失效）；命中从不跳
  过调用方校验、从不缓存 PASS 结论。
- 对象/manifest 层（`ABI-O*`）T-03/T-05 按同约定追加：body hash、
  toolchain 指纹（§5）、flags、runtime/服务表入 key。

## 9. 性能配方（冻结；T-05/T-07 执行）

- 参考机：本机（Windows 11 x64；CPU/OS 明细 T-05 采样时随报告记录）。
- 六场景各 30 样本：fresh、layout 热点、对象热点、重启磁盘热、
  body-only edit、ABI/layout edit（后者加 GC 并发资源观察）。
- **硬门**（P95，含本次必要校验/cache/emit/link/IPC 与发布，不扣真实用
  户等待）：代表单函数 AutoABI producer body edit、复用不变 C consumer
  的"请求→正确 PE 执行结果" < 1000ms；只改 descriptor 后应拒绝的请求→
  正确诊断 < 1000ms。全冷 C 工具构建单列报告，不冒称亚秒。
- 阶段 P50/P95/P99、peak memory/bytes、错误样本全保留；阶段 P95 不相加
  成总 P95；不按结果回调总阈值。
- D-796 后经正式 AutoDown 通道复测（T-06）。

## 10. 与 774 内部 ABI 的分域声明

774 的 ABI-A–E（`probes/data-resources`，Cranelift caller→显式 ABI Rust
shim）是 770/771/782 的**内部**冻结面；本计划不改其接受面/资源表示，
`autoabi` 域与其**无兼容关系**，任何"两者可互换"的说法都是错误（SD-05
落地时在 spec 层重申）。

## 11. T-01 未尽事项（后续任务入口，非缺口）

- descriptor/validate/admission 全量字段与状态码表 → T-02。
- record/buffer/handle/callback 三腿扩展与嵌套调用 → T-03/T-04。
- 六场景周转/512 churn/trace → T-05；正式 AutoDown 注册 → T-06（等
  D-796/D-788）；freeze proposal/支持矩阵 → T-07。

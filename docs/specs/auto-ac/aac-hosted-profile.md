# auto-ac: AAC hosted 标准库消费 profile（PLAN-1045 交付稿）

> **Status**: PLAN-1045 work 交付稿（review 通过后由 merge 正式沉淀；本轮不宣称 reviewed）。
> 本 profile 描述 AAC（Auto 编译器主体）在 Windows x64 hosted+alloc 目标下**实际消费**的
> 标准库/宿主接口面：已验证行为、所有权、执行端与支持边界。逐行 CS/CL 分账以
> `autolang.aac-baseline/1`（PLAN-1044 TSV + 校验器）为权威；本文件不新增公共 `std` 操作，
> 不改变 MCW core/alloc 模块的可选性。

## 1. 为什么需要本 profile

Design59 D-S01..D-S04/D-S07 给出 AAC 启动所需的库操作族；1044 冻结 CS-201..208 行与资源
预算。本包把这些行**落到真实消费**：以 Gen0（Rust 前端）编译的 Auto 客户端在真实执行端
（AV executor + 真 Win32PlatformIo、AC/mn 原生机器码、A2R 真 rustc 可执行）验证每个操作族，
并修复沿默认 provider 的真实缺口（§5）。

## 2. 包与所有权

| 轴 | 值 | 说明 |
|---|---|---|
| 应用层客户端 | `experimental/ace/aac/host/{host_client,host_files,host_publish,host_diag,host_main}.at` | AAC 应用层 hosted 需求客户端，非公共标准库面 |
| 执行证据 | `experimental/ace/tests/aac-host/`（固定组 `ace-aac-host`，nextest-full 组内 4 线程） | AV 全串 oracle + mn 原生折减 oracle + sys/A2R 真进程 oracle |
| 消费的支撑包 | `aac-support` 1.0.0（PLAN-1056：digest/codec/atom_data） | 摘要/身份分帧单源，本包不另建第二份实现 |
| 公共 std 面 | `stdlib/ace/asm/*`（聚合集）+ `stdlib/ace/sys/*`（sys 集，973/1039/1040/1041/1065） | 冻结身份；本包仅修 §5 所列缺口，无 API 新增 |
| 语言面 | `packages-aggregates-ct`（+sys 装配）；A2R 端真 rustc 消费 | 前端预算 `FrontBudget::WORK_OWNED_64MIB`（1044 冻结值） |

## 3. 已验证的 hosted 消费面（对 1044 CS 行）

| CS 行 | 客户端入口 | 已验证行为（独立 oracle） |
|---|---|---|
| CS-201 | `host_client.bytes_span_report/intern_report`；`host_files.nul_path_report` | 嵌零 bytes 保长（"a\0b"=3 字节 97,0,98）；α 字节 span [0,2)；intern 首见索引+按内容去重；**OS 字符串内 NUL 具名拒绝（-9）且拒绝≠截断**（真实前缀文件 "a" 反证） |
| CS-202 | `host_client.map_report/sort_report/int_report` | Str→int 映射末写胜+恰存项+字节字典序输出（V-04：a→1,b→2,a→3 ⇒ [(a,3),(b,2)]）；确定性选择排序（Rust `Ord` 同序）；wrap32/FNV 纪律；checked-add 溢出为独立分类 |
| CS-203 | `host_client.budget_report` | arena 树遍历显式步数预算：预算内 ok / 超限具名资源拒绝（非部分静默结果） |
| CS-204 | `host_files.*`；`host_publish.*` | 非ASCII路径模块读取（内容嵌零=内容，BLAKE3 对账）；目录/模块定位/空目录；env 往返+unset-空区分；真 cmd.exe 子进程（退出码/超时击杀 -11/缺工具 -1）；**rust-lld 发现算法 Auto 复刻与 Rust 算法逐字节一致**；发布=临时+原子替换，注入两型 rename 失败后**旧工件 hash 逐字节保持**，重试成功且 temp 清除 |
| CS-207 | `host_diag.*` | 同一真可执行从两个 cwd 调用：α 源 span 稳定（[0,2) 字节级自核验）、cwd 往返+恢复、OS 路径 NUL 拒绝、缺工具、同源重试全部不变，仅起始 cwd 不同；语言类型错误于编译阶段拒绝（阶段可区分）。**argv 值子腿具名阻塞于 D-1045-01**（A2R argv 读取经本模块形状非确定损坏——内容与长度均曾红；会话内探针曾全量 equal() 绿+码点逐位正确，证据在 PLAN-1045 §9；关闭 oracle 即 `host_diag.at` 的 `args_tail_ok`/`argv_shape`） |
| CS-205/209（消费） | `host_client.digest_report` | aac-support 的 sha256/blake3 于 AV 与 mn 原生两端逐字节对上 sha2/blake3 crate |

## 4. 执行端与观察通道（冻结事实）

- **AV+Win32PlatformIo**：sys 集程序在 AV 执行器上以真 Win32 后端运行（真文件/真子进程）；
  观察通道为返回值全串。
- **AC/mn 原生**：`MnDriver::build_collections_checked` 把聚合 lane 的 checked 产物经共享
  MIR 降为逐函数 COFF、真链接 PE 并运行；观察通道为 exit 折减码（对**独立期望串**重算的
  FNV-1a，配区分度负控；本包实例=1355556778）。sys 集在 mn 的 console lane
  （sys973/1039/1065 Win32 实装）为既有 owner 入口，本包不宣称 packages-sys 原生 lane。
- **A2R 真可执行**：真 rustc 消费 A2R 产物编译为 exe，以受控 argv/cwd 作为真子进程运行；
  **进程 exit 是 runtime 自身成功码，非 main 返回值**——观察通道为报告文件（PLAN-1064 纪律）。

## 5. 沿默认 provider 修复的真实缺口（共享实现，单写合入）

1. `crates/native/src/collections_lower.rs`：typed **I64 And/Or/Xor** 与 typed **I32 位运算**
   发射（PLAN-972 具名后继切片，此前无主认领；AAC 原生摘要消费必需——digest 算法为 xor 密集
   且 `digest.at` 的 i32 chunk-flag 折叠产生 I32 Or）。Shl/Shr/narrow 算术/float 位运算保持
   具名拒绝。
2. `crates/collections-runtime/src/lib.rs` `mn_coll_str_concat`：空⊕空拼接此前把
   `try_alloc(0)` 的合法空表示（null+0）误读为 RESOURCE 拒绝 → trap 70；现返回正确空 TEXT
   节点（owned lane 的 negative_guard/def_pair 同约定）。
3. `stdlib/ace/sys/sysplat_win.at` `put_scalar`：OS 字符串内 NUL 具名拒绝（-9），不再静默
   截断于 Win32 NUL 终止 UTF-16 边界；`stage_inner` 传播具名码。

## 6. 具名边界与后继

- Str 键 Map/泛型容器仍属 PLAN-963 全泛型切片（本包客户端以平行 arena 表达，std.map 保持
  int 键具名边界）。
- mn console lane 的 sys 平台 op 实装归 PLAN-973/1065/1063 owner；本包未新增 packages-sys
  原生 lane。
- A2R 端 argv 读取经本模块形状非确定损坏（内容与长度均曾红）：具名债务 D-1045-01
  （KNOWN-DEBT-AND-RISKS），owner=a2x-rust runtime；V-05 argv 值子腿具名阻塞，
  不交付弱化 fallback 绑定。
- Linux host 运行留 PLAN-1063；`cargo t ace-perf` 不承载本包用例（无 `::performance::` 行）。

## 7. 验证（真实命令与结果）

固定组 `ace-aac-host`：**22/22 绿**（conformance 10 + host_files 8 + host_sdk 2 +
host_diag 2；单例最大 v05_host_diag_scenario≈14s，一次冷 rustc 构建主导——外部工具成本
具名记录并列为用户裁决项，见 PLAN-1045 §10）。共享改动触及的完整
组在同一冻结工作树复跑：`ace-aac`（26 例）、`ace-aac-host`（21）、`ace-source`、`ace-native`
全绿（结果 JSON 在 plan-artifacts scope 摘要）。`ace-abi` 未选：本包无 ABI 面改动（选择理由
入 PLAN-1045 §8 T-06 行，review 核）。

## 8. 与后继的交接

- **1068**：本包 host 客户端/SDK 消费收据可作 AAC 实证输入；V-01..V-05 独立 oracle 全绿。
- **1046/1047**：CS-101..107 的语法/语义主体不在本包（本包仅库消费面）。
- **1052**：工件/恢复消费 digest/codec 时沿 `aac-support` 单源（1056 §9 既有交接不变）。

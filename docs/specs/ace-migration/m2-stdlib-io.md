# ACE M2 系统库（File/path/env/process/time/rand/console）——规范与证据边界

> Status: verified Windows (PLAN-973 + PLAN-1065 + [PLAN-1064](../../plans/archive/1064-ace-stdlib-byte-str-deferred-surfaces.md) M2-A 交付)。
> 本文件规范 973/1064/1065 已交付的系统资源库合同：能力面、平台操作冻结、六路径覆盖、资源账本、
> 默认自有实现边界、拒绝语义与已裁定移交缺口。实现状态以归档计划收据与固定组为准；
> 未列能力不代表支持。

## 1. 能力面与公开合同

- 模块集：`stdlib/ace/sys/` 11 模块（file/path/env/process/time/rand/storage/zip/syscommon/sysplat_win/sysplat_lnx），经 1043 来源装配（`#in sys is current` 按目标选平台模块）。
- 公开 API 保旧 sentinel 约定逐面：read_text/read_text_range/dir_names/env get 族/canonical/cwd+args+spawn_with_output 捕获文本/ACE-KV 存储/pure-Auto ZIP/read_bytes `?List<int>`（D-01..D-09）已由 [PLAN-1064](../../plans/archive/1064-ace-stdlib-byte-str-deferred-surfaces.md) 交付：read_text 出错 `""`（严格 UTF-8+NUL 保真）、read_text_range exact `058` 10 行 JSON envelope、write 0/-1、exists bool、size -1、`env.get` 未设 `""`/`env.get_or` 区分存在空串 `""` 与未设 `-203/-8`、`path.canonicalize` 缺失路径真报错 `-8`（移除 `GetFullPathNameW` 回退）、`process.args()` Win32 引号还原、`spawn_with_output` JSON envelope（64 KiB/流上限+失败/超限 kill+release）、`std.storage` `ACE-KV/1` `.acekv.tmp` 原子替换（上限 4096 条/256 KiB）、`std.zip` 纯 Auto `Stored`+`Deflated`(BTYPE 00/01/10)+IEEE CRC-32+流式 `max_bytes`、超时杀全树返回 -11、exit=受控 VM 终止（AV：`ExecOutcome::Exit`，永杀宿主为否）。
- 目录/权限/替换：create_dir/remove_dir/rename/replace_atomic/copy_file/is_read_only/mtime_ms 真实 OS 语义（Windows 全绿；Linux 代码臂 NOT_RUN→1063）。
- watch：旧公开 API 无此面（调研记录，无义务）。
- 旧 `random.rand_int`/`Random`（Plan 250 非凭据 PRNG）不移植——熵面以 `entropy.rand_fill`/`secure_hex` 承接（973 §10 具名裁定）。

## 2. 平台操作冻结（四表对齐 50/50/50/51）

50 个 `i32→i32` 冻结操作（io_stdin×1、file×11、path/dir/canonical/replace/attrs×17、env×4、clock×4、entropy×1、proc×12），64 位值（size/offset/now_ms/mtime）以 lo+hi 两 i32 组合，配 `file_hi`/`clock_hi` 高半查询。路径协议：Windows 收 UTF-16LE units region，Linux 收原始 bytes view；UTF-8↔UTF-16 转换在共享 Auto 源内完成。四表对齐点：

| 表 | 位置 |
|---|---|
| source 冻结表 | `experimental/ace/crates/core/src/source/platform_io.rs`（`PLATFORM_SOURCE_OPS` 973 段） |
| av-core 运行表 | `experimental/ace/crates/av-core/src/platform_io.rs`（`PlatformOpSpec` 973 段） |
| AC native shim | `experimental/ace/crates/native/src/platform_io.rs`（`mn_pio_*` COFF 符号 + `SYS973_KINDS`） |
| A2R/A2PY 运行时端口 | `a2x-rust/src/rt.rs`（50 个 `a2x_*`）/ `a2x-python/src/ports.rs`（51 含 `mem.read_u8` 辅助） |

命名派生自共享 source 表（a2x 侧结构性不可漂移）。新增操作须四处同步并对账。

## 3. 默认自有实现与 FFI 分账（SD-NATIVE）

默认系统 provider：Windows 直声明 kernel32/bcrypt 公开 DLL 入口（`extern "system"`，CreateFileW/ReadFile/…/BCryptGenRandom），Linux 原始 syscall（code-only，1063 认证）；**无 libc/Rust std::fs/std::io 默认委托、无旧 VM 桥**。A2R 目标运行时（Rust std backed）与 A2PY（Python 运行时）为目标侧显式依赖分账，不构成默认 provider 违约。独立 C/Rust 库经 967 受检帧显式选用（AV×C/AV×Rust 外库腿覆盖写路径族 dispatch；读路径出参帧=967 域具名缺口）。

## 4. 资源账本与拒绝语义

统一资源账本（`PlatformResourceLedger`）：文件/目录/进程/socket/listener 载荷、代际 handle、读写字节预算（65_536/call、1_048_576/run、handle 1024 沿 1039 冻结值）、cleanup 恰好关闭一次（net 载荷按 master 语义仅显式 `net_tcp_close` 释放）。拒绝诚实分态：不支持=具名错误码（Windows：36 个 W 形已为真身，14 个 bytes 形 N 具名 `-10`，Linux 全 stub→1063）、越权/过期 handle/重复 close/非法编码分态，拒绝时 OS 副作用计 0；无静默 fallback、无 link hole。

## 5. 车道门与装配（SD-LOAD）

sys 模块经 packages-aggregates 装配集冻结：`stdlib_sys_files()` + `is_frozen_std_sys_module()` + `assemble_stdlib_sys_inputs`（binding 双点身份）。车道门：resolve/verify（Aggregates/Closures 平台准入）、lowering 内在路径、abc 编解码、host 平台导入准入、mir_verify 能力拼写 KNOWN 准入（运行账本为权威）、`ExecOutcome::Exit` 六消费点。`#in` 目标选择经 966 `#if/#is`（Windows/Linux 装配绑定已验，Linux 运行→1063）。

## 6. 覆盖矩阵（SD-COVERAGE，2026-11-16 实跑）

| 路径 | 状态 | 证据 |
|---|---|---|
| AV/native（Win32 真后端） | verified（含 PLAN-1064 D-01..D-09 + `bstr_1064`） | `stdlib_sys_973.rs` 15 例 + `bstr_1064` 7 例 + `kv_zip_1064` 4 AV 例 + `sys_d01_d06_1064` 3 AV 例 |
| AV×C / AV×Rust 外库 | verified（写帧 + PLAN-1064 读出参 `OutBytes` 帧） | `sys973_av_foreign.rs` 2 例 + `out_frame_1064` 3 例 (`ResKind::OutBytes` / `OutBufferFrame`) |
| AC（直接编译） | verified Windows（PLAN-1065 `36W+14N` + PLAN-1064 对齐 `canonical_w16` 缺失 `-8` / `env_get_w16` `SetLastError(0)`；`std.bytes` 编解码 `mn` 5 端 PASS；`D-01..D-09` `packages-aggregates`×`PlatformCall` 合流与 `OutBytes` native 机器调用归 PLAN-1068 T-03 具名 B-handover） | native `stdlib_sys_973.rs` 11/11（含 `sys973_win_canonical_and_env` 空串vs未设+缺失canonical、PE导入审计 `kernel32+SetLastError`+`bcrypt!BCryptGenRandom`、V-09窗口）+ `bstr_1064_v_codec_five_ends` `mn` 腿 |
| A2R（真 rustc+运行） | verified（含 PLAN-1064 D-01..D-09） | `sys973_a2r_real_rustc_leg` + `bstr_1064` (`v01_large_64k_a2r`/`v_codec`/`v_d09`) + `kv_zip_1064` (KV/ZIP 4096) + `sys_d01_d06_1064` |
| A2PY（真 OS Python） | verified（含 PLAN-1064 D-01..D-09） | `sys973_a2py_real_python_leg` + `bstr_1064` (`v_codec`/`v_d09`) + `kv_zip_1064` (KV/ZIP 4096) + `sys_d01_d06_1064` |
| 宿主 facade | verified | ace-host e0/t02/t08 11 例 |
| A2C | 按原适用域不减（`bstr_1064_v_codec_five_ends` + `stdlib_asm_972`/`v08` C 腿全 PASS；完整 D-01..D-09 对齐为增强项） | — |

测试车道：`cargo nextest -p ace-av --test conformance stdlib_sys_973` 等；单例 ≤10s（冷缓存首跑超限按环境归因+热复跑记录）。

## 7. 已裁定移交缺口（不冒已交付）

| 缺口 | owner | 裁定 |
|---|---|---|
| ~~byte→Str 面：read_text/dir_names/env get 族/canonical/cwd/cmdline/捕获文本/存储重建/zip inflate/KV storage/read_bytes Option\<List\> 形状（D-01..D-09）~~ **已交付**（AV/A2R/A2Py 完整 + AC `std.bytes` 5 端 + AC `sys973` 原语对齐；`packages-aggregates`×`PlatformCall` `AC` 合流与 `OutBytes` AC 调用移交下表 PLAN-1068 T-03） | [PLAN-1064](../../plans/archive/1064-ace-stdlib-byte-str-deferred-surfaces.md) | 2026-11-17 交付 |
| M2-B AC 三端最终补齐：`packages-aggregates` 下 `std.{file,path,env,process,storage,zip}` 与 `mn_pio_*` `PlatformCall` 合流 + `ResKind::OutBytes` AC 机器调用（`AC-AC-STAGED` 具名 B-handover） | [PLAN-1068](../../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md) (`T-03`) | 2026-11-16 r3 |
| ~~AC 50 操作真身+V-09 AC 正例腿~~ **已交付**（Windows 36W+14N+V-09；Linux 腿仍属 1063） | [PLAN-1065](../../plans/archive/1065-ace-ac-sys-real-bodies.md) | 2026-11-16 交付 |
| Linux 全部实跑认证（L-01..L-12 + 1064 `sysplat_lnx.at` 增补臂，含 env set/remove、spawn 子序、canonical 符号链接） | [PLAN-1063](../../plans/1063-ace-stdlib-linux-deferred-validation.md) | 2026-11-14 裁定 |
| ~~967 读出参缓冲帧（AV×C/Rust 读路径 dispatch）~~ **AV+auto-bindgen 已交付**（`ResKind::OutBytes` + `OutBufferFrame`；AC native 侧归 PLAN-1068 T-03） | [PLAN-1064](../../plans/archive/1064-ace-stdlib-byte-str-deferred-surfaces.md) / 1068 | 2026-11-17 AV 闭合 |
| master 预存红 ace-abi×1/ace-runtime×19（rust-lld CRT 符号）/ace-a2x-c `runtime_admission_preserves_bytes_and_the_certification_pin` | 批量回归 triage | 非 973/1064 面 |

## 8. 来源

[PLAN-973](../../plans/archive/973-ace-m2-stdlib-io.md)（r13 final pass @0a6fb9100→rebased；归档收据含完整验证记录）、[Design55](../../design/55-ace-m2-complete-language-and-product-replacement.md)、[Design56](../../design/56-ace-m2-ffi-and-native-stdlib.md)、[平台 I/O 设计](../../design/stdlib/native-platform-io.md)、[覆盖设计](../../design/stdlib/backend-provider-coverage.md)。

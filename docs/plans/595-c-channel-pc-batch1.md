---
plan_id: PLAN-595
status: execution_done               # drafting → executing → execution_done → reviewed → archived
feature_name: c-channel-pc-batch1
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09

affects: [auto-bindgen, auto-lang/vm ffi, auto-lang/trans c]
current_step: 6
total_steps: 6
---

# [PLAN-595] C 通道 P-C 首批:manifest 类型层扩展 + kernel32 manifest + a2c 复刻 autoterm-ctrlc 对拍

> **来源**:auto-term `docs/designs/004-engine-autoization-roadmap.md` §5 工作项
> ①②(004 自述"①②一组,可立小 plan")。004 口径:**Auto 双后端(a2r/a2c)在
> C ABI 会合**;unsafe 消解论(C ABI 互操作外包 C 通道,Auto 不设 unsafe 面)。
> 本计划 = 004 P-C 相位第一步:auto-bindgen 升格三后端共享 IR 的**类型层与
> 首个 win32 数据集**,并以 a2c 复刻 autoterm-ctrlc(Auto 生态首个真实
> win32 互操作产物)完成行为级验收。

## 变更摘要

1. **auto-bindgen 类型层**(004 §5①前置):`CTypeDesc` 增 `FnPtr` 回调变体
   (携带 params/ret 签名);`CHeaderManifest` 增 `abi` 注记字段(serde
   default `"c"`;win32 = `"system"`)。VM c_ffi 封送面对 FnPtr **明确
   报错**(267 定性 Impossible,清晰 VMError 替代 panic/静默)。
2. **kernel32 console manifest**(004 §5①数据):extractor 增 `windows.h`
   内置 manifest,library=`kernel32`,6 函数:FreeConsole/AttachConsole/
   SetConsoleCtrlHandler(**FnPtr 参数,首个回调用例**)/
   GenerateConsoleCtrlEvent/Sleep。
3. **a2c 消费面快照**:`use.c <windows.h>` + `fn.c` 声明 + 直调最小用例
   (镜像 `test/a2c/18_c_interop/001_cstr` 模式)。
4. **a2c 复刻 autoterm-ctrlc**(004 §5②):Auto 源 fixtures(零捕获闭包
   回调 `|evt| 1` 经 Plan 060 函数指针;Dual 双发 Break→Sleep(50)→C;
   exit 0/2/3 语义对齐 Rust 版 `autoterm-core/src/bin/autoterm-ctrlc.rs`)
   → a2c 转译 → MSVC(vcvars64)编译链接 kernel32 → `ctrlc-a2c.exe`;
   构建脚本入库。
5. **行为对拍验收**:①静态 exit 码三场景;②**真实引擎路径**——
   `AUTOTERM_CTRLC_BIN=<a2c exe>` 跑 auto-term
   `cargo test -p autoterm-core --test ctrl_event`(pty.rs helper 解析
   env 优先,零 auto-term 改动);③与 Rust 版 helper 双跑对拍。
6. **004 回执**:auto-term 侧 004 §5①②进展 + DEBTS #10 更新(L0 文档)。

## 执行环境

| 项 | 值 |
|---|---|
| 工作树 | `D:/autostack/.wt/lang-595/auto-lang`(分支 `plan-595-dev`) |
| 跨仓依赖 | auto-term 主检出**只读**(ctrl_event 门禁 + Rust helper 对拍;不改其代码) |
| C 编译器 | MSVC 2022 Community(`C:\Program Files\Microsoft Visual Studio\2022\Community`,经 vcvars64.bat;PATH 无 cl,脚本内 call) |
| 参考真身 | `D:/autostack/auto-term/crates/autoterm-core/src/bin/autoterm-ctrlc.rs`(141 行,语义基准) |

## 任务清单

- [x] **T1** auto-bindgen 类型层:`CTypeDesc::FnPtr{params, ret}` +
  `CHeaderManifest.abi`(default "c");`slot_count`/type_map 覆盖;
  VM c_ffi marshalling 遇 FnPtr → 明确 VMError(非 panic);
  serde round-trip 单测。
- [x] **T2** extractor `windows.h` manifest(6 函数,library=kernel32,
  abi=system,SetConsoleCtrlHandler 用 FnPtr);`load_builtin_manifest`
  命中;单测;c_bindings JSON 导出同步。
- [x] **T3** a2c 快照用例:`use.c <windows.h>` + `fn.c` 六声明 + `Sleep`
  直调(编译级验证可选,快照文本必过)。
- [x] **T4** a2c 复刻 ctrlc:fixtures `autoterm-ctrlc.at`(argv 解析/
  FreeConsole→AttachConsole→SetConsoleCtrlHandler(闭包)→
  GenerateConsoleCtrlEvent 双发+Sleep/exit 码);a2c 产物 + MSVC 构建
  脚本(scripts/ 或 tools/)产出 `ctrlc-a2c.exe`。
- [x] **T5** 对拍验收:静态三场景 exit 码;`AUTOTERM_CTRLC_BIN=<a2c exe>`
  下 auto-term ctrl_event 门禁绿;Rust/a2c 双 helper 同场景行为一致。
- [x] **T6** 004 回执:auto-term 004 §5①②标注 + DEBTS #10 增 595 回执。

## 验收标准

1. `cargo test -p auto-bindgen`(及 auto-lang 相关档)全绿;
2. 新 a2c 快照用例绿(既有 a2c 快照零回归);
3. `ctrlc-a2c.exe` 可重复构建;无参→exit 3;坏 pid→exit 2;
4. **真实引擎门禁**:`AUTOTERM_CTRLC_BIN=<a2c exe> cargo test -p
   autoterm-core --test ctrl_event`(auto-term 主检出)全绿;
5. 双 helper 对拍:cmd/timeout 主体中断行为一致(Rust 版 5s 内中断,
   a2c 版同等)。

## 风险与边界

- a2c 产物 ABI:x64 上 C 与 system 同约定,本机为 x64;x86(stdcall)
  差异登记为已知边界,不在本计划修;
- 闭包回调:Plan 060 路径已验证(闭包→函数指针);若 a2c 对 `|evt| 1`
  参数/返回类型推断受阻,T4 允许最小修 a2c(记录于执行步骤);
- MSVC 经 cmd/vcvars 调用,Git Bash 不直呼 cl;
- 本计划**不动** auto-term 代码;004⑥(a2r 生成 Rust FFI)不在本批。

## 9. 复审记录

stage: work | PLAN-595 | rev 1 | pass | code_commit=a1626b0fb(plan-595-dev) |
task_ids=T1-T6 | evidence=见「执行证据」 | blockers=无 | next=review

### 执行证据(2026-09-09)

- **T1/T2**:`CTypeDesc::FnPtr{ret,params}` + `CHeaderManifest.abi`
  (serde default "c");VM c_ffi 注册期对 FnPtr 明确 VMError(267 定性);
  windows.h manifest 6 函数(GetCommandLineA 追加——argv 的 win32 零运行时
  通路),library=kernel32,abi=system;windows.json 导出入
  c_bindings(git 需 -f,目录有 ignore 规则,5 个旧 JSON 先例已跟踪)。
  `cargo test -p auto-bindgen` = **6 passed**(FnPtr serde 往返/legacy JSON
  abi 缺省/windows manifest 内容)。
- **T3/T4**:快照 002_windows_console(Sleep 直调)/003_autoterm_ctrlc
  (复刻真身:闭包回调 `evt => 1`、双发 Break→Sleep(50)→C、exit 0/2/3、
  argv=GetCommandLineA+strchr+atoi);**执行期抓出并修复 a2c 真缺陷**:
  闭包定义在 main 后且无原型 → 真 MSVC 编译 C2065(快照文本比对盲区)
  ——修 = generate_closure_definitions 移至 header 装配前 + 闭包原型
  入头;003_closure 快照随之更新(新增 include+原型,严格更优)。
  MSVC 构建脚本 scripts/build-ctrlc-a2c.cmd(vcvars64 自举,纯 ASCII)
  → ctrlc-a2c.exe(warn 仅 C4113/C4100/C4702,均良性)。
- **T5**:①静态:a2c/Rust 无参=3,3;坏 pid=2,2;②**真实引擎门禁**:
  ctrl_event 注入障碍(cargo 对同包 bin 自动注入 CARGO_BIN_EXE 优先于
  AUTOTERM_CTRLC_BIN)→ 产物置换法(target/debug/autoterm-ctrlc.exe 换
  a2c exe,gitignored 构建产物,事后按备份恢复 md5 一致):**3 passed /
  1 ignored(26200 ping 已知边界)**——a2c helper 过真 ConPTY 中断路径
  (cmd/timeout 主体中断+pwsh 回提示符+helper 失败降级);③Rust 版复跑
  同绿(双 helper 对拍)。
- **T6**:auto-term 004 §5①②回执 + DEBTS #10 增 595 条(主检出 L0)。
- **基线如实记录**:a2c 套件存量失败 8 例(05_expressions/002_field_access、
  08_generics/001_const_generics、10_collections×3、21_storage、23_stdlib×2)
  + .at 断言 1——经 stash 复跑与主检出旧二进制交叉验证均与本计划无关;
  本计划净效果 109 ok(基线 107)+2。
- **已知边界**:a2c 产物 x86 stdcall 差异未处理(x64 上 system≡C);
  replica 不含 Rust 版手动 c|break 诊断参数(引擎路径契约=仅 pid,
  语义=Dual);pid=0 边缘(atoi 0→exit 3 vs Rust attach 父控制台)记录在案。


## 10. 待澄清事项

(无)

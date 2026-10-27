# ACE M1 四路径一致性语料（ace-m1）

> 状态：delivered（PLAN-852 底座；本规范描述已交付事实，解码器/脚本即合同）。
> 位置：`crates/auto-lang/test/cases/ace-m1/`；验证入口
> `powershell -File scripts/verify-ace-m1-852.ps1 -Scope <contract|source|snapshot|certify|lifecycle|acceptance>`。

## 同源四路径

一份 Auto 源码（`src/*.at`，byte-identity 由 binding.tsv 的 cases.json hash 轴冻结）经四条真实路径独立判决，父侧判定，worker 不接收 expected：

| 路径 | recipe/adapter | 判决面 |
|---|---|---|
| AC 共享 MIR native | `ac/mir-native-scalar-v1`（probe/mn） | 完整 i32 exit（含符号；trap=exit 70 为 818 冻结实验合同） |
| AV ABC | `ac-av/core-run-v1`（probe/av） | 完整 i32 exit + 类型化 trap 码（`av.trap.*`） |
| 旧 AutoVM | `avm/run` | legacy-display 完整十进制 result + 类型化 stage/code |
| a2r | `a2r/emit` | 真实产物逐字节比较冻结金样（`oracle/m1_*.rs`）；行为核验见独立认证规范；日常快照含严格收据验证（类型化精确键集 schema）与认证面进程观察（周期性 CIM 祖先链采样——是采样观察器而非无损创建计数；直接 rustc/m1_* 二进制 0 观测；引擎 worker 的 memoized 维护构建属 818 纪律、单独计数报告，不算认证面） |

宿主包装只允许结果编码（认证 wrapper 计入身份），不改算法/不删分支/不用四份不同源码。

## 分母登记（binding.tsv，schema ace.m1.binding/1，12 列冻结）

63 corpus 行（58 源例 + 5 typed 入口）+ 40 scenario 行 = M1 required 分母的登记处。
逐行给出四路径单元状态：

- `ok` / `ok-emit`：能力已交付且已正式接线（base 套件真绿）。
- `diff-*`：引擎真实执行但与冻结 oracle 语义分歧（如 B08/B11 旧引擎 10/-7 vs
  oracle 7；B13/B15 旧 i64 不陷阱；B16 a2r 不做名解析）——如实登记，永不冒绿；
  改已议语义须携反例交用户裁定。
- `gap:<owner>` / `wait:D82x`：未交付能力（native 扩面、822~824 类型族、native
  在线 loader 等），带 owner/关闭 oracle；不阻塞底座，但阻止 M1 全绿。
- `snapshot-only-cert-gap`：a2r 复合产物为伪 Rust（`List<int>`/`int`），快照可比
  较、不可编译认证。

selector/局部 run 不削减 required 分母；零选择、缺行、仅 emit、SKIP=PASS 均不能
作为 M1 绿证据。

## 底座套件（ace-m1-base.suite.ad）

只装证据绿检查：36 例 97 检查（T-02 标量三路径、T-03 金样快照、T-05 会话生命
周期、类型化负例）。负例 oracle（改错期望必红）与金样漂移（必红）以 cargo 测试
`crates/auto-lang/src/tests/ace_m1_tests.rs` 常驻。完整分母（含 skip 行）由
acceptance scope 从 binding.tsv 报告，两轴结论分开：底座 PASS ≠ M1 全绿。

## 会话生命周期（T-05 事实）

同一冻结源（B09）上：av warm/hot/body 编辑（fact(5)→fact(6)=720）/跨进程编辑
持久（内容寻址 store，cache_hit_abc 观测）；mn fresh/edit+重启（新 worker 进程，
store 跨进程，cache_hit_coff）。native 在线 HMR（I08）不由本规范声明——需要
native loader 能力包，重启 exe 不满足。

## 下游绑定（823~826 T-11）

下游仅在本目录**追加** binding/证据行；改 binding schema、金样、通用合同或
SDK adapter 须拿 852 交付后的测试窗口（关联核心合同另拿核心窗口）。消费时记录
输入 profile/wire/semantic 版本与脚本/证书身份。

## PLAN-824：动态 typed 能力绑定

D01–D09 的 `av-dyn-824` 行绑定 `ac-av/dynamic-run-v1` 和真实 typed MIR/ABC/AV、portable 恢复及父侧 oracle；W01–W05 保留整数和浮点精确位型验证，I04–I06/I10–I12 追加动态会话、失效与资源证据。源码、复合 native/CTE 及旧 AVM/a2r 的 Dyn API 未由 typed fixture 关闭，仍按各自 owner/gap 记录。

## PLAN-823：闭包 typed 能力绑定

K01–K06 与 scenario R05 的 `av-closures-823` 行绑定 `ac-av/closures-run-v1`
（AVAD v5/profile 6）真实 typed MIR→ABC→AV、代码缓存恢复与父侧 oracle；
I01/I02/I04–I06/I10/I12 追加闭包会话身份、env 漂移、跨工程身份、512 修订
churn 与 judge 反控证据（`-Scope baseline` 常驻，stale-evidence 身份控制钉
41 文件 hash+依赖 HEAD）。源码闭包语法（两 legacy 路径 parse 拒）、a2r 伪 Rust
复合认证与 native 闭包 ABI 仍按各自 owner/gap 记录，不因 typed 行关闭。

## PLAN-825：标量 Loaded 能力绑定

B01/B04/B09/B13/B14 五源（852 `src` 与 825 fixture SHA-256 逐位同源）绑定 `ac-av/loaded-run-v1` 与 loaded/reference 双路径独立 oracle：优化路径索引 build/PC/参数计数非零、reference 扫描/拷贝为零，B13/B14 绑定 overflow/div-by-zero trap。I01/I02/I05–I07/I10–I12 追加 30 修订结构+P95、两函数编辑身份、内容命中零重建、暂停/恢复同 pinned LoadedProgram、512 修订+GC 资源与 judge 反控证据（`-Scope baseline` 常驻）。源码四路径/native/M1 缺口按各自 owner 保留，typed/loaded 行不冒领同源 source 能力；绑定与 raw 见计划 attachments/825/852-validation（提交 97cecc89d）。

## PLAN-826：标量紧凑 wire 能力绑定

- 852 底座分母追加本计划实际支持起点：binding 表（`docs/plans/archive/826-av-compact-abc-format-and-size.md` 的 attachments/826/852-validation/binding.md）逐行映射 B01/B04/B09/B13/B14 双 wire 对拍与 I01/04–07/10–12 场景；`verify-av-826.ps1 -Scope baseline` 为正式入口。
- 语义 ID → 真实起点 → V 子例 → 正式入口 → 独立 oracle → 证据逐行可核；optimized==reference 双路径冻结期望。
- 范围外 gap 保留 owner：复合类型/字段布局源码编辑、source/native/AJ 面不冒领；typed fixture 不替代同源四路径。

# PLAN-816 T-13 / T-03 受影响复审 R2 — needs_fix

2026-10-21（客户端日期），Plan revision 2，review_scope=phase。本聊天 Codex /root
独立于实施会话，未创建 agent；具体模型/会话 ID 未暴露。仅复审 T-03 修复及其受影响
合同，不是 M816-B 最终验收；已发布 M816-A 的审查、合入、规范与读回收据保持有效。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`。
- 受审 `ae6eb14dc063a4896cfebfb40202536caf3ae848`，入场/收尾代码 clean。
- 修复基线 `1629aa60f0bc890c06a6a403aad5e6c07697c35c`，14 文件 +455/-43。
- T-03 原始基线 `7a2323cc364000dca8777d06c8d16629d1b8456f`。
- main `b81580553052b8663cdf3f183ed6888f78835825`；auto-down
  `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-03 仍未合入 main。
- [99 项输入指纹及 6 个旧版本 blob](inputs.json)。Plan hash 在本轮簿记前采集。
  [冻结 SD 候选](spec-delta-frozen.md) 与 R1 相同，SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。

**原五项具体反例闭合；新增两项 P2 修复回归，T-03 不能通过。**
另有当前依赖变更使 777 基线过期，须随修复重新验证，不能沿用 R1 的 strict pass。

## 原五项闭合核验

| 既有 ID | 当前结果 | 直接证据及覆盖边界 |
|---|---|---|
| F-816-T03-R1-01 | 原例 CLOSED；修复引入 R2-01 | 分别启动新 consumer 进程，旧 `Automan::create_jet` 无 install / 显式默认 install 均实际生成 Jetpack；集成测试还核验 Gradle 文件。默认行为恢复不等于所有 seam 注入语义已保持。 |
| F-816-T03-R1-02 | 原例 CLOSED；新 adapter 引入 R2-02 | 无 UI 的独立 runtime consumer、有效 API、真实占用动态端口，返回 `bind failed ... os error 10048`，确实到达真实 bind；workspace/member 改走 `auto_api_gen::rust_workspace`，独立消费返回非空真实路径和 `service-back`。生产 `serve_rust` 两站点也直接使用该 owner。 |
| F-816-T03-R1-03 | CLOSED | 六条 `auto_man::util` 旧路径在独立 consumer 编译通过；显式模块单源转发 runtime util 与 UI 端口 util。 |
| F-816-T03-R1-04 | CLOSED | 原真实 Git 临时仓 attached/detached、clean/dirty 控制恢复；新增库测试四状态均通过。判定重新使用 stdout 空代表 detached。 |
| F-816-T03-R1-05 | CLOSED | 两条纯 adapter 回归在 runtime，两个 PacSource 回归在 compat；核对旧断言后执行均通过。测试列表 121 个描述符 / 121 唯一，`apply_evaluated_root_b_group` 仅登记 1 次，重复属性告警消除。 |

[原例重放入口](replay.ps1)、[最终完整回放](replay-final-output.txt)，失败组为 0。
compat / legacy / 两条外部 adapter 原例源码沿用 R1；runtime probe 因三个非 UI 方法已从
UiOps 移除，把 workspace 调用改为生产当前采用的 auto-api-gen owner，服务入口、有效
API 与保留端口输入不变，并将错误 oracle 收紧为实际 bind/address 分类。
新增 runtime-only UI 拒绝控制确认仍返回 `UI ops not installed`。

## F-816-T03-R2-01 [P2]：默认 ctor 抢占调用方首次显式 UI provider 注入

类型：F-01 修复引入的回归。影响 T-03 / AC-01 的依赖倒置与 compat 注入语义。
位置：`crates/auto-man/src/compat.rs:211–218`；相关 `auto-man-runtime/src/ui_ops.rs`
的 `OnceLock` / `install`。

T-03 的 UiOps 是服务与上层实现的公开注入点，现有 `install(Arc<dyn UiOps>)` 采用
首次安装保留、重复安装幂等的实现。调用方可先安装其实现，再由 compat 的默认安装
流程兜底。新 `#[ctor]` 在 main 前直接占用同一个 OnceLock，调用方在 main 的第一次
显式安装被 `let _ = INSTALLED.set(ops)` 静默丢弃。调用仍进入真实 CompatUiOps，
调用方无法按声明的 seam 选择实现；测试替身也不能隔离真实 UI/npm 等副作用。

输入：[同一注入 consumer](injection-probe.rs)，构造独立 `Arc<dyn UiOps>`，调用公开
`install`，以 `Arc::ptr_eq` 核验 `current`，再调用默认安装并检查幂等。两个独立 manifest
使用相同代码：runtime-only 无 compat feature；compat 开启 feature 并实际调用
`auto_man::compat::install_default_ui_ops`，保证真实 compat crate 被链接。

```text
runtime-only: explicit_first_install_accepted=true; display=auto
              after_default_preserves_explicit=true; exit 0
compat:       explicit_first_install_accepted=false; display=pnpm
              after_default_preserves_explicit=false; exit 101
```

[runtime 对照](injection-runtime-output.txt)、[compat 反例](injection-output.txt)。
根因已由 ctor→默认 install→OnceLock.set 的完整路径和真实链接进程确认，不是
对静态构造器形式的笼统禁令，也不要求后来的显式安装覆盖已有显式 provider。

闭合 oracle：新进程旧库默认入口继续生成真实产物；调用方首次显式注入在 runtime-only
和链接 compat 两种消费形态均被采用，之后默认安装保持该实现；runtime-only 未注入时
仍具名拒绝 UI，闭包仍零 UI。默认兜底应与显式注入共存。

本修复新增 `ctor` 还变更 `crates/auto-man/Cargo.toml`，使 777 锚定输入失效：

```text
python scripts/modularity-audit.py --mode strict --output <report>
exit 2, strict verdict=fail, violations=1 (stale_baseline)
锚定输入自 base_revision 起已变更: ['crates/auto-man/Cargo.toml']
```

[原始报告](modularity.json)、[命令输出](modularity-output.txt)。当前 132 条声明边与
cargo metadata 一致，未检出新增禁向边；3 条既存逐边注记仍在。这里的红是基线失效，
不能表述为 strict 已绿。完成代码修复后按 777 合同处理锚定输入/显式重采样，保留精确
逐边约束并重跑 strict；不以放宽规则消红。这作为 R2-01 的配套健康检查收口。

## F-816-T03-R2-02 [P2]：语言 adapter 遗漏旧 run_file 的脚本参数复位

类型：F-02 修复引入的行为回归。影响 T-03 / AC-01 的语言 adapter 保持合同。
位置：`crates/auto-man-runtime/src/language.rs:15–27`，实际调用点为
`http_service::serve_vm` 与 `Automan` 的 VM server 臂。

旧 CompatUiOps 直接委托 `auto_lang::run_file`；该函数调用
`run_file_with_args(path, Vec::new())`，在读文件及执行前调用 SDK `set_script_args`。
既有注释明确每次调用覆盖前值、同进程串行运行不残留。新 adapter 直接调用
`execute_autovm_with_path`，没有执行这个入口语义。Plan §1 要求脚本 argv/默认保持，
本轮语言 adapter 也声明与历史入口同语义；搬出 UI seam 不应丢失这一步。

输入：[真实语言回放](language-probe.rs)。先经 SDK 设置之前一次调用的合法脚本参数
`previous-invocation-only`，然后分别用历史 `run_file` 和新 adapter 执行同一有效 `.at`：

```auto
use auto.process
process.args()
```

两条入口均进入真实 VM 并成功，使用 native `shim_process_args` 核对可见 argv：

```text
native argv length: historical=1 adapter=2; leaked_sentinel=true
historical run_file result=Ok("[-7]") residual_args=[]
runtime adapter result=Ok("[-7, -8]") residual_args=["previous-invocation-only"]
empty args legitimate control=Ok("[-7]")
missing file control: old_error=true old_args=[] new_error=true new_args=["previous-invocation-only"]
```

[原始输出](language-output.txt)，最终旧/新参数一致断言失败，exit 101。VM 返回值里的
负数是其列表元素的内部呈现；argv 长度与 sentinel 是否泄漏由真实 native 返回的
`Vec<String>` 单独确认，不用内部数字推断字符串值。

无先前参数时正控相同；有先前参数时新调用继承了本不属于当前服务调用的参数；
读文件失败出口也未按旧入口清空。此处不要求 SDK 的低层 executor 自行复位全局状态，
应由替代历史 run_file 的入口 adapter 保持已有初始化语义。

闭合 oracle：同一有效源文件、先前非空参数、默认空参数正控与读取失败控制均保持旧
入口的 argv 初始化结果；非 UI 有效 API 继续到达真实 bind，模块路径解析与语言错误
仍正常传播。为新 adapter 补对应定向回归，避免只验证 HTTP 失败不含 UI missing。

## 其他核验与负控质量

- runtime lib：117 passed / 4 ignored；[原始结果](runtime-tests.txt)。
  [测试列表](runtime-list.txt) / [唯一性](test-list-summary.json)：121 / 121，无重复。
- 新增定向集成测：runtime 2、compat 5，共 7/7；
  [非 UI 集测](non-ui-tests.txt)、[compat 集测](compat-tests.txt)。
- 原外部 adapter 2/2、管理核心 8/8，后者包含真实 npm/node build 成功与非零失败传播；
  [管理测试](management-tests.txt)。
- `Pac` / `Target` / `Automan` 跨 facade/runtime 类型身份通过编译；仍是 re-export。
- [闭包](closure-summary.json) / [完整树](runtime-tree.txt)：normal/build 420 包，
  auto-lang、auto-ui-tools/compiler/runtime、iced/iced_* 均为零；runtime consumer
  与 compat 分别构建。未将当前仍链接 compat 的 CLI 称为 Core profile，后者归 T-05。
- 三包 check exit 0，[结构化诊断](check-diagnostics.json)；fmt --check、git diff --check
  通过。全 clippy exit 101，60 描述符 / 30 唯一 unused_io_amount 错误仍只在两份
  与 T-03 前基线逐字同构的旧 Ninja builder / CMake 文件，
  [源码一致证明](clippy-baseline-proof.json) / [诊断](clippy-diagnostics.json)。
  不把既存错误计为本轮新 finding，也不宣称 clippy 全绿。
- 当前新 HTTP 集测接受错误含任意 `failed`，workspace 名断言的末支又重复非空条件，
  单靠这两条测试无法证明它声称的全部路径。独立回放以有效 API 的实际 bind 错误和
  生产 workspace 直接委托补证本轮原例闭合；建议修复时收紧永久断言。
- [新回归重放命令](replay-regressions.ps1) 的三次执行分别返回 0 / 101 / 101，
  [汇总](regressions-summary.txt)。闭合要求其全部为 0，而不是把预期失败算成产品通过。
- 保留被排除的探针配置记录：missing bin target、缺 compat feature 的 consumer，
  未导入 println 的语言源、导入旧 list 模块导致的编译错误。它们分别记录于
  `injection-fixture-*` / `language-fixture-*`，均不作为 finding 证据。最终 manifest
  声明 feature、强制链接 compat，并用已实际成功执行的 process.args 源重放。
  初次有效直返列表回放另存 `language-direct-list-output.txt`。沙箱和宿主 TEMP 不同，
  最终通过同一宿主脚本建 manifest/运行，并固定 CARGO_TARGET_DIR。曾发现两个独立 consumer 在共享 target 下使用同名 bin，缓存执行命中了另一包的顶层产物；错误记录为 injection-fixture-shared-bin-shadow.txt，不作产品证据。最终将两 bin 分别命名 runtime-injection-probe / compat-injection-probe，输出 cfg feature 身份，并重复冷/热运行确认，避免混用证据。
- 用户贴出的两条文本替换失败日志没有当作源码状态；当前提交已包含 util 显式模块和
  service 的直接语言 adapter 调用。

## AC、Spec delta 与后续门

AC-01 / T-03：单源、闭包、旧五例部分通过；显式注入和 argv 初始化失败，整体 fail。
A 的 AC-07/08 历史 pass、AC-09 已发布保持。AC-02/03/04 的 B 面完整链仍由后续任务
验收，没有用当前局部测试宣称整 Plan 通过。

SD-03 的层次方向与当前源码相符，SD-04 的入口搬迁方向成立；两处回归修好前不能
证明默认/执行语义完整保持。SD-01-A / SD-05-A 已发布内容不改。本轮 SD 候选无语义
变化，精确 spec-impact 路径和 GOAL-003 元数据仍有效；B 的 as-built 文本由既定后续
任务收口，未新加本轮发布规范的要求，未修改 canonical Specs 或 live ledger。

原树 work 一次处理 R2-01、R2-02、配套 777 基线及定向正反控制，提交后受影响复审。
T-03 与本次复用 T-13 重新打开，`executing`，current_step=4/14；A 的 T-01/T-02/
T-12/T-14 完成状态保持。T-04 的门仍是 T-03 + 已发布窄 tool；窄能力不必等 M815-B，
完整 UI 正控/profile/bootstrap 等 M815-B，本轮因 T-03 未通过不进入 T-04。

未运行整档 t/tv/tt/tb/tu/taa/tf、完整 GUI 或未完成的完整 profile。最终 B 代码/验收/
规范审查通过后，裸 cargo t、最终 diff 触面档、实际 consumer/来源/完整集成与 Spec/
ledger 收口仍归 merge-owned pending，代码变更后绑定新冻结提交重验。未修实施、提交、
合入、归档或清理 worktree。


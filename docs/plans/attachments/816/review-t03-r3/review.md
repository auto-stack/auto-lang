# PLAN-816 T-13 / T-03 受影响复审 R3 — needs_fix

2026-10-21（客户端日期），Plan revision 2，review_scope=phase。本聊天 Codex /root
独立于实施会话，未创建 agent；具体模型/会话 ID 未暴露。本轮仅覆盖 T-03 修复，
不撤销已发布 M816-A，不代替 M816-B 最终验收。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾 clean。
- 受审 `d93106fd20a10ca6e4a435b8edbbdfc1518e0e34`，实现
  `2f2b54888587b4ec0f620a778232a8adb8060b20`，其后仅 777 基线簿记。
- 修复基线 `ae6eb14dc063a4896cfebfb40202536caf3ae848`，8 文件 +452/-18；
  T-03 原始基线 `7a2323cc364000dca8777d06c8d16629d1b8456f`。
- main `b81580553052b8663cdf3f183ed6888f78835825`；auto-down
  `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-03 尚未合入 main。
- [113 项输入指纹及 6 个原始 blob](inputs.json)，Plan hash 取本轮簿记前。
  [冻结 SD](spec-delta-frozen.md) 与 R1/R2 相同，SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。

**R2 两项产品反例已闭合，777 已恢复 pass；仍有一项 P2 定向回归缺口，不能给 T-03 pass。**

## R2 及既有闭项核验

| ID / 合同 | 结果 | 证据与边界 |
|---|---|---|
| F-816-T03-R2-01 | CLOSED | `EXPLICIT` 与 `DEFAULT_OPS` 分槽；公开 `install` 首装保留，ctor/compat 默认安装只写默认槽，current 显式优先。独立 runtime/compat consumer 同源注入原例均 exit 0，打印 cfg 身份避免混用产物。 |
| 注入相邻状态 | pass | runtime 独立集测覆盖显式先装→默认→第二显式，首个显式保持；另一独立进程覆盖无显式时默认生效；compat 集测实际链接 ctor，再显式注入和重复默认安装，显式保持。旧 Jetpack 默认入口与显式默认正控均真生成。 |
| F-816-T03-R2-02 产品原例 | CLOSED | adapter 在读文件前 `set_script_args(Vec::new())`。原有效源两入口真 VM 均成功，native argv 长度旧 1 / 新 1、sentinel=false；空参数和读取失败控制保持。**永久测试的 VM 内观察 oracle 未闭，见 R3-01。** |
| R2-01 配套 777 | CLOSED | strict exit 0 / 0 violations / 3 grandfathered，132 声明边与 metadata 一致。基线绑定 2f2b54888；diff 仅 base_revision 和文本候选 30→29，未改规则、声明边、既存逐边注记。 |
| F-816-T03-R1-01..05 | 保持 CLOSED | 原独立回放 0 失败组：旧入口默认真实产物、有效非 UI API 到真实 bind 冲突、六 util 路径和类型身份编译、Git attached/detached 与 clean/dirty、原 adapter 回归均保持。runtime-only 未安装仍具名拒绝 UI。测试列表 122 描述符 / 122 唯一，原重复属性未再现。 |

[R2 原例重放](replay-regressions.ps1) 的结果为 runtime injection 0 / compat injection 0 /
language 0，[汇总](regressions-summary.txt)。[语言原始结果](language-output.txt)：

```text
native argv length: historical=1 adapter=1; leaked_sentinel=false
historical run_file result=Ok("[-7]") residual_args=[]
runtime adapter result=Ok("[-7]") residual_args=[]
empty args legitimate control=Ok("[-7]")
missing file control: old_error=true old_args=[] new_error=true new_args=[]
```

[R1 原例回放](replay.ps1)、[原始输出](replay-output.txt)；两份注入 bin 保持不同名字，
没有重用 R2 曾发现的同名 bin 缓存歧义。probe `.rs` 沿用 R2，脚本仅调整 R3 存储目录。
用户附带的替换失败日志不代表最终状态：非 UI 服务断言已去掉任意 `failed` 兜底，
workspace 命名断言也已去掉重复非空末支。

## F-816-T03-R3-01 [P2]：VM 内 argv 回归把标量长度按片段数检查，漏掉实际泄漏

类型：R2-02 修复所需回归的验证缺口；不认定当前生产 adapter 仍泄漏参数。
影响 T-03 / AC-01，延续 R2-02 已要求的“VM 内无前次 argv”闭合 oracle，未增加新规模
或新支持面。位置：`crates/auto-man-runtime/src/language.rs:77–79`。

新增测试实际执行：

```auto
use auto.process
return process.args().len()
```

这里 VM 返回的是标量长度字符串。正确实现实际返回 `"1"`，泄漏前次 sentinel 时返回
`"2"`。但测试先 trim 方括号，再按逗号拆分并计算非空片段数；两种返回值都只产生一个
片段，因此 `assert_eq!(n, 1)` 同时接受正确和泄漏结果。执行后的 `script_args().is_empty()`
只能观察返回后的表，无法排除 VM 已经读到旧参数、入口随后才清空的实现。

独立验证不是对测试源码的猜测，已用真实 VM 做可重放变体验证：

1. [受审源码的副本](language-late-reset-mutant.rs) 只将 argv 复位从入口移至执行完成，
   并保留读失败时的清空；SDK、VM、输入文件和返回值都是真实实现。
2. 原提交的整个 `#[cfg(test)]` 测试块逐字保留，
   [两个测试块 hash 相同的证明](mutation-proof.json)。未修改工作树生产源码。
3. [观察用例](argv-oracle-mutant.rs) 还独立执行当前正确 adapter 与晚复位变体，
   验证真实 VM 返回分别为 1 和 2，变体返回后全局表为空。
4. [回放命令](replay-mutation.ps1) 使用独立 consumer、单线程测试避免 argv 全局态
   相互污染，完整结果在 [最终输出](mutation-final-output.txt)。

```text
test language::tests::run_language_file_resets_script_args_like_legacy_run_file ... ok
real_vm_correct="1" real_vm_late_reset="2" existing_oracle_n=1
test observe_real_vm_and_existing_oracle_discrimination ... ok
test result: ok. 2 passed; 0 failed
```

变体故意违反已明确的入口顺序合同，原永久测试仍全绿，确认它不能证明所声明的
“VM 内 argv 无哨兵”。R2 产品反例修好、原例绿与这个回归缺口是分开的结果。

修正方向：对真实 VM 返回的**标量长度值**断言为 1，或在 VM 内直接断言 argv 长度/
sentinel，而不是数返回字符串的片段。保持成功、空参数及读失败控制；用晚复位/泄漏
负控确认测试会红，当前正确实现应绿。当前变体文件冻结的是本受审提交的原测试，
修复后验证须从新提交重生成相同变体并核对测试块，不能把冻结旧测试重新跑绿当闭合。

闭合边界仅为同进程串行调用的既有 argv 初始化合同，不提出进程全局 argv 并发隔离
或额外 UI 行为要求。两槽 provider 的生产实现未发现新增问题。

## 其余定向验证与健康

- runtime lib 118 passed / 4 ignored，[完整结果](runtime-tests.txt)；
  [列表](runtime-list.txt) / [唯一性](test-list-summary.json)：122 / 122、无重复。
- runtime 集成 4/4，compat 集成 6/6，共 10/10；
  [runtime](runtime-integration-tests.txt)、[compat](compat-tests.txt)。
  外部旧 adapter 2/2，管理核心 8/8，包含真实 npm/node 成功与非零失败传播，
  [管理结果](management-tests.txt)。
- 三包 check exit 0 / 0 error；[诊断](check-diagnostics.json)。fmt --check、
  git diff --check 过。clippy 仍 exit 101 / 60 描述符 / 30 唯一旧 unused_io_amount
  错误，只在两份与 T-03 前基线同构的旧 Ninja builder / CMake 文件；
  [诊断](clippy-diagnostics.json)、[源码一致证明](clippy-baseline-proof.json)。
  不计为本轮新增，也不宣称 clippy 全绿。
- [runtime normal/build 图](runtime-tree.txt) / [摘要](closure-summary.json)：420 包，
  auto-lang、auto-ui-tools/compiler/runtime、iced/iced_* 禁项为零；独立 runtime
  consumer 与 compat 分别构建。Pac / Target / Automan 仍是同一类型的 re-export。
- [777 strict](modularity.json) / [输出](modularity-output.txt)：pass，132 边、
  0 violations / 3 grandfathered；本轮未替实施者重采样或放宽规则。
- M816-A 设施/客户端和已发布规范不在本修复 diff 中，历史交付保持；没有把当前
  仍链接 compat 的 CLI 声称为已完成的 Core profile，后者属 T-05。

## AC、SD 与后续

AC-01 / T-03 的分层、旧五例、两项产品修复、777 当前通过；所需定向回归未通过质量
核验，整体 needs_fix。SD-03/04 的层次与入口方向仍成立，未改变既有 SD 候选合同；
现有 spec-impact 精确路径和 GOAL-003 元数据保留，B 的 as-built 文本继续按既有任务
收口。未发布 canonical Specs 或改 live ledger，不重开已交付的 SD-01-A / SD-05-A。

回原树 work 修正这一条回归 oracle，保留真实 VM 的正反控制，提交后受影响复审。
T-03 和本次复用 T-13 重开，`executing`、current_step=4/14；A 的 T-01/T-02/T-12/
T-14 保持完成。T-04 门仍为 T-03 + 已发布窄 tool；窄能力不等 M815-B，完整 UI
正控/profile/bootstrap 等 M815-B。本轮 T-03 未 pass，不启动 T-04。

未运行整档 t/tv/tt/tb/tu/taa/tf，未修改生产实现、提交、合入、归档或清树。最终 B
代码/验收/规范审查通过后的冻结裸 cargo t、最终 diff 触面档、完整 consumer/profile/
来源及 Spec-ledger 收口仍归 merge-owned pending；本局部 needs_fix 不触发整档门。

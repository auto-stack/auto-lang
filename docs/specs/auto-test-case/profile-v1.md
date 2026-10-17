# TestCase Profile v1（autolang.case/1 / suite/1 / test-report/1）

> 状态：active（A 阶段冻结；B 阶段扩展已交付为 v2，见下）。
> 本文是 v1 合同的 current-state 规范；逐字段 schema 以
> `crates/auto-test-case/src/contract.rs` 的解码器为准（解码器即合同）。

## v2 导航与版本拒绝边界

- B 阶段（PLAN-788）新增 `autolang.case/2` / `suite/2` / `test-report/2`：
  解锁 native（ac check/build/run）与 syntax（parse/edit）通道、hir-atom 输入、
  预算/cache/capabilities/exit/perf 等字段——完整合同见
  [profile-v2-native.md](profile-v2-native.md)。
- 版本拒绝边界：三入口按 schema 字符串**精确相等**分派；`wrong_schema` 仍精确
  拒绝未知版本（`case/2` 现为受支持分派，`case/3`/`case/1x` 等仍拒绝）。v1
  文档塞任何 v2 字段按 unknown-field 定位拒绝的行为不变。

## case 合同

- `schema: "autolang.case/1"`；`id` 全局唯一（选择集合内）；case 内 check id 唯一。
- `source` 为 block/file/text 恰一 + `syntax`（auto/auto-script）+ 虚拟相对 `path`
  （扩展名匹配 syntax）；`files` 附加多文件（虚拟路径唯一）。
- `entry.mode`: script/expression/program/module（module 要求 files；avm/run 不支持
  module entry）。
- `checks[]`: id/adapter/action/tier/options/expect/product_from/skip；
  合法组合矩阵=v1 仅 avm/run 与 a2r/emit。
- `product_from` 只能引用**更早声明**的 check（DAG 无环），消费语义见"产物交接"。
- suite：include（显式列表，无 glob）/exclude/defaults（tier 回退 + options 对象递
  归深合并、数组整体替换，本地值胜）/required.adapters/fixture_root（`file:` 载荷
  与 expected artifact 的单一解析根，导入语料原位不动）。

## 载荷与字节

- `block:` 锚点小节（到下一同级/上级标题）内**恰有一个直接 fence**；载荷取 closing
  fence 前真实 bytes（保末尾换行）；精确无末尾换行用 text/file。
- 非 UTF-8 oracle 支持 bytes；非 UTF-8 source 送 UTF-8 adapter 前 ERROR。
- `.as` 语法经管线按 entry path 后缀内部 lower；提交侧预 lower bytes 只作证据哈希。

## 观测与比较

- 完成状态与断言判定分离：**成功永不满足 error completion**（FAIL，非 PASS/ERROR）。
- error oracle 判定顺序 stage → code → message；阶段/代码来自类型边界，非文本猜测。
- 默认 bytes；`lf` 仅 CRLF→LF；legacy-display result 不 trim；未声明 stdout 与空
  stdout 是不同覆盖。
- 多产物：声明集合 = 实际集合（缺/多皆 FAIL），逐项按 compare 模式比对。
- 产物交接：比较状态 ≠ 产物可用性。执行过且产出产物的 check（PASS 或 FAIL）注册
  产物；消费者读**真实 actual bytes** + manifest 哈希校验后再按自身 oracle 判定；
  金样 FAIL 不切断交接，只有真执行失败（无产物）才 BLOCKED；未实现的消费模式
  （error-completion 消费者、result/stdout/stderr 观测消费者）= 显式 ERROR。
- pilot 的"下游真编译"不存在：不得宣称 Rust 已编译，不得从 expected 取 actual。

## 选择与只读

- selector（case/adapter/check）在**执行前**解析验证：未知 adapter 名、零匹配 =
  契约错误；未选项记 NOT_RUN 且**零副作用**（不建 scratch、不调 adapter、不读
  fixture）；局部运行 NOT_RUN>0 → exit 1，不当 gate pass；selector 擦不掉
  selected case 的 required 覆盖。
- 输出域：out/json-out/proposal-dir 与 suite 根/fixture 根 canonical 双向不相交
  （相等/包含/被包含皆拒）+ 全链 reparse 审查；module_name 与产物逻辑路径必须是
  单一安全路径分量。失败产物与候选基准只写隔离输出目录；proposal 仅生成候选
  （旧新 sha256 + 单 hunk unified diff + 共享消费者 + 来源报告），accept 是独立
  人工动作；读失败传播为 ERROR。

## 资源与报告

- admission 限额（frozen defaults/hard caps，runner 可在区间内调）：document/
  inline/sidecar 字节数、cases/checks/files/suite-files 计数、**全 run 展开
  check 总量**（load 时强制，lint 同见）；读取一律先检后有界（读 max+1 判溢出）。
- 报告：逐 check 状态 PASS/FAIL/ERROR/BLOCKED/SKIP/NOT_APPLICABLE/UNDECLARED/
  NOT_RUN + identity 五轴 + toolchain 四轴；退出码 0=所选 gate 满足、1=断言/覆盖、
  2=契约/基础设施（并存取 2）；**零动态检查 = 2**（聚合绿不许遮蔽空跑）；JSON 与
  console 同源；XFAIL/XPASS unsupported。

> 来源：docs/plans/773-autodown-testcase-protocol-pilot.md（Design 35 A 阶段，
> R1 修复轮后冻结）；解码器/裁决实现即合同细节的单一事实来源。

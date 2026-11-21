# 按语言特性组织的共享 AutoDown 语料（feature-organized corpus）

> Status: **delivered（PLAN-1078 AVP-00 / PLAN-1079 AVP-01，2026-11-19）**。公共语言测试按语言特性
> 集中组织：同一份 Auto 源码供 AV/AC/A2R 等后端各自追加目标检查，避免重复维护。
> 设计合同见 [docs/design/testing/ace-feature-organized-autodown-corpus.md](../../design/testing/ace-feature-organized-autodown-corpus.md)；
> 本文件记录**实际交付**的作者/选择/迁移/忽略机制与支持事实。

## 1. 作者与选择（交付）

- 唯一活跃作者根：`crates/auto-lang/test/cases/language/`（按语言行为命名，
  不以 AV/AC/A2R、M1/M2 或计划号划分）。公共总选择 `language.suite.ad`。
- 每个公共 Case 一份 `### Auto` 源码（或多文件工程闭包）；各后端在 Case 内
  声明检查。suite/固定组只引用、选择，不复制 Case 形成后端视图。
- 固定组 `case-av-language-parity` 使用 `--target av` 选择已注册 AV recipe 的 run 检查。
  AV 视图按所选检查判定退出状态，报告记录目标，SKIP 和 NOT_RUN 各自保留。

## 2. 检查级 `#ignore`（交付）

- 裸 `#ignore` 紧跟检查小节标题、位于属性之前 → 该检查映射 `CheckSpec.skip`
  （reason=`AV阶段暂缓非AV检查`、issue=`AV-ONLY-1078-1096`、owner=PLAN-1078）。
- 只影响该检查；不能标整 Case 或共享 Auto 源；AV required 检查不得被关闭。
- 忽略的检查不启动其绑定/编译/链接/执行/外部工具准备（FC-08：prepare/
  dispatch=0），不阻 AV 门；后端任务逐项移除本阶段 `#ignore`，保原
  oracle/平台条件/历史 ignore，按同一源真实运行。
- `#ignore` 不放松公共文档结构/schema 准入；未注册的未来 recipe 保
  Notes/义务表 pending。

## 3. 迁移映射（交付）

- `language/migration/legacy-map.tsv`：旧义务（repo 路径/Case ID/check ID/动作/
  源指纹/旧 oracle/条件）→ canonical Case/check + 原因 + owner + 状态
  （mapped / research-hold / no-such-shape / parser-only-hold）。数据映射，
  不作新注册器；旧 ID 不丢，合并不减义务。

## 4. 支持/边界

- AV-ONLY 阶段（1078–1096）：非 AV 检查加 `#ignore` 或保持未选；共享文档/源码、
  AV 必需公共编译与语义验证仍正确。非 AV required 关闭是对应后端/1068/985
  的最终门，AV 阶段的忽略不当通过。
- 平台：Windows 执行；Linux/WSL 腿归专用 Linux 环境。

## 5. AVP-01（PLAN-1079）增量

- 新增唯一作者文件：`language/numbers/literals-conversions.case.ad`（3 个 Case：
  wide-exact / mixed-width-explicit / overflow-is-not-wrap）、
  `language/text/interpolation-unicode.case.ad`（2 个 Case：text-nul-and-format /
  text-literal-escapes）。`language.suite.ad` 只追加两条 canonical 引用。
- 固定组 `case-av-language-parity` 复跑 8/8（1078 三例 + 本包五例），单例
  0.45–0.54s，均在 10s 预算内；非 AV 行继续 `#ignore`/NOT_RUN，不充 PASS。
- `legacy-map.tsv` 追加 68 行（PLAN-1079 主承接的旧 VM 文档）：保留原 case ID、
  check ID、声明 oracle 与 Auto 源 SHA16；分类 mapped 22 / research-hold 34 /
  no-such-shape 12。hold 行在 reason 里写明形态、owner 与关闭条件，不缩分母。
- 门禁控制（真实执行）：oracle 变异（成功例 `exit: 41`）→ FAIL
  `exit[exit] mismatch: expected "41", actual "Some(42)"`；全部 AV 检查 `#ignore`
  的视图 → 组级 `NOT_APPLICABLE` 且单独运行 exit=1（零选中不绿）。


## PLAN-1080 增量（2026-11-19，calls 家族）

`language/calls/binding-pipeline.case.ad` 为 calls 家族唯一活跃作者
（22 例：§6 正/负例 + 10 份旧 VM 迁移 + PIPE-NEIGHBOR/CALL-SOURCE-ORDER
补充 ID）；`language.suite.ad` 增 canonical 引用。迁移判等以整份旧
`.case.ad` 文件字节 SHA256 前 16 hex 对账（10/10 零漂移）；保留词冲突的
`double`→`twice` 改名是唯一具名审计规范化（语义/oracle 不变）。旧
`test/vm` 文件零改动（具名冻结历史）；legacy-map 本包行 owner=PLAN-1080。
故意红元控制（oracle 三处变异各红、复原绿）实证语料判决器真实。

## PLAN-1081 增量（2026-11-20，functions 家族）

canonical 作者 `crates/auto-lang/test/cases/language/functions/references-closures.case.ad`（12 case：§6 合同 4 例 + 9 份旧文档迁移），`language.suite.ad` 追加 include。legacy-map.tsv 本包 9 行：mapped×7（27_function_reference 三例 `double`→`twice` 保留词规范化；007 untyped lambda 参数加 `int` 注解；d01b error-presence 精确化为 `bind/source.nested-fn-no-capture`），research-hold×2（010/013 `List.map` 算法归 stdlib/HOF owner——闭包+捕获语言面由 canonical `closure-hof-capture` 承接，oracle 3 不变）。`test/vm` 零改动（具名冻结历史）；AV-ONLY 阶段非 AV 检查 `#ignore`/NOT_RUN 不充 PASS。

## PLAN-1082 增量（2026-11-20，collections 家族）

canonical 作者 `crates/auto-lang/test/cases/language/collections/list-places.case.ad`（25 case：§6 合同 4 例 + PLACE-SIDE-EFFECT/BOUNDS 五面 + 迁移 12 例 + 具名反例 3 例），`language.suite.ad` 追加 include。legacy-map.tsv 本包 62 行（33 mapped 带审计规范化 / 29 research-hold 具名 owner：AVP-06×8、AVP-05×1、AVP-08×1、stdlib 容器×19）；跨批关闭 statement-surface 复合语句 6 行（mapped 至 `compound-statement-family`，原始 oracle 全保）；21_conv 2 行重定向 stdlib 容器批。`test/vm` 零改动（具名冻结历史）。故意红元控制：单 oracle 变异 42→43 → scratch 套件 FAIL=1/exit 1。infra：`fixed-tests.py` per-group `check` 选择器经 prepared 清单丢失已修（run-prepared 丢 `--check av` 致全量套件对滤后期望误判）；AV-ONLY 阶段非 AV 检查 `#ignore`/NOT_RUN 不充 PASS（67 AV 绿/34 NOT_RUN）。

## PLAN-1083 增量（2026-11-20，types/patterns 家族）

canonical 作者 `crates/auto-lang/test/cases/language/types/variants.case.ad`（23 case：§6 合同 1 例 + 016-019 全 oracle 迁移 + 标量枚举/int 码臂 + 28_enum 方法四例 + Option/Result 族 + idiom2 读面/034-036/008）与 `crates/auto-lang/test/cases/language/patterns/nested-match.case.ad`（12 case：§6 合同 2/3/4 例 + PATTERN-BIND 五面 + 尾值族），`language.suite.ad` 追加两 include（新 types/patterns 目录）。legacy-map.tsv 本包 52 行（45 mapped 含 7 具名拒映射 / 7 research-hold：AVP-08×4（m01/m02/m08/m09 方法字段写腿）+ 10.036 Str-through-element 腿 + AVP-10（28.005 泛型）+ stdlib/modules（28.006））；跨批 1 行（15_nested_mutation.015）owner 由本包改 AVP-08（读面已落、写缝归借用批）。具名审计规范化：fn-local enum 提升、fstr 包裹去包裹、display oracle 留 map、整 owned 元素绑定→经元素投影、`.helper(3)`→`self.helper(3)`、str 词臂→int 码、wildcard helper 内联（1080 未类型尾调用缺口）。`test/vm` 零改动（具名冻结历史）；SHA16=整文件字节前 16 hex 对账。故意红元控制：单 oracle 变异 exit 42→43 → 套件 FAIL exit=1，复原复绿。AV-ONLY 阶段非 AV 检查 `#ignore`/NOT_RUN 不充 PASS（111 AV 绿/41 NOT_RUN）。

## PLAN-1084 增量（dynamic 家族）

`language/dynamic/objects-null.case.ad` 是对象、混合集合与 null 的活跃作者。19 份迁入源字节和原检查 ID 均保留；AV 与旧 VM 参考共用作者源。原纸本路径保历史签章；普通 legacy 文档读者按精确迁移行读取活跃作者。

固定组 `case-av-language-parity` 以运行目标和 recipe 家族选择实际检查。显式阶段忽略报告 SKIP，其他未选报告 NOT_RUN。具 AV required 标签的 Case 必须有活跃的 AV run 检查。源码变更使旧 AV 结果陈旧，陈旧性检查按普通 AV worker 的实际身份对账。控制与复现摘要见 [PLAN-1084](../../plans/archive/1084-av-parity-dynamic-objects-and-null.md)。

## PLAN-1085 增量（modules 家族）

`language/modules/imports-globals.case.ad` 是本包统一作者，含 `03_variables` 八个原例与 `17_modules/001_use_fn` 的原 case/check、源和原 VM oracle。单独的 AV probe 共享工程文件闭包。原 VM 文档按 1066 保留为历史，生产清单与 Rust 包装读者沿迁移映射访问 canonical 文档。阶段 `#ignore` 仅作用于原 VM 检查。AV 切片用 `--target av` 按动作和 recipe 选择。工程依赖文件内容变化使旧证据陈旧，坏公共结构和缺必需 AV 检查使 AV 门失败。来源见 [PLAN-1085](../../plans/archive/1085-av-parity-modules-and-global-state.md)。

## PLAN-1086 增量（ownership 家族）

`crates/auto-lang/test/cases/language/ownership/borrows-moves.case.ad` 组织借用、写回、移动和资源值的同源语料。28 条主承接与 11 条接续保留原 Auto 字节、原检查身份和 VM 预期。原 VM 检查按阶段 `#ignore`，生产包装器按 `legacy-map.tsv` 读取唯一作者，旧文档作为冻结历史保留。普通 AV 按目标与 recipe 独立判定，新增 22 个具名检查覆盖视图、记录与文本写回、诊断、发布和复制。源身份对账和有意判错的控制见 [PLAN-1086](../../plans/archive/1086-av-parity-borrows-moves-and-storage.md)。

## PLAN-1087 增量（errors 家族）

`language/errors/catch-finally.case.ad` 是跨调用错误、展开和相邻控制流的唯一活跃作者。25 份原源和 27 条原 VM 检查在 `language/migration/legacy-map.tsv` 逐项保留原 SHA16、检查 ID、预期和执行条件。原 VM 检查阶段标记为 `#ignore`。具名差异各有独立的 AV 诊断检查，同一个 Auto 源继续承接旧 VM 预期。

普通 AV worker 向父端报告处理器进入计数和最近的错误类型、虚拟文件和字节区间。父端验证该位置属于提交源码，并独立判定 AV 的值、双流和诊断。语料控制与运行证据见 [PLAN-1087](../../plans/archive/1087-av-parity-errors-and-unwinding.md)。

## PLAN-1089 增量（generators 家族）

[generators/yield-close.case.ad](../../../crates/auto-lang/test/cases/language/generators/yield-close.case.ad) 是生成器挂起、取项、耗尽和关闭的活跃作者。四份旧 VM 文档的源码字节、case/check、输出和末尾换行通过 legacy-map.tsv 保留。旧 VM 生产发现和包装读者按映射读 canonical 文档，原文档供历史核对。原 VM 检查与新增间接返回的旧参考均应用阶段 #ignore。AV 实际检查按运行目标选择并独立比较。

三项父端资源观察重用同一 AutoDown 作者的源。实验分别从 AV 记录早停后的 Str/List/Variant、挂起期间的宿主取消以及关闭错误捕获后的释放。选择、oracle 与来源反控见 [PLAN-1089](../../plans/archive/1089-av-parity-generators-and-stream-lifetimes.md)。

## PLAN-1088 增量（generics 家族）

泛型、spec 分派与方法决议的活跃作者是 `language/generics/` 下的 `spec-methods.case.ad`、`list-methods.case.ad`、`spec-dispatch.case.ad` 和 `record-templates.case.ad`。主承接七份旧 VM 文档，附加承接泛型枚举 28.005，共八条路径依原 Case/check ID 映射至 canonical 作者。m08/m09 使用 ownership 家族既有条目。

同一 Auto 源下按阶段选择真实 AV 检查，原 VM oracle 随 `#ignore` 检查保留。旧路径由 `CanonicalVmCases` 通过迁移数据解析到 canonical 作者。可执行控制验证源码身份漂移、AV oracle 错误、重复活跃 Case、无有效 AV 检查及未选目标的工具隔离。实际结果及输入身份见 [PLAN-1088](../../plans/archive/1088-av-parity-generics-specs-and-methods.md)。

## PLAN-1091 增量（comptime 家族）

编译期装配、真实标量执行和宏的唯一公共作者文件为 [evaluation-macros.case.ad](../../../crates/auto-lang/test/cases/language/comptime/evaluation-macros.case.ad)，由 `language.suite.ad` 引用。21 个普通 AV 检查分别固定值、精确双流或具名错误。验证用真实 CT 调用次数和运行期调用次数区分编译期回灌与运行期计算。选择负控覆盖错 AV 预期、零条有效 AV 和重复 ID。非 AV 检查在对应视图按阶段报告 SKIP 或 NOT_RUN。

本包主承接旧 VM 文档为零。`legacy-map.tsv` 记录旧示例、编译器内部断言与 cookbook foreign-provider 文档的独立角色和原预期来源，这些记录不作为 AV 运行结果。源码身份和父侧检查结果见 [PLAN-1091](../../plans/archive/1091-av-parity-comptime-and-macros.md)。

## PLAN-1090 增量（async 家族）

`language.suite.ad` 引用任务和异步的唯一普通 AV 作者，共 41 条 AV 检查。13 份旧 Actor 文档保持原 Case ID、源码与 stdout，另保留 3 个历史 Rust 内联源码和 1 个纯语言 Cookbook 源码。迁移表区分原 VM ignored、原 A2R 预期角色和 AV 阶段选择。非 AV 原检查保留其原始条件、预期和 owner。[任务和异步语料](../../../crates/auto-lang/test/cases/language/async/tasks-actors.case.ad)

父端按目标独立判定值、双流与异常。控制覆盖改动源身份、故意改错 AV 预期、旧活作者重复、零有效 AV 与非 AV 工具隔离。冻结结果及原文件身份见 [PLAN-1090](../../plans/archive/1090-av-parity-tasks-async-and-actors.md)。

## PLAN-1092 增量（metadata 家族）

标注与字段元数据的唯一公共作者为 [attributes.case.ad](../../../crates/auto-lang/test/cases/language/metadata/attributes.case.ad)，由 `language.suite.ad` 引用。18 条普通 AV 检查独立判定值、双流和具名错误。其中一个共享 Case 持有依阶段忽略的 AC 检查。父端反控覆盖选错预期、零有效 AV、重复 Case 和源身份失效。

本包主承接旧 VM 文档为零。迁移表保 963/971/555/936/967、旧 Task shim 及独立的 A2R/A2C 目标预期。原消费端及判决见 [PLAN-1092](../../plans/archive/1092-av-parity-attributes-and-metadata.md)。

## PLAN-1093 增量（FFI 家族）

受检外来 provider 语言接面的唯一公共作者为
[imports-callbacks.case.ad](../../../crates/auto-lang/test/cases/language/ffi/imports-callbacks.case.ad)，
由 `language.suite.ad` 引用。5 条普通 AV 检查真实运行：Rust `f0_text`（9/0 与 panic
边界）、C `parity1093_c`（真实 `<string.h>` strlen=5）、缺 cap 与 provider 不匹配的
具名拒绝。case 携带声明性 `foreign` 绑定，父端经 `REGISTERED_FOREIGN_PROVIDERS`
核验并在 `ACE_AV_PROVIDER_ROOT` 缺省时具名失败（不静默 SKIP）。本包主承接 70 份旧
VM FFI 文档：迁移表逐行保留原身份与 owner，AV 阶段 `#ignore`。父端反控覆盖选错
oracle（必红）、缺 provider 闭包（必失败）与源身份失效。

## PLAN-1094 增量（script/Python 家族）

脚本模式与 Python 对象协议的唯一公共作者为
[python-context.case.ad](../../../crates/auto-lang/test/cases/language/script/python-context.case.ad)，
由 `language.suite.ad` 引用。12 条普通 AV 检查真实运行：嵌套 py 调用、ValueError
捕获/未捕获对（`python.5` 分类＋原始 span oracle）、context-manager `__exit__` 恰一次、
Unicode/NUL 往返、py `.len()` 分派与纯 Auto 原生不变、错误消息原文、py
TypeError 可捕获、WITH-EXITS（break/嵌套）关闭计数。case 携带声明性 `foreign:
{kind, provider: "py"}`，父端在 `py-unavailable` 标记存在时显式 NOT_RUN（"never
PASS"）。本包主承接 7 份旧 VM 文档（99_py_dispatch×2、99_script_err×5）：迁移表 7
行保原 ID/check/oracle/SHA16 与 owner=PLAN-1094，AV 阶段 `#ignore`（非 AV 目标
pending 交原 owner）；旧 obj_call 对 Auto 值的假阳性不复制。父端反控覆盖选错 oracle
（必红）、重复 Case ID（必红）与源身份失效。

来源 [PLAN-1094](../../plans/1094-av-parity-script-and-python-protocols.md)。


## PLAN-1095 增量（composition/domain 家族）

新增 canonical 作者 `language/composition/ordinary-projects.case.ad`（
module-closure-generic / task-language-composition / cte-and-ordinary-library）
与 `language/domain/node-values.case.ad`（node-ordinary-coexist），均加入
`language.suite.ad`。AV 检查真实执行；`node-ordinary-coexist` 使用新增
`nodes` oracle（`nodes: { Name: [texts] }`，仅普通 language-source 成功完成
允许）断言 979 producer 对同一源的真实 view 文本——变异必红（2026-11-22 故意红
实测）。129 份旧 VM 文档义务行追加至 `language/migration/legacy-map.tsv`
（owner=PLAN-1095，原 case/check/条件/oracle/SHA16 保留，canonical 按语言族
映射）。非 AV 检查在本包保持 AV-ONLY `#ignore`/未选，不 prepare、不 dispatch、
不计 PASS；后端任务逐 check 打开后按同一源真实运行。

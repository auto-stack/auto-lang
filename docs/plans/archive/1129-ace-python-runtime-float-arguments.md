---
plan_id: PLAN-1129
status: archived
feature_name: ACE Python 运行时浮点实参
author: [codex]
created_at: 2026-11-23
updated_at: 2026-11-24
plan_revision: 4
depends_on: [PLAN-1127, PLAN-1110, PLAN-1107, PLAN-967, PLAN-978]
supersedes_spec_components:
  - docs/specs/auto-av/python-interop.md
  - docs/specs/auto-a2x/python-ecosystem-integration.md
  - docs/specs/auto-test-case/feature-organized-corpus.md
  - docs/specs/ace-migration/m2-host.md
  - docs/specs/auto-bindgen/project.md
  - docs/specs/ace-migration/replacement-scope-and-evidence.md
new_spec_components: []
touched_goals: []
affects: [auto-av, auto-a2x, auto-test-case, ace-migration, auto-bindgen]
current_step: 6
total_steps: 6
---

# [PLAN-1129] ACE Python 运行时浮点实参

## 0. 变更摘要

2026-11-23，承接 PLAN-1127 的已交同源切片基线，将 Python 调用中只接受字面量或字面量来源的浮点实参扩到普通 Auto 运行时浮点值。首个使用例是 `SGD(model.parameters(), lr: rate)`，其中 `rate` 由 Auto 实际计算。目标与后继顺序见 [Design 58 §8](../../design/58-python-standard-syntax-interop.md#8-python-后继路线)。

## 1. 目标

- AV 和 A2PY 共享 `f32/f64` 实参准入，普通函数调用和绑定方法均能消费位置实参、具名实参。值可来自参数、运行时计算、变量重赋值、函数返回，或现有 Python 标量提取结果。
- 验收 Python 实际收到的类型和值，以及混合实参的单次求值和源序。以同一份 Auto 训练用例及独立原生 Python 核 `lr: rate` 的实际优化器值与训练结果。
- 沿 1110 核对 Python 原异常类、控制流和原 Auto 逻辑路径与范围。沿 967/978 管理所需 provider、接口与工件身份。
- 动态整数宽度扩展、通用容器过桥、回调、零复制、异步和调用加速各自另行设计。Python → Auto `.to(float/double)` 继续遵循现有入站转换合同。

## 2. 架构方案

普通 Auto 源 → 共用受检浮点操作数事实 → AV 的版本化 Python 调用值编码或 A2PY 的原生 Python 标量 → 实际 Python 函数。AV 沿已有 967 调用、错误及资源合同，A2PY 沿已有 Codegen 和原异常观测。后端应消费同一个运行时操作数，保留 Auto 已发生的舍入。

## 3. 技术栈

ACE `resolve`、`PyOpFact/PyOperand`、967 Python provider 与 978 工件装配、A2PY lower/运行 helper、AutoDown TestCase。正式验证以 1107/1127 已交 Windows x64、CPython 3.14.2、CPU PyTorch 2.14.0+cpu 为环境基线，T-01 核实际解释器、包及 provider 的精确身份。标准库 `math/struct/fractions` 用于观测类型、位值、签名与错误；`fractions.Fraction` 观察 Python 对象运算符右侧的浮点操作数。

## 4. 需求分析与背景调查

本轮授权为 auto-lang 仓内的设计、路线图与计划文档，由 [Specs 总览](../../specs/overview.md) 导航。静态代码基线 `8a52d7364e5b`。实施、构建、产品执行和 worktree 留待工作阶段。

| 来源 | 已核对事实 |
|---|---|
| [AV Python 规范](../../specs/auto-av/python-interop.md)、[Python 生态规范](../../specs/auto-a2x/python-ecosystem-integration.md) | 动态浮点实参仍是明确缺口。输出侧已经有 `.to(float/double)`，入站有独立边界。 |
| [共享 resolve](../../../experimental/ace/crates/core/src/source/resolve.rs)、[操作事实](../../../experimental/ace/crates/core/src/py_ops.rs) | `py_val_class`、`py_operand_literal`、`py_frame_val` 目前只承认浮点字面量及 `py_lit_locals` 中记录的来源。运行时 float 落入 `source.py-domain`。 |
| [AV provider](../../../experimental/ace/crates/foreign-contract/py/py_bridge_provider.rs)、[A2PY lowering](../../../experimental/ace/crates/a2x-python/src/lower.rs) | AV 旧 provider 把文本前缀 `F:` 当浮点字面量标记。A2PY 已有按受检表达式获取非字面量操作数的路径。需验证它能获取真实浮点值而没有采用帧构造节点或旧常量。 |
| [1107 PyTorch 源](../../../crates/auto-lang/test/cases/ace-m2/python-1107/pytorch_train.as)、[PLAN-1110](1110-ace-a2py-python-exception-mapping.md)、[PLAN-1127](1127-ace-python-object-slice-syntax.md) | 现有训练把 `lr: 0.1` 写为字面量。1110 异常调度和 1127 双端参考基线已交付。原始 1107 夹具保留作对照。 |
| [967 的标量合同](../../../experimental/ace/crates/foreign-contract/src/lane.rs)、[M2 构建规范](../../specs/ace-migration/m2-build.md)、[PLAN-967](967-ace-m2-host.md)、[PLAN-978](978-ace-m2-build.md) | 通用 ABI 有精确 `F64Bits` lane，当前 Python 调用仍有自己的帧合同。扩展必须由现有 provider 与装配选择正式承载。通用标量 lane 的存在并不证明它已进入 Python 参数帧。 |

2026-11-23 用户以 `/goal 完成计划1129` 授权执行、复审、合同内修复和合入归档。实施基面 `0486981556bd8d128c69ded9888319ac4a65fc02`，专属树 `D:/autostack/.wt/lang-1129/auto-lang`（`plan-1129-dev`），同组只读 auto-down 检出 `456c86a`。计划材料统一放 `D:/autostack/plan-artifacts/plan-1129/`。

r2（2026-11-24）：有界修订，授权沿用上述 `/goal`。work 在冻结提交上实测后订正 T-01 冻结文本的 payload 形态、发射、A2PY 舍入 helper 与身份表，并把帧线路波及的三份 canonical 规范补入规范增量（依据与被取代措辞见 §9「new r2」）。目标、验收标准、任务 ID、仓集合与预算不变。

r3（2026-11-24）：有界修订，授权沿用上述 `/goal`，起因是 review r2 的发现（§9）。只订正 §4 冻结文本里与实现不符或漏写的三处——`f` 标签的规范编码上界（R-05）、A2PY 取负实参的取值规则（R-01）、身份表「无 `use.py` 的程序」一行对 A2PY 的限定（范围观察）——并把语料例数改为 14。目标、验收标准、任务 ID、仓集合与预算不变。

### T-01 冻结结果

（r2：下列线路合同、发射、A2PY、身份表与测试入口是 work 在冻结提交上实测后的现行文本；r1 被取代的措辞留存在 §9「new r2」。基线实测与表示选择未变。r4 只回写 A2PY 段的一句，见 §9「new r4」。）

**环境身份**。Windows x64，rustc 1.98.0。AV 用 `C:/Python314` 内嵌 CPython 3.14.2；A2PY 与原生参考用用户级 `A2PY_PYTHON` 指向的 venv（`-I -B`）。两端都读到 torch 2.14.0+cpu（用户 site）与 numpy 2.5.1。调试工具准备 `cargo t case-av-language-parity case-a2py-language-parity ace-python --prepare-only` 退出 0，总墙 4 分 33 秒。

**基线实测**（改动前的专属树，探针与报告摘要见 `t01/t01-baseline.txt`，SHA256 `ed639389cd36149ad75c7c3046cba62bb39dee19160a40b4f8812130c27e943b`）：

| 探针 | AV 实际 | A2PY 实际 | 结论 |
|---|---|---|---|
| `pack(">f", 0.1)`、`pack(">d", 0.1)`、`pack(">d", 0.1d)` | `3dcccccd`、`3fb99999a0000000`、`3fb999999999999a` | 同 AV | `0.1` 是 f32 再加宽，`0.1d` 是 f64；字面量路径两端一致 |
| `repr("F:1.25")` | `1.25` | `'F:1.25'` | **基线缺陷 F-1**：AV 的旧 `F:` 前缀把普通文本解码成浮点，两端不一致 |
| `var rate = 0.1; rate = 0.2; pack(">f", rate)` | `3dcccccd`（0.1） | `3dcccccd`，生成码先赋 0.2 再传 0.1 | **基线缺陷 F-2**：`py_lit_locals` 只在初始化时登记、重赋值不移除，两端都传旧字面量 |
| 运行时 f32/f64 实参 | `diag source.py-domain` | `a2x.compile` 退出 2 | 现状缺口，共享前门拒绝 |
| inf、NaN、f64/f32 次正规、`-0.0d` | `1 1 1 1 1 Infinity NaN 0` | 同（多 `value:0`） | inf、NaN 与两种宽度的次正规都可从源面到达；显示丢符号零的符号 |

**表示选择**。比较两条路：(A) 消费现有 ABI 精确浮点 lane——`LaneKind::F64` 只属于 937 的 typed-declaration 家族，py provider 走 manual/resource 家族（`ResKind` 无浮点），AV foreign exec 没有 F64 物理 callable（`dispatch_view_scalars` 只收 `&[i32]`），且公开 lane 集没有 f32；接通需要跨家族的 ABI 改动，不是最小路径。(B) 向 Python 调用帧追加有版本的 typed 值——只动帧语法、解码器与 provider，可行。**选 B**。

**线路合同（冻结）**。`p967.frame/2` 是 `/1` 的超集，布局不变，新增两个值标签：

- `d <len>\n<w2> <w1> <w0>\n` 是 f64，位型是其 64 位模式；
- `f <len>\n<w2> <w1> <w0>\n` 是 f32，位型是其 32 位模式；因此 payload 另要求高 32 位为零（`w2 == 0` 且 `w1 < 1024`），这是规范编码的一部分，超出即具名拒绝，不截断（r3，R-05）。

`<w2> <w1> <w0>` 是三个无符号十进制字，把 64 位位型按 21/21/22 位自高位拆开（`bits = w2 << 43 | w1 << 22 | w0`）；每字有界（`w2 < 2^21`、`w1 < 2^21`、`w0 < 2^22`），一个值只有一种编码。payload 恰为“数字 空格 数字 空格 数字”，否则是具名 `foreign.frame` 拒绝，派发前生效，接收者、位置与具名三个值槽规则相同；拒绝消息带实际帧版本（`p967.frame/<版本> rejected: <原因>`，r3，R-06）。AV 校验器（`validate_frame_wire`）与生成的 shim 解码器是同一字节集的双实现，须同步修改；解码器源文本单独成文件（`crates/auto-bindgen/src/frame_shim_prelude.rs`），由生成器 `include_str!` 拼入 provider、由单元测试 `include!` 直接编译。`/2` 帧里 `t` 恒为文本，不再有 `F:` 解码。`/1` 帧保原五种标签和旧行为。resolver 为自己构造的每个带值帧（`py_call`、`py_getitem`、`py_binop`）发 `/2`；导入帧与只带接收者的帧不带值，保持 `/1`。

**发射**。浮点不论字面量、local 还是运行时表达式，都走 typed 标签；删除 `py_lit_locals`（F-2 的根因）。浮点字面量的位型编译期已知，是帧里的静态 typed 文本。其余浮点是对冻结源码 helper `__py_chunk_d(v)`/`__py_chunk_f(v)` 的一次普通调用，因此操作数恰求值一次并保持源序。helper（`__py_chunk_d`、`__py_chunk_f`、`__py_chunk_words`、`__py_chunk_f64`、`__py_chunk_f32`）以普通 Auto 算术读取位型，只显示 i32；唯一无法用源码表达的一步是 `-0.0` 的符号位（IEEE 比较看不出它，Auto 的浮点 `/` 对零除数陷阱），由 intrinsic `__py_sign_bit(double) -> i32` 提供，沿 `__py_f64_bits` 先例在 HIR→MIR 内联发射，不加 MIR 指令或 wire tag。解析器只在 `__py_chunk_f64` 体内放行该名字：用户调用它按未知被调方拒绝，与 1026 的两个重建 intrinsic 一致。

**边界值合同**。有限数（含次正规）与有符号零两端逐位保留。`+/-inf` 逐位保留。NaN 以 NaN 通过，payload 不主张：AV 传规范静默 NaN（`0x7FF8000000000000`、`0x7FC00000`），A2PY 传原生 NaN。

**A2PY**。事实带字面量则发字面量，否则按 span 取受检表达式原生 lowering；`py_find_value` 跳过全部 py 车道帧 helper。解析器给一元节点的是其操作数的 span，两者同 span：与实参 span 相同的表达式里有一元节点时，只有该节点所在取负链的链底不是 Python 对象才取最外层的那个（后序遍历的最后一个）；链底是 Python 对象时不取任何一元节点，回落到 py 操作调用、局部变量读取、字面量、其他的优先级（r3，R-01；链底守卫是 r5 修复 R-15 后的现行规则，见 §9「work r5」与 A2X 规范）。取操作数会把一元负号丢掉。既有缺口 F-3：`lower_num_bin` 对 F32 的 `+ - * /` 不舍入（`a2x_fdiv` 直接返回双精度），f32 类型的算术结果在 A2PY 上不是 f32 值。§5.1 要求“先按 Auto 原有规则完成运算”，故本计划让 A2PY 的 F32 算术逐运算经新增的 `a2x_f32_round` 舍入（ties-to-even，溢出为 `+/-inf`；既有 `a2x_cast_f32` 在溢出时抛 `OverflowError`，不合 Auto 语义），py 调用边界的 f32 操作数再显式过同一次舍入。**这条舍入属于 A2PY 目标自身，对全部 A2PY 程序生效，不以 `use.py` 为条件**（r3）。程序计算的 Auto 文本作为实参时经新增的 `a2x_py_text` 以它所保存的 `str` 传入；文本字面量仍按字面量发射。

**身份与兼容表**（冻结）：

| 场景 | 承载机制 | 结果 |
|---|---|---|
| 旧工件（`/1` 帧、`F:` 字面量）配旧 provider | 代码未动 | 按原样运行 |
| `/1` 帧字节到达新 shim/provider | 解码器同时接受 `/1` 与 `/2`，旧解码模式只按帧版本选择 | 值与原先相同 |
| 新工件配未声明新能力的旧 provider | `py_call`/`py_getitem`/`py_binop` 的注解升为 `frame=p967.frame/2`；注解是 provider manifest 内容，`content_version` 随之变化，stub `lookup` 的身份轴不匹配 | 用户调用前具名 `foreign.signature`/`host.unauthorized` |
| 已准备的 provider 闭包过期 | `load_prepared_py_bridge` 核 provider 摘要、shim 摘要、rustc 身份与 DLL SHA256 | 具名 `foreign.prepared: … stale or incomplete provider closure` |
| provider、shim 或 helper 源变化 | AV：构建缓存键含 provider 摘要与生成 shim 摘要，帧注解是 provider manifest 内容。A2PY：`a2x_f32_round` 在运行时正文，正文 SHA-256 绑定生成片段的运行时依赖；`a2x_py_text` 在只对 `use.py` 模块追加的对象 helper 中，工程构建身份以 `python-object-runtime` 轴纳入其内容哈希 | 自然失效并重建，工件 SHA 变化 |
| SDK 运行兼容 | `RuntimeCompatibility` 的逐符号签名哈希对带帧声明显式纳入帧版本 | `/2` 需求遇 `/1` 供给是具名 `m2build.compat.symbol-signature`；新工件 SHA 须重新 pin（仓内无 py provider 或 shim 的工件钉值，无需改钉） |
| 中立契约 | `PY_CONTRACT_CONTENT_VERSION` 升版并逐 op 记录帧版本 | 缓存与准入身份换代 |
| 无 `use.py` 的程序 | AV：无 helper 注入、无 py 帧、不加载 provider。A2PY：`a2x_f32_round` 在运行时正文里，正文哈希绑定每个生成片段 | AV 原路径不变。A2PY 的 `float` 算术同样逐步舍入，片段的运行时依赖身份同样换代；`a2x_py_text` 不追加（r3 限定） |

**测试入口**。固定组入口 `cargo t <组>`（nextest 组以 `--debug-tools` 准备，`--run-prepared` 消费同一清单）：共享事实与解析器 `ace-source`，中立契约 `ace-foreign-contract`，AV 帧校验与执行 `ace-av`，内联发射 `ace-lowering`，provider、shim、帧身份与 session `ace-abi`、`ace-python`，A2PY lowering 与运行时 `ace-a2x`，工程装配 `ace-m2-build`，导入器与 shim 前导 `auto-bindgen`，受检 HIR 与共享 lowering 的消费者 `ace-semantics`、`ace-native`；双端真运行 `case-av-language-parity` 与 `case-a2py-language-parity`。Windows 的 Linux/WSL 腿按仓规具名排除。

## 5. 详细设计

### 5.1 值与求值合同

- 共享绑定区分 Auto 浮点值、Python 对象和普通文本。支持运行时 `f32/f64`。先按 Auto 原有规则完成运算，`f32` 按实际 32 位舍入结果精确提升为 Python `float`，`f64` 保持其二进制 64 位值。有限数及有符号零为必验范围。T-01 核查实际源面可产生的次正规值、无穷和 NaN，若可到达，冻结两端一致的保留或具名拒绝合同及见证。NaN payload Bit-for-Bit 只沿已定义的现有规则主张。
- float 可用在受检调用的任一现有位置和关键字槽，沿既有默认、关键字混合与求值顺序。显式读取当前赋值和函数结果，禁把重新赋值后参数替换为先前记录的字面量。未支持的集合/对象形态继续遵循现有准入。
- 运行时浮点的线路要有明确的类型身份，避免沿用 `F:` 文本伪装。新编码消费端须让普通文本 `"F:1.25"` 仍为文本。T-01 固定兼容路径，旧发射件可继续由匹配旧契约的 provider 消费。若新版同时消费旧工件，旧解码模式只由帧的线路版本选择（`/1` 保留 `F:` 字面量行为，`/2` 的文本恒为文本），不按 payload 内容猜测。未声明新能力的 provider 在用户调用前具名拒绝，禁止旧 provider 静默把新值解释成文本。
- A2PY 从 Checked HIR 的真实 float 实参发射对应的值与宽度转换，继续直接调用原生 Python。两端维持真实 Python 的类型、默认值、自定义对象调用以及原异常类和源码位置。编译器或 provider 自身的准入错误仍按各自阶段报告。

### 5.2 工程和训练入口

T-01 比较消费现有 ABI 精确浮点 lane 与向 Python 调用合同追加有版本的 typed 值表示，给出最小全路径可行选择和兼容表。冻结 `BuildIdentity` 对实际读到的源、语义、helper/provider 及版本的失效，`RuntimeCompatibility` 对接口能力的匹配，以及 978 原 provider 工件 pin 选择。两端沿正常工程/执行入口验证，现有声明及锁定装配复用。

训练用例沿 1107 的 CPU Linear/MSELoss/SGD 骨架，将 `lr` 改为普通 Auto 函数接收参数后计算所得。候选表达式为 `var rate = learning_rate(training_step)` 和 `var optimizer = SGD(model.parameters(), lr: rate)`。训练以精确可表示的固定参数与输出阶段钉住 Auto 侧值。先核 `optimizer.param_groups[0]["lr"]` 的实际值，再比较已冻结的 loss 轨迹、至少一个最终参数或预测及形状。保持一份 Auto 源与一份独立原生 Python 参考。

### 规范增量

| ID | 动作 | 目标 | 拟沉淀规则 | 验收 |
|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/python-interop.md` | 记录两种宽度的实参、AV 值编码、旧帧和新版能力边界，保原入站合同 | AC-01/03/04 |
| SD-02 | modify | `docs/specs/auto-a2x/python-ecosystem-integration.md` | 删除运行时 float 阻断项，记录共享语义、A2PY 运行时值（含取负实参，r3）、工程版本和对照；舍入规则不以 `use.py` 为条件（r3） | AC-01/02/03/04 |
| SD-03 | modify | `docs/specs/auto-test-case/feature-organized-corpus.md` | 登记新增单作者语料及双端与原生参考（14 例，r3） | AC-02/04 |
| SD-04 | modify | `docs/specs/ace-migration/m2-host.md` | IF-CALL 帧记录两个线路版本：`/2` 的 typed 浮点标签与三字 payload、`F0Val`/`F0Frame` 类型、帧注解进入接口身份与签名哈希；`/1` 行为不变；`f` 的高 32 位须为零（r3） | AC-03/04 |
| SD-05 | modify | `docs/specs/auto-bindgen/project.md` | `frame=` 注解接受 `p967.frame/1` 与 `p967.frame/2`，生成的 shim 含两版帧类型与同一解码器；其他版本仍具名拒绝；解码器源文本单独成文件（r3） | AC-03/04 |
| SD-06 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | v1 前端边界里「动态浮点关键字实参已证拒绝」指向本次交付，不再作为现状 | AC-04 |

## 6. 测试设计

每个用例一份 `### Auto` 源，同一逻辑路径、字节及 SHA256 供 AV 普通源码执行和 A2PY `run-py`。独立手写 Python 参考用声明解释器直接调用相同库及 API，核观察与固定预期。输入在真实执行时变化，测试避免因常量折叠而漏验。冻结 AV 和 A2PY 解释器/包及各执行入口。缺环境按现有准入处理，实际缺行不满足验收。

| 验证 | 区分性用例与独立观察 | 关联 |
|---|---|---|
| V-01 动态值 | 用参数/计算/重赋值生成不同 `f32/f64`，经 Python `struct.pack` 或等价方式读回 Python float 类型、精确位值和有符号零。覆盖函数/方法、位置与具名槽、嵌套含副作用实参的单次源序。加入需要 f32 先舍入的反例、重赋值后旧值反例以及普通 `F:1.25` 文本、旧字面量和对象参数控制。最外层是一元负号的运行时实参（局部变量、乘积、函数结果、f32、整数、关键字、双重取负）取反后到达 Python，取负的运行时操作数同样触发 `ValueError`（r3，R-01）。 | AC-01/04 |
| V-02 CPU PyTorch 学习率 | 同源训练真实运行，断言优化器 `param_groups[0]["lr"]` 等于两个不同运行输入实际算出的学习率。各次与手写 Python 比较 loss 轨迹和终态参数或预测、形状，以 1107 容差作基准，调错 lr 或两次都拿常量必须失败。 | AC-02/04 |
| V-03 错误和定位 | 让真实 Python 调用以运行时负值触发 `math.sqrt` 的 `ValueError`，分别观察捕获仅一次并继续、未捕获失败和 A2PY 非零退出。父侧核原类、分类 `python.5`、原逻辑路径与半开 UTF-8 字节范围。嵌套实参先失败及源位移的反例须锁定真正失败位置。 | AC-03/04 |
| V-04 兼容与回归 | 正式装配对比新/旧 provider、接口能力缺失、换代或实际 helper 变化失效，以及无 Python 工程仍走原路径。兼容的旧字面量工件按原匹配运行。缺能力的新工件在进入 Python 用户代码前具名拒绝。非规范 `f` payload（高 32 位非零）在三个值槽具名拒绝而不截断，拒绝消息带实际帧版本（r3，R-05/R-06）。保 1107、1110、1126、1127 的实体语料与同源结果。 | AC-03/04 |

正式入口为 `case-av-language-parity`、`case-a2py-language-parity`；完整职责组为 `ace-source ace-foreign-contract ace-av ace-lowering ace-abi ace-python ace-a2x ace-m2-build auto-bindgen ace-semantics ace-native`（nextest 组以 `--debug-tools` 准备，在冻结提交上 `--run-prepared`）。`ace-adapters` 不选：语言侧桥接测试不消费 ACE py 车道、帧线路或本语料。Windows 的 Linux/WSL 腿按仓规具名排除。

## 7. 验收标准

- [x] AC-01（owner_stage=work）V-01 双端实跑验证运行时 `f32/f64` 类型、位值和求值顺序，固定错误控制能使对照失败。
- [x] AC-02（owner_stage=work）V-02 同一份 Auto 训练源经 AV/A2PY 与独立 Python 对照，核真实学习率和结果。
- [x] AC-03（owner_stage=work）V-03/04 保留错误类别和源位置、真实失配拒绝，以及新旧身份和缓存失效边界。
- [x] AC-04（owner_stage=work）共享作者入口、匹配的实际运行和三方参考可复跑，验证原有浮点字面量、Python 对象、普通文本及早期共享语料。
- [x] AC-05（owner_stage=review）独立冻结复审核 AC-01..04、协议选择、dtype/边界判定、引用消耗、下游回归及规范增量。
- [x] AC-06（owner_stage=merge）冻结职责和消费者门、规范沉淀、归档及按仓规保护式清理。

work 验收结果（r3，证据见 §9；review r2 曾将 AC-01 退回，review r3 将 AC-03 退回，修复轮 2 已处理，见 §9「work r4」）：AC-01 由语料 `runtime-values`、`types-and-text`、`order-and-once`、`rounds-first`、`boundaries`、`signed-zero-f32`、`binop-float`、`negated-operands` 的两端检查、`p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text`、20 条检查器变体控制与三组负控覆盖；AC-02 由 `torch-lr-computed` 两端检查、`native_python_torch_computed_learning_rate`、`p1129_t04_engineering_entry_passes_the_computed_learning_rate` 与学习率三变体控制覆盖；AC-03 由 `sqrt-*` 四例两端检查、两个 `p1129_v04_*`、`frame_identity_1129_tests` 与 `each_float_helper_sits_in_the_section_whose_hash_binds_its_identity` 覆盖；AC-04 由 14 例语料的 28 条检查、14 个原生参考对照，以及 1107、1110、1126、1127 既有语料在统一跑中通过覆盖。取负实参的整数形态只在 A2PY 侧钉住，AV 侧是早于本计划的缺陷 D-1129-01，不计入本计划的验收。work r5 修复 review r4 的 R-15 后，AC-01 的取负证据还包括三个矩阵用例（32 格：对象链 12、对象兄弟 9、Auto 数值 11）与 38 格 × 3 个 worker 的对照——对象链 22/22 与修复前逐格相同（输出与层数），Auto 数值链 16/16 等于原生（§9「work r5」）。

## 8. 执行步骤

有原生 TODO 时先登记本节的任务 ID。实施工作树为 `D:/autostack/.wt/lang-1129/auto-lang`，分支 `plan-1129-dev`。实施授权后从 T-01 开始。

- [x] T-01 冻结实参表示与兼容合同
  - owner_stage: work。核 1127/1110/1107、967/978 已交接口。用最小实际调用核两端运行时值、可到达的边界值、精确构建和运行标记，确认旧工件/新能力的收口路径及测试命令。关联 AC-01..04。
  - 证据：§4「T-01 冻结结果」。改动前的专属树 `048698155` 上实跑四组探针，坐实基线缺陷 F-1（AV 把文本 `F:1.25` 解成浮点，两端不一致）、F-2（重赋值后两端都传旧字面量）与现状缺口（运行时浮点被 `source.py-domain` 拒绝）；inf、NaN、两种宽度的次正规都可从源面到达。选定版本化 typed 帧 `p967.frame/2`。原始材料 `D:/autostack/plan-artifacts/plan-1129/t01/t01-baseline.txt`，SHA256 `ed639389cd36149ad75c7c3046cba62bb39dee19160a40b4f8812130c27e943b`。r2：r1 记录的两字 helper 形态探针（`u64::MAX → "2147483647 2147483647"`）实为截断，已由三字位型取代；四组基线探针仍然有效。
- [x] T-02 接通共享浮点和双端调用
  - owner_stage: work。前置 T-01。扩充共享事实与受检调用值，接 AV provider 和 A2PY lowering，关闭 V-01。沿既有工件合同接通所需能力标记。关联 AC-01/03。
  - 证据：共享事实把任意 `f32/f64` 操作数记为 `Float`，只有字面量带字面量值，`py_lit_locals` 删除（`ace-core` conformance `py_float_args_1129` 7 例）。线路为版本化帧 `p967.frame/2`（三字 payload，§4 线路合同）：中立契约 `p1107-py-ops-v2` 逐操作记录帧版本，AV 校验器 `av-core/src/foreign_exec.rs`（`frame_wire_1129_tests` 5 例）、生成 shim 与 provider（旧解码只按帧版本选择）接通；A2PY 按 span 取受检表达式、f32 逐步舍入、计算文本以 `str` 过界（`ace-a2x-python` `py_float_args_1129` 15 例）。AV 发射用 `__py_sign_bit`（§4 发射），只在冻结 helper `__py_chunk_f64` 体内可解析，用户调用按未知被调方拒绝。提交 `77453fa9b`、`de913f15e`。review r2 退回待修：R-01（A2PY 丢弃最外层取负的运行时实参）、R-05（`f` 标签未拒绝非规范编码）、R-06（拒绝消息的帧版本号）；已完成部分保留。r3（修复轮 1，提交 `4ba315c9c`）：`py_find_value` 在同 span 有一元节点时取最外层；`f` 标签要求高 32 位为零，校验器与解码器同时生效（`frame_wire_1129_tests` 6 例、`frame_shim_prelude_tests` 5 例，解码器源文本移入 `crates/auto-bindgen/src/frame_shim_prelude.rs`）；拒绝消息带实际帧版本。冻结提交 `60f4e661b` 上 `ace-a2x` 380/380、`auto-bindgen` 15/15、`ace-abi` 的 `py_float_1129` 全部通过，并集新增红 0（§9）。review r3 退回待修：R-10（两个解码器接受带前导零的字，「一个值只有一种编码」只对 `f` 的高位闭环）、R-11（R-01 的修复让 A2PY 把取负的 Python 对象实参截断为 0 或以原始 `TypeError` 崩溃，是修复引入的回归面）、R-12（本计划改写的拒绝文案说 bool 可过界，运行时 bool 两端都不能）；已完成部分保留。r4（修复轮 2，提交 `3c7aebf5f`）：两个解码器拒绝带前导零的多位字，原因字符串相同；两个实现对同一张样本表 `crates/auto-bindgen/src/frame_wire_samples.rs`（583 行）受测；一元优先只对操作数不是 Python 对象的节点生效，29 格操作数类别矩阵成为永久用例，「哪些操作产生对象」只在 `PyOpKind::yields_object` 定义一次；拒绝文案写明 bool 仅字面量。以上用例在冻结提交的并集里全部通过，两侧变异对照成立（§9「work r4」）。review r4 退回待修：R-15（一元优先的守卫只看一元节点的直接操作数，嵌套取负压在 Python 对象上时仍是修复轮 1 的行为——截断为 0 或以原始 `TypeError` 退出，没有回到修复前；矩阵没有「取负层数」这一维）；已完成部分保留。r5（修复轮 3，提交 `43d9dc1dc`）：一元优先按整条取负链的链底判断（`py_unary_bottom`），三个表驱动用例固化 32 格（对象链 12、兄弟 9、Auto 数值 11）；38 格 × 3 个 worker 的对照里对象链 22/22 与修复前逐格相同、Auto 数值 16/16 等于原生，三种撤规则变异各自变红并恢复。冻结提交 `43d9dc1dc` 上 11 组并集 4166 执行、基线 51、非基线 0；语料组同提交已跑，AV 侧唯一不 PASS 的是 torch 墙钟例（§9「work r5」）。
- [x] T-03 交付同源训练及异常
  - owner_stage: work。前置 T-02。新增唯一 Auto 训练与标量/异常用例及原生 Python 参考，冻结输入、运行身份、观察和反例，关闭 V-02/03。关联 AC-02/03/04。
  - 证据：语料 `crates/auto-lang/test/cases/language/script/python-float-args.case.ad` 共 13 例，由独立原生参考 `ace-m2/python-1129/float_args_ref.py` 生成冻结预期，源码 SHA256 逐例钉在 `ace-a2x-python` 测试里；语料可由作者源与参考逐字节重新生成。冻结提交统一跑：AV 13/13 PASS、A2PY 13/13 PASS（§9 计时表）。V-02 区分性控制见 §9。提交 `77453fa9b`。review r2 退回待补：语料缺「最外层取负的运行时浮点实参」用例（R-01 的永久回归）；已完成部分保留。r3：语料增至 14 例，新增 `negated-operands`（取负的浮点实参：局部变量、乘积、函数结果、f32、关键字、双重取负，以及取负操作数的 `ValueError`），其余 13 例重新生成后逐字节不变；冻结提交 `60f4e661b` 上 AV 14/14、A2PY 14/14 PASS。整数形态不进语料，由 `generated_python_negates_an_int_argument` 只在 A2PY 侧钉住——AV 对程序算出的负整数实参另有早于本计划的缺陷（D-1129-01，§9）。提交 `4ba315c9c`、`60f4e661b`。
- [x] T-04 验证兼容与真实消费者
  - owner_stage: work。前置 T-02/03。关闭 V-04，确认双端 Case、按实际触面准备的职责组及工程入口，写入拟沉淀的规范差额。关联 AC-03/04。
  - 证据：`ace-abi` `p1129_v04_a_stale_prepared_closure_is_rejected_by_name`、`p1129_v04_a_provider_without_the_typed_frame_is_refused_before_the_call`；`ace-m2-build` `frame_identity_1129_tests`（帧版本进入带帧符号的签名哈希，未声明帧者不变）与 `p1129_t04_engineering_entry_passes_the_computed_learning_rate`；`ace-a2x-python` `runtime::plan1129_tests::each_float_helper_sits_in_the_section_whose_hash_binds_its_identity`（两个 A2PY helper 各自在以其哈希绑定身份的那一节）；`ace-core` `the_float_helpers_belong_to_the_py_lane_only`（无 `use.py` 的程序不注入 helper、不产生新帧）。职责组选择、红例分类与检查器控制见 §9。规范增量 SD-01..06 已在专属树提交（`f3c9565a9`），r2 入表。review r2 退回待修：R-02（计划新增行上的一条编译警告）、R-03 与 R-04（两处规范陈旧句）、身份表「无 `use.py` 的程序」一行需限定（A2PY 的 f32 逐步舍入与 `use.py` 无关）；已完成部分保留。r3：以上四项已修（规范句与身份表在 `4ba315c9c`、`60f4e661b`）；最终提交 `60f4e661b` 上的完整冻结并集（11 组加两个语料组）已跑，新增红 0；计划新增行上编译警告 0 条、rustfmt 差异 0 处；20 条检查器变体控制在该提交上重放，全部仍被判掉（§9）。债务候选 D-1129-01..07 已登记。review r3 退回待修：R-13（A2X 规范里的代码示例被写成弯引号 `pack(“>d”, -rate)`）、R-14（work r3 记录有三处与原始结果不符，见 §9）；已完成部分保留。r4（提交 `3c7aebf5f`）：规范里的代码示例改回 ASCII 引号，「取负实参」限定为 Auto 数值操作数，两份规范的已知边界写明对 Python 对象取负（D-1129-09），「一个值只有一种编码」写明最短形式；规范改动行自检脚本在本轮改动之前报出那处弯引号（1 处），现在 0 处。work r3 的三处已在 §9「work r4」订正；对账脚本改为只按事件名计数并重算。冻结提交 `3c7aebf5f` 上：语料组 14/14 × 2，本计划的全部用例通过，`ace-abi`、`ace-a2x`、`ace-m2-build` 等组与基线逐名相同；`ace-av` 与 `ace-native` 有时间预算类的非基线失败，运行与重放期间机器被其他会话占到约 40% CPU，空闲时的隔离重放待补（§9「work r4」）。债务候选 D-1129-01..11 已登记。r5：`43d9dc1dc` 上 11 组并集与两个语料组已在安静机器上跑完（非基线失败 0），torch 语料例仍撞 2 秒 AV 墙钟，隔离重放见 §9「work r5」。
- [x] T-05 独立复审
  - owner_stage: review。前置 T-04。核冻结身份及全部 work 验收，复核兼容、错误和下游实际消费。关联 AC-05。
  - 复审提示（r2）：逐条核 §4 现行冻结合同与实现一致；master 自实施基面前进，4 个共享源文件文本重叠（§9「证据复用」），以专属树当前提交为冻结对象，同步 master 归合入。
  - 复审提示（r3，修复轮 1 之后）：只核受影响项——R-01..R-06 的闭环与负控、冻结提交 `60f4e661b` 的并集对账、D-1129-01 的处置（不在本计划修、语料收窄到浮点、整数形态只在 A2PY 侧钉住）是否成立。
  - 复审提示（work r4 之后）：只核 R-10..R-14 的闭环、两侧变异对照、诊断里列出的三行 oracle 调整是否落地，以及修复是否引入新的回归面。冻结提交 `3c7aebf5f` 的并集是在带负载的机器上跑的：`ace-av` 与 `ace-native` 的时间预算类失败需要在空闲时隔离重放后才能定为环境归因（§9「work r4」）。R-01 对 Auto 数值操作数的闭环、D-1129-01 的处置沿用 review r3 的结论。
  - 复审提示（修复轮 3 之后）：只核 R-15——规则按整条一元链的表述与实现、矩阵新增的「取负层数」各格是否钉在实测的修复前输出、去掉守卫时新格是否变红、A2X 规范那一句是否对所有层数成立、修复是否再引入回归面。R-10、R-12、R-13、R-14 的闭环沿用 review r4 的结论；语料组与 20 条检查器变体已在安静机器上用冻结工具重放（§9「review r4」）。work r5 的冻结对象是 `43d9dc1dc`：11 组并集与两个语料组已在安静机器上跑过，非基线失败 0；AV 侧的 `torch-lr-computed` 仍撞 2 秒墙钟，隔离重放的实测与判读见 §9「work r5」，不按代码失败处理的话须在复审记录里写明依据。
- [x] T-06 合入归档
  - owner_stage: merge。前置 T-05 pass。跑冻结并集、核正式合入和规范来源、归档并按仓规清理。关联 AC-06。
  - 合入提示（r2）：先把 master 同步进分支，同步后的冻结并集是最终门的输入。

## 9. 复审记录

2026-11-23，`stage: new | PLAN-1129 r1 | status: drafting | progress: 0/6 | outcome: pass | next: work T-01（待实施授权）`。已核交付基线、相关边界与源码入口；本轮仅有文档证据。执行合同 T-01..06、AC-01..06。

### work r1（2026-11-24）

`stage: work | plan_id: PLAN-1129 | plan_revision: r1 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | code_commit: 77453fa9b（实现、测试、语料）→ f3c9565a9（canonical 规范增量）→ de913f15e（`__py_sign_bit` 准入修复）→ 6c05a257a（AV 语料用例拆分）→ 481f0a31d（helper 身份归属测试，当前 HEAD） | dependency_revisions: 同组 auto-down `456c86a`（只读，未改） | task_ids: T-02、T-03、T-04 已实现并在冻结提交上验证；T-01 冻结合同文本与实现有偏离，待有界修订 | evidence: 本节下列各表，原始结果在 `D:/autostack/plan-artifacts/plan-1129/` | blockers: 无待用户裁决项；偏离 D-1..D-3 与规范补列 SD-04..06 见 §10 | next_responsible_role: new（有界修订 r2）→ work → review`。

**授权与续作对账**。2026-11-23 用户以 `/goal 完成计划1129` 授权（§4）。本轮接手时专属树内有上一会话未提交、未记账的 T-02..T-04 实现；其日志所用的 `av-worker.exe`、`a2x-worker.exe`（21:31–21:32 构建）早于 `av-core/src/foreign_exec.rs`（21:55）与语料（22:29）的最后修改，故旧日志只作参考，以下证据均在提交后的冻结状态上重新取得。本轮补齐两项：`__py_sign_bit` 的准入收窄到冻结 helper（此前用户在 `use.py` 程序里可直接调用，AV 运行并输出 `1`，A2PY worker `Failure(2)`）；AV 语料用例由单个 13.035 秒的测试拆为逐案例测试。

**本轮实跑**。固定组入口 `scripts/fixed-tests.py`（即 `cargo t` 驱动透传的同一入口，nextest 带 `--no-fail-fast`）。

| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| 默认工具档 13 组 `--prepare-only`（`de913f15e`），第 1 次 | 工具 `store-gc` 的 rustc 以 `STATUS_HEAP_CORRUPTION` 退出，未完成（同机另有会话在编译） | 786.6 s | — | — | — | 13m06s |
| 同上，第 2 次 | PREPARED 13；两轮累积使 `ace-source` 条目的 5 个工具收据哈希陈旧 | 319.1 s | — | — | — | 5m19s |
| 同清单 `--run-prepared`（00:16） | 入口 fail-closed：`PREPARATION_REQUIRED: tool artifact missing or tampered`，未执行 | 复用 | — | — | — | 0.35 s |
| 清 `target/test-groups/prepared` 后单轮 `--prepare-only`（同提交） | PREPARED 13；19 条收据与 11 个归档哈希一致 | 42.2 s | — | — | — | 42.6 s |
| 默认工具档 13 组 `--run-prepared`（`de913f15e`，空闲机器 00:33） | 退出 1，见「红例分类」 | 复用 | 未单列 | 未单列 | 未单列 | 517.3 s |
| 调试工具档（`--debug-tools`）11 个 nextest 组 `--prepare-only`（`de913f15e`） | PREPARED 11，`run-debug`，18 条收据与归档哈希一致 | 51.8 s | — | — | — | 52.4 s |
| 同清单 `--run-prepared` 第 1 次（空闲机器 01:02） | 退出 1，见「红例分类」 | 复用 | 未单列 | 未单列 | 未单列 | 411.7 s |
| 同清单 `--run-prepared` 第 2 次（原意 `--test-name scheduler_runs_are_repeatable` 单测重放；入口未按名过滤，实为整组重跑） | 退出 1；除 `ace-native` 外与第 1 次逐名相同 | 复用 | 未单列 | 未单列 | 未单列 | 397.2 s |
| 调试工具档 `ace-abi ace-python --prepare-only`（`6c05a257a`） | PREPARED 2，哈希一致 | 16.8 s | — | — | — | 16.8 s |
| 同清单 `--run-prepared`（空闲机器 01:19） | `ace-abi` 368 执行：340 过、28 红、8 排除；`ace-python` 12/12 PASS | 复用 | 未单列 | 57.7 s / 1.9 s | 未单列 | 72.6 s |
| 调试工具档 `ace-a2x --prepare-only`（`481f0a31d`） | PREPARED 1，哈希一致 | 7.9 s | — | — | — | 7.9 s |
| 同清单 `--run-prepared`（空闲机器 01:31） | `ace-a2x` 377 执行：377 PASS、7 排除，退出 0 | 复用 | 未单列 | 82.7 s | 未单列 | 103.9 s |

等待其他会话的构建与测试让出机器：四次空闲窗口的有界轮询合计 1105 s（208 + 312 + 311 + 274），与门的墙钟分列。工具档位：调试工具档把 `release` 工具改按 debug、原生运行时改按 `runtime-dev` 准备，是本仓现行做法（测试纪律、PLAN-125 及同日 PLAN-117 的全部 `ace-native` 回执）；语料组所用的 case 工具（`case-parent`、`av`、`a2x-python-debug`、`av-providers`）本就是 debug worker。

原始结果在 `D:/autostack/plan-artifacts/plan-1129/`，SHA256 前 16 位：`t03/union-01/results.json` `ce44e5b0f9e585fe`；`t03/union-02-debug/results.json` `4122b29a07448539`；`t03/replay-sched-repeatable/results.json` `74dbfc3d1ebabbe3`；`t03/union-03-abi-debug/results.json` `d01735fddbfb27f5`；`t03/union-02-debug-failures.json` `6ccf3de8e928cf4d`；`t03/union-04-a2x-debug/results.json` `d90b1bafc15f660c`；`t02/lr-controls.txt` `df48b54adc788836`；`t03/mutants-av.txt` `8fad3c5eaf286acf`；`t03/mutants-av-warm.txt` `dcda02a4ee27cb2d`；`t03/mutants-a2py.txt` `2d43f172112e7495`；`t03/torch-av-replay.txt` `ee6f1b97700671cc`；`t03/torch-wall-av.txt` `e36648512e3846b9`；`t03/user-signbit-*-before.txt` 与 `-after.txt`（用户调用探针前后）。

**红例分类**（逐名对照 PLAN-1123 `4a2054466`、PLAN-1127 的基线清单，以及 PLAN-117 同日不含本计划提交的树上的运行）。

| 组 | 结果 | 归因 |
|---|---|---|
| `ace-source` | 1002 执行：997 过、5 红、1 排除 | 5 红全为 `text_record_source::*`，失败码 `verify.double-move`，与 PLAN-1127 基线同集 |
| `ace-av` | 1634 执行：1621 过、13 红、23 排除 | `av_method_modes`、`av_method_owned`×4、`av_owned_first_case`、`av_owned_recipe`、`av_test_batch::c_data_profile`、`component_event_sessions_1106::v02b`、`resource_915`×3、`layout::tests`，与 PLAN-1123 逐名清单一致（D-1055-BASE-01、D-1088-LAYOUT-01） |
| `ace-abi`（`6c05a257a`） | 368 执行：340 过、28 红、8 排除 | 28 红全为 `source_foreign` 族（D-1109-ABI-C-RUNTIME-PIN），无其他失败 |
| `ace-native`（第 2 次） | 330 执行：325 过、5 红、19 排除 | 5 红为 `conformance` 既有集，与 PLAN-117 同日调试档运行的 5 红逐名相同。第 1 次另有 `sched_m2_1067 scheduler_runs_are_repeatable` 红（`scheduler runtime: prep lock: 拒绝访问 (os error 5)`，预备锁瞬时竞争），第 2 次 PASS 6.404 s；首红保留，环境归因 |
| `case-av-language-parity`（`de913f15e`） | 564 执行：562 PASS、2 FAIL、213 NOT_RUN、161 SKIP | 2 红为 `av.parity.1093.explicit-c-library`、`c-header-spelling-rejected`（1093 C 闭包既有债，PLAN-1125 记录在案） |
| `ace-foreign-contract` 59、`ace-lowering` 9、`ace-python` 12、`ace-a2x` 376、`ace-m2-build` 50、`auto-bindgen` 10、`ace-semantics` 295、`case-a2py-language-parity` 27 | 全部 PASS | — |

**本计划新增红：零。** 职责组选择：`ace-source`（事实、解析器、helper）、`ace-foreign-contract`（中立契约）、`ace-av`（帧校验器与 AV 执行）、`ace-lowering`（符号位内联发射）、`ace-abi` 与 `ace-python`（provider、shim、帧身份、session）、`ace-a2x`（A2PY lowering 与运行时、语义视图、store 过滤）、`ace-m2-build`（签名哈希、工程入口）、`auto-bindgen`（导入器与 shim 前导）、`ace-semantics`（受检 HIR 与事实的消费者）、`ace-native`（共享 HIR→MIR 适配器的消费者）、两个语料组。排除 `ace-adapters`：语言侧桥接测试只含旧引擎自己的 `use.py` 桥，不消费 ACE py 车道、帧线路或新语料。

**证据复用**。`6c05a257a` 相对 `de913f15e` 只改 `experimental/ace/crates/abi/tests/cases/py_float_1129.rs`；`481f0a31d` 相对 `6c05a257a` 只在 `ace-a2x-python` 的 `runtime.rs` 追加一个 `#[cfg(test)]` 模块。共用前者的 `ace-abi`、`ace-python` 已在 `6c05a257a` 重跑，`ace-a2x` 已在 `481f0a31d` 重跑；其余 nextest 组沿用 `de913f15e` 的两次调试档结果，语料组沿用 `de913f15e` 的结果（语料文件与两端 worker 自 `de913f15e` 起未变）。复审与合入的冻结并集在最终提交上重跑。本轮期间 master 前进了 10 个提交（PLAN-117 落主），与本分支在 `a2x-core/src/semantic_view.rs`、`core/src/source/adapter.rs`、`core/src/source/resolve.rs`、`lowering/src/hir_adapter.rs` 四个共享源文件上文本重叠；`git merge-tree master plan-1129-dev` 预演无冲突。文本无冲突不等于语义兼容，同步 master 之后的验证归合入阶段的冻结并集。

**新增用例**（每个测试取其实际执行的各次统一跑中的最大值；首次 13.035 s 的单个语料测试已拆分，见上）。

| 新用例 | 验证的行为与 oracle | profile | 实测秒 | 结果 / ≤10 s |
|---|---|---|---|---|
| `ace-core::every_float_operand_is_a_float_and_only_a_literal_carries_a_literal` | 每个 f32/f64 操作数记为 `Float`；只有字面量带字面量值，重赋值的局部变量不带 | debug/test | 0.195 | 通过 |
| `ace-core::value_carrying_frames_speak_version_two_and_valueless_frames_stay_on_one` | 调用/索引/二元帧为 `/2`，导入帧与仅接收者帧为 `/1`；浮点字面量是静态 typed 文本，不再发 `F:` | debug/test | 0.284 | 通过 |
| `ace-core::the_typed_payload_is_three_bounded_words_with_one_spelling_per_value` | `<w2> <w1> <w0>` 的 21/21/22 位拆分与还原，一个值一种拼写 | debug/test | 0.186 | 通过 |
| `ace-core::unsupported_value_kinds_stay_a_named_rejection_at_their_own_span` | `1i64` 操作数在自身 span 具名 `source.py-domain` 拒绝；f32/f64、参数、函数结果放行 | debug/test | 0.291 | 通过 |
| `ace-core::the_float_helpers_belong_to_the_py_lane_only` | 无 `use.py` 的程序不注入任何 `__py_` helper、无 py 计划；有则注入五个浮点 helper | debug/test | 0.199 | 通过 |
| `ace-core::the_sign_bit_primitive_is_not_user_callable` | 用户调用 `__py_sign_bit`（五种形态）按未知被调方拒绝 | debug/test | 0.316 | 通过 |
| `ace-core::the_aggregates_entry_admits_runtime_float_operands` | Aggregates 语法入口同样产出 `Float` 事实与 `/2` 帧 | debug/test | 0.138 | 通过 |
| `ace-core source::py::tests::the_sign_bit_owner_is_the_one_helper_that_calls_it` | 冻结 helper 源里 `__py_chunk_f64` 定义一次并持有 `__py_sign_bit` 的唯一调用点 | debug/test | 0.153 | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::version_one_keeps_exactly_its_five_tags` | `/1` 帧五种标签不变，`d`/`f` 是未知标签 | debug/test | 0.146 | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::version_two_admits_typed_floats_in_every_value_slot` | `/2` 帧在接收者、位置、具名槽接受 typed 浮点，`/1` 规则不变 | debug/test | 0.122 | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::a_typed_float_payload_is_three_bounded_decimals` | payload 必须是三个有界十进制字；缺失、多出、非十进制、越界各有具名原因 | debug/test | 0.016 | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::an_unknown_wire_version_is_rejected_before_its_body` | `/3`、`/12`、空头部在帧体之前具名拒绝 | debug/test | 0.015 | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::the_decoder_and_the_admission_accept_the_same_bytes` | 准入判定的样本与 shim 解码器是同一字节集 | debug/test | 0.154 | 通过 |
| `ace-foreign-contract py_ops::tests::exactly_the_value_carrying_ops_speak_the_typed_frame` | 中立契约里恰有三个带值操作声明 `/2`，其余 `/1`，无未知版本 | debug/test | 0.143 | 通过 |
| `ace-abi::py_float_1129::p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text` | 手工 `/2` 帧经真 provider 与 CPython：f64/f32 精确位值、f32 加宽、`F:2.25` 在 `/1` 为浮点而在 `/2` 为文本（TypeError）、具名槽位值、有符号零、最小次正规、inf/NaN、畸形 payload 派发前 `foreign.frame` | debug/test | 1.137 | 通过 |
| `ace-abi::py_float_1129::p1129_corpus_lists_every_case_in_file_order` | 语料案例清单与文件逐项一致，每例都有 AV 探针 | debug/test | 0.066 | 通过 |
| `ace-abi::py_float_1129::p1129_v04_a_stale_prepared_closure_is_rejected_by_name` | provider 摘要、shim 摘要、rustc 身份、DLL SHA256 任一不符即具名 `foreign.prepared` 或 `DLL digest mismatch` | debug/test | 0.596 | 通过 |
| `ace-abi::py_float_1129::p1129_v04_a_provider_without_the_typed_frame_is_refused_before_the_call` | 按 typed 接口身份准入的声明向旧身份 provider 派发即 `foreign.signature`，callee_count 不变 | debug/test | 0.314 | 通过 |
| `ace-abi::py_float_1129::p1129_t02_neutral_frames_match_the_live_provider` | 中立契约与活 provider 逐操作帧版本一致，内容版本 `p1107-py-ops-v2` | debug/test | 1.134 | 通过 |
| `ace-a2x-python generated_python_passes_the_runtime_value_and_never_a_remembered_literal` | 生成的 Python 以变量传运行时值、字面量为自身位值、重赋值后读当前值、f32 每步舍入，运行输出等于 oracle | debug/test | 0.725 | 通过 |
| `ace-a2x-python generated_python_hands_python_the_text_value` | 计算文本经 `a2x_py_text` 过界，文本字面量仍是字面量，运行输出等于 oracle | debug/test | 0.432 | 通过 |
| `ace-m2-build rust_project::frame_identity_1129_tests::the_frame_version_is_part_of_only_the_typed_symbols_signature` | 一个声明的帧版本由 `/1` 移到 `/2`：只有该符号的签名哈希改变 | debug/test | 0.065 | 通过 |
| `ace-m2-build rust_project::frame_identity_1129_tests::an_undeclared_frame_changes_no_signature` | 未声明帧的符号哈希等于裸事实哈希；声明帧后改变 | debug/test | 0.022 | 通过 |
| `ace-a2x-python runtime::plan1129_tests::each_float_helper_sits_in_the_section_whose_hash_binds_its_identity` | `a2x_f32_round` 只在运行时正文（其 SHA-256 绑定片段的运行时依赖），`a2x_py_text` 只在 `use.py` 模块追加的对象 helper（工程入口的 `python-object-runtime` 轴）；二者不串节，无 `use.py` 的模块不携带对象 helper | debug/test | 0.009 | 通过 |
| `ace-m2-build p1129_t04_engineering_entry_passes_the_computed_learning_rate` | 978 工程入口编译同一份 PyTorch 源，生成的 Python 以 `a2x_f32_round(变量)` 传 lr 并复现 oracle，优化器自报 lr 等于两次运行各自算出的值 | debug/test | 3.798 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_runtime_values` | 语料 `runtime-values` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.355 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_types_and_text` | 语料 `types-and-text` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.346 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_order_and_once` | 语料 `order-and-once` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 1.132 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_rounds_first` | 语料 `rounds-first` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.670 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_boundaries` | 语料 `boundaries` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 1.826 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_signed_zero_f32` | 语料 `signed-zero-f32` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.364 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_binop_float` | 语料 `binop-float` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 1.839 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_containers_and_binop` | 语料 `containers-and-binop` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 1.459 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_torch_lr_computed` | 语料 `torch-lr-computed` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 4.795 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_sqrt_runtime_negative` | 语料 `sqrt-runtime-negative` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.301 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_sqrt_uncaught_fails_as_written` | 语料 `sqrt-uncaught` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle；以具名语言错误失败且无部分输出 | debug/test | 0.303 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_sqrt_nested_first` | 语料 `sqrt-nested-first` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.350 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_sqrt_nested_shifted` | 语料 `sqrt-nested-shifted` 经真实 session 的普通语言入口在 AV 运行，逐行匹配冻结 oracle | debug/test | 0.355 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_runtime_values` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：参数/计算/重赋值/函数结果产生的 f32/f64 经 `struct.pack` 读回精确位值与有符号零 | debug/test | 0.138 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_types_and_text` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：Python 实际收到的类型（`__class__.__name__`、`repr`）；普通文本 `F:1.25` 保持文本；对象与文本值实参 | debug/test | 0.197 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_order_and_once` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：嵌套带副作用实参的单次求值与源序；运行时负值触发 `ValueError` 捕获一次后继续 | debug/test | 0.105 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_rounds_first` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：f32 先按 32 位舍入再提升（十个 `0.1f` 累加为 `3f800001`）；f64 不舍入 | debug/test | 0.129 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_boundaries` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：无穷、NaN、两种宽度的次正规与有符号零从源面到达 Python | debug/test | 0.106 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_signed_zero_f32` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：程序算出的 f32 负零、加宽后的负零、正零与字面量负零的精确位值 | debug/test | 0.108 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_binop_float` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：`py_binop`：Python 对象运算符右侧的运行时 f32 与 f64 操作数（`fractions.Fraction`） | debug/test | 0.133 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_containers_and_binop` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：绑定方法 `get/setdefault` 与索引键里的浮点；对象 `+`/`*` 的文本与整数操作数 | debug/test | 0.214 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_torch_computed_learning_rate` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：CPU PyTorch：优化器 `param_groups[0]["lr"]` 等于两次运行各自算出的学习率，loss 轨迹与终态预测逐值一致 | debug/test | 3.169 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_sqrt_runtime_negative` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：运行时负值触发 `math.sqrt` `ValueError`：捕获恰一次并继续；原类、`python.5`、半开字节范围 | debug/test | 0.196 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_sqrt_uncaught` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：未捕获的同一异常：AV 以具名语言错误失败且无部分输出，A2PY 非零退出 | debug/test | 0.199 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_sqrt_nested_fails_first` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：嵌套实参先失败：范围锁定内层失败调用而非外层 | debug/test | 0.209 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_sqrt_nested_shifted` | 独立原生参考的输出与 AV、A2PY 两组冻结预期逐行一致，源码 SHA 钉住：同一失败置于 UTF-8 多字节前缀之后：范围按字节长度整体平移 | debug/test | 0.199 | 通过 |

原生新用例 51 个，最慢 4.795 s。

| 语料检查（`auto-case` 普通入口） | AV 所选检查 ms | A2PY `run-py` 所选检查 ms | 结果 |
|---|---|---|---|
| `py.float-args.1129.runtime-values` | 1026 | 1465 | 两端 PASS |
| `py.float-args.1129.types-and-text` | 1010 | 2124 | 两端 PASS |
| `py.float-args.1129.order-and-once` | 979 | 1489 | 两端 PASS |
| `py.float-args.1129.rounds-first` | 1060 | 1449 | 两端 PASS |
| `py.float-args.1129.boundaries` | 992 | 1503 | 两端 PASS |
| `py.float-args.1129.signed-zero-f32` | 938 | 1463 | 两端 PASS |
| `py.float-args.1129.binop-float` | 948 | 1415 | 两端 PASS |
| `py.float-args.1129.containers-and-binop` | 981 | 1501 | 两端 PASS |
| `py.float-args.1129.torch-lr-computed` | 2961 | 3583 | 两端 PASS |
| `py.float-args.1129.sqrt-runtime-negative` | 959 | 1459 | 两端 PASS |
| `py.float-args.1129.sqrt-uncaught` | 1029 | 1480 | 两端 PASS |
| `py.float-args.1129.sqrt-nested-first` | 968 | 1634 | 两端 PASS |
| `py.float-args.1129.sqrt-nested-shifted` | 1889 | 1453 | 两端 PASS |

语料检查 26 条，最慢 AV 2961 ms、A2PY 3583 ms。

**区分性控制**（oracle 是否能判掉错误实现）。

- 改动前基线（T-01，`t01/t01-baseline.txt`）：`F:1.25` 文本在 AV 被解成浮点（F-1）、重赋值后两端都传旧字面量（F-2）、运行时浮点被 `source.py-domain` 拒绝（现状缺口）。
- V-02（`t02/lr-controls.txt`）：原生参考与冻结预期一致；学习率加倍（M1）在第 2/4/6/8/10/12 行、两次都给常量（M2）在第 8/10/12 行、输入恒定（M3）在第 7/8/10/12 行被判掉，差异恰落在优化器自报的 `lr` 与后续 loss/预测上。
- 检查器控制（已备工具，`de913f15e`；`t03/mutants-av.txt`、`t03/mutants-a2py.txt`）：10 个单点变体——Auto 源改错（`lr: 0.03125`）、范围差一字节、范围落在外层调用、错误类别写错、F-1 旧输出、F-2 旧输出、f32 不舍入、丢有符号零、文本值、求值顺序。两端各 10/10 FAIL。AV 的 `x1-constant-lr` 首测为 ERROR（torch 冷缓存撞普通入口 2 秒墙钟，到不了比对；首测保留），缓存热时重放为 FAIL（2525 ms），恰在第 8 行优化器自报的 `lr` 处不符（预期 `0.09375`、实际 `0.03125`）；重放记录 `t03/mutants-av-warm.txt` `dcda02a4ee27cb2d`。
- `__py_sign_bit` 用户调用：修复前 AV 输出 `1`、A2PY worker `Failure(2)`；修复后 AV 为 `source.undefined` 诊断、A2PY 为编译失败退出 2，与其他编译期拒绝同型。

**健康检查**。26 份归档与工具构建日志共 203 条不同的编译器警告，落在本计划新增行上的 0 条，落在本计划所触及文件内的 0 条。非测试代码无新增调试打印；`git diff --check` 无空白错误；测试内的耗时打印与邻近的 1026/1107 测试同一惯例。

**环境风险（如实记录，非本计划引入）**。AV 普通语言入口的执行墙钟为 2 秒，含第三方包导入。`py.float-args.1129.torch-lr-computed` 实测：缓存热时 torch 导入 1.24–1.42 s、整条检查 2.61 s PASS；缓存冷（大规模构建后首次导入）时 7–9 s，以 `av.resource.execution-wall` ERROR，同机负载无关；训练本身极便宜（训练轮数由 2 增到 16，总耗时只增 0.16 s）。该文件在 `language.suite.ad` 中排在 1127 的 torch 用例之后，整组冷跑时先吃冷启动的是既有用例。同一份 AV 源另在 `ace-abi` 以显式 120 s 墙钟经真实 session 运行（`p1129_corpus_av_torch_lr_computed`）。本计划未改墙钟。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r1 | round: 1 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 481f0a31d | next: new`。

### new r2（2026-11-24，有界修订）

`stage: new | plan_id: PLAN-1129 | plan_revision: r2 | status: executing | outcome: pass | next: work（对照 r2 复核后交 review）| changed: frontmatter（plan_revision、supersedes_spec_components、affects）、§3、§4 授权记录与 T-01 冻结文本（线路合同、发射、A2PY、身份表、测试入口）、§5.1 第三条、规范增量表 SD-04..06、§6 入口段、§8 T-01/T-02/T-04 证据与 T-05/T-06 提示、§10 | unchanged: 目标、验收标准 AC-01..06、任务 ID T-01..T-06、仓集合、预算`。

**修订依据**。`work r1` 在冻结提交 `481f0a31d` 上实测后给出偏离 D-1..D-3 与规范补列 SD-04..06（本节上文）。按修订规则，它们属于「设计不再成立而目标与验收不变」（D-1）与「合同内的路径/符号订正」（D-2、D-3、SD-04..06）：保留已完成的工作与 2026-11-23 的授权，只改合同文本，状态保持 `executing`。修订前的 r1 文本副本 SHA256 `c9383c2f3281bab1d8b4a4e50bffdf52cd740772acff66ee2d0cee071893b5c9`（`t04/plan-1129-r1-before-revision.md`）。

**r1 被取代的措辞（留作历史，不再是现行合同）**：

- 线路合同：payload 写为 `d <len>\n<hi> <lo>\n`（f64）与同形的 `f`（f32），位型 `hi << 31 | lo`，`hi`、`lo` 各自小于 2^31，payload 恰为“数字 空格 数字”。两个字各 31 位只能承载 62 位，表示不了符号位与 NaN、无穷的最高两位。
- 发射：写为「两个只在 py 车道出现的 intrinsic `__py_f64_to_bits(double) -> u64`、`__py_f32_to_bits(float) -> u64`」，并记「helper 形态已在 AV 实跑：`2^30 → "0 1073741824"`、`0x80000001 → "1 1"`、`u64::MAX → "2147483647 2147483647"`，0.657 秒」。末例实为截断，该探针作废。
- A2PY：写为「F32 算术逐运算经 `a2x_cast_f32` 舍入」。
- 身份表：写为「构建缓存键是 provider 摘要与生成 shim 摘要；A2PY 构建身份含 Python helper 内容哈希」，未区分两个 A2PY helper 所在的两条身份轴。
- §5.1 第三条：写为「旧解码模式只按已准入的接口身份选择」。
- 测试入口：写为「候选完整组 `ace-source ace-python ace-a2x ace-abi ace-m2-build`，按真实变化调整 `ace-semantics`、`ace-lowering`、`ace-adapters` 与必要下游用例」。
- 规范增量表只列 SD-01..03，frontmatter 的 `supersedes_spec_components` 与 `affects` 未含帧线路波及的 `ace-migration`、`auto-bindgen`。

**受影响的既有验证**。T-01 的两字 helper 形态探针作废，由三字位型的 `float_words`、`frame_wire_1129_tests` 与 `p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text` 取代。T-01 的基线实测与表示选择、T-02..T-04 的验证不受本次文本修订影响：代码自 `481f0a31d` 起未变。

**规范增量 SD-04..06 的落地位置**（专属树提交 `f3c9565a9`）：`docs/specs/ace-migration/m2-host.md` 的 IF-CALL 节新增版本化超集一段；`docs/specs/auto-bindgen/project.md` 的注解导入器条目改为接受两个版本；`docs/specs/ace-migration/replacement-scope-and-evidence.md` 的 v1 边界句指向本次交付。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r2 | round: 2 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 481f0a31d | next: work`。

### work r2（2026-11-24，对照 r2 复核）

`stage: work | plan_id: PLAN-1129 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | code_commit: 481f0a31d59dcf74e69ea4cd137b6218b59d3742（r2 修订后代码未变） | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | task_ids: T-01、T-02、T-03、T-04 | evidence: 本节「合同逐条核对」「就绪记录」与 work r1 各表，原始结果在 D:/autostack/plan-artifacts/plan-1129/ | blockers: 无 | next_responsible_role: review（T-05）`。

**合同逐条核对**（r2 §4 现行文本对 `481f0a31d`）。「直读」指在冻结提交上直接查看源码；「用例」指 work r1 统一跑中通过的断言。

| r2 条目 | 核对方式 | 结果 |
|---|---|---|
| `/2` 是 `/1` 的超集，新增 `d`、`f`；`/1` 保五种标签与旧行为 | 用例 `version_one_keeps_exactly_its_five_tags`、`version_two_admits_typed_floats_in_every_value_slot` | 一致 |
| payload 三字 21/21/22 位，每字有界，一个值一种编码 | 用例 `the_typed_payload_is_three_bounded_words_with_one_spelling_per_value`、`a_typed_float_payload_is_three_bounded_decimals` | 一致 |
| 畸形 payload 是具名 `foreign.frame` 拒绝，派发前生效 | 用例 `p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text`：四种畸形帧均为 `foreign.frame`，`callee_count` 不变 | 一致 |
| AV 校验器与 shim 解码器是同一字节集的双实现 | 用例 `the_decoder_and_the_admission_accept_the_same_bytes`（样本重放）；两处实现同读 | 一致 |
| `/2` 的 `t` 恒为文本；`/1` 的 `F:` 字面量照旧解码 | 用例同上：`F:2.25` 在 `/2` 为 TypeError、在 `/1` 得 1.5；语料 `types-and-text` | 一致 |
| 带值帧发 `/2`，导入帧与仅接收者帧保持 `/1` | 直读：解析器写帧头 7 处，`/2` 在调用、索引（两处）、二元运算，`/1` 在 `py_f64_lo` 静态帧、`py_recv_frame`、`py_getattr`；导入帧在 `py::import_frame`。用例 `value_carrying_frames_speak_version_two_and_valueless_frames_stay_on_one` | 一致 |
| 字面量是静态 typed 文本；其余浮点是对 helper 的一次调用，单次求值、保持源序 | 用例同上（`0.5f32` 字面量是静态 typed 文本）；语料 `order-and-once` | 一致 |
| `py_lit_locals` 删除 | 直读：`experimental/ace/crates` 内无该符号 | 一致 |
| helper 以普通 Auto 算术读位型，只显示 i32 | 直读：`HELPERS` 的 f-string 只渲染 `d` 与 `f`；语料 `boundaries`、`signed-zero-f32` | 一致 |
| `__py_sign_bit(double) -> i32` 内联发射，不加 MIR 指令或 wire tag | 直读：`av-core/src/mir.rs`、`abc.rs` 与 `core/src/hir.rs` 本计划未触及；`emit_py_sign_bit` 只用 `Conv`、`TextDisplay`、`StrLenBytes`、`BorrowEnd`、`Const` 与 `Branch`/`Jump` | 一致 |
| 该名字只在 `__py_chunk_f64` 体内放行，用户调用按未知被调方拒绝 | 直读：放行条件含 `owner_name == PY_SIGN_BIT_OWNER`；用例 `the_sign_bit_primitive_is_not_user_callable`、`the_sign_bit_owner_is_the_one_helper_that_calls_it` | 一致 |
| A2PY：字面量发字面量，其余按 span 取受检表达式；帧 helper 全部跳过 | 用例 `generated_python_passes_the_runtime_value_and_never_a_remembered_literal`；直读：`is_py_lane_helper` 按前缀覆盖全部 `__py_chunk_*` | 一致 |
| F32 算术逐步经 `a2x_f32_round` 舍入（溢出为 inf），调用边界再过一次 | 直读 `lower_num_bin` 与调用边界；语料 `rounds-first`、`boundaries` | 一致 |
| 计算文本经 `a2x_py_text` 过界，文本字面量仍为字面量 | 用例 `generated_python_hands_python_the_text_value` | 一致 |
| `/1` 字节到达新 provider，值不变 | 用例（`/1` 的 `F:2.25` 得 1.5） | 一致 |
| 新工件配未声明新能力的旧 provider：用户调用前具名拒绝 | 用例 `p1129_v04_a_provider_without_the_typed_frame_is_refused_before_the_call`（`foreign.signature`，`callee_count` 不变） | 一致 |
| 过期的已备闭包具名拒绝 | 用例 `p1129_v04_a_stale_prepared_closure_is_rejected_by_name` | 一致 |
| provider 摘要随帧注解变化 | 用例 `v_int_1026_named_domain_rejections`（两份源码的 `provider_digest` 不同）、`p1026_py_bridge_prepared_and_admits` | 一致 |
| 两个 A2PY helper 各在其身份轴上 | 用例 `each_float_helper_sits_in_the_section_whose_hash_binds_its_identity`；`py_project_1107` 钉 `python-object-runtime` 轴 | 一致 |
| 带帧符号的签名哈希纳入帧版本，未声明帧者不变 | 用例 `frame_identity_1129_tests` 两条 | 一致 |
| `/2` 需求遇 `/1` 供给为 `m2build.compat.symbol-signature` | 直读 `compat.rs` 的具名路径与 `compat_selection` 的既有用例（统一跑通过）。本计划新增的是哈希层的用例，没有新增端到端的供需错配用例 | 一致；覆盖边界如左 |
| 中立契约升版并逐操作记录帧版本 | 用例 `exactly_the_value_carrying_ops_speak_the_typed_frame`、`p1129_t02_neutral_frames_match_the_live_provider` | 一致 |
| 无 `use.py` 的程序不注入 helper、不产生 py 帧 | 用例 `the_float_helpers_belong_to_the_py_lane_only` | 一致 |
| 旧工件配旧 provider 按原样运行 | 旧 provider 二进制不由本计划重建，与旧工件配对时两端都未变 | 由构造成立，未单独实跑 |

无遗漏条目，无未决的合同项。

**就绪记录**。

- 工作阶段验收 AC-01..AC-04 的证据映射见 §7，逐例计时表见 work r1。本轮无代码变更，无新增用例。
- 合同内的未决发现、必需的覆盖缺口、未决合同决策：均为空。
- 既有反例与同族路径：学习率三变体、20 条检查器变体、`__py_sign_bit` 用户调用的前后对照见 work r1；它们所走的运行时代码自 `de913f15e` 起未变。
- 本轮实跑：没有执行测试，没有新的用例耗时。原拟在 `481f0a31d` 上把 13 组完整并集重跑一遍，但两次有界等待（543 s、248 s）内机器始终被其他会话占用（PLAN-112 的 `cargo test -p auto-ui-runtime … -j 3` 编译、PLAN-1054 的 `gen-next` 生成器），未启动。其间做过的一次 11 组调试档准备在预检时发现 `ace-source` 条目的 `a2x` 工具收据陈旧（同一源状态此前单独准备过 `ace-a2x`），已清掉已备缓存，未执行。
- 证据复用的依据（文件级）：`de913f15e..481f0a31d` 只改两个文件。`experimental/ace/crates/abi/tests/cases/py_float_1129.rs` 只编入 `ace-abi` 的 conformance 目标；`experimental/ace/crates/a2x-python/src/runtime.rs` 的改动只是一个 `#[cfg(test)]` 模块，只编入该库自己的测试，依赖该库的 `ace-av`（dev）、`ace-m2-build`、`ace-abi`（dev）都以普通库方式编译它。`6c05a257a..481f0a31d` 只改后一个文件。语料文件与两端 worker 的非测试代码自 `de913f15e` 起未变。
- 证据身份：`ace-source`、`ace-foreign-contract`、`ace-av`、`ace-lowering`、`ace-m2-build`、`auto-bindgen`、`ace-semantics`、`ace-native` 与两个语料组测于 `de913f15e`；`ace-abi`、`ace-python` 测于 `6c05a257a`；`ace-a2x` 测于 `481f0a31d`（377/377）。
- 冻结并集：最终提交上的完整重跑仍待执行，归复审按需重放与合入（同步 master 之后）。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r2 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 481f0a31d | next: review`。

### review r2（2026-11-24，终审）

`stage: review | review_scope: final | reviewer_identity/limitations: 实现会话（主上下文）负责确定性核查与记录；另起一个全新上下文的复审代理做代码级独立核查与反例探针（general-purpose 代理，模型身份未暴露）。主上下文逐条复核了代理的发现后才下结论。两者都没有执行 cargo 或 nextest | plan_id: PLAN-1129 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | reviewed_commit: 481f0a31d59dcf74e69ea4cd137b6218b59d3742 | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | spec_inputs: 六份规范在复审提交上的 SHA256 前 16 位依次为 python-interop 719ef4713070f7e5、python-ecosystem-integration 10b171b3d962da52、feature-organized-corpus 1e5eeb60c1b550c4、m2-host fcb4c85c88c86a1b、auto-bindgen/project ec6e827232d1635a、replacement-scope-and-evidence 84a1a09e5a0ccaf6；规范增量的冻结副本 SHA256 e67b80c950088ad96044af41c8d629b8e5276744ecab005ab556af3a5a84e05e，实现差异的冻结副本 dee49a86a97b78243d754dae725eee48a58551317e1912dc81848a03d4c22489（`D:/autostack/plan-artifacts/plan-1129/review/`） | acceptance_results: AC-01 fail，AC-02 pass，AC-03、AC-04 partial；T-02 fail，T-01、T-03 pass，T-04 partial | pending_closeout_gates: 同步 master 之后 13 个职责组的冻结并集、规范落地核验、归档与清理 | findings: R-01（高）、R-02..R-06（低或外观），另有观察 R-07..R-09 与一条范围观察 | evidence: 本节下列各表；原始结果在 `D:/autostack/plan-artifacts/plan-1129/`（`review/` 为复审代理的探针、脚本与输出） | next: work（修复轮 1/3）`。

**基线**。计划 r2、状态 `execution_done`；专属树 `plan-1129-dev`，HEAD `481f0a31d…`，基面 `048698155…`，33 个文件 +4221/−229；复审前后树均无脏改动；§9 引用的 14 个证据哈希与磁盘文件一致；计划没有写旧账本。

**验收证据映射**。

| 项 | 结论 | 方法与结果 |
|---|---|---|
| AC-01 | **fail** | 运行时取负的浮点实参在 A2PY 上传成取负前的值（R-01，本轮两端复现）；其余运行时形态两端一致：复审代理 `neg-zero-runtime`、`f64-extremes`、`f32-extremes`、`f32-step-rounding`、`binop-index-runtime` 全部按位一致，主会话的 `extract-then-pass`、`f32-arith-edges` 两组探针两端与原生参考一致 |
| AC-02 | pass | `torch-lr-computed` 两端 PASS（union-01）；学习率三变体被判掉；`native_python_torch_computed_learning_rate` 与 `p1129_t04_engineering_entry_passes_the_computed_learning_rate` 通过（union-04 / union-02）；复审代理在本提交上重放两端 PASS |
| AC-03 | partial | 错误类别与范围由复审代理的 `f64-nested-error-span` 探针（`python.5`，129..147，两端）与四个 `sqrt-*` 语料用例在运行时确认；身份拒绝（`foreign.signature`、`foreign.prepared`）只有日志证据（输入未变的复用）；SDK `/2` 需求遇 `/1` 供给没有端到端用例（R-07） |
| AC-04 | partial | 复审代理在本提交上重放 13 例语料：A2PY 13/13，AV 12/13 加 torch 一例先 ERROR 后 PASS（R-09）；原生参考与既有语料的结果是复用的日志证据 |
| T-01 | pass | 基线记录哈希与文件一致；r2 合同与代码一致，`f` 标签「一个值只有一种编码」在实现上未成立（R-05） |
| T-02 | **fail** | A2PY 操作数查找丢弃最外层取负（R-01）；校验器与解码器未拒绝 `f` 的非规范编码（R-05） |
| T-03 | partial | 交付物成立：13 份作者源由 SHA256 钉住；各失败范围都等于失败调用的半开 UTF-8 字节范围；语料缺取负实参的用例，已并入 T-03 待补 |
| T-04 | partial | 最终提交上的完整冻结并集未重跑；复用理由按文件范围成立（`de913f15e..6c05a257a` 只改一个 abi 测试文件，`..481f0a31d` 只改一个测试专用模块） |

**发现**（每条含复现包）。

| ID | 级别 | 关联 | 违反的规则 | 位置或输入 | 期望对实际 | 已确立的原因 | 关闭用的 oracle 与覆盖边界 |
|---|---|---|---|---|---|---|---|
| R-01 | 高 | AC-01、T-02、T-03 | §5.1「A2PY 从 Checked HIR 的真实 float 实参发射对应的值」：实参的值必须原样到达 Python | 输入 `var rate = 0.2d; print(pack(">d", -rate).hex().to(str))` 等，见 `review/verify-neg.as`；入口 `python tools/a2py_direct.py review/verify-neg.as <out>` 后运行 `a2x_main.py`，或 `corpus_run.py` 跑 `review/verify-neg.case.ad` | 期望 `bfc999999999999a`（AV 与原生 Python 一致），A2PY 实际 `3fc999999999999a`。`-(rate * 2.0d)`、`-twice(1.0d)`、f32 的 `-seed`、具名实参 `round(2.5d, ndigits: -digits)`（得 2.5 而非 2.0）、`sqrt(-x)`（得 2 而非 `ValueError`）同样出错；先绑定局部变量 `var bound = -rate` 则正确 | 生成代码直接传内层值（`[">d", l2_rate]`），取负从未发射。`py_find_value` 在与实参 span 相同的表达式里按优先级选：py 操作调用 0、局部变量读取 1、字面量 2、其他 3，取负节点属于「其他」。该规则在基面逐字相同，整数实参同样中招（`repr(-n)` 得 `3`），所以缺陷早于本计划，本计划把它带到了新交付的运行时浮点上。取负节点为何与内层节点同 span，待修复时以受检 HIR 转储确认 | 新增语料（位置实参、具名实参、f32 与 f64、局部变量、调用结果、括号表达式）在两端通过；`ace-a2x-python` 断言生成的代码发射了取负；整数兄弟路径一并核。AV 端此形态正确，只需修 A2PY |
| R-02 | 低 | T-04（健康） | AGENTS.md §3 健康检查：计划新增的代码零未处理编译警告 | `experimental/ace/crates/abi/tests/cases/f0_int1026.rs:424`（`77453fa9b` 新增）；`t03/s2-abi-corpus-split.log`、`t01/check-ace-3..5.log` | `unused variable: legacy` | work r1 的「新增行警告为 0」是错的：当时的扫描脚本用反斜杠路径与正斜杠路径做后缀匹配，永远对不上。改用按路径分量匹配（`review/warnings_audit.py`，带匹配自检）重扫留存的 21 份编译日志：触及文件内既有警告 6 条（不在新增行），新增行上 1 条 | 删除未用的绑定或使用它，abi 测试目标重编后无该警告。留存的编译日志不含已清理的归档构建，合入阶段的构建需再扫一次 |
| R-03 | 低 | T-04（SD-02） | 规范描述当前行为 | `docs/specs/auto-a2x/python-ecosystem-integration.md:25` | 仍写 content `p1107-py-ops-v1`；代码与同文件新增段落是 `v2` | 1107 的契约描述句在本计划换代时漏改 | 改为 `v2`。文本核对 |
| R-04 | 低 | T-04（SD-01） | 规范句子不自相矛盾 | `docs/specs/auto-av/python-interop.md:147` | 「仍未交：…动态浮点实参（PLAN-1129 已交）」 | 1026 的「仍未交」清单加了括注，而没有把该项移出 | 把该项移出「仍未交」并指向本次交付。文本核对 |
| R-05 | 低 | T-02 | §4 线路合同：每个值只有一种编码，畸形 payload 是具名 `foreign.frame` 拒绝 | `av-core/src/foreign_exec.rs` 的 `check_tag_value`；`crates/auto-bindgen/src/rust_import.rs` 的 `__f0_val`。入口：AV 准入与生成的 shim 解码器 | `f` 的非规范 payload 被接受并静默截断：`0 0 0`、`2097151 0 0` 与 `0 1048576 0` 都解码成 +0.0。发射端总是写规范编码 | 校验器对 `d` 与 `f` 用同一套字界检查，解码器对 `f` 以 `as u32` 截断，`f` 的高位不受约束。代码阅读确认，未实跑 | 校验器与解码器对 `f` 要求 `w2 == 0` 且 `w1 < 1024`，其余具名拒绝；`frame_wire_1129_tests` 与 abi 的帧用例补非规范 `f` 负控（位置槽、具名槽、接收者槽）。`/1` 帧不带 `f`，不受影响 |
| R-06 | 外观 | T-02 | 拒绝消息如实报告 | `av-core/src/foreign_exec.rs`，`p967.frame/1 rejected: …` | `/2` 帧被拒时消息仍写 `/1`；错误码 `foreign.frame` 正确 | 消息里的版本号是写死的常量 | 按实际版本报告，既有帧用例覆盖。只影响消息文本 |

**观察（不构成本计划的阻断项，已登记为债务候选）**。

- R-07：SDK 运行兼容一句（`/2` 需求遇 `/1` 供给是具名 `m2build.compat.symbol-signature`）只由构造成立。唯一的需求生产者是 `ConsumerRequirements::native_program`；所有带帧符号的签名哈希（含 `/1` 的）都变了，仓内没有钉值因此失效。
- R-08：`.as(float)` 收窄溢出时两端都失败，但形态不同：AV 是 `av.trap.float-overflow`，A2PY 以未归类的 `OverflowError` 退出 1。`a2x_cast_f32` 与 AV 的陷阱都早于本计划。
- R-09：AV 的 `torch-lr-computed` 在 torch 冷缓存导入时以 `av.resource.execution-wall` 出错（7770 ms、7960 ms），缓存热后 PASS（2620 ms）。与 work r1 的记录一致。
- 范围观察：A2PY 的 f32 逐步舍入作用于所有 A2PY 程序，与是否使用 `use.py` 无关；身份表「无 `use.py` 的程序 → 原路径不变」一行需要限定。`ace-a2x` 377/377 通过，说明没有既有用例依赖未舍入的结果。
- R-01 的整数兄弟路径（`repr(-n)` 得 `3`）早于本计划。
- 浮点字面量不支持指数记法（`1.0e3` 被判 `source.literal-range`），早于本计划；不带指数的长写法解码与 Python 一致（23 个字面量相对误差全为 0）。

**原始结果对账**（`review/review_evidence.py`）。四次固定组运行加默认档的语料运行共 25 条组级计数声称，与 `results.json` 及 nextest 原始输出逐项吻合；失败集全部落在既有基线族内（`ace-source` 5、`ace-av` 13、`ace-abi` 28、`ace-native` 5 加一例瞬时锁竞争、语料组 2），没有归不到基线的失败。新增用例按基面对账：原生 51 个、语料 13 例 26 条检查，与声称一致，51 个原生用例都有通过的原始耗时记录。

**测试成本**。新增用例最慢 4.795 s（`p1129_corpus_av_torch_lr_computed`），其次 3.798 s、3.169 s；语料检查最慢 AV 2.961 s、A2PY 3.583 s；没有超过 10 s 的。已退役的单个语料测试的首测 13.035 s 保留在 work r1。新增测试不起子进程编译：provider 经只认已备工件的加载器取得，缺了是具名失败；其余子进程是声明的 Python 解释器。

**规范增量审读**。`supersedes_spec_components` 的六条与差异里的六份规范一一对应，`affects` 与它们的目录一致；`new_spec_components` 为空（没有新建规范文档）；`touched_goals` 为空，本计划不改变任何 GOAL 的达成状态。SD-03、SD-05、SD-06 准确。SD-01 除 R-04 外准确，其中「`.to(float/double)` 的提取结果可作实参」一句由主会话的探针在两端确认，但交付语料没有用例钉住。SD-02 除 R-03 外准确，「与 TypeScript 目标既有的每步 round 规则一致」属实（`a2x-ts` 的 `num_bin_expr`，本计划未触及），但未写明舍入改动不限于 `use.py` 程序，并含「此前…」的叙事式措辞。SD-04 的「一个值只有一种编码」对 `f` 标签不成立（R-05）。

**健康检查**。`git diff --check` 无空白错误；新增的非测试代码没有调试打印或 TODO。编译警告见 R-02：触及文件内既有 6 条，计划新增行上 1 条；work r1 的「0 条」结论作废。`cargo fmt --check` 未运行。

**主会话的探针**（已构建的调试工具，无构建；说明见 `review/probes/README.md`）：`extract-then-pass` 两端 PASS（713 ms / 984 ms）；`f32-arith-edges` 两端 PASS（575 ms / 1036 ms），f32 溢出得 ±∞、`inf − inf` 与 `inf × 0` 得 NaN、次正规、下溢得 ±0、f64 溢出，与原生参考一致；语料由作者源与原生参考重新生成后与交付文件逐字节相同（SHA256 `8ed8ef14359a9d239aaf3fb7d44f66ef59afd9000f161e1ccb0b3649bfbdd092`）。主会话犯了三处探针错误并已更正归类（预期块粘行、参考里的字面量量级数错、格式串漏写 `>`），原始记录保留。

**本轮执行与复用**。执行：复审代理 24 个反例用例（两端，每批约 10 s）、本提交上的 13 例语料两端重放（02:43–02:45）、若干单例重放；主会话 6 个探针程序两端、`verify-neg`、`verify-neg2`。未执行：nextest 的定向重放与任何构建——复审代理三次有界等待（5、5、3 分钟）都没有等到空闲窗口，其他会话在跑 cargo、rustc、link 与 Python oracle；51 个新增用例的结果复用哈希核对过的原始日志。复审代理的一次 A2PY 语料运行在十进程争用下以 `0xC0000005` 退出，三次重放通过，未归因。

**路由**。`needs_fix`：状态回到 `executing`，T-02、T-03、T-04 与 AC-01 退回（`current_step` 重算为 1），已完成部分保留；交 work 做修复轮 1/3。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r2 | round: 4 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 481f0a31d | next: work`。

### work r3（2026-11-24，修复轮 1/3）

`stage: work | plan_id: PLAN-1129 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | code_commit: 60f4e661b46cb8d374649c7d2e3e3891bf03ba5c（修复提交 4ba315c9c5920204755f4efc44ee1a50a72a13de，订正提交 60f4e661b…） | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | task_ids: T-02、T-03、T-04 | evidence: 本节各表，原始结果在 D:/autostack/plan-artifacts/plan-1129/t06/ | blockers: 无 | next_responsible_role: review（T-05）`。

**r3 有界修订**（在修复轮内完成，授权沿用 `/goal`，起因是 review r2 的发现）。只订正 §4 冻结文本里与实现不符或漏写的三处，并更新例数；目标、验收标准、任务 ID、仓集合与预算不变。被取代的 r2 措辞：

| 位置 | r2 被取代的措辞 | r3 现行文本 |
|---|---|---|
| §4 线路合同，`f` 标签 | 「`f …` 是 f32，位型是其 32 位模式。」，没有上界；拒绝消息未提版本 | 加上「高 32 位为零（`w2 == 0` 且 `w1 < 1024`），超出即具名拒绝、不截断」；拒绝消息带实际帧版本；解码器源文本单独成文件 |
| §4 A2PY 段 | 「`py_find_value` 跳过全部 py 车道帧 helper。」之后直接是既有缺口 F-3；没有说舍入作用于哪些程序 | 加入取负实参的取值规则；加入「舍入属于 A2PY 目标自身，不以 `use.py` 为条件」 |
| §4 身份表「无 `use.py` 的程序」 | 「无 helper、无 py 帧、不加载 provider / 原路径不变」 | 分 AV 与 A2PY：AV 原路径不变；A2PY 同样逐步舍入，片段的运行时依赖身份同样换代 |
| 规范增量 SD-02..05、测试设计 V-01/V-04、T-03 证据 | 不含取负实参、`f` 上界；例数 13 | 补入；例数 14 |

**闭环表**。

| 发现 | 违反的规则 | 触及路径 | 修复与永久用例 | 结果 | 覆盖边界 |
|---|---|---|---|---|---|
| R-01 | §5.1：实参的值原样到达 Python | `experimental/ace/crates/a2x-python/src/lower.rs` 的 `py_find_value` | 与实参同 span 的表达式里有一元节点时取最外层（受检体按后序列出，最后一个是最外层）；语料 `negated-operands`；`generated_python_negates_an_argument_wrapped_in_unary_minus`、`generated_python_negates_an_int_argument`、`native_python_negated_operands`、`p1129_corpus_av_negated_operands` | 语料两端 PASS；`ace-a2x-python` 该文件 18/18 | 整数形态只在 A2PY 侧：AV 对程序算出的负整数实参另有早于本计划的缺陷（D-1129-01） |
| R-02 | AGENTS.md §3 健康检查 | `experimental/ace/crates/abi/tests/cases/f0_int1026.rs` | 删除未用的绑定。本轮新写的 bindgen 解码器测试模块一度带来 5 条 dead_code 警告，已在该测试模块上限定消除 | 冻结提交 `cargo check --tests`：计划新增行上 0 条 | 触及文件内既有的 5 条不在新增行 |
| R-03 | 规范描述当前行为 | `docs/specs/auto-a2x/python-ecosystem-integration.md` | content `p1107-py-ops-v1` 改 `v2` | 文本核对 | — |
| R-04 | 规范不自相矛盾 | `docs/specs/auto-av/python-interop.md` | 「动态浮点实参」移出「仍未交」，指向本次交付 | 文本核对 | — |
| R-05 | §4：一个值只有一种编码 | `experimental/ace/crates/av-core/src/foreign_exec.rs`；`crates/auto-bindgen/src/frame_shim_prelude.rs`（自 `rust_import.rs` 逐字移出） | `f` 要求高 32 位为零；`an_f32_payload_is_canonical_or_rejected`、解码器 5 例；abi 的畸形帧循环补位置、接收者、具名三槽 | 校验器 6/6、解码器 15/15、abi 帧用例通过 | `/1` 帧不带 `f`，不受影响 |
| R-06 | 拒绝消息如实 | `av-core/src/foreign_exec.rs` 的派发点 | 消息取 `FRAME_HEADERS` 里实际匹配到的帧头；无匹配报 `p967.frame/?` | abi 帧用例断言 `/1`、`/2`、`/3` 三种消息 | 只影响消息文本 |
| 范围观察 | 身份表如实 | 计划 §4；A2X 规范 | 「无 `use.py`」一行按 AV、A2PY 分述；规范写明舍入不以 `use.py` 为条件 | 文本核对；既有用例 `each_float_helper_sits_in_the_section_whose_hash_binds_its_identity` | — |

**负控与变异对照**（把修复撤掉后新用例必须变红；每次撤掉后都按 SHA-256 核对恢复）。

| 撤掉的修复 | 结果 |
|---|---|
| R-01：`lower.rs` 换回 `481f0a31d` 的查找 | 六个 `pack` 实参为 `["l4_rate", "t13", "t19", "a2x_f32_round(l5_seed)", "a2x_f32_round(l5_seed)", "l16_bound"]`，取负深度 `[0, 0, 0, 0, 0, 1]`，期望 `[1, 1, 1, 1, 2, 1]`；整数用例同时变红 |
| R-05：校验器的上界判断改成恒真 | `an_f32_payload_is_canonical_or_rejected` 与 `the_decoder_and_the_admission_accept_the_same_bytes` 变红 |
| R-05：解码器的守卫改成恒真 | `an_f32_with_upper_bits_is_rejected_in_every_slot_not_truncated` 变红 |

**冻结跑中发现并订正的问题**。第一次冻结跑（`4ba315c9c`，源状态 `758a700ee7255554`）对账出一条新增红：`p1129_corpus_av_negated_operands` 在 AV 上以 `foreign.frame: p967.frame/2 rejected: payload not LF-terminated` 失败，语料组同一例为 ERROR，A2PY 端通过。当时语料例含两个整数形态（`repr(-n)`、`round(2.5d, ndigits: -digits)`）。AV 探针共 28 个程序，结论：

- 浮点各形态在 AV 上与原生 Python 逐位一致：`-rate`、`-(rate * 2.0d)`、`-twice(1.0d)`、`-twice(0.1d)`、`-(rate)`、`-(rate + rate)`、f32 的 `-seed`、`-(-seed)`、`-(seed * seed)`，以及 `sqrt(-seed)` 抛 `ValueError`。
- 整数实参只要是程序算出的负数就失败，与表达式无关：`repr(a - b)`、返回负数的函数、先绑定到局部变量、`repr(-n)`、结果为 -10 与 i32 最小值。非负的运行时整数与负字面量 `repr(-3)` 正常。
- 根因是冻结 helper `__py_dlen` 只数非负值的十进制位数（`experimental/ace/crates/core/src/source/py.rs` 的模块注释自述「线路上只携带非负 i32」），负数帧声明的 payload 长度少一位。本计划没有改这两个 helper；1107 放行运行时整数实参后，1026 的这条假设不再成立。

该缺陷早于本计划，也不在「浮点实参」的合同内，按仓规登记为 D-1129-01，没有在本计划里改冻结 helper。订正提交 `60f4e661b`：语料例去掉整数形态，关键字槽改用浮点观察 `complex(real: 1.5d, imag: -two)`（得 `(1.5-2j)`，丢符号则为 `(1.5+2j)`），源码 SHA256 `651e5093f7e2…`；其余 13 例重新生成后与原文件逐字节相同；整数形态由 `generated_python_negates_an_int_argument` 只在 A2PY 侧钉住。订正两点记录：review r2 的 R-01 写「AV 端此形态正确」只对浮点成立，整数兄弟路径当时的探针没有覆盖；语料生成器的预期只来自原生参考、不跑 AV，所以这个问题迟到冻结跑才暴露。

**订正**。review r2 的观察一节称「已登记为债务候选」，当时并没有写入 `KNOWN-DEBT-AND-RISKS.md`。本轮补登记 D-1129-01（AV 上程序算出的负整数实参）、D-1129-02（R-07）、D-1129-03（R-08）、D-1129-04（R-09）、D-1129-05（指数记法字面量）、D-1129-06（默认工具档下 `ace-native` 链接失败）、D-1129-07（对象 helper 在 worker 身份里没有独立轴）。

**冻结并集**（`60f4e661b`，源状态 `74f42807146f`，固定组配置 `a560d21924cbef24`）。调试工具档一次准备 53.3 秒（11 组，按组没有拆分计时），清单收据 65 条与 11 个归档的哈希全部核对通过，`--run-prepared` 不构建；机器等 17 秒后空闲；整命令 385.9 秒，其中发现 56.2 秒、测试 328.3 秒、运行器其余约 1.4 秒。清理没有单独的阶段，未观测。峰值 Job 内存 5.72 GB。

| 组 | 结果 | 执行 | 条件排除 | 发现秒 | 测试秒 | 失败集对账（与 `de913f15e`、`6c05a257a`、`481f0a31d` 的冻结跑逐名比对） |
|---|---|---|---|---|---|---|
| `ace-source` | FAIL（基线） | 1002 | 1（ignored） | 2.3 | 11.5 | 5，`text_record_source::*`，相同 |
| `ace-foreign-contract` | PASS | 59 | 0 | 0.6 | 1.0 | 0 |
| `ace-av` | FAIL（基线） | 1635 | 0 | 4.7 | 46.7 | 14 条记录，13 个用例（`layout_rejects_non_record_and_empty` 在两个目标下各记一次），相同 |
| `ace-lowering` | PASS | 9 | 0 | 1.4 | 2.4 | 0 |
| `ace-abi` | FAIL（基线） | 369 | 0 | 3.9 | 60.1 | 28，全是 `source_foreign` 族（D-1109-ABI-C-RUNTIME-PIN），相同 |
| `ace-python` | PASS | 12 | 0 | 1.6 | 3.8 | 0 |
| `ace-a2x` | PASS | 380 | 4（ignored） | 10.0 | 92.9 | 0 |
| `ace-m2-build` | PASS | 50 | 0 | 12.4 | 17.9 | 0 |
| `auto-bindgen` | PASS | 15 | 0 | 0.6 | 1.1 | 0 |
| `ace-semantics` | PASS | 295 | 0 | 13.2 | 15.7 | 0 |
| `ace-native` | FAIL（基线） | 330 | 0 | 5.5 | 75.2 | 5，相同；上轮瞬时锁竞争的 `sched_m2_1067 scheduler_runs_are_repeatable` 本次通过 |
| 合计 | 新增红 0 | 4156 | 5 | 56.2 | 328.3 | 基线红 51 个用例，逐名相同 |

新增红 0。对账脚本解析 nextest 的 JSON 事件流。它的首版按人读格式去读 stdout，又没有守卫，报出「全部 0 红」；同一份结果里 6 个组的结论是 FAIL，这个空转结果因此被识破。加上「FAIL 组解析不到任何测试事件就拒绝下结论」的守卫并改读事件流之后，第二版仍读错了目录（少了每组一个的序号子目录），守卫拦住了它；订正路径后才有上面的对账。语料组（`case-av-language-parity`、`case-a2py-language-parity`，默认工具档，准备 2.7 秒，收据 7 条，整命令 128.0 秒）：本计划 14 例 AV 14/14、A2PY 14/14 PASS；AV 组另有既有的 2 个 `av.parity.1093.*`。Windows 的 Linux/WSL 腿按仓规具名排除，未执行，不充 PASS。

第一次冻结跑（`4ba315c9c`，源状态 `758a700ee7255554`，准备 209 秒，整命令 386.6 秒）对账出一条新增红，见上文「冻结跑中发现并订正的问题」。那一次 `ace-av`、`ace-a2x`、`ace-m2-build` 的退出码是 124：工作负载自身的退出码是 100、0、0（`ace-a2x` 379 全过，`ace-m2-build` 50 全过），运行器在工作负载退出后计量到仍存活的后代进程 `vctip.exe`（MSVC 遥测进程，`cleanup_complete: true`）。与 PLAN-1098、963、888 记录的现象相同，属环境观察；第二次冻结跑没有复现，三组退出码为 100、0、0。

**本会话实跑**（修复轮 1；「—」表示该阶段不适用或未单独计时）。

| 测试模块/实际范围 | 结果与执行/排除计数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| 冻结并集 `60f4e661b`：11 组，调试工具档，`--run-prepared` | 新增红 0；执行 4156，排除 5 | 53.3 s | 56.2 s | 328.3 s | 无单独阶段 | 385.9 s |
| 语料组 `60f4e661b`：`case-av-language-parity`（565）、`case-a2py-language-parity`（28） | 本计划 14×2 PASS；AV 组另有基线 2 FAIL | 2.7 s | — | 71.7 s、47.4 s | — | 128.0 s |
| 冻结并集与语料组 `4ba315c9c`（第一次） | 新增红 1（AV 新语料例），已订正 | 209 s | — | — | — | 386.6 s、126.6 s |
| `cargo test -p ace-a2x-python --test py_float_args_1129`（串行，直接运行） | 18/18 | 5.4 s 增量编译 | — | 3.1 s | — | 8.6 s |
| `cargo test -p ace-av-core --lib frame_wire_1129_tests` | 6/6 | 3.4 s | — | 0.01 s | — | 3.5 s |
| `cargo test -p auto-bindgen --lib` | 15/15 | 0.9 s | — | 0.01 s | — | 1.1 s |
| 负控与变异对照 4 次（撤掉修复，预期变红） | 每次符合预期：2 或 1 例红 | 每次 ≤3.4 s 增量编译 | — | ≤0.5 s | — | 每次 ≤5.4 s |
| AV 探针 28 个程序，经 `auto-case` 普通入口 | 浮点 15 个（其中 10 个带取负）与原生 Python 逐位一致；程序算出的负整数 8 个复现畸形帧；整数对照 5 个正常 | 复用已备 worker 与 provider | — | 每个约 0.55 s | — | 约 15 s |
| 语料新例修订版的 AV、A2PY 直跑 5 次 | 两端一致 | 复用 | — | 每个 ≤0.7 s | — | 约 3 s |
| 检查器变体控制 20 条（冻结提交重放） | AV 10/10 FAIL、A2PY 10/10 FAIL，符合预期 | 复用 | — | 所选检查合计 AV 7.1 s、A2PY 20.0 s | — | 约 40 s |
| `cargo check --tests`：本计划触及的 9 个 crate | 退出 0；计划新增行警告 0 | 23.8 s | — | — | — | 23.8 s |
| rustfmt 计划行核对与选择性应用（`fmt_scope.py`、`fmt_apply_plan_lines.py`） | 计划新增行差异 78 → 0；每个文件记号序列前后相同 | — | — | — | — | 每次约 20 s |

资源等待：为避开其他会话的构建与测试，修复期间有界等待 484 秒（03:27–03:35）、两次冻结跑之前各 17 秒、警告审计之前不到 70 秒，合计约 10 分钟，不计入上表。复用的证据：学习率三变体（work r1，原生参考层面，不经过编译器）；`ace-source`、`ace-foreign-contract`、`ace-lowering`、`ace-semantics`、`ace-native` 的失败集基线（`de913f15e` 的冻结跑，仅用于对账，本轮已在 `60f4e661b` 上全部重跑）。

**检查器变体控制**（冻结提交上重放）：10 个单点变体两端各 10/10 FAIL，没有一个通过；AV 的 `x1-constant-lr` 缓存热，2525 ms 处在第 8 行优化器自报的 `lr` 上被判掉（预期 `0.09375`，实际 `0.03125`）。

**测试成本**（每个用例自己的 nextest 时间，调试档；语料检查是 case runner 所选检查的时长）。

| 新用例 | 验证的行为 | 实测秒 | ≤10 s |
|---|---|---|---|
| `ace-av-core foreign_exec::frame_wire_1129_tests::an_f32_payload_is_canonical_or_rejected` | 规范 `f` payload 被接受；任一高位被置位的六种拼写在位置、接收者、具名三个槽具名拒绝 | 0.010 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::version_one_decodes_its_five_tags_and_keeps_the_version` | 解码器对 `/1` 五种标签的解码（`include!` 同一份前导文本） | 0.222 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::version_two_decodes_typed_floats_from_their_exact_patterns` | `/2` 的 `d`/`f` 精确还原为 `F64(u64)`/`F32(u32)` | 0.218 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::an_f32_with_upper_bits_is_rejected_in_every_slot_not_truncated` | 解码器对非规范 `f` 在三个槽具名拒绝 | 0.213 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::typed_tags_never_ride_a_version_one_frame` | `d`/`f` 不出现在 `/1` 帧 | 0.118 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::malformed_float_words_keep_their_named_reasons` | 解码器的畸形原因与 AV 校验器逐字相同 | 0.312 | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_negates_an_argument_wrapped_in_unary_minus` | 六个 `pack` 实参的取负深度 `[1, 1, 1, 1, 2, 1]`；关键字与失败调用同样取负；输出等于 oracle | 0.375 | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_negates_an_int_argument` | 整数形态只在 A2PY 侧：`-3`、`0.0` | 0.312 | 通过 |
| `ace-a2x-python::py_float_args_1129 native_python_negated_operands` | 原生参考与两组冻结预期逐行一致，失败范围 `sqrt(-seed)` | 0.071 | 通过 |
| `ace-abi::conformance py_float_1129::p1129_corpus_av_negated_operands` | 语料新例经普通语言入口在 AV 运行，匹配 oracle | 0.363 | 通过 |

改动并重跑的 4 个用例：`version_two_admits_typed_floats_in_every_value_slot` 0.016 s、`the_decoder_and_the_admission_accept_the_same_bytes` 0.017 s、`p1129_corpus_lists_every_case_in_file_order` 0.096 s、`p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text` 0.389 s。新增与改动共 14 个，最慢 0.389 s。语料检查 28 条全部 PASS：`negated-operands` AV 1197 ms、A2PY 1421 ms；最慢是 `torch-lr-computed`，AV 2712 ms、A2PY 3678 ms；其余 AV 899–1519 ms、A2PY 1311–1943 ms。没有超过 10 秒的。新增测试不起子进程编译：解码器测试 `include!` 源文本，A2PY 测试走进程内的 lower，语料与 abi 用例经已备的 worker 与 provider。

**健康检查**。`git diff --check` 无空白错误。`cargo check --tests`（本计划触及的 9 个 crate，22 秒）：计划新增行上编译警告 0 条；触及文件内既有 5 条（`resolve.rs` 4 条、`hir_adapter.rs` 1 条），都不在新增行。rustfmt：本计划新增或改动的行原有 78 处差异（review r2 已指出当时没有跑 `cargo fmt --check`），本轮只对这些行应用，现为 0 处；这些文件里既有的 106 处偏差没有动。应用时每个文件的记号序列（忽略空白、括号与逗号）前后相同才写入；两处 `mod` 声明的重排工具拒绝自动应用，手工完成（`abi/tests/conformance.rs`、`core/tests/conformance.rs`）。新增代码没有调试打印，没有遗留 TODO。

**本会话的探针与工具错误**（均已更正，不是产品缺陷）：关键字槽的观察先选了 `Fraction(numerator: 1.5, …)`，Python 的 float 不是 `Rational`，原生就抛 `TypeError`，两端一致地失败，改用 `complex`；AV 校验器单测里 `f` 关键字样例的长度先写 9、后写 8，帧格式的长度不含结尾换行，实为 7；浮点探针的预期列有 4 处手算错误，以原生 Python 重算为准，AV 全对；`fmt_scope.py` 首版路径没有归一化，报了空转的 0；`compare_runs.py` 两次读错输出文件；时长表脚本的用例名与输出文件各错一次。占用采样曾与启动放在同一条命令里，没有按采样结果等待（03:25，当时另一会话的 nextest 正在跑，本会话的 10 秒测试照常启动）；此后改为单独等待。

**证据哈希**（SHA256 前 16 位，`D:/autostack/plan-artifacts/plan-1129/` 下）：`t06/r02-union/results.json` `112f21b336dc560d`；`t06/r02-cases/results.json` `32ad226dc089e2a3`；`t06/r02-union-compare.txt` `7a0c7f57a3a89d8c`；`t06/r02-case-table.md` `512a95fadadfb89f`；`t06/mutants-av.txt` `cabd0de0689eca3e`；`t06/mutants-a2py.txt` `9b4eba3b504a1dcb`；`t06/warnings-audit.txt` `86559be65171cd3b`；`t06/r01-union/results.json` `ba354fa691b91ea1`（第一次冻结跑）；`t06/r01-cases/results.json` `1b1b27273eb2c66b`。

**就绪记录**。

- 工作阶段验收 AC-01..AC-04 的结果见 §7，逐例计时见上表。
- 合同内的未决发现、必需的覆盖缺口、未决合同决策：均为空。合同外新发现 1 项，已登记 D-1129-01，不在本计划修。
- 同族路径：R-01 的永久用例覆盖位置、具名、f32 与 f64、局部变量、调用结果、括号表达式、乘积与双重取负，以及整数兄弟（仅 A2PY）；探针另覆盖 `-twice(0.1d)`、`-(rate)`、`-(rate + rate)`、`-(seed * seed)`，AV 与原生 Python 一致，没有做成永久用例。R-05 覆盖校验器与解码器各自的位置、接收者、具名三槽；R-06 覆盖 `/1`、`/2` 与未知帧头。以上是实际闭环，不是只修了原始反例。
- 冻结并集已在最终提交 `60f4e661b` 上执行。同步 master 之后的冻结并集是合入阶段的门，本阶段不执行。
- 证据身份：代码 `60f4e661b46cb8d374649c7d2e3e3891bf03ba5c`，基面 `0486981556bd8d128c69ded9888319ac4a65fc02`，依赖 auto-down `456c86a`（只读），固定组配置 `a560d21924cbef24`，rustc 1.98.0，调试工具档。
- 复用：学习率三变体（work r1）是原生参考层面的对照，不经过编译器，沿用原结果；其余本轮均已重跑。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r3 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 60f4e661b | next: review`。

### review r3（2026-11-24，修复后再复审）

`stage: review | review_scope: final（修复后再复审，只核受影响项） | reviewer_identity/limitations: 实现会话（主上下文）负责确定性核查、帧级探针与记录；另起一个全新上下文的复审代理做独立反例探针、原始结果对账与修复差异扫描（general-purpose 代理，模型身份未暴露；只读，不构建，不写任何检出）。代理的发现由主上下文逐条复核后才采纳；代理自报并更正了两处探针错误（关键字实参用了 Python 的 = 写法；前四个探针的 A2PY 一侧误走了 WSL 的 bash），这两处没有作为证据。两者都没有运行统一回归，冻结并集沿用 work r3 在同一提交上的结果 | plan_id: PLAN-1129 | plan_revision: r3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | reviewed_commit: 60f4e661b46cb8d374649c7d2e3e3891bf03ba5c | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | spec_inputs: 六份规范在复审提交上的 SHA256 前 16 位依次为 python-interop 9c98ab4fff34c92f、python-ecosystem-integration 0fd3cc365e1fb75b、feature-organized-corpus 488edf76d645be85、m2-host 3eeda82a469adcec、auto-bindgen/project 77a8477f82fcf921、replacement-scope-and-evidence 84a1a09e5a0ccaf6；冻结副本 SHA256 前 16 位：规范增量全量差异 d5cb33d43a91e6be、修复轮规范差异 c7980cb8c2d25398、实现全量差异 28e35b19035a6adc、修复轮实现差异 04a6bd4b7890deb3（D:/autostack/plan-artifacts/plan-1129/review-r3/frozen/） | acceptance_results: AC-01 pass，AC-02 pass，AC-03 fail，AC-04 pass；T-02 fail，T-01、T-03 pass，T-04 partial | pending_closeout_gates: 同步 master 之后的冻结并集、规范落地核验、归档与清理 | findings: R-10（低）、R-11（中）、R-12（外观）、R-13（外观）、R-14（低，记录订正） | evidence: 本节各表；原始结果在 D:/autostack/plan-artifacts/plan-1129/review-r3/ | next: work（修复轮 2/3），提交之前先写诊断检查点`。

**基线**。计划 r3、状态 `execution_done`；专属树 `plan-1129-dev`，HEAD `60f4e661b…`，复审前后 `git status` 均为空；基面 `048698155…`，34 个文件；修复轮差异（`481f0a31d..60f4e661b`）25 个文件 +1152/−303。固定组配置 `a560d21924cbef24`。差异不触及 `.autoos/`、`specs.json`、`index.json`、`attachments`。过程观察：r3 的合同文本修订是在修复轮里由 work 自行完成并自增版本号的，没有经过 `new` 阶段（r2 经过）。它的目标、验收标准、任务、仓集合与预算都没有变，授权范围之内；本轮复审把 r3 的规范与合同增量当作复审对象逐句核过，见「规范增量审读」。

**复审范围**（check → finding/invariant）。

| 不变量 | 本轮的核法 | 结论 |
|---|---|---|
| 实参的值原样到达 Python（R-01） | 代理自选 97 个源，每个先算原生预期，再跑 A2PY 与 AV；主上下文补关键字槽与修复回归面 | Auto 数值操作数闭环；Python 对象操作数出现修复引入的回归面（R-11） |
| 一个值只有一种编码，畸形 payload 具名拒绝（R-05） | 独立挑非规范拼写，在真实代码上实测两个解码器的三个值槽；代理做代码审读 | `f` 的高位闭环；前导零未闭环（R-10） |
| 拒绝消息如实（R-06） | 代码审读加运行期消息 | 闭环 |
| 修复不改变已放行输入的行为 | 比较修复前后 `py_find_value` 的选择；查解析器里全部同 span 构造与合成 `Unary` | 一处回归面（R-11）；一处目前不可达的隐患（观察 3） |
| 规范描述当前行为 | 逐句对照代码与实测 | 一处代码示例错误（R-13），两句需要限定范围（随 R-10、R-11 修） |
| 冻结并集新增红 0 | 代理用自己的脚本从原始日志重算 | 成立 |

**验收证据映射**（再复审只更新受影响项）。

| 项 | 结论 | 方法与结果 |
|---|---|---|
| AC-01 | pass | R-01 对 Auto 数值操作数的闭环由独立探针证实，原生 = A2PY = AV：f32 与 f64 的局部变量、函数结果、方法结果 `.to(double)`、多重取负（带括号、不带括号、三层）、和差积商、嵌套调用、关键字槽（`round(number: -rate, ndigits: 3)`、`complex(real: -rate, imag: -(rate * 2.0d))`、仅关键字的 `isclose(a: -rate, b: …)`）、Python 对象右侧操作数（`Fraction(3, 2) * -rate`）、字典与集合的 `k` 与 `-k`、同一调用里两个取负实参、副作用与求值顺序。整数形态 A2PY 正确，AV 是 D-1129-01 |
| AC-02 | pass | 沿用：`torch-lr-computed` 两端检查与两个 torch 用例在冻结跑中通过（5.126 s、4.018 s） |
| AC-03 | **fail** | V-04 的「非规范 payload 具名拒绝」与 §4 的「一个值只有一种编码」只对 `f` 的高位闭环，带前导零的字仍被接受（R-10）。错误类别、范围与身份拒绝的既有证据保留 |
| AC-04 | pass | 语料 14 例 AV 14/14、A2PY 14/14；学习率三变体与 20 条检查器变体两端全部被判掉。20 条里 A2PY 的 x1、x6、x8 在保留文本中被截断，复审从留存的报告里读出了差异行：第 8 行优化器自报的 `lr`、第 4 行旧字面量、最后一行零的符号 |
| T-02 | **fail** | R-10、R-11、R-12 |
| T-03 | pass | 语料如实，整数形态的处置有记录 |
| T-04 | partial | R-13、R-14；规范里两句的范围要随 R-10、R-11 订正 |

**既有发现的闭环判定**。

| 发现 | 原始例子 | 不变量 | 说明 |
|---|---|---|---|
| R-01 | 闭环 | 部分闭环 | Auto 数值操作数全部正确；一元优先规则对 Python 对象操作数也生效，带来 R-11 |
| R-02 | 闭环 | 闭环 | 计划新增行警告 0，复算成立 |
| R-03、R-04 | 闭环 | 闭环 | 文本核对 |
| R-05 | 闭环 | **未闭环** | `f` 的高位被拒；前导零是同一不变量下的缺口（R-10） |
| R-06 | 闭环 | 闭环 | `/1`、`/2`、未知帧头三种消息都有断言 |

**发现**（每条含复现包；探针源与输出在 `review-r3/`，`agent/` 是代理的 97 个源，`probes/` 是主上下文的源）。

| ID | 级别 | 关联 | 违反的规则 | 位置或输入 | 期望对实际 | 已确立的原因 | 关闭用的 oracle 与覆盖边界 |
|---|---|---|---|---|---|---|---|
| R-10 | 低 | T-02、T-04；AC-03 | §4 线路合同：一个值只有一种编码；畸形 payload 具名拒绝 | `av-core/src/foreign_exec.rs` 的 `check_float_words`；`crates/auto-bindgen/src/frame_shim_prelude.rs` 的 `__f0_float_bits`。输入：`00 0 0`、`0000000 0 0`、`01 0 0`、`0 00 0`、`0 0 01`、`0000000 0000000 0000000`、`f` 的 `00 1023 4194303` | 两个实现在位置、接收者、具名三个槽都接受这些拼写，并解码成与 `0 0 0`、`1 0 0`、`1` 相同的值。必拒对照（`0 +1 0`、越界字、`f` 的高位）确实被拒 | 两处字解析只查「全是数字、长度 1..=7、小于界」，没有禁止前导零。发射端总是写规范形式，所以运行中的程序不受影响；这是合同与实现不一致。注释与三份规范都写着「一个值只有一种编码」 | 两个实现对带前导零的多位字具名拒绝，三个槽各有正反例，发射端不受影响。如果改 §4 的冻结文本来写明这条，经 `new` 做有界修订；只改实现与测试则不动版本号。附带：`the_decoder_and_the_admission_accept_the_same_bytes` 只调用 `validate_frame_wire`，解码器并没有在它里面运行，名不副实；校验器与解码器对同一张样本表的判定应被实际执行 |
| R-11 | 中 | T-02；SD-01、SD-02 | 修复不得给已放行的输入带来新的失败形态；规范描述当前行为 | `pack(">d", -sqrt(0.25d))`、`repr(-o.real)`、`repr(-o)`（`o = complex(0.25d, 0.5d)`）、`print(-r)`（`r = sqrt(0.25d)`）。入口同上，`agent/w01`、`w02`、`w05`、`w08`、`w10`、`k06` | `pack(">d", -sqrt(0.25d))` 原生 `bfe0000000000000`，A2PY `0000000000000000`，AV 以 `foreign.frame: p967.frame/2 rejected: payload not LF-terminated` 失败。`repr(-o.real)` 原生 `-0.25`，A2PY `0`。`repr(-o)` 在 A2PY 上以未归类的 `TypeError: int() argument must be … not 'complex'` 退出 1。`print(-r)` 在 AV 上打印 `-2`（句柄的相反数）。先提取再取负是对的：`repr(-sqrt(0.25d).to(double))` 得 `-0.5` | 前门只对数值操作数做带类型的一元负号，其余回退为 i32 且不拒绝（`resolve.rs` 的 `Task::UnaryOut`，来自 `a505011bd7`，本计划未改）。R-01 的修复之后，A2PY 选中这个一元节点，`NegI32` 发射 `a2x_neg(对象)`，即 `-int(对象)`（`lower.rs:1246`；生成码实测 `t5 = a2x_neg(t4)`）。修复前 A2PY 传入的是操作数本身，丢符号但不崩——这一条是代码阅读，没有执行修复前的二进制 | 二选一并写明：(a) 一元优先规则只对 Auto 数值操作数生效，Python 对象操作数保持修复前的行为；(b) 前门对 Python 对象的一元负号两端具名拒绝，并提示先 `.to(double)`。(b) 改变现有准入，须经 `new` 记入合同。无论哪种：A2X 规范「取负实参」一句限定为 Auto 数值操作数；两份规范的已知边界写明这个形态；永久用例覆盖所选方案；AV 一侧登记 D-1129-09（已登记） |
| R-12 | 外观 | T-02；SD-01 | 诊断文案如实 | `resolve.rs` 的拒绝文案；输入 `var flag = true` 加 `repr(flag).to(str)`、`repr(!flag)` | 两端都以「this value kind cannot cross into a Python call (int, bool, text, float and Python objects cross)」拒绝，而 bool 正是这句话说可以过界的 | 运行时 bool 不能过界（`py_frame_val` 没有这一分支，既有行为）。基面上的文案是「v1: int/bool/text literals and Python objects」，是准确的；`77453fa9b` 改写时去掉了「字面量」 | 文案与实际一致，例如写明 bool 只有字面量可以过界。补运行时 bool 过界超出本计划合同，不要求 |
| R-13 | 外观 | T-04；SD-02 | 规范里的代码示例必须是合法源码 | `docs/specs/auto-a2x/python-ecosystem-integration.md:170` | `pack(“>d”, -rate)` 用了弯引号 | 修复轮编辑带入 | 改回 ASCII 引号；顺带把 python-interop.md 里「（这句只说 AV。…）」这句对句子本身的说明改写成直接陈述 |
| R-14 | 低 | T-03、T-04 的证据记录 | 记录如实 | work r3 的三处 | (1) `ace-av` 一行写「14 条记录，13 个用例（同一用例在两个目标下各记一次）」。原始事件流是 13 个失败、13 个不同名字，该用例只属于一个目标；14 是对账脚本把同一失败的两种标签写法并进了集合。(2) 第一次语料组运行里 `torch-lr-computed` 也是 ERROR（3055 ms，`av.resource.execution-wall`），记录只写了新语料例。(3) 发现时间合计是逐组四舍五入值相加的 56.2 s，精确值 56.349 s；运行器余量是 1.22 s，不是「约 1.4 秒」 | 对账脚本按事件名去重的缺陷，以及记录时的疏漏 | 修复轮的记录里逐条订正（§9 只追加，不改旧记录）。基线红合计仍是 51 个用例。对账脚本改为只按事件名计数 |

**观察（不构成本计划的阻断项）**。

1. D-1129-01 的处置成立。计划的目标、值合同与验收标准只针对 `f32/f64`；§5.1 写明未支持的对象形态继续遵循现有准入。代理独立证实：不用一元负号得到的负整数在 AV 上同样失败，`__py_dlen` 与 `__py_chunk_i` 的文本在基面与复审提交上逐字节相同。
2. 代理另报了几类既有现象，都没有纳入发现：帧的长度与计数字段接受前导零与前导负号（`d 05`、`d -5`、`p -0`），`/1` 的整数与句柄 payload 接受 `003`、`+42`、`007`。主上下文在真实代码上复测，两个实现逐行相同。这是 `/1` 布局的既有行为，本计划的合同要求它不变；规范里「一个值只有一种编码」只应理解为说字三元组。已登记 D-1129-10。校验器与解码器对四种非浮点畸形帧的原因字符串不同，接受与否则一致。
3. 隐患：`resolve.rs` 的 `any_all_typed_list` 把循环内部的 `Unary{Not}` 停止标志放在整个 `xs.all(…)` 表达式的 span 上。如果该表达式直接作 Python 调用实参，一元优先规则会选中它。实测目前到不了：带 `use.py` 的程序里这种闭包写法在 bind 阶段被两端一致拒绝（已登记 D-1129-08），不带 `use.py` 时同一写法在 AV 上正常输出。下一次动 `py_find_value` 时宜把选择规则收窄到句法上的一元节点。
4. 代理报告的 A2PY 独有现象，与本计划无关：`use.py` 程序里的 f64 数组字面量被拒（`a2x.unsupported-type`）；整数的 `print((a - b).to(str))` 不可 lower（`a2x.unsupported-expr`）。
5. 语料组运行器在被选检查全部 PASS 时进程退出码仍是 1；判定要看包装器的逐检查状态。
6. A2PY 变体证据里「第 N 字节不符」的偏移落在 `\r` 上，是换行符造成的，不能当证据；判定本身是语义的（见 AC-04 一行）。

**规范增量审读**。`supersedes_spec_components` 的六条与差异里的六份规范一一对应，`new_spec_components` 为空，`touched_goals` 为空。SD-03、SD-05、SD-06 准确。SD-01（python-interop.md）：R-04 已改；「一个值只有一种编码」随 R-10 订正或落实；「（这句只说 AV。…）」随 R-13 改写；AV 负整数实参的已知边界与实测一致。SD-02（python-ecosystem-integration.md）：R-03 已改；「取负实参」一句没有限定操作数类别，对 Python 对象不成立（随 R-11 限定）；代码示例的弯引号（R-13）；舍入不以 `use.py` 为条件的说明与 `each_float_helper_sits_in_the_section_whose_hash_binds_its_identity` 一致。SD-04（m2-host.md）：「每字有界，故一个值只有一种编码」随 R-10；其余与代码一致，包括解码器源文本单独成文件。

**原始结果对账**（代理用自己的脚本独立重算，主上下文抽查）。`t06/r02-union`：11 组执行合计 4156、排除 5；每组的执行数、排除数、发现秒数、测试秒数与 work r3 的表逐项相同（到 0.1 秒）；每组 nextest 的判定行数等于执行数，通过加失败等于执行数；失败 51 个用例——`ace-abi` 28（全是 `source_foreign`）、`ace-av` 13、`ace-source` 5、`ace-native` 5；与修复前四次冻结跑逐名比对，新增 0，消失 1（`sched_m2_1067 scheduler_runs_are_repeatable`，本次通过）。`t06/r02-cases`：本计划 14 例两端各 14/14，AV 组非 PASS 恰为 2 个 `av.parity.1093.*`。计划里列出的 9 个证据哈希前 16 位全部相符。`t06/r01-union` 的三组退出码 124 与 `vctip.exe` 记录相符，`ace-a2x` 379 全过、`ace-m2-build` 50 全过。

**测试成本**（复审阶段实际执行的测试；统一回归没有重跑，沿用 work r3 在同一提交上的结果）。

| 测试模块/实际范围 | 结果与执行/排除计数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| 代理的案例级探针：97 个源，每个原生、A2PY、AV 各一次 | R-01 的 Auto 数值形态三方一致；10 个源在 A2PY 或 AV 上与原生不一致（R-11）；另有若干形态被两端一致拒绝，无法比较运行期 | 复用已备 worker 与 provider | — | A2PY 每个约 0.2–0.6 s，AV 每个约 0.5–0.6 s | — | 约 30 分钟（含写脚本与对账） |
| 主上下文的案例级探针：13 个自写源加 3 个代理源的复跑 | 关键字槽三方一致；其余是对「py 车道拒绝闭包」的刻画与 R-11 的复核 | 复用 | — | 每个 ≤0.7 s | — | 约 20 s |
| 帧级探针：AV 校验器，临时单测 2 个（跑后按 SHA-256 恢复并删除） | 前导零 9 组 payload × 2 标签 × 3 槽，另 4 个应接受的对照与 3 个必拒对照；帧头与计数 18 行 | 增量编译，每次约 3.4 s | — | 0.01 s | — | 每次约 3.5 s |
| 帧级探针：shim 解码器，临时单测 2 个（同上） | 同一批输入，三个槽 | 每次 ≤1 s | — | 0.01 s | — | 每次约 1.1 s |
| rustfmt 计划行核对（`cargo fmt --check`，9 个包） | 计划新增行差异 0，既有 106 | — | — | — | — | 约 20 s |
| 警告审计复算（既有编译日志） | 计划新增行 0，既有 5 | — | — | — | — | 不到 1 s |

本轮没有新增用例。资源等待：复审期间机器空闲，各次占用采样均为 0 个构建、0 个测试。

**健康检查**。复审提交上 `git diff --check` 无空白错误；修复差异里没有新增或删除调试打印与 TODO 标记；新增的属性只有测试模块上的 `#[cfg(test)]` 与带说明的 `#[allow(dead_code)]`。修复差异删除 19 行断言、新增 49 行，除下面一处之外对应 rustfmt 重排（当时的格式化工具按「记号序列相同」才写入）。那一处实质改动是 `p1129_t02…`：原断言在畸形帧循环之后要求 `callee_count` 不变，现在是规范对照调用之后要求 `before + 3`。性质相同（畸形帧一次派发都没有），只是定位从循环末尾挪到了用例末尾；不算缺陷，随 R-10 修复时恢复成循环末尾的断言。另一处实质改动是 `f0_int1026` 去掉未用绑定（R-02）。

**诊断检查点（连续第二次终审不通过）**。review r2 与 review r3 都以 `needs_fix` 结束。按 work 技能的规定，下一次修复提交之前，work 须先在 §9 写出诊断并落实 oracle 调整，不能拿同样的证据再送一次复审。本轮给出的诊断材料如下，work 须逐项确认、补充并落实：

| 没有闭环的不变量 | 之前的检查为什么没发现 | 受影响的路径与状态 | oracle 调整（下一次复审核它是否落地） |
|---|---|---|---|
| 一个值只有一种编码 | R-05 的修复与用例只覆盖了被报告的形状（`f` 的高位），没有枚举其他非规范拼写；名为「同一字节集」的用例并不运行解码器，两个实现各自对着自己的样本测 | 两个实现的字解析；位置、接收者、具名三个槽；`d` 与 `f` 两个标签 | 一张共享的样本表（规范写法与全部非规范拼写：每个字的前导零、补零到 7 位、高位、符号、分隔符、字数），对两个实现执行同一张表并断言相同的判定；再加一条生成式检查——对一批随机位型，发射端的写法在它的各种补零变体里是唯一被接受的 |
| 修复不改变已放行输入的行为；实参的值原样到达 | R-01 的负控与语料只用了 Auto 数值操作数；没有按操作数类别枚举「同 span 的一元节点」这条规则会选到什么 | `py_find_value` 的一元优先规则 × 操作数类别（f32、f64、i32 的字面量、局部变量、调用结果；Python 对象；`.to(double)` 提取结果）× 槽位（位置、关键字、二元运算、索引） | 一张类别矩阵，每格写明期望（值，或具名拒绝），两端执行；修复前的行为用执行取得（构建修复前的 worker，或留下修复前的生成码），不靠阅读 |
| 记录与规范如实 | 对账脚本把同一失败的两种标签写法并进了集合，数字没有回到原始事件流核对；规范改写后没有再通读 | work 记录的计数与时间；A2X 规范与 AV 规范的本计划段落 | 计数只按事件名；记录里的每个合计数由脚本从原始结果算出并附命令；规范改动的行做一次自检——代码跨度里不得有非 ASCII 引号，每个「X 可以」的断言都有对应的实测或用例 |

**本会话的探针与工具错误**（均已更正，不是产品缺陷）：主上下文为「合成 `Unary` 的回归隐患」写了 7 个探针才确认该形态在 py 车道编译不过（其中 3 个把 `.to(str)` 链在了闭包后面，1 个用了本语言不存在的 `all()`/`any()` 函数形式）；帧头探针里接收者槽与关键字槽的 4 行长度写错，无效，没有计入证据。代理的两处见本节开头的说明。

**路由**。`needs_fix`：状态回到 `executing`，T-02、T-04 与 AC-03 退回，`current_step` 重算为 2；T-01、T-03、AC-01、AC-02、AC-04 的结果保留。交 work 做修复轮 2/3，范围是 R-10..R-14，先写诊断检查点。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r3 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 60f4e661b | next: work`。

### work r4 诊断检查点（2026-11-24，修复轮 2/3，提交之前写入）

`stage: work | plan_id: PLAN-1129 | plan_revision: r3 | checkpoint: two-failure-diagnosis | basis: review r2 与 review r3 连续两次 needs_fix | written_before_any_repair_change: true（写入时专属树仍是 60f4e661b，`git status` 为空；本轮只在 plan-artifacts 里构建了修复前的 worker、跑了矩阵与一次 HIR 转储） | next: 落实下表三行 oracle 调整并验证，之后才提交`。

**执行基线**（替代 review r3 里靠阅读得出的「修复前」）。把 `lower.rs` 临时换成 `481f0a31d` 的版本，`cargo build -p ace-a2x-worker --bin a2x-worker`（13.0 秒），二进制留存为修复前 worker；源文件随后按 SHA-256 恢复。修复轮 1 的 worker 是冻结跑准备时构建的那个。29 个最小程序（操作数类别 × 槽位，每个先算原生预期）依次经两个 worker 运行，原始表 `t07/matrix-before.md`，SHA256 前 16 位 `6bf7553f807645a8`：修复前与原生相同 1/29，修复轮 1 为 20/29。

| 类别 | 格 | 修复前 | 修复轮 1 | 结论 |
|---|---|---|---|---|
| Auto 数值：f64、f32、i32 的局部变量与函数结果，位置槽与关键字槽 | A1–A6，12 格 | 丢符号 | 全部正确 | R-01 的修复在这一类上闭环 |
| Auto 数值：`.to(double)`、`.to(float)`、`.to(int)` 的提取结果 | A7–A10，5 格 | 丢符号 | 全部正确 | 同上 |
| Python 对象：py 调用结果、对象局部变量、运算结果 | B1p、B1k、B3p、B4p | 传入操作数本身（丢符号，不崩） | 截断为 0；B3p 以原始 `TypeError` 崩溃 | 第一轮修复改变的正是这 4 格 |
| Python 对象：属性读取 `-o.real` | B2p、B2k | 截断为 0 | 截断为 0 | 两版相同：取负节点是该 span 上唯一的候选，一直被原生 lowering |
| Python 对象：调用后紧跟 `.to()`，如 `-sqrt(0.25d).to(double)` | B5p、B5k、B6p | 截断为 0 | 截断为 0 | 两版相同：取负经 `Cast` 的参数被原生 lowering，不经过 `py_find_value` |
| Python 对象：`-len("abc").to(int)` | B7p | `-3` | `-3` | 碰巧正确（整数对象经 `int()` 不损失） |

对 review r3 的三处订正：

1. R-11 的关闭 oracle (a)「Python 对象操作数保持修复前行为」的准确含义是：`py_find_value` 对这类操作数回退到修复前的排序。属性读取那一格修复前就是截断为 0，(a) 不改变它。
2. `-sqrt(0.25d).to(double)` 不是「提取结果取负」。HIR 是 `Unary{NegI32, py 调用}` 再 `Cast` 到 F64：前导负号先作用在 py 调用上，属于对 Python 对象取负一族（D-1129-09）。本机探针 `t07/hir-a7.log`。
3. review r3 说修复前 A2PY 对 `-o.real` 传入 `0.25`，这是代理靠阅读得出的；实测是 `0`。

**诊断**（每行：为什么之前的检查没发现；受影响的路径与状态；本轮要落实并验证的 oracle 调整；怎么证明调整有判别力）。

| 没有闭环的不变量 | 之前的检查为什么没发现 | 受影响的路径与状态 | oracle 调整 | 判别力检查 |
|---|---|---|---|---|
| 一个值只有一种编码；畸形 payload 具名拒绝（R-05 → R-10） | R-05 的修复和用例是从复审给的三个样例（`f` 的高位）直接推出来的，只枚举了被报告的形状。两个实现各对着自己手写的样本测；名为「解码器与准入接受同一字节集」的用例只运行了校验器，解码器从没跟校验器对过同一张表 | AV 校验器的 `check_float_words` 与 shim 解码器的 `__f0_float_bits`；标签 `d`、`f`；位置、接收者、具名三个槽；状态：规范写法、某个字带 1 到 6 个前导零、补零到 7 位、`f` 的高位、符号、非数字字符、分隔符、字数、超过 7 位、越界。长度与计数字段的宽松是 `/1` 布局的既有行为（D-1129-10），不在本行 | (1) 两个实现拒绝带前导零的多位字，原因字符串相同。(2) 一张共享的样本表（一个源文件，两个测试模块 `include!` 同一份），含手写行和一个确定性生成器：对一批位型，规范写法被接受，每个字的各种补零变体都被拒绝，不同的位型得到不同的规范写法。两个实现对同一张表给出相同判定 | 把前导零规则去掉，两个共享表用例必须变红；只去掉一侧的实现，只有那一侧变红（证明两侧都在跑这张表） |
| 实参的值原样到达 Python；修复不改变已放行输入的行为（R-01 → R-11） | R-01 的修复从浮点例子推出来，负控与语料只用了 Auto 数值操作数；新规则「同 span 有一元节点就取它」对所有一元节点一视同仁，没有按操作数类别区分；961 把非数值操作数的一元负号回退成 i32 这件事没有被考虑到；修复前的行为是读代码得出的，没有执行 | `py_find_value` 的一元优先规则 × 操作数类别（f32、f64、i32 的局部变量、函数结果、提取结果；Python 对象的调用结果、属性、局部变量、运算结果）× 槽位（位置、关键字）。另有两条不经过该查找的路径（属性读取后的取负、`Cast` 参数里的取负），本轮不改 | (1) 一元优先只对操作数不是 Python 对象的节点生效；Python 对象操作数回退到修复前的排序。(2) 上面的 29 格矩阵做成永久的表驱动用例：数值格断言原生值与「实参被取负」的生成形态；对象格断言实测到的修复前输出，并注明 D-1129-09。(3) 「哪些操作产生对象」只留一份定义，放在中立契约的 `PyOpKind` 上，解析器与 A2PY 都用它 | 把规则改回「任何一元节点都优先」，对象格必须变红；改成「一元节点从不优先」，数值格必须变红。两侧都要试 |
| 记录与规范如实（R-12、R-13、R-14） | 对账脚本把事件名与人读失败行并成一个集合，同一条失败的两种标签写法被数了两次；我没有回到原始事件流核对，还给多出来的那一条编了一个说得通的解释。合计数是用逐组四舍五入值相加的。第一次语料组运行的第二个 ERROR 在归纳时被漏掉。规范改写后没有再通读：带进了弯引号，「取负实参」一句比实测到的行为宽。拒绝文案改写时没有拿它描述的程序形态去跑 | 对账脚本的计数；work 记录里的每个合计数；A2X 与 AV 两份规范的本计划段落；`resolve.rs` 的拒绝文案 | (1) 只按事件名计数，重算后订正记录。(2) 记录里的合计数由脚本从原始结果算出，命令随记录写下。(3) 规范改动行自检脚本：代码跨度里不得有非 ASCII 引号；本计划新增的规范行里每一条「X 可以」都列在实测证据旁边。(4) 改写用户可见的诊断文案时，用它描述的每种程序形态实跑一遍 | 自检脚本对本轮改动之前的规范差异必须报出弯引号那一处；修后报 0 |

以上三行的实现与验证完成之前不提交、不送审；完成情况记在随后的 work r4 记录里。

### work r4（2026-11-24，修复轮 2/3）

`stage: work | plan_id: PLAN-1129 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | code_commit: 3c7aebf5fa1221ea6d1d728e1b891ee1dd7e4dac | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | task_ids: T-02、T-04（修复 review r3 的 R-10..R-14） | evidence: 本节各表；原始结果在 D:/autostack/plan-artifacts/plan-1129/t07/ 与 t06/r03-* | blockers: 无 | next_responsible_role: review（T-05）`。

**修复**（提交 `3c7aebf5f`，14 个文件 +1003/−61）。

- R-10：AV 校验器与 shim 解码器拒绝以 `0` 开头的多位字，原因字符串相同（`typed float word has a leading zero`）。两个实现的测试改为 `include!` 同一张样本表 `crates/auto-bindgen/src/frame_wire_samples.rs`（手写行加确定性生成器）；原先那个名为「解码器与准入接受同一字节集」却只运行校验器的用例已删除，它的样本并入样本表。
- R-11：`py_find_value` 只在一元节点的操作数不是 Python 对象时才优先取它；是 Python 对象时回到修复前的排序。「哪些操作产生对象」只在 `PyOpKind::yields_object` 定义一次，解析器的 `py_is_object` 与 A2PY 的 lowerer 都用它。
- R-12：拒绝文案改为「int, text, float and Python objects cross; bool only as a literal」。
- 规范：A2X 的「取负实参」限定为 Auto 数值操作数；两份规范的已知边界写明对 Python 对象取负（D-1129-09）；「一个值只有一种编码」写明最短形式；代码示例改回 ASCII 引号；「（这句只说 AV。…）」改为直接陈述；共享样本表写入三份规范。

**闭环表**。

| 发现 | 不变量 | 触及路径 | 修复与永久用例 | 结果 | 覆盖边界 |
|---|---|---|---|---|---|
| R-10 | 一个值只有一种编码；畸形 payload 具名拒绝 | `av-core/src/foreign_exec.rs` 的 `check_float_words`；`crates/auto-bindgen/src/frame_shim_prelude.rs` 的 `__f0_float_bits` | 两个实现拒绝前导零；共享样本表 583 行（接受 306、拒绝 277），`the_admission_judges_the_shared_wire_samples_as_recorded` 与 `the_decoder_judges_the_shared_wire_samples_as_recorded` 各自跑同一张表；`the_canonical_spelling_of_every_generated_pattern_is_admitted`、`the_canonical_spelling_decodes_back_to_its_pattern`；abi 的 `p1129_t02…` 循环新增三个前导零的帧并恢复循环末尾 `callee_count` 不变的断言 | 两侧共享表用例通过；abi 用例通过（安静重放 0.369 秒） | 长度与计数字段的宽松拼写是 `/1` 布局的既有行为，不在本行（D-1129-10） |
| R-11 | 实参的值原样到达 Python；修复不改变已放行输入的行为 | `a2x-python/src/lower.rs` 的 `py_find_value` 与 `py_object_locals`；`core/src/py_ops.rs`；`core/src/source/resolve.rs` 的 `py_is_object` | 一元优先只对非对象操作数生效；29 格矩阵成为两个永久用例：`generated_python_negates_every_auto_value_operand_class`（19 格）与 `generated_python_keeps_the_earlier_behaviour_for_object_operands`（10 格）；`exactly_the_five_object_lanes_yield_an_object` | 19 格等于原生，10 格等于实测的修复前输出（下表） | Python 对象取负本身仍是既有缺陷（D-1129-09） |
| R-12 | 诊断文案如实 | `core/src/source/resolve.rs` 的拒绝文案 | 文案写明 bool 仅字面量；`the_cross_boundary_rejection_states_the_boundary_as_it_is` | 通过 | — |
| R-13 | 规范里的代码示例必须是合法源码 | `docs/specs/auto-a2x/python-ecosystem-integration.md` | 改回 `pack(">d", -rate)` | 规范自检 0 处 | — |
| R-14 | 记录如实 | 对账脚本 `compare_runs.py`；work r3 的三处 | 只按事件名计数，失败行数只作交叉核对；合计数由脚本算出；三处见下文订正 | 重算结果与 review r3 代理的数字一致 | §9 只追加，旧记录不改 |

**诊断的落实**（对应上一节的三行）。

| 不变量 | 落实的 oracle | 判别力检查 |
|---|---|---|
| 一个值只有一种编码 | 共享样本表：两个测试模块 `include!` 同一个文件；生成器对 64 个位型产出规范写法（接受）与每个字的补零变体（拒绝），外加 `f` 的高位变体 | `t07/mut-r10.txt`：只关掉校验器的规则，仅 AV 侧变红（6 过 1 红），bindgen 17 全过；只关掉解码器的规则，仅 bindgen 侧变红（16 过 1 红），AV 侧 7 全过；两次都按 SHA-256 恢复 |
| 一元规则 × 操作数类别 | 29 格矩阵：操作数类别（Auto 数值的 10 类、Python 对象的 7 类）× 槽位（位置、关键字）；修复前的行为由**执行**取得——把 `lower.rs` 临时换成 `481f0a31d` 的版本构建 worker（13.0 秒），留存二进制后按 SHA-256 恢复源文件 | `t07/mut-r11.txt`：改回「任何一元节点都优先」，对象格在 B1p 变红、数值格不受影响；改成「一元节点从不优先」，数值格在 A1p 变红、对象格不受影响；恢复后两者都通过 |
| 记录与规范如实 | 对账脚本只按事件名计数；规范自检脚本 `spec_selfcheck.py` 扫本计划新增的规范行（代码跨度里的非 ASCII 引号；列出每条带断言的行） | 自检脚本对本轮改动之前的状态（`60f4e661b`）报出弯引号 1 处、退出码 1；它随后抓到我重写那一段时**第二次**写成弯引号，改正后报 0 处。诊断文案按它描述的每一类程序形态实跑（见 `the_cross_boundary_rejection_states_the_boundary_as_it_is`） |

**执行基线与修复后**（29 个最小程序，每个先算原生预期，再经三个 worker 运行：修复前 `481f0a31d`、修复轮 1 `60f4e661b`、修复轮 2 `3c7aebf5f`；`t07/matrix-before.md` `6bf7553f807645a8`、`t07/matrix-after.md` `893da1d32ad99edd`）。

| 类别 | 格数 | 修复前 | 修复轮 1 | 修复轮 2 |
|---|---|---|---|---|
| Auto 数值：f64、f32、i32 的局部变量与函数结果，`.to(double)`、`.to(float)`、`.to(int)` 的提取结果；位置槽与关键字槽 | 19 | 全部丢符号 | 全部等于原生 | 全部等于原生 |
| Python 对象：py 调用结果、对象局部变量、运算结果 | 4 | 传入操作数本身（丢符号，不崩） | 截断为 0；B3p 以原始 `TypeError` 崩溃 | 等于修复前 |
| Python 对象：属性读取 | 2 | 截断为 0 | 截断为 0 | 等于修复前 |
| Python 对象：调用后紧跟 `.to()` | 4 | 截断为 0（B7p 碰巧正确） | 同修复前 | 等于修复前 |
| 与原生相同的格数 | 29 | 1 | 20 | 20 |

修复轮 2 与修复前逐格相同的对象格：10/10。

**冻结并集**（`3c7aebf5f`，源状态 `d4516be83436d0ce`，固定组配置 `a560d21924cbef24`）。调试工具档一次准备 194 秒（shim 源文本变了，provider 已重建），收据 65 条与 11 个归档哈希全部核对；等 17 秒后按进程数判为空闲并启动。并集运行期间（05:55–06:09）PLAN-1124 复审会话的数值核对任务（`check_numbers.py` 四个进程与 `shard_numbers.py`，05:56 启动）占着机器，之后实测这类任务在跑时 CPU 约 40% 到 47%；并集整命令 837.9 秒（发现 65.806 秒、测试 771.388 秒），是上一轮空闲时的 2.2 倍。失败按事件流计数，以 `60f4e661b` 那次空闲运行的 51 条为基线（`t07/baseline-failures-60f4e661b.txt`）。

| 组 | 执行 | 并集测试秒（带负载） | 安静重放测试秒 | 并集里的非基线失败 | 安静重放里的非基线失败 |
|---|---|---|---|---|---|
| `ace-source` | 1004 | 10.8 | 未重放 | 0 | — |
| `ace-foreign-contract` | 59 | 1.0 | 未重放 | 0 | — |
| `ace-av` | 1636 | 61.8 | 52.9 | 1（时间预算） | 0 |
| `ace-lowering` | 9 | 1.9 | 未重放 | 0 | — |
| `ace-abi` | 369 | 126.8 | 63.9 | 0 | 0 |
| `ace-python` | 12 | 4.9 | 未重放 | 0 | — |
| `ace-a2x` | 382 | 138.3 | 93.9 | 0 | 0 |
| `ace-m2-build` | 50 | 38.3 | 17.5 | 0 | 0 |
| `auto-bindgen` | 17 | 0.9 | 未重放 | 0 | — |
| `ace-semantics` | 295 | 17.0 | 未重放 | 0 | — |
| `ace-native` | 330 | 369.6 | 72.6 | 25（时间预算） | 0 |

- 基线红合计 51 个用例（`ace-source` 5、`ace-av` 13、`ace-abi` 28、`ace-native` 5），在并集与两次安静重放里逐名相同。
- 并集里的 26 条非基线失败全是时间预算类（`Deadline`、平台调用的 wall budget、「不得挂起」的时长断言），由脚本分类（`t07/r03-union-classified.txt` `d248c491515b6b4f`）。它们所在的 native 运行时、调度器与 AV 运行时 crate 本轮没有改动（`git diff --stat 60f4e661b 3c7aebf5f` 不含这些路径）。
- 带负载的重放（5 组，766.6 秒）里这两组的非基线失败是 2 条与 20 条，**与并集的不是同一批**（`ace-native` 两次都红的非基线用例只有 4 个）；另有两条在 134 秒、152 秒后以断言失败收场。
- 安静重放：其他会话的数值核对任务结束后，CPU 10% 到 19% 时用固定运行器重放。`ace-av` 加 `ace-native` 整命令 135.9 秒，`ace-abi` 加 `ace-a2x` 加 `ace-m2-build` 201.1 秒。五组的非基线失败都是 0，退出码是各自工作负载的退出码（`t06/r03-isolated/results.json` `0a7c40a8b913f39b`、`t06/r03-isolated-b/results.json` `59b2eff341aaf213`）。
- 并集与带负载重放里 `ace-av`、`ace-a2x`、`ace-m2-build` 的退出码 124 仍是 `vctip.exe`（工作负载退出码 100、0、0，`cleanup_complete: true`）；安静重放里没有出现。
- 一次无效的重放没有计入证据：脱离固定运行器、直接在已备归档上过滤重放 `ace-native` 的 41 条，其中 19 条以 `PREPARATION_REQUIRED` 失败——这些用例依赖运行器准备阶段的内容绑定戳记。
- 语料组（带负载，248.4 秒）：本计划 14 例 AV 14/14、A2PY 14/14；AV 组非 PASS 恰为既有的 2 个 `av.parity.1093.*`（`t06/r03-cases/results.json` `1a1e9d257ea93ddf`）。语料组没有在安静机器上重跑；检查全部通过，AV 所选检查最慢 4196 ms（torch），其余 2118–2272 ms。

**测试成本**（修复轮 2 实际执行的测试；统一跑见上）。

| 测试模块/实际范围 | 结果与执行/排除计数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| 冻结并集 `3c7aebf5f`：11 组，调试工具档，`--run-prepared` | 执行 4163，排除 5；非基线失败 26（时间预算，带负载） | 194 s | 65.8 s | 771.4 s | 无单独阶段 | 837.9 s |
| 语料组 `3c7aebf5f`（带负载） | 14×2 PASS；AV 组另有基线 2 FAIL | 4.6 s | — | 156.4 s、76.8 s | — | 248.4 s |
| 安静重放：`ace-av`、`ace-native` | 非基线失败 0 | 复用 | 10.3 s | 125.5 s | — | 135.9 s |
| 安静重放：`ace-abi`、`ace-a2x`、`ace-m2-build` | 非基线失败 0；退出码 0/100/0 | 复用 | 25.7 s | 175.3 s | — | 201.1 s |
| 带负载重放：5 组 | 非基线失败 22（时间预算） | 复用 | 47.5 s | 718.8 s | — | 766.6 s |
| AV 校验器 `cargo test -p ace-av-core --lib frame_wire_1129_tests`（直接运行） | 7/7 | 约 1.3 s | — | 0.00 s | — | 1.5 s |
| auto-bindgen `cargo test -p auto-bindgen --lib` | 17/17 | 约 0.8 s | — | 0.01 s | — | 0.9 s |
| a2x-python `py_float_args_1129`（直接运行，串行） | 20/20 | 1.2 s | — | 7.60 s | — | 8.9 s |
| ace-core conformance `py_float_args_1129` | 8/8 | 10.2 s | — | 0.29 s | — | 10.7 s |
| R-10 两侧变异对照 | 6 次运行，结果如诊断表 | 增量 | — | 每次 ≤0.02 s | 按 SHA-256 恢复 | 每次 1–2 s |
| R-11 两侧变异对照 | 3 次运行，结果如诊断表 | 增量 | — | 2.34 / 1.12 / 3.11 s | 按 SHA-256 恢复 | 每次约 8 s |
| 矩阵驱动：3 个 worker × 29 格，加原生参考 | 174 次 worker 运行、58 次原生参考 | worker 构建 13.0 s、14.3 s | — | — | 源文件按 SHA-256 恢复 | 约 2 分钟 |
| 单独重放 torch abi 用例（已备归档） | 3 次通过：2.07、1.92、1.89 秒 | 复用 | — | — | — | 每次 6–7 s |
| `cargo check --tests`（触及的 9 个 crate） | 新增行警告 0 | 8.2 s | — | — | — | 8.2 s |
| rustfmt 计划行核对与选择性应用 | 新增行差异 0 | — | — | — | — | 每遍约 20 秒，共 3 遍 |
| 探针：HIR 转储 1 次、出站边界 6 次、矩阵脚本试跑 | — | — | — | — | — | 约 1 分钟 |

资源等待：并集前 17 秒；并集之后两次有界等待——06:15–06:27 约 12 分钟（按进程数判忙，后来因为我误读了一次「CPU 1%」而提前停止），06:40–06:58 满 18 分钟到期（64 个样本，CPU 平均 38.7%，最低 14.1%、最高 69.7%，`t07/wait-cpu-quiet.log`）。合计约 30 分钟，不计入上表。

**新增与改动用例的时长**（每个用例自己的 nextest 时间；标「安静」的取自安静重放，其余取自带负载的并集，这些用例都在 0.25 秒以内）。

| 用例 | 验证的行为 | 秒 | ≤10 s |
|---|---|---|---|
| `ace-av-core foreign_exec::frame_wire_1129_tests::the_admission_judges_the_shared_wire_samples_as_recorded` | 校验器对共享样本表 583 行逐行给出记录的判定，浮点行的原因字符串逐字相同 | 0.016（安静） | 通过 |
| `ace-av-core foreign_exec::frame_wire_1129_tests::the_canonical_spelling_of_every_generated_pattern_is_admitted` | 64 个生成位型的规范写法在 `d`、`f` 下都被接受 | 0.017（安静） | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::the_decoder_judges_the_shared_wire_samples_as_recorded` | 解码器对同一张样本表给出相同判定与相同原因 | 0.028 | 通过 |
| `auto-bindgen rust_import::frame_shim_prelude_tests::the_canonical_spelling_decodes_back_to_its_pattern` | 规范写法还原出原位型 | 0.045 | 通过 |
| `ace-core py_ops::tests::exactly_the_five_object_lanes_yield_an_object` | 五个对象操作为真、四个提取操作为假 | 0.145 | 通过 |
| `ace-core::conformance py_float_args_1129::the_cross_boundary_rejection_states_the_boundary_as_it_is` | 运行时 bool 被拒且文案写明仅字面量；文案列出的每一类都能编译 | 0.222 | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_keeps_the_earlier_behaviour_for_object_operands` | 10 格 Python 对象：输出等于实测的修复前输出，形态与登记的一致 | 2.444（安静） | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_negates_every_auto_value_operand_class` | 19 格 Auto 数值：实参被取负，输出等于原生 | 5.571（安静） | 通过 |
| 改动：`ace-abi::conformance py_float_1129::p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text` | 畸形帧循环新增三个前导零的帧；循环末尾恢复 `callee_count` 不变 | 0.369（安静） | 通过 |

本轮新增 8 个、改动 1 个，最慢 5.571 秒。名字含 `1129` 的既有用例里对负载最敏感的 `p1129_corpus_av_torch_lr_computed`（`ace-abi`）四次测量：安静重放 4.228 秒，带负载的并集 25.58 秒，带负载的重放 9.998 秒，脱离并集单独重放 1.89–2.07 秒。新增用例都不起子进程编译：共享表用例 `include!` 源文件，矩阵用例走进程内的 lower 加声明的 Python 解释器，abi 用例经已备的 worker 与 provider。

**健康检查**。复审提交上 `git diff --check` 无空白错误；`cargo check --tests`（9 个 crate）计划新增行上编译警告 0 条，触及文件内既有 5 条；rustfmt 只应用到本计划新增的行（8 处），现为 0 处；新文件 `frame_wire_samples.rs` 整体经 rustfmt。新增代码没有调试打印与 TODO。修复没有新增 `#[allow]`。删掉的用例只有一个，就是那个名不副实的，它的 8 个样本都在共享样本表里。

**规范改动行的证据对照**（`spec_selfcheck.py` 列出 17 行；`t07/spec-selfcheck-now.txt` `12012e965d805827`）。

| 规范里的断言 | 位置 | 证据 |
|---|---|---|
| 每字最短形式，一个值只有一种编码；`f` 高位为零即拒绝 | m2-host.md、python-interop.md | 共享样本表两侧通过；两侧变异对照；abi 用例 |
| Auto 数值操作数取负后到达 Python；操作数是 Python 对象时不优先取负节点 | python-ecosystem-integration.md | 29 格矩阵；两侧变异对照 |
| 取负规则由 `negated-operands`、`generated_python_negates_an_int_argument` 与两个矩阵用例钉住 | 同上 | 这些用例在冻结提交上全部通过 |
| 对 Python 对象取负：A2PY 截断为 0 或抛原始 `TypeError`；调用结果与对象局部变量原样传入 | python-ecosystem-integration.md、python-interop.md | 修复轮 1 的 worker 实测（B1p 全 0、B3p 崩溃）与修复轮 2 的 worker 实测（B1p `3fe0000000000000`、B3p `(0.25+0.5j)`）；AV 一侧是 review r3 代理的 w01、w05 |
| 非正规、非有限浮点不可过界（指 Python → Auto 的提取） | python-interop.md | 两端实测：AV `float("inf")` 配 `.to(double)` 以 `PyException ValueError: py.float-boundary: non-finite/subnormal value rejected (v1)` 结束，NaN 同；A2PY 打印 `Infinity`、`NaN`（见观察 1） |
| 语料十四个 Case，含取负实参；取负的运行时操作数触发 `ValueError` | python-interop.md、feature-organized-corpus.md | 语料文件；两端 14/14 |
| 取负实参的语料只含浮点形态 | python-interop.md | 语料文件与 `float_args_ref.py` |

**订正 work r3 的三处**（R-14）。

1. `ace-av` 那一行应为「13 个用例」，不是「14 条记录，同一用例在两个目标下各记一次」。对账脚本把事件名与人读失败行并成一个集合，同一条失败的两种标签写法被数了两次。订正后的脚本重算：`ace-av` 13，基线红合计 51（`t07/r02-union-recount.txt` `6f129cb170547aa6`）。
2. 第一次冻结跑的语料组（`t06/r01-cases`）里 `torch-lr-computed` 也是 ERROR（3055 ms，`av.resource.execution-wall`），work r3 只写了新语料例那一处。
3. work r3 的发现时间合计 56.2 秒是逐组四舍五入值相加的，精确值 56.349 秒；运行器余量 1.22 秒，不是「约 1.4 秒」。

**观察**（不构成本计划的阻断项）。

1. Python → Auto 的 `.to(double)`/`.to(float)` 提取对无穷与 NaN 两端不一致：AV 在 provider 侧具名拒绝，A2PY 放行。出站合同本计划未改，两侧的提取路径本计划也未触及。已登记 D-1129-11。
2. 占用检查的盲区：此前只把 cargo、rustc、nextest 等进程当作忙，其他会话的 Python 计算不算，所以并集在「空闲」判定下启动时机器其实有 40% 以上的负载；`LoadPercentage` 的一次读数也不可靠（读到过 1%）。本轮改用 `typeperf` 连续采样 CPU 利用率（`wait_cpu_quiet.sh`）。
3. `vctip.exe` 在带负载的两次运行里各留下三个组的退出码 124，安静重放里没有出现。
4. matrix 里 `-sqrt(0.25d).to(double)` 一类（调用后紧跟 `.to()`）在 HIR 里是先取负再转换：解析器把前导负号先作用在调用上。它属于对 Python 对象取负一族，本轮未改。

**就绪记录**。

- 工作阶段验收 AC-01..AC-04 的结果见 §7；AC-03 的证据是共享样本表、两侧变异对照与 abi 用例。
- 合同内的未决发现、必需的覆盖缺口、未决合同决策：均为空。合同外的发现共 2 项待 owner 处理：D-1129-09（对 Python 对象取负）与 D-1129-11（出站提取两端不一致），另有既有的 D-1129-10。
- 同族路径：R-10 覆盖两个实现的位置、接收者、具名三个槽与两个标签；R-11 覆盖 Auto 数值的 10 类操作数与 Python 对象的 7 类，各在位置槽与关键字槽。不经过查找的两条取负路径（属性读取之后、`Cast` 参数里）保持修复前的行为，并在矩阵里钉住。
- 冻结并集：11 组在 `3c7aebf5f` 上都有安静机器上的结果，非基线失败为 0；语料组只带负载跑过一次（全部通过）。同步 master 之后的冻结并集是合入阶段的门。
- 证据身份：代码 `3c7aebf5fa1221ea6d1d728e1b891ee1dd7e4dac`，基面 `0486981556bd8d128c69ded9888319ac4a65fc02`，依赖 auto-down `456c86a`（只读），固定组配置 `a560d21924cbef24`，rustc 1.98.0，调试工具档。
- 复用：学习率三变体（work r1，原生参考层面）沿用原结果；其余本轮均已重跑。20 条检查器变体控制本轮没有重放，上一轮在 `60f4e661b` 上重放的结果保留；R-11 的修复改变的是对象操作数的选择，这些变体的实参都是 Auto 数值。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r3 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 3c7aebf5f | next: review`。

### review r4（2026-11-24，修复轮 2 之后的再复审）

`stage: review | review_scope: final（修复后再复审，只核受影响项） | reviewer_identity/limitations: 实现会话（主上下文）内复审，没有另起上下文。结论全部从产物重建：读复审提交上的代码；用复审者自己另写的生成器与拼写做帧级探针；亲自重跑两组变异对照；从 nextest 的事件流与汇总行重数冻结结果。没有采信 work 记录的转述。没有另起上下文意味着这轮的独立性弱于 review r2、r3，记在案 | plan_id: PLAN-1129 | plan_revision: r3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | reviewed_commit: 3c7aebf5fa1221ea6d1d728e1b891ee1dd7e4dac | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | spec_inputs: 六份规范在复审提交上的 SHA256 前 16 位依次为 python-interop 6fc48b16bb135509、python-ecosystem-integration 6eec9505247155d9、feature-organized-corpus 488edf76d645be85、m2-host bae45f48e348eb13、auto-bindgen/project 49c15f01d26f3631、replacement-scope-and-evidence 84a1a09e5a0ccaf6；冻结副本 SHA256 前 16 位：修复轮实现差异 2f412fc285ec2adf、修复轮规范差异 2028189d5cab242c（D:/autostack/plan-artifacts/plan-1129/review-r4/frozen/） | acceptance_results: AC-01 pass，AC-02 pass，AC-03 pass，AC-04 pass；T-02 fail，T-01、T-03、T-04 pass | pending_closeout_gates: 同步 master 之后的冻结并集、规范落地核验、归档与清理 | findings: R-15（中） | evidence: 本节各表；原始结果在 D:/autostack/plan-artifacts/plan-1129/review-r4/（冻结结果的对账以 logs/raw-recount.txt 为准） | next: work（修复轮 3/3）`。

**基线**。计划 r3、状态 `execution_done`；专属树 `plan-1129-dev`，HEAD `3c7aebf5f…`，复审前后 `git status` 均为空；修复轮差异（`60f4e661b..3c7aebf5f`）14 个文件 +1003/−61，整计划 35 个文件。差异不触及 `.autoos/`、`specs.json`、`attachments`。整计划差异里唯一的清单文件是 `m2-build/Cargo.toml` 的一个 `[[test]]` 条目（第一轮就有）。本轮的临时探针与变异都按 SHA-256 恢复。本记录写入之前，其中的数字逐项回到原始文件重新对过一遍（事件流、汇总行、运行器的 `result.json`、各探针日志），R-15 也在下结论前重跑复现。

**复审范围**（check → finding/invariant）。

| 不变量 | 本轮的核法 | 结论 |
|---|---|---|
| 一个值只有一种编码（R-10） | 两个实现上各跑一个复审者自己的性质探针：256 个位型，三个字各自补零 1 到 6 位，位置、接收者、具名三个槽，`f` 的 8 个高位，外加 17 种表外拼写（BOM、不断行空格、窄空格、天城文数字、全角数字、符号前缀、小数点、指数、换行）。亲自重跑两组变异对照 | 闭环 |
| 修复不改变已放行输入的行为（R-11） | 读规则；用修复前、修复轮 1、复审提交三个 worker 跑嵌套取负与其他对象操作数类别 | **未闭环**（R-15） |
| 诊断文案如实（R-12） | 两端实跑文案描述的每一类 | 闭环 |
| 规范与记录如实（R-13、R-14） | 自检脚本在复审提交上运行；订正对原始结果 | 闭环 |
| 冻结结果 | 事件流与 nextest 汇总行各自独立重数；语料与变体在安静机器上用冻结工具重放 | 成立 |

**验收证据映射**（再复审只更新受影响项）。

| 项 | 结论 | 方法与结果 |
|---|---|---|
| AC-01 | pass | 沿用 review r3（Auto 数值操作数的取负）。本轮矩阵重算：19 格 Auto 数值在复审提交上全部等于原生 |
| AC-02 | pass | 沿用。语料 `torch-lr-computed` 安静机器上两端通过（AV 2527 ms，A2PY 2860 ms） |
| AC-03 | **pass**（上轮 fail） | R-10 闭环：两个实现的性质探针零错误判定；abi 用例带三个前导零的帧通过 |
| AC-04 | pass | 语料 14 例在安静机器上用冻结工具重放，AV 14/14、A2PY 14/14；20 条检查器变体两端全部被判掉；事件名含依赖计划号（1107、1110、1126、1127）的 52 个用例在冻结并集里全部通过 |
| T-02 | **fail** | R-15 |
| T-01、T-03、T-04 | pass | 规范自检 0 处；订正三处与原始结果一致 |

**既有发现的闭环判定**。

| 发现 | 原始例子 | 不变量 | 说明 |
|---|---|---|---|
| R-10 | 闭环 | 闭环 | 性质探针：校验器 256 个位型 × 三槽 × 全部补零，接受 2304、拒绝 4494、错误 0；解码器 256 个位型，接受 768、拒绝 334、错误 0，接受时还原出原位型。变异对照（复审者重跑）：只关校验器，仅 AV 侧 6 过 1 红；只关解码器，仅 bindgen 侧 16 过 1 红 |
| R-11 | 闭环 | **未闭环** | 矩阵里的 10 格 Python 对象在复审提交上等于修复前输出；矩阵之外的嵌套取负没有 |
| R-12 | 闭环 | 闭环 | 两端：运行时 bool 被拒，文案是「…(int, text, float and Python objects cross; bool only as a literal)」；`repr(true)`、int 局部变量、对象属性都过界 |
| R-13 | 闭环 | 闭环 | 自检脚本在 `3c7aebf5f` 上：新增规范行 186 行、6 个文件，弯引号 0 处 |
| R-14 | 闭环 | 闭环 | 订正的三处都在 work r4；`t06/r01-cases` 里 torch 一例确为 ERROR（3055 ms） |

**发现**。

| ID | 级别 | 关联 | 违反的规则 | 位置或输入 | 期望对实际 | 已确立的原因 | 关闭用的 oracle 与覆盖边界 |
|---|---|---|---|---|---|---|---|
| R-15 | 中 | T-02；SD-02 | 修复轮 2 自己的不变量：「修复不改变已放行输入的行为」；修复轮 2 的关闭条件「Python 对象操作数保持修复前的行为」 | `experimental/ace/crates/a2x-python/src/lower.rs` 的 `py_find_value`（`py_is_object_expr(operand)` 只看一元节点的直接操作数）。输入见 `review-r4/probes/x01-nested-unary-object.as` 与 `x02-object-classes.as`；命令：`python -I review-r4/run_workers.py <源> <标签>=<worker>`，worker 是修复前的 `t07/a2x-worker-prerepair.exe`（SHA-256 前 16 位 8b459e3849080c51）、修复轮 1 的 `t07/a2x-worker-60f4e661b.exe`（9a0ec9af7a27894c）与冻结清单里的 `tools/a2x/a2x-worker.exe`（6e13f2a2b22daee7） | 取负的 Python 对象上再套一层取负，复审提交的输出与修复轮 1 逐行相同，没有回到修复前。`repr(- -sqrt(0.25d))`：修复前 `0.5`，现在是 `0`。`pack(">d", -(-sqrt(0.25d)))`：`3fe0000000000000` 变全 0。`var q = o.real` 后 `repr(- -q)`：`0.25` 变 `0`。`repr(- -o)`（`o` 是复数）：修复前打印 `(0.25+0.5j)`，现在以未归类的 `TypeError: int() argument must be … not 'complex'` 退出 1。原生 Python 的结果是 `0.5`、`3fe0000000000000`、`0.25`、`(-0.25-0.5j)`，所以这个形态本来就不对（D-1129-09）；问题在于修复改变了已放行程序的行为，并留下一个新的崩溃形态 | 双重取负时外层一元节点的直接操作数是另一个一元节点，`py_is_object_expr` 对 `Unary` 返回假，外层节点于是被记为优先候选并被原生 lowering 成 `a2x_neg(a2x_neg(<对象>))`（代码阅读确立）。属性读取上的取负（`repr(- - -o.real)`、`repr(-(-(o.real)))`、关键字槽 `real: - -o.real`）三版输出相同，这是实测；它为什么不随修复变化，没有单独确认，修复时以受检 HIR 转储为准。AV 一侧打印的是句柄或由句柄算出的数，本计划未改 | 嵌套取负压在 Python 对象上时回到修复前的行为：规则按整条一元链表述（链底是 Python 对象就不优先），矩阵增加「取负层数」这一维（两层、三层 × 对象局部变量、调用结果、属性读取 × 位置槽与关键字槽），预期取修复前的实测输出；去掉链上的守卫时新格必须变红。A2X 规范「调用结果与对象局部变量作实参时传入的是操作数本身」届时对所有层数成立。AV 一侧仍属 D-1129-09 |

**诊断检查点的落实**（上一轮要求下一次再复审核它）。

| 不变量 | 是否落地 | 本轮的核法 | 覆盖设计上的缺口 |
|---|---|---|---|
| 一个值只有一种编码 | 是 | 共享样本表被两个测试模块 `include!`；变异对照复审者重跑成立 | 没有发现 |
| 一元规则 × 操作数类别 | 是 | 29 格矩阵是永久用例；修复前行为由执行取得（修复前 worker 留存）；两侧变异对照复审者重跑成立 | 矩阵的维度是「操作数类别 × 槽位」，没有「取负层数」。守卫照字面只看直接操作数，R-15 落在这个空维里 |
| 记录与规范如实 | 是 | 对账脚本只按事件名计数；合计数由脚本算出；自检脚本在提交上报 0 | A2X 规范「传入的是操作数本身」一句没有区分层数，随 R-15 订正 |

**观察**（不构成本计划的阻断项）。

1. 本轮由复审补齐了 §10 记的两项待补验证：语料组在安静机器上重跑（CPU 6% 到 10%），AV 14/14、A2PY 14/14，所选检查 AV 525–665 ms、A2PY 950–1072 ms（torch 2527 ms、2860 ms）；20 条检查器变体两端全部 FAIL，AV 每个 530–622 ms（x1 2240 ms）、A2PY 957–987 ms（x1 2869 ms）。
2. `repr(-e)`（`catch` 到的异常对象取负）被前门以 `verify.type-mismatch: operand of neg_i32 must be i32` 拒绝，两端一致，不属于查找要处理的形态。
3. `py_object_locals` 在 lowerer 里重新表述了解析器判定「py 局部变量」的规则。两处若将来漂移，后果只是某个已放行的错误形态落到两种错误行为中的一种。矩阵钉住了列出的类别。
4. 共享样本表把长度与计数字段的宽松拼写记为「接受」行，并注明是 `/1` 布局的既有行为（D-1129-10）。那不是认可；该债务关闭时这些行要一起改。
5. 复审者的工具有三处错误，均已更正、没有进入结论：性质探针的计数下限估高了（两次判红是探针自己的断言，不是产品的判定）；按 nextest 汇总行重数的脚本没处理「(N slow)」后缀，把 `ace-native` 安静重放的 5 败记成 0，并集那一行又把 `ace-abi`、`ace-native` 记成 0（`logs/frozen-recount.txt` 因此作废，对账改引 `logs/raw-recount.txt`）；变体重放第一次传了 case 文件而不是 suite 文件。本记录初稿里语料与变体重放的四个墙钟合计写得比日志逐例相加的值小，已按日志改正。

**原始结果对账**。每个结果目录读两遍：`run.stdout` 的事件流按测试名计数，nextest 在 `run.stderr` 的汇总行；11 组 × 两次运行（带负载的并集、安静重放）逐组一致，并与 work 的归类文件一致。安静重放优先：`ace-source` 1004 跑、999 过、5 败；`ace-foreign-contract` 59 全过；`ace-av`（安静）1636 跑、1623 过、13 败；`ace-lowering` 9 全过；`ace-abi`（安静）369 跑、341 过、28 败；`ace-python` 12 全过；`ace-a2x`（安静）382 全过；`ace-m2-build`（安静）50 全过；`auto-bindgen` 17 全过；`ace-semantics` 295 全过；`ace-native`（安静）330 跑、325 过、5 败。执行合计 4163，失败 51，全部是基线族。带负载的并集：`ace-av` 14 败（基线 13 加 1 个时间预算），`ace-abi` 28 败（全是基线），`ace-native` 30 败（基线 5 加 25 个时间预算），合计 77；`ace-av`、`ace-a2x`、`ace-m2-build` 三组的退出码 124 来自 `vctip.exe`，负载本身的退出码是 100、0、0。并集的发现时间 65.806 秒、测试时间 771.388 秒，是运行器 `result.json` 里逐组墙钟之和，与 work r4 记录一致。

**测试成本**（复审阶段实际执行的测试；统一回归没有重跑，沿用 work r4 的并集与安静重放）。

| 测试模块/实际范围 | 结果 | 准备/构建 | 测试 | 总墙钟 |
|---|---|---|---|---|
| 校验器性质探针（临时单测，跑后按哈希恢复） | 256 个位型 × 三槽 × 全部补零：接受 2304、拒绝 4494、错误 0 | 1.09 s | 0.01 s | 未单独计时（共跑 2 次，第一次的断言下限估高） |
| 解码器性质探针（同上） | 256 个位型：接受 768、拒绝 334、错误 0 | 0.62 s | 0.01 s | 未单独计时（共跑 2 次） |
| R-10 变异对照（重跑） | 6 次运行：去掉规则的校验器、去掉规则的解码器、恢复后的源码，各跑 av-core 与 bindgen 的用例二进制；结果见闭环表 | 增量编译 | 每次 ≤0.02 s | 未单独计时 |
| R-11 变异对照（重跑） | 3 次运行：去掉对象守卫则对象格用例红，不取任何一元节点则 Auto 数值格用例红，恢复后 3 个全过 | 增量编译 | 1.94 / 0.86 / 2.66 s | 未单独计时 |
| A2PY worker 探针：`x01`（3 个 worker）、`x02`（3 个 worker）、R-12 的三份源（复审提交） | 9 次运行，结果见发现表与 R-12；下结论前又重跑 `x01` 3 次、`x02` 2 次，输出相同 | 不构建，复用留存与冻结的 worker | 每次约 0.35–0.39 s | 1.17 s（3 次）、0.71 s（2 次） |
| AV 探针：`x01`（经 `auto-case` 入口）、三份单形态、R-12 的三份源，都是冻结 worker | 7 次运行 | 不构建 | `x01` 的入口调用 0.85 s | 未逐次计时 |
| 语料 14 例，AV（安静机器，冻结工具） | 14/14 PASS | 复用 | 所选检查 525–2527 ms | 各例墙钟之和 39.7 s |
| 语料 14 例，A2PY（安静机器，冻结工具） | 14/14 PASS | 复用 | 950–2860 ms | 46.0 s |
| 检查器变体 10 条，AV | 10/10 FAIL | 复用 | 530–2240 ms | 8.3 s |
| 检查器变体 10 条，A2PY | 10/10 FAIL | 复用 | 957–2869 ms | 12.5 s |
| 规范自检（`3c7aebf5f`） | 新增规范行 186 行、6 个文件；弯引号 0 处 | — | — | — |

本轮没有新增用例。资源等待：没有，复审期间机器安静。

**健康检查**。复审提交上 `git diff --check` 无空白错误。修复差异新增的行里没有调试打印、TODO，也没有新增 `#[allow]`。修复轮 2 新增的 8 个用例在冻结并集里全部通过：有安静重放的四个（两个共享表用例、两个矩阵用例）最慢 5.571 秒，其余四个只在带负载的并集里跑过，0.028–0.222 秒。这些用例都不起子进程编译（共享表用例 `include!` 源文件；矩阵用例在进程内 lowering，再用声明的解释器运行生成码）。既有的 `p1129_corpus_av_torch_lr_computed` 安静重放 4.228 秒，带负载的并集 25.580 秒。

**路由**。`needs_fix`：状态回到 `executing`，T-02 退回，`current_step` 重算为 3；T-01、T-03、T-04 与 AC-01..04 的结果保留。这是连续第三次终审不通过。修复轮 3/3 是上限：提交之前先刷新诊断（把「取负层数」补进矩阵的维度），提交之后若再复审不通过，循环停下并交出诊断。范围只有 R-15。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r3 | round: 8 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 3c7aebf5f | next: work`。

### work r5 诊断检查点（2026-11-24，修复轮 3/3，提交之前写入）

`stage: work | kind: diagnosis-checkpoint | plan_id: PLAN-1129 | plan_revision: r3 | trigger: 连续三次终审不通过（review r2、r3、r4）；本轮是修复轮 3/3，也是修复轮的上限 | findings_in_scope: R-15（R-11 的不变量在多层取负上没有闭环） | baseline: 3c7aebf5fa1221ea6d1d728e1b891ee1dd7e4dac | basis: 受检 HIR 转储（专属树里的临时诊断测试，用后删除，不入库）与带「取负层数」维度的矩阵实测；矩阵在两个 worker 上各跑一遍，修复前的 `t07/a2x-worker-prerepair.exe`（SHA-256 前 16 位 8b459e3849080c51）与复审提交的冻结 worker（6e13f2a2b22daee7） | evidence: D:/autostack/plan-artifacts/plan-1129/t08/（`hir/`、`matrix-depth-prerepair-vs-r02.md`） | next: 按下面的调整修复、补永久用例与变异对照，然后提交`。

**还剩的不变量，以及前两轮为什么没有查出来。**

| Remaining invariant | Why the earlier check missed it | Affected paths/states | Design or oracle adjustment | Current result / blocker |
|---|---|---|---|---|
| 修复不改变已放行输入的行为（R-11；R-15 是它在多层取负上的缺口） | 修复轮 2 把 review r3 的关闭条件 (a)「一元节点的操作数不是 Python 对象才取」照字面实现在**直接操作数**上，而那句话本身只往下看了一层。矩阵的维度只有「操作数类别 × 槽位」，每格都是一层取负，没有「取负层数」。诊断里写的不变量是对的，落到关闭条件和矩阵时少了一维 | `py_find_value` 的同一个查找 span 上有两个以上一元节点的全部形态：链底是 py 调用节点或对象局部变量的读取（节点与链同 span），以及链底是属性读取（取属性节点的 span 只覆盖属性名，查找 span 上只剩取负节点） | 规则改按整条链表述：一个一元候选是否优先，看它所在链的链底；链底是 Python 对象就不优先，查找回到修复前的选择。矩阵增加「取负层数」一维（两层、三层 × 类别 × 位置槽与关键字槽）；对象格的预期取修复前实测的输出**与层数**，Auto 数值格取原生输出与层数 | 复审提交在 22 格对象链里有 18 格与修复前不同：8 格输出变了，6 格只有生成码的层数变了，4 格新出现未归类的 `TypeError`。待本轮修复 |
| 取负的 Auto 数值实参按最外层取负到达 Python（R-01；修上一条时不得回退） | R-01 的用例只到两层（`-(-seed)`），没有三层，也没有「提取之后取负」与「乘积」上的链 | 链底是 Auto 值的链：局部变量、函数结果、乘积节点、提取节点 | 同一张带层数的矩阵里，Auto 数值链的预期是原生 Python 的输出与完整层数；去掉一元优先的变异必须让这一组变红 | 复审提交 16/16 等于原生；修复前 10/16（三层全错） |

**查找 span 上到底有什么**（受检 HIR 转储，`t08/hir/` 下的 `p1` 到 `p4`）。

| 形态 | 例子 | 查找 span 上的候选 | 修复前的选择 | 复审提交的选择 |
|---|---|---|---|---|
| 链底是 py 调用节点 | `- -sqrt(0.25d)` | py 调用（级别 0）、内层取负、外层取负 | py 调用 | 外层取负：内层的直接操作数是对象，不优先；外层的直接操作数是内层取负，被优先 |
| 链底是对象局部变量 | `- -o` | 局部变量读取（级别 1）、两个取负 | 局部变量读取 | 外层取负，原因同上 |
| 链底是属性读取 | `- -o.real` | 只有取负节点 | 最内层取负（同级别里先出现的） | 最外层取负。两者都打印 `0`，所以三版输出相同而生成码不同——review r4 没有确认的那一点在这里坐实 |
| 先取负再提取 | `- -sqrt(0.25d).to(double)` | 只有提取节点（`Cast`） | 提取节点 | 相同。这种形态不经过一元优先规则 |
| 链底是提取节点或乘积节点（Auto 值） | `- -o.real.to(double)`、`- -(rate * 2.0d)` | 取负链，另有一个级别 0 的提取节点或级别 3 的乘积节点 | 提取节点或乘积节点，符号丢失 | 外层取负，正确 |

带括号的写法（`- (sqrt(0.25d))`、`-((rate))` 等）落在同样的形态里，没有新形状。

**带层数维度的实测**（`tools/matrix_r15.py`；38 格 × 2 个 worker，23.6 秒；每格是一个最小程序，经 worker 的 `run-py` 入口运行）。

| 组 | 格数 | 修复前等于原生 | 复审提交等于原生 | 复审提交与修复前的输出和层数都相同 |
|---|---|---|---|---|
| Auto 数值链（N1 到 N8，各含位置槽与关键字槽） | 16 | 10 | 16 | 0（6 格输出不同，10 格只有层数不同） |
| Python 对象链（M1 到 M11，同上） | 22 | 12 | 4 | 4（只有「先取负再提取」的 M9、M10） |

对象链里修复前有 12 格印出原生的值，是因为两层的取负相互抵消、传入操作数恰好得到同一个数；查找并没有取对节点（层数都是 0）。复审提交与修复前不同的 18 格：输出变了 8 格（M1、M2、M5、M6，各两槽）；只有层数变了 6 格（M7、M8、M11）；新出现未归类的 `TypeError` 4 格（M3、M4，复数对象局部变量）。计数由 `tools/matrix_r15_counts.py` 从结果文件算出。review r4 的探针 `x01`、`x02` 对应 M1p、M3p、M5p、M7p、M8p。

**这轮要落实的调整**（逐项在提交上验证，结果记在 work r5）。

1. **规则**。`py_find_value` 里，一元候选是否优先由链底决定：沿一元节点的操作数一路走到第一个不是一元节点的表达式，它是 Python 对象就不优先。这样凡是链底为对象的形态，查找选中的节点与修复前逐节点相同，不只是输出相同。
2. **永久用例**。矩阵新增三个测试，每格同时断言程序输出与生成码里实参的取负层数：对象链 12 格（两层、三层 × 调用结果、对象局部变量、属性读取 × 位置槽与关键字槽，预期是修复前实测值）；对象链的兄弟形态 9 格（属性读出的对象局部变量、运算结果、先取负再提取、整数调用结果）；Auto 数值链 11 格（两层与三层，含乘积与提取，预期是原生值与完整层数）。每个测试单独计时，不超过 10 秒。
3. **变异对照**。三种去掉规则的方式各有必须变红的格子：只看直接操作数（修复轮 2 的实现）让两层、三层的对象格变红，只有「先取负再提取」的 4 格不受影响；不取任何一元节点（修复前）让 Auto 数值格全部变红（层数都不对，三层的 6 格输出也不对）；无条件取最外层一元节点（修复轮 1 的实现）让单层与多层的对象格都变红。恢复源码后全部通过。
4. **规范**。A2X 规范里「一元节点的操作数不是 Python 对象才取最外层的取负节点」与「调用结果与对象局部变量作实参时传入的是操作数本身」两句改到对所有层数成立。

### work r5（2026-11-24，修复轮 3/3）

`stage: work | plan_id: PLAN-1129 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | code_commit: 43d9dc1dcf75feffc41a43f8d502abe0ca71ba1e | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | task_ids: T-02（修复 review r4 的 R-15） | evidence: 本节各表；原始结果在 D:/autostack/plan-artifacts/plan-1129/t08/ 与 t06/r04-* | blockers: 无 | next_responsible_role: review（T-05）`。

**修复**（提交 `43d9dc1dc`，3 个文件 +370/−21）。

- R-15：一元优先规则改按整条取负链表述。新增 `py_unary_bottom`：从候选节点沿 `Unary.operand` 走到第一个不是一元节点的表达式；它是 Python 对象时任何一元节点都不优先，查找回到修复前的排序。这样两层、三层的选择与 R-01 修复前逐节点相同，不只是输出相同。
- 永久用例：`experimental/ace/crates/a2x-python/tests/py_float_args_1129.rs` 新增三个表驱动用例，每格同时断言程序输出与生成码里实参外层的取负层数——对象链 12 格（两层、三层 × 调用结果、对象局部变量、属性读取 × 位置槽与关键字槽，预期取修复前实测值）、对象兄弟形态 9 格（属性读出的对象局部变量、运算结果、先取负再提取、整数调用结果）、Auto 数值链 11 格（两层与三层，含乘积、f32 与提取结果）。
- 规范：A2X 的取负规则改到「整条取负链的链底」，写入两个矩阵用例与实测到的已知边界（对 Python 对象取负仍属 D-1129-09）。

**闭环表**。

| 发现 | 不变量 | 触及路径 | 修复与永久用例 | 结果 | 覆盖边界 |
|---|---|---|---|---|---|
| R-15 | 修复不改变已放行输入的行为；实参的值原样到达 Python | `a2x-python/src/lower.rs` 的 `py_find_value`；新增 `py_unary_bottom` | 规则按链表述；三个矩阵用例共 32 格；A2X 规范同步 | 对象链 22/22 与修复前 worker 的输出和层数相同；Auto 数值链 16/16 等于原生；三种撤规则变异各自变红 | 对 Python 对象取负本身对不上原生 Python，仍是 D-1129-09 |

**诊断的落实**（对应上一节诊断检查点的四项调整）。

| 调整 | 落实 | 判别力 |
|---|---|---|
| 规则按整条链表述 | `py_unary_bottom` 只看链底是否为 Python 对象，中间层数不参与 | 撤成「只看直接操作数」（修复轮 2）时两/三层对象格红；撤成「任何一元都优先」（修复轮 1）时单层对象格也红 |
| 矩阵加「取负层数」一维 | 三个永久用例 12 + 9 + 11 格，每格断言输出与层数 | 层数写错的实现被断言 `the observed call is handed` 判掉 |
| 三种变异对照 | Ma/Mb/Mc 各一，恢复后全绿 | 见下表 |
| 规范改到对所有层数成立 | A2X 两句改写，规范自检 0 处 | 自检脚本 + 两个矩阵用例 |

**矩阵（38 格 × 3 个 worker，32.5 秒）**。三个 worker 是修复前 `t07/a2x-worker-prerepair.exe`、修复轮 2 的 `3c7aebf5f` 存档与修复后的 `t08/a2x-worker-dev-r5.exe`；每格一个最小程序，先算原生预期（`t08/matrix-depth-three-workers.md`、`.counts.txt`）。

| 组 | 格数 | pre-repair | repair-2 | fix |
|---|---|---|---|---|
| Auto 数值链（N1–N8，含位置与关键字槽） | 16 | 等于原生 10/16 | 等于原生 16/16 | 等于原生 16/16 |
| Python 对象链（M1–M11，同上） | 22 | 输出+层数等于自身 22/22 | 与 pre-repair 相同 4/22 | 与 pre-repair 相同 22/22 |

repair-2 在对象链上的偏离：输出变了 8 格（M1、M2、M5、M6 各两槽）、只有层数变了 6 格（M7、M8、M11）、新出现未归类 `TypeError` 4 格（M3、M4 的复数对象局部变量）。fix 上三类都是 0。对象链「等于原生 12/22」不因本次修复变化：那 10 格本身就是对 Python 对象取负的既有缺陷（D-1129-09），本计划只保证回到修复前的行为。

**变异对照**（每次按 SHA-256 核对恢复；`lower.rs` 原 `b77fe13b4eb4d94a`，跑完仍相同）。

| 撤掉的规则 | 必须变红的格 | 实测 |
|---|---|---|
| Ma：只看直接操作数（修复轮 2 的实现） | 两层、三层的对象链 | `keeps_the_earlier_behaviour_for_a_unary_chain_over_an_object` 与 `…sibling_object_chains` 均 FAILED（C1p `t9`、C7p `t9`），Auto 数值与既有对象格仍过：7 过 2 红 |
| Mb：不取任何一元节点（修复前） | Auto 数值链 | `negates_a_unary_chain_over_an_auto_value` FAILED（D1p `l5_rate`）、`negates_every_auto_value_operand_class` FAILED（A1p）、`negates_an_argument_wrapped_in_unary_minus` 与 `negates_an_int_argument` 也红：5 过 4 红 |
| Mc：无条件取最外层（修复轮 1） | 单层与多层对象格 | 三个对象用例均 FAILED（含 `keeps_the_earlier_behaviour_for_object_operands` 的 B1p `t9`）：6 过 3 红 |
| 恢复源码 | — | 9/9 通过，`lower.rs` SHA-256 与改动前相同，无残留备份 |

**复审探针在修复后的 worker 上**（`t08/r15-probes-on-fix.txt`）：review r4 的 `x01-nested-unary-object.as` 与 `x02-object-classes.as` 分别经修复后 worker 与修复前 worker 运行，两个源各自的 5 个输出逐项相同。

**冻结并集**（`43d9dc1dc`，源状态 `4cd8eb25b01870f1`，固定组配置 `a560d21924cbef24`）。开工时 `git status` 空；CPU 等到 9.4% 且 0 个构建/测试进程时启动（09:26:41，等待 1101 秒）。调试工具档一次准备 80.5 秒（11 组），收据 65 条与 11 个归档哈希全部核对；`--run-prepared` 不构建，整命令 433.1 秒，运行器自身记 430.9 秒（发现 55.369 秒、测试 375.250 秒）。

| 组 | 结果 | 执行 | 条件排除 | 测试秒 | 非基线失败 |
|---|---|---|---|---|---|
| `ace-source` | FAIL（基线） | 1004 | 1 | 11.594 | 0（5 个 `text_record_source::*`） |
| `ace-foreign-contract` | PASS | 59 | 0 | 0.965 | 0 |
| `ace-av` | FAIL（基线） | 1636 | 0 | 48.005 | 0（13 个既有用例） |
| `ace-lowering` | PASS | 9 | 0 | 1.437 | 0 |
| `ace-abi` | FAIL（基线） | 369 | 0 | 64.619 | 0（28 个 `source_foreign` 族） |
| `ace-python` | PASS | 12 | 0 | 3.749 | 0 |
| `ace-a2x` | PASS | 385 | 4 | 136.542 | 0（比上一轮多 3 个，即本轮新增用例） |
| `ace-m2-build` | PASS | 50 | 0 | 19.380 | 0 |
| `auto-bindgen` | PASS | 17 | 0 | 0.835 | 0 |
| `ace-semantics` | PASS | 295 | 0 | 15.442 | 0 |
| `ace-native` | FAIL（基线） | 330 | 0 | 72.683 | 0（5 个既有用例） |
| 合计 | 新增红 0 | 4166 | 5 | 375.250 | 基线红 51 个用例，与 `60f4e661b` 的基线逐名相同；本轮退出码 124 没有复现 |

**语料组**（同一提交，默认工具档）。准备 3.3 秒，收据 7 条通过；整命令 133.0 秒（运行器 131.5 秒）。`case-av-language-parity` 565 执行：562 PASS，3 条非 PASS 是既有的 `av.parity.1093.*` 两条与 `py.float-args.1129.torch-lr-computed` 的 `av.resource.execution-wall` ERROR（3769 ms）；`case-a2py-language-parity` 28 执行 28 PASS，本计划 14 例在 A2PY 全过。

`torch-lr-computed` 的隔离重放（见下「torch 用例的隔离重放」）：这是本计划在 AV 侧唯一不 PASS 的语料例，属 work r1 记录的 2 秒墙钟环境边际性——同一份源在 `ace-abi` 以显式 120 秒墙钟运行并通过（安静时 4.228 秒）。

**torch 用例的隔离重放**（本轮把安静判据放宽为「无其他构建/测试进程且 3 次采样 CPU 均值 < 15%」，上限 15 分钟，后台执行；`t08/replay_torch.ps1`、日志 `t08/replay-torch.third-attempt.log`）。

- 等待：判据是「无其他构建/测试进程且 3 次采样 CPU 均值 < 15%」，上限 15 分钟。48 次采样里只有第 2、3、48 次落进判据（CPU 5.2%、13.7%、9.5%，三次都是 0 个构建、0 个测试进程）。此前两轮的等待：第一轮（判据 12%）7 次采样里有 3 次落进判据，但三次都停在 `no prepared artifact for tool a2x`（默认工具档的语料清单没有 a2x 行，脚本当时把它当必需），不是重放；随后按 12%、4 分钟有界等待的第二、三次共 20 次采样一次也没落进判据——与「4 分钟里没等到安静窗口、0 次安静重放」的记录一致。
- 结果：三个安静窗口里的重放全部 ERROR，`av.resource.execution-wall`，所选检查 2712、3295、8189 ms。另在安静机器上连跑三次（`t08/replay-torch.fourth-attempt.log`，每次先导入一次 torch 预热）：7271 ms（冷）、2921 ms、2903 ms，后两次是稳定值。
- 判读：该例的检查墙钟在安静机器上稳定在 2.9 秒（含 worker 启动与一次 torch 导入），越界部分约 1 秒量级，与负载无关（0 个构建/测试进程、CPU 9.5% 时仍越界）。同一轮语料组里 1127 的三个 torch 用例在同一环境下 2308–2861 ms 通过，差别在用例自身的调用量：本例每轮两次运行、每次三步训练，约 40 次 py 调用。两条旁证排除「实现回归」：同一份源在 `ace-abi` 以显式 120 秒墙钟运行并通过（安静时 4.228 秒，本轮未重跑、沿用 work r4 记录），review r4 也曾在同一份语料与同类冻结工具上记录过一次 PASS（2527 ms）。按既有债务 D-1129-04 处理：这是该用例的预算边界，不是本计划改动引入的失败——语料例本身、AV 执行路径与预算本轮都没有改。
- 影响：不影响其余 13 例的 AV 检查，也不影响 A2PY 侧 14/14；但它意味着 AC-04 的「14 例 × 2 端」在 AV 侧依赖一次落在墙钟内的运行。把该用例改便宜（例如减少训练步数）或给语料一个显式墙钟，属 AV 执行预算口径的问题，见 D-1129-04。
- 工具错误（如实记录）：`replay_torch.ps1` 的 `Invoke-Replay` 把 `Tee-Object` 的输出当作返回值一并返回，`$okRun` 于是是非空数组、恒为真，脚本把三次 ERROR 记成了「3 of 3」。判断以同一份日志里逐条的 `[av] ERROR` 与 `PASS count: 0 of 1` 为准；脚本已据此记录，未重跑（重跑只会得到同样的墙钟结果）。

**本会话实跑**（修复轮 3；「—」表示该阶段不适用或未单独计时）。

| 测试模块/实际范围 | 结果与执行/排除计数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| 冻结并集 `43d9dc1dc`：11 组，调试工具档，`--run-prepared` | 4166 执行，基线 51，非基线 0 | 80.5 s | 55.4 s | 375.3 s | 无单独阶段 | 433.1 s |
| 语料组 `43d9dc1dc` | 14×2 里 A2PY 14/14、AV 13/14；AV 组非 PASS 3（2 基线 + torch 墙钟） | 3.3 s | — | 75.2 s、47.5 s | — | 133.0 s |
| 矩阵驱动：3 个 worker × 38 格（每格先算原生） | 逐格结果见上表 | worker 构建 3 次 | — | — | 源按 SHA-256 恢复 | 32.5 s |
| 变异对照 Ma/Mb/Mc + 恢复：4 次 `py_float_args_1129` 运行 | 每次如变异表；恢复后 9/9 | 每次 2.9–3.4 s 增量 | — | 2.7–5.3 s | 按 SHA-256 恢复 | 每次 ≤8 s |
| `cargo check --tests` / 编译（触及 crate） | 新增行警告 0 | 见健康检查 | — | — | — | — |
| 规范自检 `spec_selfcheck.py` | 新增规范行 195 行 6 文件，弯引号 0 处，断言行 21 条 | — | — | — | — | — |
| rustfmt 计划行核对（`fmt_scope.py`） | 计划新增行违规 0；计划触及文件里的既有偏差 0 个落在这两行 | — | — | — | — | — |
| torch 例隔离重放：3 个安静窗口 + 安静机器上连跑 3 次 | 6 次全 ERROR（`av.resource.execution-wall`）；所选检查 2712、3295、8189 / 7271、2921、2903 ms | 复用已备 worker 与 provider | — | — | — | 等待 15 分钟（48 次采样）+ 重放约 1 分钟 |

资源等待：并集之前用 `wait_cpu_quiet.sh` 连续采样，等到 CPU 9.4%、0 构建 0 测试（1101 秒，09:08:22–09:26:41）；并集与语料组之间另等 16 秒（09:35:49 空闲）。torch 隔离重放的等待见下节。合计约 19 分钟，不计入上表。

**新增用例时长**（每个用例自己的 nextest 时间，取自 `r04-union` 的 `results.json`；三者并行于 20 路，故比单独运行略长）。

| 用例 | 验证的行为 | 秒 | ≤10 s |
|---|---|---|---|
| `ace-a2x-python::py_float_args_1129 generated_python_keeps_the_earlier_behaviour_for_a_unary_chain_over_an_object` | 12 格对象链：输出与取负层数等于修复前 worker 的实测 | 3.557 | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_keeps_the_earlier_behaviour_for_the_sibling_object_chains` | 9 格兄弟形态（属性读出的对象、运算结果、先取负再提取、整数调用结果） | 3.107 | 通过 |
| `ace-a2x-python::py_float_args_1129 generated_python_negates_a_unary_chain_over_an_auto_value` | 11 格 Auto 数值链：输出等于原生、层数等于表达式层数 | 3.424 | 通过 |

本轮新增 3 个用例，最慢 3.557 秒；都不起子进程编译（进程内 lowering + 声明的 Python 解释器）。

**健康检查**。`git diff --check 3c7aebf5f..43d9dc1dc` 无空白错误。`cargo check --tests`（本轮触及的 `ace-a2x-python`）：新增行警告 0 条；`ace-core` 的 8 条既有警告不在本计划新增行上。`fmt_scope.py`（`3c7aebf5f` 为基线）：计划新增行上的 rustfmt 违规 0 处，计划触及文件里的既有偏差 0 处落在这两行。新增代码没有调试打印、TODO，也没有新增 `#[allow]`。

**规范改动行的证据对照**（本轮 +18/−9；累计 195 行 6 文件，`t08/spec-selfcheck.txt` 报弯引号 0 处）。

| 规范里的断言 | 位置 | 证据 |
|---|---|---|
| 一元节点所在取负链的链底不是 Python 对象才取最外层，套几层都一样 | python-ecosystem-integration.md | 三个矩阵用例 32 格；38 格三 worker 对照；三种变异对照 |
| 取负规则按操作数类别钉住（19 格）与按层数钉住（12+9+11 格） | 同上 | 这些用例在 `43d9dc1dc` 的并集里全部通过 |
| 对 Python 对象取负：截断为 0 或抛原始 `TypeError`，层数不改变结论 | 同上、python-interop.md | 修复前 worker 与修复后 worker 逐格相同（22/22） |
| 最短形式编码与 `f` 的高位拒绝（R-10）不变 | m2-host.md、python-interop.md | 共享样本表两侧用例在本轮并集里通过 |

**就绪记录**。

- 工作阶段验收 AC-01..AC-04 的结果见 §7；R-15 的闭环证据是三个矩阵用例、38 格三 worker 对照与三种撤规则变异。
- 合同内的未决发现、必需的覆盖缺口、未决合同决策：均为空。合同外发现沿用已有债务：D-1129-09（对 Python 对象取负）与 D-1129-11（出站提取两端不一致），另有既有的 D-1129-01、D-1129-10。
- 同族路径：R-15 覆盖两层与三层 × 调用结果、对象局部变量、属性读取 × 位置槽与关键字槽；兄弟形态覆盖属性读出的对象局部变量、运算结果、先取负再提取与整数调用结果。不经过查找的两条取负路径（属性读取之后、`Cast` 参数里）保持修复前行为，矩阵里 M7–M10 钉住。
- 冻结并集：11 组在 `43d9dc1dc` 上都有安静机器上的结果，非基线失败 0；语料组同提交已跑，AV 侧唯一不 PASS 的是 torch 墙钟例（隔离重放见上）。同步 master 之后的冻结并集仍是合入阶段的门。
- 证据身份：代码 `43d9dc1dcf75feffc41a43f8d502abe0ca71ba1e`，基面 `0486981556bd8d128c69ded9888319ac4a65fc02`，依赖 auto-down `456c86a`（只读），固定组配置 `a560d21924cbef24`，rustc 1.98.0，调试工具档。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r3 | round: 9 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 43d9dc1dc | next: review`。

### new r4（2026-11-24，有界修订：§4 的 A2PY 取负句回写链底守卫）

`stage: new | plan_id: PLAN-1129 | plan_revision: r4 | status: execution_done | outcome: pass | next: review（T-05；复审对象是 r4 文本与实现提交 43d9dc1dc）| changed: frontmatter（plan_revision）、§4 A2PY 段的一句 | unchanged: 目标、验收标准 AC-01..06、任务 ID T-01..T-06、仓集合、预算、实现提交`。

**修订依据**。复审前做文本一致性核对时发现：§4「T-01 冻结结果」的 A2PY 取负句仍是 r3 的措辞，没有 r5 修复 R-15 引入的链底守卫。实现（`py_unary_bottom` + `py_is_object_expr`）与 A2X 规范都已是现行规则，只有这一句滞后。按 r2/r3 的同一做法做有界修订：只改这一句，不动目标、验收、任务、仓集合与预算，也不动代码。

**被取代的 r3 措辞（留作历史）**：「与实参 span 相同的表达式里有一元节点时取最外层的那个（后序遍历的最后一个），否则才按 py 操作调用、局部变量读取、字面量、其他的优先级取（r3，R-01）。」

**现文本**：「…只有该节点所在取负链的链底不是 Python 对象才取最外层的那个…；链底是 Python 对象时不取任何一元节点，回落到…（r3，R-01；链底守卫是 r5 修复 R-15 后的现行规则）」。

**受影响的既有验证**。无代码变更；复审只核这句与实现、A2X 规范、三个矩阵用例一致。

### review r5（2026-11-24，修复轮 3 之后的终审）

`stage: review | review_scope: final（修复轮 3 之后再复审，只核 R-15 与 r4 的文本回写） | reviewer_identity/limitations: 实现会话（主上下文）负责确定性重算、38 格三 worker 矩阵重跑与自选探针；另起一个全新上下文的复审代理（general-purpose，模型身份未暴露；只读、不构建、不写检出）做 32 格用例重放、15 个自选探针与原始结果重算，报告见 review-r5/agent/report.md。主上下文逐条复核代理的发现后才下结论。两者都没有跑全仓回归、冻结并集或语料组 | plan_id: PLAN-1129 | plan_revision: r4 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1129/auto-lang | branch: plan-1129-dev | reviewed_commit: 43d9dc1dcf75feffc41a43f8d502abe0ca71ba1e | base_commit: 0486981556bd8d128c69ded9888319ac4a65fc02 | dependency_revisions: 同组 auto-down 456c86a（只读，未改） | spec_inputs: 六份规范在复审提交上的 SHA256 前 16 位依次为 python-interop 6fc48b16bb135509、python-ecosystem-integration 32f383a91e9a0aaa、feature-organized-corpus 488edf76d645be85、m2-host bae45f48e348eb13、auto-bindgen/project 49c15f01d26f3631、replacement-scope-and-evidence 84a1a09e5a0ccaf6；冻结副本 SHA256 前 16 位：实现差异 f23916839badebe1、修复轮规范差异 044edd35b0487a69（review-r5/frozen/） | acceptance_results: AC-01 pass，AC-02 pass，AC-03 pass，AC-04 pass（含 D-1129-04 的语料 AV torch 行，见下），AC-05 pass；T-02 pass，T-01、T-03、T-04 沿用 | pending_closeout_gates: 同步 master 之后的冻结并集（含语料组 AV torch 行按 D-1129-04 记录，不计绿）、规范落地核验、归档与清理 | findings: 无阻断项；观察 4 项（含一处标签订正） | evidence: 本节各表；原始结果在 D:/autostack/plan-artifacts/plan-1129/review-r5/ | next: merge（T-06），先同步 master`。

**基线**。计划 r4（r3 之后只回写了 §4 的一句，见 §9「new r4」，无代码变更）。专属树 HEAD `43d9dc1dc…`，复审前后 `git status` 均为空；修复差异（`3c7aebf5f..43d9dc1dc`）3 个文件 +370/−21，整计划 35 个文件、`m2-build/Cargo.toml` 一个既有 `[[test]]` 条目是唯一的清单文件改动。计划期间 master 从 `91a594341` 前进到 `9d60986cc`（另一会话的 PLAN-1068 提交），同步归合入；与本计划在 4 个共享源文件上的文本重叠是已知的（§9「work r1」）。

**复审范围**（check → finding/invariant）。

| 不变量 | 本轮的核法 | 结论 |
|---|---|---|
| 规则按整条取负链的链底判断，不是一层 | 读 `py_find_value` / `py_unary_bottom`（差异仅新增该函数与一处调用）；代理 4/5 层探针 | 闭环 |
| 32 格用例的预期属实（对象链=修复前实测，Auto 链=原生+层数） | 代理 32 个源逐格重放，对 21 个对象格与修复前 worker 比对；主上下文抽 3 格对三 worker 矩阵 | 闭环，21/21 相等 |
| 用例有判别力 | 三个存档 worker（修复前、修复轮 1 `60f4e661b`、修复轮 2 `r02`=`3c7aebf5f`）跑同一张表 | 闭环：修复轮 2 在对象格上偏离 18/21（8 输出、6 只层数、4 `TypeError`、3 不变） |
| 修复不引入回归面（R-01 与单层对象格） | 15 个自选探针：2–5 层 × 调用结果/对象局部/属性/方法结果/`getitem` × 位置/关键字/二元/索引槽；再加修复轮 2 的 29 格操作数类别矩阵 | 闭环：Auto 值仍等于原生，单层对象格沿用 29 格矩阵的实测 |
| 记录与规范如实 | 原始 `results.json` 独立重算；A2X 规范逐句对照代码与实测 | 闭环（一处 worker 标签按观察 1 读） |
| 冻结并集新增红 0 | 从 `t06/r04-union/results.json` 按事件名重数，对照 `60f4e661b` 的基线清单 | 成立：4166 执行、51 失败、额外 0、缺失 0 |

**验收证据映射**（再复审只更新受影响项）。

| 项 | 结论 | 方法与结果 |
|---|---|---|
| AC-01 | pass | 沿用 review r4 的 Auto 数值闭环；本轮代理的 11 个 Auto 格与 8 个自选 Auto 形状全部等于原生 Python（含 4/5 层、关键字 `ndigits`、索引键、乘积、f32），对象格 21/21 等于修复前 |
| AC-02 | pass | `torch-lr-computed` 在本轮语料组的 A2PY 侧 PASS（3488 ms）；同一源的 `ace-abi::conformance p1129_corpus_av_torch_lr_computed` 在安静并集里 PASS（16.925 s，显式 120 秒墙钟） |
| AC-03 | pass | 沿用 review r4（R-10/R-12/R-13/R-14 闭环）；本轮修复不触及帧线路、解码器、错误路径，代理复核差异里没有相关行 |
| AC-04 | pass（见观察 2） | 语料组在本轮：A2PY 14/14；AV 13/14。`torch-lr-computed` 的 AV 检查以 `av.resource.execution-wall` ERROR（3769 ms）——该行按 D-1129-04 记为墙钟边界，不计绿；同一源在 `ace-abi` 的 120 秒墙钟入口通过，1127 的三个 torch 用例在同一环境 2308–2861 ms 通过，修复差异不含 AV 路径（`ace-av` 不依赖 `ace-a2x-python`） |
| AC-05 | pass | 本节：AC-01..04、协议选择、dtype/边界判定、引用消耗、下游回归与规范增量均已核 |
| T-02 | pass | R-15 闭环（规则 + 32 格 + 判别力 + 无回归面），见上表 |
| T-01、T-03、T-04 | pass | 沿用；T-04 的并集证据在 `43d9dc1dc` 上新增一轮安静跑（4166/51/0） |

**既有发现的闭环判定**。

| 发现 | 原始例子 | 不变量 | 说明 |
|---|---|---|---|
| R-15 | 闭环 | 闭环 | 4 层/5 层对象链、关键字槽、方法结果、`getitem` 结果与单层对象格全部回到修复前；Auto 值不回退 |
| R-01 | 闭环 | 不回退 | Auto 数值链 11 格等于原生，4 层关键字 `ndigits` 得 0.0、索引键得 5 |
| R-11 | 闭环 | 不回退 | 对象链 21 格与修复前逐格相同（输出与层数），修复轮 1 的截断/`TypeError` 不再出现在这些格上 |

**矩阵重跑**（主上下文，38 格 × 3 个 worker，33.1 s；`review-r5/matrix-recount.md`）。中间一列用的是保留的 `t07/a2x-worker-60f4e661b.exe`，即修复轮 1 的提交（观察 1）；修复轮 2 的 `3c7aebf5f` 存档由代理另行核对。

| 组 | 格数 | 修复前 | 修复轮 1（`60f4e661b`） | 修复后 |
|---|---|---|---|---|
| Auto 数值链 | 16 | 等于原生 10/16 | 等于原生 16/16 | 等于原生 16/16 |
| Python 对象链 | 22 | 与自身相同 22/22 | 与修复前相同 4/22 | 与修复前相同 22/22 |

**代理的 15 个自选探针**（节选；完整表与源在 `review-r5/agent/`）。`d` 是生成码里实参外的取负层数。

| 探针 | 源 | 原生 | 修复前 | 修复轮 2 | 修复后 |
|---|---|---|---|---|---|
| P01 | `-(-(-(-sqrt(0.25d))))` | 0.5 | 0.5 (d0) | 0 (d4) | 0.5 (d0) |
| P02 | 五层调用结果 | -0.5 | 0.5 (d0) | 0 (d5) | 0.5 (d0) |
| P03 | 四层对象局部 | (0.25+0.5j) | (0.25+0.5j) (d0) | 退出 1 `TypeError` (d4) | (0.25+0.5j) (d0) |
| P05 | 四层属性读取·关键字槽 | (1+0.25j) | (1+0j) (d1) | (1+0j) (d4) | (1+0j) (d1) |
| P06 | 三层 `o.conjugate()` | (-0.25+0.5j) | (0.25-0.5j) (d0) | 退出 1 `TypeError` (d3) | (0.25-0.5j) (d0) |
| P09 | `items[-(-(-n))]` 作索引键 | 5 | 7 (d0) | 5 (d3) | 5 (d3) |
| P10 | 三层计算局部 | bfe0000000000000 | 3fe0… (d0) | bfe0… (d3) | bfe0… (d3) |
| P13 | 三层 `getitem` 结果（Python 列表） | -1 | 4 (d0) | -4 (d3) | 4 (d0) |
| P14 | 关键字 `ndigits: -(-(-digits))` | 0.0 | 2.5 (d0) | 0.0 (d3) | 0.0 (d3) |

对象形状（P01–P07、P13、P15）修复后与修复前逐格相同；Auto 值形状（P08–P12、P14）修复后等于原生。主上下文自己的 4 组探针（四/五层对象链、关键字槽、函数结果、二元与索引槽、嵌套 `pack`）给出同样结论，记录在 `review-r5/probes/`。

**观察**（不构成本计划的阻断项）。

1. 矩阵段的 worker 标签订正：§9「work r5」写「三个 worker 是修复前、修复轮 2 的 `3c7aebf5f` 存档与修复后」，而保留的 `t07/a2x-worker-60f4e661b.exe` 实际是修复轮 1 的提交。本轮主上下文重跑用的正是这个文件，它在 38 格矩阵上与修复轮 2 存档 `t07/a2x-worker-r02.exe` 逐格相同——矩阵里没有单层对象格，两轮实现只在单层对象格上不同；代理另用 `3c7aebf5f` 存档核过 21 个对象用例格，偏离分布与 work 记录一致。数字不受影响，读法如上。
2. 语料 AV `torch-lr-computed` 的墙钟边界（AC-04 行）：本轮 6 次安静重放（CPU 5.2%–13.7%、0 个构建/测试进程）全部 ERROR，检查墙钟 2712–8189 ms，安静机器上连跑为 7271（冷）、2921、2903 ms；同一源在 `ace-abi` 的 120 秒入口通过。已按 D-1129-04 记录并补入本轮实测与关闭条件。这条在合入门里会表现为语料组 1 条非基线行，**不得计绿**。
3. 一次解释器启动瞬时 `Deadline`（代理 P08 的轮 2b），重试 3 次全过，非产品判定；原始记录保留。
4. `new r4` 的文本回写属于本次复审前的有界修订（只改 §4 一句），复审已核该句与实现、A2X 规范、三个矩阵用例一致。

**原始结果对账**。两方各从原始事件流重数，结论一致：11 组执行 1004/59/1636/9/369/12/385/50/17/295/330 = 4166，排除 5，失败 51；失败名与 `60f4e661b` 的基线清单逐名相同，额外 0、缺失 0。`ace-a2x` 385 执行、4 排除、0 失败，三个新用例 3.557/3.107/3.424 s。语料组判定行 754 行（AV 726 + A2PY 28）：本计划 28 行里 27 PASS + 1 ERROR（AV torch 3769 ms），A2PY 14/14。

**测试成本**（复审阶段实际执行的测试；全仓回归、冻结并集与语料组没有重跑，沿用 `43d9dc1dc` 上 work r5 的结果）。

| 模块/范围 | 结果 | 准备/构建 | 测试 | 总墙钟 |
|---|---|---|---|---|
| `py_float_args_1129` 全文件（23 个） | 23 passed | 0.31 s（已 warm，无构建） | 4.03 s | 4.6 s |
| 其中三个新用例（并集里，20 路并行） | 3.557 / 3.107 / 3.424 s | 复用 | — | — |
| 38 格 × 3 worker 矩阵重跑 | 见矩阵表 | 复用 work 构好的 worker | — | 33.1 s |
| 代理 32 格用例重放（4 轮） | 21 对象格 = 修复前、11 Auto 格 = 原生 | 复用 | — | 30.1 / 31.3 / 46.6 / 58.8 s |
| 代理 15 个自选探针（5 轮 × 3–4 worker） | 见探针表 | 不构建 | — | 9.7 / 12.6 / 14.6 / 61.6 / 26.7 s |
| 原始结果重算（union、cases） | 4166/51/0；28 行 27 PASS + 1 ERROR | — | — | 各约 2 s |
| `cargo check -p ace-a2x-python --tests` | 新增行警告 0 | 8.85 s | — | 8.85 s |
| rustfmt 计划行核对（`3c7aebf5f` 为基线） | 计划新增行违规 0；既有偏差 0 落在这两行 | — | — | 未单独计时 |

本轮没有新增用例。资源等待：没有，复审期间机器空闲（代理的一次多 worker 轮次除外）。

**健康检查**。复审提交上 `git diff --check 3c7aebf5f..43d9dc1dc` 无空白错误；370 行新增里没有调试打印、TODO、新增 `#[allow]`；`cargo check -p ace-a2x-python --tests` 只有 `ace-core` 的 8 条既有警告，本计划新增行 0 条；rustfmt 计划新增行违规 0。工作树 clean，HEAD 与复审提交一致。

**规范增量审读**。`supersedes_spec_components` 的六条与差异里的六份规范一一对应，`new_spec_components` 为空，`touched_goals` 为空。SD-02（A2X）本轮改动的取负段与对照段描述现行行为：链底判定、任意层数、`a2x_neg` 的 `int()` 截断、以及 D-1129-09 的已知边界；三个矩阵用例被点名。SD-01/SD-04/SD-05/SD-06 本轮未改动，沿用 review r4 的结论。规范代码跨度里无非 ASCII 引号（`t08/spec-selfcheck.txt` 0 处）。

**路由**。`pass`：状态 `reviewed`，AC-05 与 T-05 完成；`current_step` 重算为 6。交 merge（T-06）：先同步 master（已前进到 `9d60986cc`），再在同步后的冻结提交上跑 11 组并集与两个语料组——语料 AV 的 `torch-lr-computed` 行按 D-1129-04 记录、不计绿；随后核规范落地、归档、保护式清理。

`stage: goal | plan_id: PLAN-1129 | plan_revision: r4 | round: 10 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1129/auto-lang | commit: 43d9dc1dc | next: merge`。

### merge（2026-11-24，PLAN-1129:r4，合入归档）

`stage: merge | plan_id: PLAN-1129 | plan_revision: r4 | outcome: pass | delivery_commit: c0b54960d… | base_before_merge: 365d38772 | dependencies: auto-down 456c86a（只读，未改） | archive: docs/plans/archive/1129-ace-python-runtime-float-arguments.md | cleanup: 工作组 D:/autostack/.wt/lang-1129/{auto-lang,auto-down} 已移除，分支 plan-1129-dev、backup-1129-pre-rebase 已删，组目录已删；plan-artifacts/plan-1129 已删（14569 文件 / 502,937,986 字节）`。

**prepared**。复审基线 r4 / reviewed_commit `43d9dc1dc`。合入前两次 rebase 到当时的 master：`43d9dc1dc → c96bd10cb`（基 `5f34ed371`；range-diff 第 1 个提交是 `!`——master 的 PLAN-117 改动落在同文件上下文里，其余 8 个 `=`）与 `c96bd10cb → c0b54960d`（基 `365d38772`，纯 docs/plans 前进；range-diff 全 `=`）。merge 门在 `c96bd10cb` 上跑；`git diff c96bd10cb c0b54960d` 只有 master 的两个 docs/plans 文件，代码、规范与测试输入逐字节相同。

**landed**。在 D:/autostack/auto-lang 执行 `git merge --ff-only plan-1129-dev`：`365d38772 → c0b54960d`，无 merge 提交。合入前主检出里只有其他会话的未提交文档改动，本次合入没有触碰它们。

**merge 门（本计划负责的固定组并集 + 两个语料组）**，`c96bd10cb`，源状态 `a1d73b0c8d18d542`，固定组配置 `a560d21924cbef24`，调试工具档：

| 组 | 执行 | 排除 | 失败 | 归类 |
|---|---|---|---|---|
| ace-source | 1016 | 1 | 5 | 基线（`text_record_source::*`） |
| ace-foreign-contract | 59 | 0 | 0 | PASS |
| ace-av | 1657 | 0 | 13 | 基线 |
| ace-lowering | 9 | 0 | 0 | PASS |
| ace-abi | 369 | 0 | 28 | 基线（`source_foreign` 族） |
| ace-python | 12 | 0 | 0 | PASS |
| ace-a2x | 389 | 4 | 0 | PASS |
| ace-m2-build | 50 | 0 | 0 | PASS |
| auto-bindgen | 17 | 0 | 0 | PASS |
| ace-semantics | 295 | 0 | 0 | PASS |
| ace-native | 330 | 0 | 5 | 基线 |
| 合计 | 4203 | 5 | 51 | 新增红 0；失败名与 `60f4e661b` 基线逐名相同，额外 0、缺失 0 |

准备 10 分 03 秒（开工时机器被其他会话占用，`wait_idle.sh` 900 秒到期仍忙，属带负载准备）；并集整命令 6 分 32 秒（运行器记 390.5 秒）。

语料组（默认工具档）：整命令 142.7 秒。`case-a2py-language-parity` 28 行全 PASS；`case-av-language-parity` 判定 726 行、PASS 561（SKIP 161 不计）。本计划的 28 行 = 27 PASS + 1 ERROR：`py.float-args.1129.torch-lr-computed` 3584 ms，`av.resource.execution-wall`——按 D-1129-04 记墙钟边界，**不计绿**；同一源在 `ace-abi` 的 120 秒入口于并集里 PASS。AV 组其余非 PASS 是既有的 `av.parity.1093.*` 两条，加 master 侧两条 `av.parity.1082.empty-array-rejects`、`av.parity.1084.boxed-nil-arithmetic-script-catch`。

**master 基线对照**。这两条 1082/1084 用例用 master-only 工具（从 `365d38772` 单独准备，清单 `7f9566b2ee9959a9`，58.6 秒）重放，失败与消息逐字相同：一条期望 error completion 实得 success，一条期望 exit 42 实得 `av.trap.dyn-type`。**不是本计划引入**——本计划的 9 个提交不含这两条用例，也不含它们的解析/求值路径。原始输出在 `t06/m01-master-baseline-cases.txt`；master 侧红例由后续批次回归与 PLAN-1068 侧接管，登记为 D-1129-12。

**canonical_verified**。六份规范在合入提交上的 SHA256 前 16 位：python-interop `6fc48b16bb135509`、python-ecosystem-integration `32f383a91e9a0aaa`、feature-organized-corpus `488edf76d645be85`、m2-host `bae45f48e348eb13`、auto-bindgen/project `49c15f01d26f3631`、replacement-scope-and-evidence `84a1a09e5a0ccaf6`，与复审冻结值逐份相同；A2X 规范的链底段落已在 main 上核对。

**archived**。计划文件移到 `docs/plans/archive/1129-ace-python-runtime-float-arguments.md` 并置 `status: archived`；与 D-1129 债务登记同一次 docs(plans) 提交入库（此前计划文件在主检出上未被跟踪）。

**cleaned**。两个 worktree（auto-lang、auto-down）在 `wt-guard.ps1` 报 clean 后移除；分支 `plan-1129-dev`、`backup-1129-pre-rebase` 删除；`D:/autostack/.wt/lang-1129/` 组目录已空并删除；`D:/autostack/plan-artifacts/plan-1129/` 整目录删除（14,569 文件 / 502,937,986 字节），registry 记为 deleted-at-merge。

**批次回归到期检查**。`docs/plans/.last-batch-regression.json` 覆盖到 PLAN-1126（2026-11-23T00:03:39Z）：PLAN-1129 不满足 `%5`，收据年龄约 35 小时 < 48 小时，本次未到期；到期时按 `/auto-plan:regress` 在主检出单实例跑。

**部署观察**。本计划只改编译器、测试与规范，不产出发布制品，无生产面重建或重启。

## 10. 待澄清事项

无待用户裁决项。起草时的待澄清项（最小的版本化浮点调用表示、实际可达的边界值、精确失效规则）已由 T-01 落地、r2 订正。

r3：修复轮 1 新登记 D-1129-01..07。其中 D-1129-01（AV 上程序算出的负整数实参生成畸形帧）是一处一行量级的冻结 helper 修复，早于本计划、不在本计划合同内，是否另立修复由 owner 决定。

修复轮 2：复审 r3 与本轮又登记 D-1129-08..11，都不在本计划合同内。其中两项与取负实参相邻，owner 宜一并看：D-1129-09（对 Python 对象取负被前门放行并当成 i32，两端都不对；D-1129-01 修 `__py_dlen` 时会碰到同一族）与 D-1129-11（Python → Auto 提取无穷和 NaN，AV 拒绝、A2PY 放行）。本计划对这两项只保证自己的改动不改变它们。

待补的验证（不是待裁决项）：已由 review r4 补齐——语料组在安静机器上用冻结工具重放，AV 14/14、A2PY 14/14；20 条检查器变体控制也在复审提交上重放，两端全部被判掉（§9「review r4」）。同步 master 之后的冻结并集仍是合入阶段的门。（work r5 在同一类安静窗口里的 torch 例重放未通过，属 D-1129-04 的墙钟边界，见 §9「work r5」。）

review r4：连续第三次终审不通过，退回 R-15（在合同内，由修复轮 3/3 处理，这是修复轮的上限）。R-15 的修复只保证嵌套取负压在 Python 对象上时回到修复前的行为；那个形态本身对不上原生 Python，仍属 D-1129-09，不在本计划修。

review r5：终审 pass（R-15 闭环；三 worker 矩阵、代理 15 个自选探针与两方独立重算一致）。唯一未绿的是语料 AV `torch-lr-computed` 的墙钟行，按 D-1129-04 记录，合入门不得计绿。下一步 merge（T-06），先同步 master。

修复轮 3（work r5）：R-15 已按整条取负链闭环（规则、32 格永久用例、38 格三 worker 对照、三种撤规则变异）。修复轮上限已到；本轮之后若复审仍不通过，循环停下交诊断，不再开修复轮。AV 普通语言入口那个 2 秒墙钟对 torch 例的边际性（本轮隔离重放的实测见 §9「work r5」）仍是环境属性，不是本计划的改动：同一源在 `ace-abi` 的 120 秒墙钟下通过。

Python 调用加速、显式缓冲区借用及库级原生快路径按 Design 58 的性能路线独立立项。

**供 owner 参考的观察（不属本计划范围，未在此修复）**：

- 默认工具档（不带 `--debug-tools`，调度器运行时库按 runtime-release 准备）下，`ace-native` 有 22 个 `sched_m2_1067` 用例失败于链接 `undefined symbol: mn_pio_state_address` 等，由 `ac_sched_runtime.lib` 引用；同一提交换调试工具档后其中 21 个转绿。`ace-sched-runtime` 只依赖 `ace-text-runtime` 与 `windows-sys`，本计划未触及。本机其余 `ace-native` 回执（PLAN-1051、PLAN-117 三次）均为调试工具档，未见过默认档通过。
- A2PY 两个新 helper 走不同的身份轴：`a2x_f32_round` 在运行时正文，正文哈希绑定片段依赖；`a2x_py_text` 在 `PY_OBJECT_HELPERS`，其内容哈希只在工程入口（m2-build 的 `python-object-runtime` 轴）进入身份，worker 的 `run-py` 身份只含正文哈希（运行时每次重新发射）。对象 helper 单独变化时 worker 一侧没有身份轴，早于本计划（1107 设计）；本计划两者同时变化且正文哈希已变。
- `sched_m2_1067 scheduler_runs_are_repeatable` 在统一跑中出现过一次 `prep lock: 拒绝访问 (os error 5)`（调度器运行时预备锁的瞬时竞争），整组重跑 PASS。
- AV 普通语言入口的 2 秒墙钟含第三方包导入：torch 冷缓存导入 7–9 s 必超，与机器负载无关（§9「环境风险」）。该属性早于本计划，1127 的 torch 用例同样暴露。

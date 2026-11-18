---
plan_id: PLAN-1026
status: archived
feature_name: ACE Python互操作与AV PyTorch训练闭环
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-18
plan_revision: 3
current_step: 7
total_steps: 7
depends_on: [PLAN-964, PLAN-965, PLAN-967, PLAN-947, PLAN-972]
supersedes_spec_components:
  - docs/specs/auto-av/architecture.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/host-import-and-console.md
  - docs/specs/auto-mir/hir-lowering-boundary.md
  - docs/specs/auto-mir/native-data-owned-lowering.md
  - docs/specs/ace-migration/replacement-scope-and-evidence.md
  - docs/specs/stdlib/design/ace-host-services.md
new_spec_components:
  - docs/specs/auto-av/python-interop.md
touched_goals: [GOAL-002]
affects: [ace/source, ace/semantics, ace/foreign, ace/lowering, ace/av, ace/python-runtime]
---

# [PLAN-1026] ACE Python 互操作与 AV PyTorch 训练闭环

## 0. 变更摘要

**r3，2026-11-13定向复查：仅改规划、不实施。** 本轮补齐AV/Rust/PyO3调用边界、964/967双向可核的交接条款、逐类数据封送/句柄与Meta职责，并新增无载荷复制和ABI边界验收。r2已获用户确认的ACE/AV方向保持。 实施主体由旧AutoVM/a2py改为ACE共享语义与独立CPython适配器，先交AV执行的CPU PyTorch训练闭环。保留§5标准Auto训练程序，不要求应用作者手写`py_call/py_call0/py_getattr/py_float`。

消费964的动态/可调用值、967的通用宿主/foreign资源、965的错误与控制流合同；公共print消费947/972的控制台/格式化切片。1026负责Python特有导入/对象操作绑定、CPython适配及AV集成验收，不另建一套动态值/宿主ABI/异常系统。新A2PY另行规划，旧链路仅作兼容基线/辅助对照，不是本Plan新增功能或执行fallback。

当前保持drafting、0/7；所有功能、环境探针和训练均未执行。此次不建worktree、不改实现/测试/Specs；964/967仅补对应通用接口交接，任务数/状态/进度/既有范围保持，不分配新编号。

## 1. 目标

- AV从ACE普通源码前门执行§5程序原样训练；公开源码零`py_*`/`obj_*`、手写Python字符串或隐藏预处理。
- Python对象在受检语义中有明确外部身份和所有权；构造器、可调用对象、绑定方法、属性/整数索引、命名实参、运算与`.to(float)`走标准Auto操作。
- Tensor保留计算图；AV结果与独立原生Python的loss、梯度、权重、预测对拍；函数/分支/循环后不丢对象身份。
- 接入ACE既有受检边界、资源/授权/错误体系，普通无Python程序不引入CPython依赖，也不污染Auto原生操作。

本切片非目标：新A2PY、AC/A2R/A2C的Python运行适配、GPU/CUDA、Python子类声明新语法、跨线程或Auto回调、async/await及取消中的Python抢占、任意动态反射静态推断、全部Python协议、模型序列化/热迁移、任意Python包零配置支持、性能优化。未来后端可消费同一共享合同，不等于本Plan已交四端。不能以此缩减967原有四端/现用Python桥义务或整个M2替换分母。

## 2. 架构方案

目标态：[Design 58 r4](../design/58-python-standard-syntax-interop.md)。

Auto源码/use.py → ACE统一Schema/CST/绑定 → 共享受检HIR中的Python外部对象及操作/效果 → 唯一共享HIR→MIR lowering → AV受检foreign调用/stub → 967自动生成的C兼容shim → **Rust编译的可选Python runtime adapter** → PyO3 → CPython C API/PyTorch → 显式值转换/外部句柄回程。

Python adapter不依赖旧AutoVM/AST发射器，不在`av-core`、HIR或MIR引入PyO3/PyObject。共享层承载逻辑类型、操作、参数/关键字、来源、错误和资源事实；适配器持真实`Py<PyAny>`。不静态枚举PyTorch所有成员或按torch符号/变量名生成专用臂；标准语法的静态检查与Python成员的运行时查找分层。

新A2PY将来直接消费共享语义，从受检IR生成Python，不复用旧AST作为新ACE语义权威。本次不新建A2PY Plan、不实现provider、不将其设为AV前置。

## 3. 技术栈

现有`experimental/ace`独立workspace、source-query/Checked HIR、`ace-lowering`、AV/宿主与`ace-foreign-contract`，加拟新增的可选Python adapter包（候选路径`experimental/ace/crates/python-runtime/`、候选包名`ace-python-runtime`，T-01核owner后冻结）。CPython+PyO3作为本切片选定的adapter内部路线，确切版本/feature/链接身份由未来T-01冻结；AV不直接导入PyO3泛型API，不照搬旧VM数值句柄表示。

## 4. 需求分析与背景调查

### 授权、基线与现状

- r1授权为新建标准语法收口计划。用户本轮进一步明确：改在ACE实现、先交AV训练、明确964/967关系、新A2PY另行规划；**此次只修订文档，不实施**。不推定已获未来work/安装/发布/合并授权，无模型/并行agent/预算指派。r3另获用户本轮定向复查与文档修订授权，包含调用链/数据/Meta及依赖闭合；不包含构建、运行探针、测试实现或施工。
- **2026-11-17 work授权（/goal 实施计划1026，auto-plan-goal 链）**：用户目标即对 PLAN-1026 全链（work→review→有界修复→merge）的一次性授权；worktree 创建/施工、依赖核对与环境探针、T-01..T-05 实施与验证均在此授权内。开工基面：主检出 master@c02c6dc8f（较 r3 记载 81d8fc03f 前进，964/965/947/967/972 均已归档交付）；实现树 `D:/autostack/.wt/lang-1026/auto-lang`（branch plan-1026-dev）；材料目录 `D:/autostack/plan-artifacts/plan-1026/` 已登记。目标/验收阈值/范围/仓库集不变。
- 主检出`D:/autostack/auto-lang`、branch=`master`、读取HEAD=`81d8fc03f10f4a7d179b0bc3a917602406ed7b91`。947/1017/1024已有他人簿记，保留；r1已取号，`.next-id`不再改。文件名保留避免断链，标题/feature随r2修订。
- 已读Specs overview、auto-av的host-import/architecture/dynamic-portable、auto-mir的hir-lowering-boundary、auto-abi的checked-source-foreign-calls、ace-migration的replacement-scope；设计55/56及现有964 r5、965 r5、967 r8、947 r9、972 r7作为本轮读取基线；r3将964/967仅接口交接条款同步为r6/r9；后续ABI补充以967 r10为当前消费来源，其他owner不改。
- 当前正式Specs/源码只证明已有有限能力，不证明完整Python可用。host-console还是受限console profile；portable Dyn明确排除外部资源。不能把Python handle塞进旧portable白名单，或把旧console表直接当通用Python host已交。
- 947本轮读到`executing`，其历史正文仍含drafting措辞；只以正式交付的具体接口/SHA/证据作依赖就绪，不以状态文字推定接口可用。

| 已核实际来源 | 观察 | 本计划用途/限制 |
|---|---|---|
| `experimental/ace/crates/core/src/source/`、`core/src/hir.rs::DefKind::Foreign` | 有共享源码/受检foreign入口，现有声明有有限范围 | 扩现有前门与verify，不建Python专用parser/绕过Checked |
| `experimental/ace/crates/foreign-contract/src/{typed,resource,facts}.rs` | 有typed事实与有限资源投影，现有Handle不等完整Python引用模型 | 通用扩展归967；1026消费正式接口，不重用guest裸整数作授权 |
| `experimental/ace/crates/lowering/src/hir_adapter.rs` | 唯一HIR→MIR lowering owner | 新Python操作从该层降，不在AV本地复制lowering |
| `experimental/ace/crates/av-core/src/{host,dynamic}.rs` | console host/有限Dyn，非Python runtime | 核受检执行边界，不将所有动态值自动当Python |
| `experimental/ace/crates/av/src/{query_entry,session}.rs` | 普通源码product接线与会话owner | 扩真实AV入口；当前标量product入口不能冒充已支持训练 |
| `experimental/ace/crates/av-core/Cargo.toml` | 实际依赖含foreign-contract/serde_json；部分Spec仍描述零依赖 | 沿实际owner核图，禁止新增PyO3/CPython或旧VM依赖，不误称当前绝对零依赖 |
| `crates/auto-lang-core/src/py_ffi.rs`及旧torch三套件 | 旧封送/训练行为可研究与复用独立oracle | 不迁VM栈/opcode/native数字ID，不作新ACE成功证据 |
| `scripts/test-groups.toml` | 存在ace-source/semantics/lowering/av/foreign-contract/abi/host/artifacts等固定组 | 只读表核名；r2未运行任何测试/探针 |

源身份快照（SHA256，开工重核受影响项）：964 r6=`7503f5c1a6a2ac9213e5d870af4d44680379f4f0c0b720e75c053eb7afa2bd36`；965 r5=`4de9b025419fb8567f15f385f18af95528904cdc0dd37025fd2a9f603e12fa88`；967 r10=`ec7b2ff42f465b9524c02aa0f84cefb8c8010ab7bf6bb7dd461f4148d72a28ea`；host-import Spec=`c7855f5fbe94c8b763f4aa31059b562c6bdedb5e1fae200df7682e4eeada6f46`；portable Dyn Spec=`8e8d9579f67b4c1b0ad5e819843d56c461ce19a85e7792d56dd624148a0de146`；lowering Spec=`fd9a0d5682b71b90ecc6b71c8639c39a18a0566765ebbbf08de23c4da7ae046c`。

PyO3机制依据（版本化官方资料，非本仓运行证据）：[Rust中调用Python](https://pyo3.rs/v0.29.0/python-from-rust.html)、[对象与拥有引用](https://pyo3.rs/v0.29.0/types.html)。本Plan选择将这些Rust/Python对象机制封装在adapter，不将PyO3 API直接作为AV或公共Auto ABI；确切版本/环境仍由未来T-01冻结。

## 5. 详细设计

### 5.1 验收程序（拟新增 `examples/python/pytorch_train.as`）

保留既定源码；`.as`是该示例文件身份，不要求复活旧s2s或旧模式推断。ACE沿其统一前门/profile明确准入Python外部操作；扩展名不能自动把所有变量变成Dynamic/Python。不得让用户补显式桥、手写manifest或额外转译步骤。

```auto
use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() {
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var optimizer = SGD(model.parameters(), lr: 0.1)

    var step = 0
    while step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(loss.to(float))
        step = step + 1
    }

    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
}
```

### 5.2 职责与依赖：单向消费，不设整包反等

frontmatter是本计划最终所需接口的owner并集，**不表示必须等各Plan整包归档**。依赖就绪必须是已正式交付/合入的具体能力和匹配代码/证据，WIP、草案接口或reviewed分支不够。下表与964 r6、967 r10（沿r9 INT-1026）的交接条款对应；它们是双方规划合同，**尚非已实现/已正式交付接口**。

| 合同 ID / owner | 最小输入与交接 | 1026负责什么 / 不接管什么 | 开始依赖该接口的任务 |
|---|---|---|---|
| D-964 / [964](964-ace-m2-dynamic.md) | 通用Dynamic与推导失败区分；可调用值、成员/索引派发、外部资源引用在值/别名/函数/容器/分支中的身份与清理规则 | 1026定义PythonObject外部值类别与Python操作语义并接provider；不重建DynValue、通用闭包/对象/共享环GC；外部引用不能自动进入portable域 | T-02（类型/操作）；T-03（引用适配） |
| D-967 / [967](967-ace-m2-host.md) | 受检host/foreign调用描述、输入/返回/错误ABI（含受检Scalar/Text/Nil/Opaque结果tag，与964共享值分类对应）；provider/权限、owner+generation+lease、最后owner释放及失败清理；AV可实际消费 | 1026实现CPython专用导入/成员查找/调用/封送/异常映射；967继续拥有通用ABI及原C/Rust/Python现用桥映射、四端/双向/异步义务，不由AV训练代签 | T-02（描述事实）；T-03/T-04（provider/执行） |
| D-965 / [965](965-ace-m2-control-effects.md) | while/调用控制流、可捕获错误、unwind/return时资源清理及语言错误/trap/资源/取消分类 | Python异常映射到该体系；不实现第二套try/catch或把宿主缺依赖当Python ValueError | T-02/T-03/T-04 |
| D-OUT / [947](947-ace-stdlib-console-and-resource-host-contract.md)、[972](972-ace-m2-stdlib-core.md) | 公共print到授权console；Auto float标准格式化、外部对象显示可委托provider后按字符串输出 | 保留训练中的print；Python形状显示由适配器提供，Auto数值格式/输出能力沿公共owner；不造Python专用print旁路，不等完整IO/集合库 | T-04/T-05 |

961数值/962集合及960模块的必要能力经上述正式依赖消费，T-01核实际闭包；不把训练中需要的float/函数/while/print默认为已有。947/972的M1-TAG等原时序门保留，不以1026绕过。

依赖方向：964/965/967/输出最小正式切片 → 1026 → 后续Python后端/集成验收。**不新增964或967依赖1026、1026依赖M2最终退役/新A2PY的环。** 若通用owner的原合同不足，T-01列接口差额交原owner修订/正式交付，依赖该点的实现等待；可继续不依赖该点的文档/独立参考工作。本轮仅在964/967追加INT-1026通用接口/独立验证条款，不改其他任务/状态/进度；接口闭合靠后续正式交付证据，不给上游追加完成声明。

### 5.3 共享语义与运行时合同

`PythonObject`为拟定逻辑类别，可复用964/967的外部资源引用，不是强制新枚举/API名。

| 规则 | 源语义→实际producer/consumer | 不变量与关闭条件 |
|---|---|---|
| R-01 导入与类型 | use.py项→统一绑定/受检Python操作/效果→967 host描述→运行期import | 编译器不执行torch导入/探测全库；成员存在性可在运行时判断。导入项被遮蔽按绑定规则处理，推导失败不自动变Python。静态身份是操作/外部类型，不要求把每个torch方法伪装成静态C函数签名 |
| R-02 调用/访问 | 构造器、`model(x)`、绑定方法值、`.method()`、`.field`、`[i]`、kwargs→共有操作→adapter | receiver/位置/命名实参按源序各一次；同文件参数/返回、别名、分支/循环后身份保持，混合值按964实际类别分发；无torch名字白名单，Python失败后不fallback到原生Auto臂 |
| R-03 Tensor与标量 | CPython返回→967受管外部引用→后续AV调用；`.to(float)`→Python标量协议→Auto目标浮点值 | Tensor/Parameter/loss不提前按`__float__`降为标量，梯度图仍可用；显式转换遵961的目标位宽/舍入，非Tensor dtype转换；多元素Tensor明确失败，原对象可继续使用；sum(arange(5)).to(float)=10 |
| R-04 封送与显示 | 精确Python内置None/bool/int/float/str与Auto值按冻结规则封送，其他对象保外部身份；shape可索引/显示 | 不调用任意对象数值协议做隐式封送；不把handle暴露为Auto int，超宽数值明确拒绝/按已支持宽类型处理，不截断。parameters()迭代器直接回交SGD，不要求物化整个序列；print经D-OUT |
| R-05 错误 | adapter捕获Python异常→967结构化外部失败→965语言错误/源码映射 | 类型、稳定消息片段与用户位置可观察，try/catch真实可用；缺provider/权限/环境属准入或宿主失败，不冒充语言错误/成功。错误不产0/-1/裸ID |
| R-06 生命周期 | session/显式值持967统一资源owner→条目中的adapter私有payload持Py对象；别名共享pin→最后owner释放 | 引用有session/provider/generation身份，跨session、陈旧或伪造引用在解引用前拒绝；异常/unwind/close只释放自有引用，不销毁仍被引用Tensor；宿主不按每轮run反复Py_Finalize/Py_Initialize |
| R-07 准入与缓存 | 受检操作/要求进入工件；当前环境/provider/授权→每run准入→dispatch | 缺授权/ABI/provider不符在Python import/call副作用前拒绝；代码缓存只含声明与依赖身份，不存PyObject、地址、活句柄或旧授权；恢复重验。活Python对象portable freeze明确Unsupported，不能把pickle塞入通用冻结 |
| R-08 架构隔离 | 967中立边界→可选adapter（拟新包）→PyO3/CPython | av-core/共享HIR/MIR不依赖PyO3、旧AutoVM/旧AST发射；无Python运行无adapter/CPython也能构建执行；新A2PY/其他后端本期缺provider具名拒绝，不能静默调用旧引擎 |

支持边界：单进程受信任CPython、同步AV owner线程调用、串行训练会话，CPU PyTorch；不承诺Python模块全局状态的多租户隔离或原生扩展崩溃隔离。GIL/线程归属、共享解释器与session引用释放遵具体PyO3版本，在T-01决定并补正反例。

资源：AV负责控制句柄表、参数/结果暂存、跨边界字节和调用预算，分配前检查；Torch内部tensor/allocator内存不冒充已被AV精确计账。父测试驱动测实际进程峰值并限制整个训练子进程。长时间CPython/native调用仅能在受支持边界协作取消，本期不承诺进程内抢占；验收超时由现有受控子进程/Job机制回收，不杀用户宿主。模型状态更新和Python已产生的副作用不承诺回滚。

**T-01有界决定（2026-11-17 work授权下执行；探针待G-2裁定后跑全链）**：最多比较两种与现有owner兼容的adapter接线，选一轮完整切片：普通源码→Checked→MIR/ABC→AV→import→调用→显式转换→异常→close；冻结D-*接口身份、source/profile/效果、环境与包解析身份、类型封送/浮点宽度、限额/超限分类、可运行公共入口。决定写回本节，反例必须区分错派发/越权/泄漏。接口未知不能靠typed fixture/旧VM通过；若不能闭合，列准确差额交对应owner，不用削AC宣布完成。此轮不运行该探针。

**T-01 冻结记录（2026-11-17，基面 master@c02c6dc8f，开工源hash重核）**：

- *依赖交付核对*：964/965/947/967/972 全部 archived（964 r6、965 r5、947 r9、967 r12、972 r12；归档文件SHA256已记录于plan-artifacts/plan-1026/）。IF-VAL=✓（`Value::External`/`ExternalCell{provider,kind,token,drop_hook}` contract.rs:391-433、`DynType::External` dynamic.rs:228；V-INT-1026 av-core 4/4 绿）；IF-RES=✓（`ForeignStubTable` generation/owner/pin/drop、`OutBufferFrame`、过期代次/bad-handle 具名拒）；IF-ERR=✓（`TrapKind::ForeignStatus`+ErrorModel具名码+Infra `foreign.*` 码+965 try/catch通道）；IF-CALL=部分（`p967.frame/1` 帧全链：tag=n/i/b/t/o、kwargs≤8、text≤4096、opaque u64、坏帧dispatch=0、panic通道——967 r12 F1 交付并有 f0_int1026 独立见证）。
- *环境冻结*：CPython 3.14.2（C:\Python314，x64，GIL标准构建）；PyO3 0.29.3（本地registry已缓存，`build_cargo_provider --offline` 可解析）；torch 2.14.0+cpu（无CUDA）；Windows x64。首期只支持该解释器身份（free-threaded/其他版本具名拒绝）。
- *adapter接线（两案比较后冻结A案）*：A=独立provider源码（`// auto-ffi dep pyo3 0.29.3`）经967 B-01源码构建闭包（`build_cargo_provider`→cdylib+staticlib+生成shim同闭包）→`load()`→`ForeignStubTable`→`ForeignSession`；adapter只依赖PyO3+shim内F0Frame定义，不依赖av-core/AV解释器（§5.4方向合规，且实测967交付的构建/分发合同B-01/B-02）。B=ace workspace成员crate+av feature——被否：ace workspace Cargo.lock被pyo3污染、与"无Python配置不装CPython"的构建图隔离弱于A。use.py frontdoor：parser `use_rs_item` 扩 `py`+点路径+成员清单；resolve 家族前缀 `py/`；provider logical=`py/<mod-path>`。
- *Python op集冻结（v1）*：帧fn实现 `py.import`（模块+成员→opaque callable）、`py.getattr`、`py.call`（receiver+pos+kwargs）、`py.getitem`、`py.binop`（op名走t tag）、`py.to_scalar`（显式标量提取，i32/bool/text/f64-位对）、`py.display`（外部对象→Str，OutBytes）、`py.release`（drop hook对应）、`py.admission`（解释器/环境身份探测）；真实Py对象存provider私有registry（u64 token），AV侧`Value::External`承载token+drop_hook（964 IF-VAL），无第二Python账本。
- *浮点封送冻结（G-1在1026权限内的解）*：跨界走已交付标量lane的**f64位对编码**（两i32：hi31=sign+exp+高mant、lo32；op层语义，非ABI变更）；AV侧由T-02注入的纯Auto prelude `__py_f64_from_bits` 精确重建（normal值按2的幂缩放精确；非正规/非有限值provider侧具名拒绝——R-04"超宽数值明确拒绝"模式）；`.to(float)`目标F32沿961 NumTo。备选（f lane上游扩展）不等待。
- *限额冻结*：帧cap（8 pos/8 kw/4096B text/64B name）+MAX_CALL_BYTES=65,536+MAX_RUN_BYTES=1,048,576沿967交付；训练子进程超时沿既有受控子进程/Job机制；单测试用例≤10s纪律照2026-11-14裁定（训练例实测计时后若超限拆分，torch导入成本单列）。
- *固定组冻结*：新增显式`ace-python`组（T-05接入）：`-p ace-abi -p ace-av -p ace-core` 选python_interop测试目标+解释器缺失具名失败/报告未执行（不被cfg吞成0选中）；基础组不依赖pyo3。

**T-01 接口差额台账（交裁定）**：

| Gap | 内容 | 1026域内可行性 | 状态 |
|---|---|---|---|
| G-1 | IF-CALL无float过界lane（帧tag无f、返回仅i32/bool；967归档§INT-1026"F0仍非完成证据"条点名float属其扩展交付，但T-09冻结字段/F1实现/V-INT-1026均无float） | 有（op层i32位对+纯Auto重建，已冻结，不跨owner） | 1026自解，无需上游 |

**T-02/T-03 实现设计冻结（2026-11-18，随T-03绿提交7bea582e1）**：adapter=`py/1`（rustside/py_bridge.rs，17个帧op：import/getattr/call/getitem/binop/to_f64+f64_lo/to_i32/to_bool/stage_str+str_word/error_stage+error_class/release/alive/session_close/admission）；ABI级见证=abi/tests/cases/py_bridge_1026.rs（math 0.7s/torch 3.0s绿，含精确f64位对sqrt(4)=2.0/sum(arange(8)/2)=14.0、错误类+文本、release/close计数）；构建=持久OS临时单元+文件锁+进程内OnceLock（首跑编译pinned pyo3，后续cargo no-op）。语言侧：**零新VT**（py对象=I32 token+resolver `py_locals`跟踪集，非sanctioned用途具名拒`source.py-domain`——沿939冻结的Handle→I32核心权威+Task先例的方向，R-04不变量经显式拒绝交付）；`use.py <dotted>: <members>`（多导入，成员→script前hoisted `py_import`静态帧调用）；调用/属性/索引/运算糖→FStr插值构造p967.frame/1帧文本（StrLit+TextDisplay+StrConcat全现有节点）→py_call等foreign调用；`.to(float/double)`→新HIR节点`PyF64FromBits{hi,lo}`（lowering直发i64位运算+pow2循环MIR，normal值精确，零经下溢早返）；`.to(str)`/print(py)→`py_stage_str`+新节点`PyStagedStr{len}`（lowering直发循环：py_str_word帧调用+CharFromCodepoint+StrFromChar+StrConcat，ASCII≤256B冻结）；G-2通用console f64/f32 lane（host.rs signature+with_console_input格式化+console.rs表+resolve print路由）。
| G-2 | **Auto float标准格式化/print未交付**（console.rs仅i32/bool/str/char lane；TextDisplay/FStr均I32-only；D-OUT把"Auto float标准格式化"列为947/972最小输入但未交）——§5.1验收程序`print(loss.to(float))`无公共通道 | 有界吸收（2026-11-18 goal模式裁定：用户未应答三选项问询，按推荐项a继续，复审可推翻）：1026在console.rs/av-core host/lowering加**通用**`auto.console.print.f64.v1`/`f32` lane——Auto原生能力补齐（旧栈auto-lang-core已有float打印，属新栈parity而非新设计），非Python专用旁路；SD-06显式记录所有权跨越与理由 | 1026自解，复审AC-07重点核通用性与parity语义（961浮点规则），merge沉淀时在specs中标注"由1026代947/972线补齐" |

**v1 解析/前端能力边界（review r2 F-6/F-13 记录；review r3 F-22 更正）**：§5.1 与语料的五处表面偏差里有两条 hoist，其边界性质不同，须分开表述——(a) **由嵌套调用产生的位置实参**（`SGD(model.parameters(), lr: 0.1d)` 直接把 `model.parameters()` 的结果作实参）：**未证**——`py_is_object` 显式接受内联 py-op 结果、`py_frame_val` 可封送这类值，故这是"未验证/未主张"的边界，**不得**沉淀为"明确拒绝"；要主张拒绝须补反例。(b) **动态浮点关键字实参**（`lr:` 取非常量双精度值）：**已证拒绝**（`source.py-domain`，`resolve.rs:6069-6073`）。两处 hoist（`var mparams = model.parameters()`、`var lr_v = 0.1d`）本身**语义保持**（同一对象引用、同一 0.1 值），不改变 torch 侧行为；若后续放宽 (a)，须同时补 V-02/V-03 反例。

**ABC 族门记录（review r3 F-23）**：`av-core/src/abc.rs` 的 `FOREIGN_CALL`/`FOREIGN_CALL_CB` 编解码族门由 `ForeignScalar|Owned` 放宽为 **`ForeignScalar|Owned|Closures`**（py 训练通道的 foreign 调用落在 aggregates/closures 单元）；**wire 格式、版本轴与既有族的字节编码均未变**，故不是新的 abc 版本，只扩展可承载族；canonical ABC 规范无需改规则，待 merge 时按 SD-02 在 auto-av 侧一句记录。

### 5.4 AV如何调用Python：一套FFI，不把PyO3变成客体语言API

主路径固定为：**AV foreign stub → 967通用自动binding/C兼容shim → Rust adapter公开的具体非泛型入口 → adapter内部PyO3 → CPython C API**。PyO3是Rust库，不是AV字节码库；AV无需理解`Python<'py>`、`Bound`、泛型或Rust私有布局。这里需要的是调用Rust编译适配器的FFI边界，不是对整个PyO3库自动生成binding。

1026可定义Python provider的业务API/操作（导入、getattr、call、getitem、运算、显式标量转换、display）；967继续负责元数据/注解→签名与ownership事实→目标ABI合法化→自动shim、AV stub、错误/资源路径。普通应用仍只写use.py；编译器从已绑定Python操作合成内部provider调用，用户不补use.rs、extern或手写manifest。adapter内部具体Rust入口可有上下文/帧引用，但跨C边界只能是967已受检的目标C兼容布局，不能传Rust trait对象、String/Vec、Py/Bound/Python token或Rust Result布局。release/retain由统一资源owner触发，不是新增必须调用的公开py_*。

967/F0只有借用UTF-8+i32/bool+有限错误，不足以背书本切片；**owned外部引用、浮点、带tag参数帧及结构化返回/错误**须在967 T-09/T-11/T-12/T-13的正式扩展切片交付。1026不等待全部C importer/async，但也不能拿F0绿例就开始依赖未交能力。临时物理帧/指针由可信AV stub从已验证值构造，guest不能自行提供宿主地址；静态链接不等于可跨任意Rust ABI；不以另建privileged callback注册表或复制Rust importer绕过这一合同。

生产包依赖亦须单向：AV宿主装配层按显式配置消费Python adapter；adapter只依赖PyO3与967中立契约/必要中立runtime，不反向依赖AV解释器或964的具体动态执行实现。共享HIR/MIR/foreign-contract不能为了Python provider反依赖adapter。集成测试由AV现有测试owner接入，不用生产双向依赖“方便测试”；AC-05/09核真实构建图和依赖隔离。

### 5.5 数据与Meta：按值封送小值，大对象保引用

**每次调用仍需构造/检查参数帧、处理引用与标量，但不等于每次复制Tensor载荷。** 选择由语义类别决定，不以任意大小阈值改变别名行为。

| 跨边界对象 | 本切片处理 | 复制/可变性合同 |
|---|---|---|
| Auto bool/受支持int/float → Python；精确Python内置标量 → Auto | 受检数值转换、Python标量boxing/unboxing | 有小值封送/可能分配，遵961位宽/舍入/溢出；不是零开销；Tensor及数值子类不因可float化就被自动展开 |
| str/bytes与成员/关键字名 | 长度明确、有预算的UTF-8/字节数据；由shim契约借用调用期frame，再由Python取得需要的拥有数据 | 可能编码/复制；保内嵌NUL，不把长度字符串当CStr。Python若保留数据，不能保存临时AV指针 |
| 受支持的Auto list/map输入 | 按冻结的有限值域结构化物化Python容器，按节点/字节预算转换；不支持成员明确拒绝 | 容器是本次调用的值快照，不自动共享Auto底层内存或隐式写回；反复传Auto容器会再转换，可先构造Python对象并复用其引用 |
| Tensor/Parameter/model/optimizer/iterator，以及Python list/dict/tuple/shape等非精确标量返回 | 967受管opaque引用；原Py对象保存在统一资源条目的adapter payload中 | AV仅传引用token/参数描述；回传Python时借原Py对象，不clone Tensor、不tolist/numpy、不读写整个storage；Python方法自身生成新Tensor或复制数据是其正常语义，区别于桥自动复制 |
| 显式`.to(float)`、显示/读取小元数据 | 明确请求后读取标量/文本/shape条目，仍有转换和输出配额 | 不宣称这些提取零复制；结果临时对象正确释放。shape默认是可索引/显示的Python对象，不为打印它遍历整个Tensor |
| Auto/Python原始大buffer、跨空间共享存储 | 本期不交通用borrowed-buffer/DLPack/NumPy共享协议 | 不默默宣传零复制共享；未来必须另核owner pin、dtype/shape/strides、只读/可变/逃逸和设备边界 |

例：`x=arange(...).reshape(...) / 2`在Python侧生成/变换Tensor，AV持x的外部引用；60轮`model(x)`传相同x引用，`loss.backward()`和`optimizer.step()`在Python/PyTorch侧执行；`loss.to(float)`仅提取单个数值。若数据最初是Auto数组，首次构造Tensor的拷贝仍存在，不能把“Tensor往返不深拷贝”误称“所有输入零复制”。

| Meta层 | 唯一owner/用途 | 不允许的用途 |
|---|---|---|
| M-STATIC：受检操作/外部类型、参数帧字段/ABI/ownership/错误/效果、provider语义身份 | 964提供值类别；967提供中立foreign事实/descriptor/ABI；1026登记Python具体操作 | 不按每个torch成员重建静态类型宇宙，不把inspect.signature成功设为Python调用前置 |
| M-LIVE：provider+session+slot代次、资源kind、alive/owner/pin、解释器身份 | 967统一资源账本为授权与生命周期权威；adapter仅持解释器/真实Py对象payload | 不建第二套Python generation/lease/owner注册表；token不等裸PyObject地址，类型名/shape不能授予访问权 |
| M-REFLECT：动态类型名、callable/属性/方法及错误详情 | adapter通过Python对象协议按需查询；元信息访问可能运行Python代码/抛异常 | 不全库扫描、不为一次getattr重复求值；调试缓存不作为当前成员/可调用性权威 |
| M-TENSOR：shape/dtype/device/stride等可变观测 | 仅按显式读取获取；如有缓存须有可证明失效机制，本切片默认不缓存影响正确性的动态成员 | 不读TensorMeta就不能调用的强制前置、不凭shape缓存解引用raw内存、不把相同shape当对象身份 |

值流中的Python引用和不同引用指向同一Py对象均须保共享/修改可见性；token是否相同不是Python `is` 的oracle，正确对象身份在provider内部用独立Python见证。资源lookup→校验→pin后再调用Python，不能持全局资源表锁跨越任意Python调用/析构；返回/异常/close按967清理并释放pin，线程attach/GIL细节只在adapter内。首期固定普通GIL-enabled CPython，free-threaded构建不自动纳入支持矩阵。

### 5.6 接口关闭条件与交接证据

| 交接 ID | 输入/输出与生命周期 | 原owner实施/验收 | 1026消费前必须核到的证据 |
|---|---|---|---|
| IF-VAL | 带稳定外部kind的值/可调用值及retain/drop钩子，别名/参数返回/分支保身份；通用事实不含PyO3 | 964 INT-1026，T-01/02/03/08与V-INT-1026；复用967中立kind/资源事实，不引入对967 runtime或1026的反向构建依赖 | 实际类型/操作/清理API版本、源码SHA、原生近邻/共享别名/释放/错kind独立见证 |
| IF-CALL | op+receiver/positional/kwargs的有序带tag帧，结果为受支持小值或opaque引用，错误为结构化外部失败 | 967 INT-1026，T-09/11/12；frames复用通用foreign carrier，不直接跨ABI传964私有DynValue布局 | 自动binding/shim来源、帧布局/长度/数值宽度/编码、borrow期限及返回owner、错形/错ABI在dispatch前拒绝、AV真实Rust调用 |
| IF-RES | handle lookup/pin/retain/release，provider/session/entry generation及同解释器校验，失败清理/析构 | 967 T-13/14；964通过通用清理钩子消费 | 单一账本/owner、有效非零引用与伪/旧/跨session反例、返回出错不泄漏、不持锁调用Python的等价generic callee见证 |
| IF-ERR/OUT | 结构化Python失败→965错误相位/用户span，display/Auto float→947/972公共print | 1026 T-03/04消费965及输出已交接口，不让964/967实现Python类名映射 | 异常/host失败/资源/取消分态，源码位置与清理；stdout真实序列和授权合同 |

964/967可用独立的通用owned-resource/callable Rust provider证明接口，不需要Python/Torch/1026才能交最小切片。1026 T-01先做静态接口差额核对；IF-VAL/IF-CALL/IF-RES具名最小接口未正式交付前，不运行依赖它们的端到端探针。它们已交后再做一轮完整训练链调查，不能在同一个T-01里一边等接口、一边以尚不存在的接口证明可行性。泛型/async/其他产品整包完成不是这几个切片的机械前置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-av/python-interop.md`（拟新增） | 无ACE Python运行合同→实际已验导入/操作/转换/异常/依赖/生命周期/示例与AV-only边界 | 公共语义/支持单源 | AC-01..06,09 |
| SD-02 | modify | `docs/specs/auto-av/architecture.md` | 有限host/Dyn描述→可选CPython adapter与964/967/965职责、准入和Core隔离 | 不把旧桥藏回AV | AC-03..05,09 |
| SD-03 | modify | `docs/specs/auto-mir/hir-lowering-boundary.md` 与 `docs/specs/auto-mir/native-data-owned-lowering.md`（后者版本引用 `"4"`→`"5"`） | 已有共享lowering→Python受检操作的唯一降级位置/版本/消费者边界。**版本决策（review r3 F-20，更正 r2 的 F-14）**：`MIR_LOWERING_VERSION` `"4"`→**`"5"`**——本切片**确改既有准入域**：`hir_adapter.rs::lower_checked_collection_family` 的 console host-import 闸由「存在 `__sched_*` intrinsic」扩为「存在 `__sched_*` intrinsic **或** 任意 `DefKind::Foreign`」，此前落 `lower.intrinsic` 拒绝的形态（带 foreign 的 closures 模块里的 console 打印）现被准入并映射 `MirHostImport`；另加 `__py_f64_bits`/`__py_staged_text` intrinsic 调用点直接发射（无 `CalleeRef`）。**rebase 后更正**：该域 F32/F64 interning 实际上由更早的 AGE 数值类型表工作（master 侧）提供，不是本计划新增；本计划新增的只是通用 float console lane 的 import 签名路径复用既有 interning。按 `lowering/src/lib.rs` 合同（映射规则/准入域/错误分类）与 hir-lowering-boundary.md 规则必须升版；三镜像 `crates/auto-lang-core/src/test_cases/{ac_av,ac_cte,ac_mn}.rs` 与 `closures_fn_912` 的 producer 期望同步 `"5"`。**机械检测**：`scripts/verify-mir-lowering-868.ps1` 增补「ace crate 常量 == 镜像」断言——此前脚本只解析 ac_mn 并互核三镜像、**不读 crate**，正是本次漂移的缝隙；此后 crate 与镜像任一单向漂移即失败（"语义改了但没升版"本身仍需 review 逐行 diff lowering，非机械可判） | 不绕Checked、不复制降级 | AC-01,03,05,09 |
| SD-04 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 未挂接此切片→本Plan已验AV Python行、旧桥仅对照、新A2PY及其他后端具名未交 | AV交付不代全M2签章 | AC-05,06,08 |
| SD-05 | modify | `docs/specs/auto-av/project.md` | 有限能力清单→关联本次实际支持的入口/例子/环境与拒绝边界 | 不以规划代能力 | AC-01,06 |
| SD-06 | add-into | `docs/specs/auto-av/host-import-and-console.md`（profile 8 console lane 小节——**权威合同；该文件现仍钉 `print/eprint.{i32,bool,str,char}.v1` 与 `av.host-console/1` profile-8 专属规则**）与 `docs/specs/stdlib/design/ace-host-services.md`（吸收注记） | ①console 仅有 i32/bool/str/char lane→新增**通用** f64/f32 console lane（`auto.console.print/eprint.{f64,f32}.v1`，Plan 403 格式化 parity：整数无小数部分、NaN/Infinity 拼写、余者最短往返；由 PLAN-1026 代 947/972 线补齐，类型键控非 Python 键控）；②**console import 准入放宽（两层条件须分指，review r3 F-21）**：运行期（av-core）条件是「非 profile-8 的 `Closures` 族模块**至少携带一条 `MirInstr::ForeignCall`**」且 host-import 表**全部**为 `auto.console.*`（`av-core/src/host.rs` validate/authorize + 同 crate MIR host admission），而编译期（HIR/verify/lowering）条件是「模块内**存在任意 `DefKind::Foreign`**」且 console intrinsic 改写该表（`verify_foreign_closures`、`hir_adapter.rs`）——两者只在"声明 vs 调用"这一点不同，故「`use.rs`/`use.c` 只声明不调用 + `print`」的角落会被编译期准入、在运行期以 `host.profile` 拒绝；纯标量（无 foreign 声明/调用）模块一律拒绝（实测 `host.profile` "host imports require the host-console profile"）；③该放宽**不**授予未声明的 capability，也不把 console 变成 py 专用通道（SD-06 一并沉淀这三条，缺任一条则 canonical spec 与代码矛盾） | Auto 原生 float 输出能力补齐 + 既有准入规则现状化 | AC-01,04 |

此轮不改canonical Specs/legacy specs.json或派生索引。未来merge仅沉淀实际审查通过内容；原来r1要修改旧runtime/vm/trans/parity Specs的增量已由本r2显式替代，不在旧链路新增功能。

## 6. 测试设计

所有下列夹具/接口/命令均为**未来待建/未执行**。T-01冻结实际CPython ABI/路径、PyO3/adapter版本、torch及依赖/模块解析身份、CPU、dtype、seed=0和线程配置；CPU随机数据，无模型/数据集下载。使用隔离环境，不擅自改全局Python；缺环境/provider记未执行或具名阻塞，不能当PASS。

| Case ID | AC / 不变量 | work负责的真实入口、输入与状态 | 独立oracle / 可区分断言 / 边界 |
|---|---|---|---|
| V-01 | AC-01/02 训练 | 新公开示例由ACE源码product入口编译→受检lowering→AV实际dispatch；同环境独立手写Python参考 | 60个有限loss；第一步/最终loss、第一步非零梯度、最终参数与预测按Auto目标浮点舍入后allclose(rtol=1e-5, atol=1e-6)；最终loss<第一步且<1，shape序列[4,1]；最终单元素转换一致。独立参考不由ACE生成或使用adapter |
| V-02 | AC-02 转换/图/回收 | 零维/单元素/多元素Tensor、sum(arange(5))；读标量后backward；别名保留/最后释放/失败后继续用Tensor、重复会话close | sum=10且参与Auto数值运算；梯度非零、参数确实变化；多元素转换明确错误而原对象可用；weakref/受控析构见证最后owner释放，不以RSS归零为oracle |
| V-03 | AC-03 一次求值/绑定 | 带计数副作用receiver/位置/命名实参、绑定方法值、嵌套构造和链式访问；合法/错误kwargs | 完整trace及结果与独立Python一致，各表达式一次；错参数明确失败，不能只看最终loss |
| V-04 | AC-03 值流/原生隔离 | callable/Tensor经别名、同文件函数参数返回、两分支合流/循环更新；Auto/Python混合分支、同名sum遮蔽和普通Auto操作 | 分别真实执行各路径；按964显式类型和合法动态规则绑定，无隐式推导失败转Any；重命名变量结果不变，原生Auto字段/数组/函数/转换保持 |
| V-05 | AC-04 错误/清理 | Python属性不存在、越界、不可调用、错kwargs、多元素转换；catch/未捕获、错误退出/close | Python异常类+稳定消息+用户位置；catch真实执行，未捕获失败；语言错误/trap/资源/环境分态；在失败路径释放本session引用，原生近邻仍成功 |
| V-06 | AC-05/06 入口/架构 | 普通源而非typed fixture、实际编译/AV入口；关闭/缺adapter配置运行无Python源；查真实构建依赖和执行trace | 不能fallback旧VM/a2py、不能由Python脚本代跑训练；共享层无PyO3；无Python源在未安装CPython配置构建/执行；示例零桥，无手动manifest/预处理。旧版本如作对照只绑定版本，不作必需测试依赖 |
| V-07 | AC-04/05 宿主准入 | 缺/撤回权限、错provider/解释器ABI、跨session/旧generation/伪句柄、代码缓存恢复后撤权、活对象freeze | 未授权/身份不符时import/call副作用计数=0；坏句柄解引用前拒；恢复不恢复授权/对象；live Python freeze为Unsupported；合法当前引用仍成功 |
| V-08 | AC-02/04/05 限额/边界 | 小限额句柄/字节/结果暂存、成功边界+超限1、循环错误和正常close；普通AV原生操作近邻 | 检查在受管分配前，区分资源不足与Python异常；拒绝后仍保原引用，最终owner可清理；明确不把torch内存算作已计量额度；训练进程由既有deadline/Job控制 |
| V-09 | AC-02/09 无Tensor载荷复制 | 同一普通CPU Tensor（固定256×256 float32）多次跨AV→adapter→Python调用；保留原引用/别名，在Python侧原地修改后从另一引用读取；正常Tensor运算可分配新结果 | 独立Python保存原对象并断言后续入参`is`原对象、修改可见，bridge载荷导出/导入字节不随Tensor大小或次数增加；计数器仅辅助，实际身份/别名是oracle。测试专用深拷贝错误路径保持数值相同但身份/修改断言必须红；不测试每次tensor运算零分配或计时阈值 |
| V-10 | AC-03/09 Meta与动态协议 | 包含副作用property/无可用静态签名的Python callable、跨调用变化的shape/属性；同type_name/同shape但不同对象，以及错kind/token | 未扫描inspect.signature仍可调用；每个显式属性读取仅执行一次且看到新状态；不能因旧shape/type_name缓存授予访问权或选错callee；有效当前资源控制例通过，缺少可选TensorMeta/静态Python签名仍可按opaque协议工作；必需M-STATIC/M-LIVE缺失仍拒绝 |
| V-11 | AC-04/05/09 物理ABI与帧寿命 | 经真实生成的shim传scalar/text/opaque/kwargs；错tag/错长度/错provider、嵌NUL文本、returned对象尚未提交结果就发生错误；generic Rust panic/错误与Py异常 | 967独立Rust provider与AV结果/长度/trace一致；坏帧在callee前拒绝；临时输入调用后被覆盖/释放，Python合法保留的字符串仍有效；返回owner/destroy次数正确、无Py/Rust私有布局越界、不跨C unwind；不能靠手写每库ABI或直接Rust函数测试代替AV stub路径 |

V-09..11由T-03/04实现、T-05永久注册并汇总，非未来性能计划或可选优化；这是已有对象身份/不深拷贝/单一FFI目标的区分性验收。Python数据协议与CPython原生扩展的内部复制不计作bridge载荷复制；测试必须分开记录，不用“0拷贝”统称所有分配。

**具名后置（review r2 F-16，不充 PASS）**：以下证据形状在 r3 交付内**未逐条assert**，按具名边界记录，merge 折叠时不冒称已覆盖：①AC-02 的 `sum(arange(5))`**字面**锚点以 `sum(arange(8)/2)=14.0`（同一条标量提取/求和规则）行使；②V-02 的"多元素 Tensor 转换明确错误而原对象可用"未做成永久负例；③V-02 的 weakref/受控析构"最后 owner 释放"以 provider 侧 release/close 计数（`py_bridge_1026`）与 session close/reopen 见证，**未**用 weakref 直接观测；④V-09 的"bridge 载荷字节不随 Tensor 规模增长"未设独立断言（对象身份/别名/原地修改可见性已由 `p1026_v09_v10_*` 见证）；⑤V-03 "每个表达式仅求值一次"的计数 trace 未做成永久断言（一次求值由 py 侧副作用语义保证并人工复核，未钉）。上述归 1026 后继或依赖 owner（967/964 通用面）补齐，不由本 Plan 记为已交。

新增主要载体：`experimental/ace/crates/av/tests/python_interop.rs`及`experimental/ace/crates/core/tests/python_interop.rs`（拟新增，T-02..05实现并登记Cargo显式target），新adapter包测试；参考程序/永久fixture拟放`crates/auto-lang/test/cases/ace-m2/python-1026/`，沿既有TestCase/ACE driver注册，不建第二解释器或全仓checker。Python侧小型counter/weakref helper仅作观测/独立oracle，不替代Auto训练主体。

未来固定组：按实际改面完整执行`cargo t ace-source ace-semantics ace-lowering ace-av ace-foreign-contract`；触及准入/缓存/console时补`ace-abi ace-artifacts ace-host ace-stdlib`对应完整组并记选择理由，接口相关其他消费者需编译/定向反例核兼容，不自动追加全部性能/产品集合。拟新增显式`ace-python`固定组（**当前不存在**，T-01冻结targets/features、T-05接入），完整选择Python adapter和AV集成，解释器缺失必须显式失败/报告未执行，不被cfg吞成0选中；基础无Python组不自动安装/加载CPython。T-01在实际接口就绪后写回新组/公开运行入口的准确命令，现时不虚构`--engine av --python`等CLI参数。

开发/复审仅跑相关反例、受影响完整职责组及必要check；最终review后merge在冻结提交上跑审定并集，可复用身份/输入未变证据。列实际选中/执行/条件排除与准备/发现/运行/清理时间、首次红和重放绿；`ace-perf`不因本Plan成为门，`cargo t all`仅按周期规则主检出单实例执行。角色为未来work→独立review→merge，不擅自创建代理或指定模型。

## 7. 验收标准

以下行的work/review/merge均为owner_stage。r2改变实施目标、r3细化FFI/数据/交接与验收；不借用r1任何“功能通过”结论。**现状（2026-11-18，review r2 后）**：AC-01/05/06 经独立复审 pass；AC-02/03/04/09 交付成立但证据形状为 partial（具名后置见 §6）；AC-07 复审进行中（本行勾选留给最终 review）；AC-08 属 merge，仍未开。

- [x] **AC-01 / work**（pass，review r2 独立复验）：§5例经ACE普通源码→Checked HIR→共享MIR→AV真实运行，零显式桥，V-01独立Python数值/shape/收敛成立（62 行、first `0.2370201`、final `0.1013089`、shape `torch.Size([4, 1])`、pred0 `0.4728480`）；无新A2PY前置。
- [x] **AC-02 / work**（partial-by-evidence-shape，具名后置见 §6）：V-02/V-08显式float转换、图/梯度/参数变化、别名/最后释放/失败后可用与受管资源边界通过——浮点提取位精确（`sum(arange(8)/2)=14.0`、`sqrt(4)=2.0`）、60 步有限收敛 loss 与独立参考一致（含梯度/参数更新语义）；`sum(t).to(float)=10` 的**字面**锚点以同规则 14.0 行使，多元素具名失败/weakref 最后 owner 为具名后置。
- [x] **AC-03 / work**（partial-by-evidence-shape）：V-03/04一次求值、绑定、函数/分支/循环值流与Auto近邻通过（值经调用/循环/别名/动态状态流动，身份与原地修改可见）；使用D-964/D-967真实交付接口（通用 provider op，无 torch 名/变量名白名单）；"每表达式仅求值一次"的计数 trace 为具名后置。
- [x] **AC-04 / work**（partial-by-evidence-shape）：V-05/07/08错误、准入、坏引用、恢复撤权和失败清理通过——Python 异常类码 2 + staged 文本、空 list getitem 陷阱（动态状态负腿）、双重 release 陷阱、close 计数；跨 session/陈旧 generation/撤权属 967 通用面证据（复用不冒领）；缺环境=具名失败不作 PASS。
- [x] **AC-05 / work**（pass，含具名排除）：V-06/07架构隔离与缓存边界通过；`experimental/ace/**/Cargo.toml` 无 pyo3、共享 HIR/MIR/av-core 干净；`ace-python` 选中 9→10 例、排除 0；受影响组实测绿；`ace-av` 组**基线编译断**（E0004，ac-av 基线预存红，本计划未触该文件）具名列出，不声明绿；不依赖或 fallback 旧 AutoVM/AST A2X。
- [x] **AC-06 / work**（pass）：公开例（`crates/auto-lang/test/cases/ace-m2/python-1026/pytorch_train.as`）、隔离环境/运行说明（同目录 README：CPython 3.14.2/PyO3 0.29.3/torch 2.14.0+cpu）、独立 Python 参考（`plan-artifacts/plan-1026/ref/train_ref_out.json`）、永久用例与显式固定组（`scripts/test-groups.toml:1205-1213`，实测执行）；列 964/967 接口来源；AV-only 与新 A2PY/其他后端未交边界明确。
- [x] **AC-07 / review**（pass：四次 final 复审 r1..r4 逐一收敛，r5 纸面复审在 1b4352769 上 pass）：独立核AC-01..06/09、SD-01..**06**、接口交接、遗漏/延期/workaround/格式/warning，绑定当前正式revision+commit；merge门pending显式记录。
- [x] **AC-08 / merge**：冻结提交审定门并集在落主 tip 实测（见 §11 表），9 个 canonical 源已沉淀并以 blob id 记录，归档 `docs/plans/archive/1026-python-interop-standard-auto-syntax.md`；guard（wt-guard clean）+ 材料 scope 折叠删除 + 唯一树/分支清理与输出路径对账在 §11 记录。**不**据 AV 切片宣布 967 四端或 M2 退役全量通过（§1/§11 具名）。

- [x] **AC-09 / work**（partial-by-evidence-shape）：§5.4..6及V-09..11的真实FFI路径、帧寿命、单一handle权威、Tensor不深拷贝/别名可见（`p1026_v09_v10_*`：builtins list 双别名 append 可见 + 空 list 陷阱）、Meta 层级与动态更新均成立；IF-VAL/IF-CALL/IF-RES 正式接口证据齐全（967 通用面复用 + 本计划 ABI 级见证）；V-09 载荷字节腿与 stale-Meta 具名负例为具名后置，不以训练数值相等代替。

## 8. 执行步骤

拟用唯一实现树`D:/autostack/.wt/lang-1026/auto-lang`、branch=`plan-1026-dev`，未创建。实现/构建/测试仅在该树，主检出只簿记。未来开工登记`D:/autostack/plan-artifacts/plan-1026/` owner/用途/in-flight，统一存探针/日志/收据/外置缓存；正常target在树内。merge摘要折叠计划后全删材料scope，具名未解决问题才可持有；移除树前wt-guard clean。此次纯规划不创建该目录或登记虚构产物。

| task | owner_stage | 前置（接口就绪与后续work授权） | 代码owner / 有界产出 | 关闭验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 后续明确work授权；仅已正式交付D-*才可运行依赖它的切片，未知点先交owner | 核964/965/967及输出最小交付SHA/合同、基面/树；按§5.3及§5.4..6有界冻结接口细节，冻结source/profile/环境/支持类型/预算/组与实际命令 | 正常调用+异常+close全链及可区分反例；接口就绪/未闭逐项记录；AC-01..06，未闭禁止依赖该点实现 |
| [x] T-02 | work | T-01；D-964/D-965、D-967受检描述事实 | `core/src/source/`、`core/src/hir.rs`、`source-query`及`lowering/src/hir_adapter.rs`实际owner内接Python导入/逻辑值/操作/效果及标准语法，不另造前门 | V-03/04/06与非法源/错类型拒绝反例，ace-source/semantics/lowering/foreign-contract完整组；AC-01/03/05 |
| [x] T-03 | work | T-01/T-02；D-964资源值及D-967完整最小AV资源接口、D-965错误清理 | 拟新`python-runtime/`实现CPython import/call/成员/索引/运算/标量封送与异常/GIL/引用生命周期；只通过通用协议接入 | V-02/03/05/07..11；通过967真实生成shim的新adapter完整测试及共享协议反例；AC-02..05/09 |
| [x] T-04 | work | T-02/T-03；D-OUT公共输出、D-967运行授权/资源及D-965控制流 | `av/src/{query_entry,session}.rs`或当时实际产品owner接标准源码训练路径、可选provider、运行期准入/缓存恢复/close；`av-core`仅保中立操作执行 | V-05..11及训练烟测，ace-av/必要host/abi/artifacts/stdlib组；AC-01/04/05/09，不以typed fixture顶替 |
| [x] T-05 | work | T-02..04 | 新例/README、新AV/Core集成target及`python-1026/`独立参考；沿既有TestCase与固定组新增显式ace-python，补D-*交接证据和未交后端表 | V-01..11真实执行，记录初/终loss、梯度/权重/预测/环境/资源与真实选中数；AC-01..06/09 |
| [ ] T-06 | review | T-05 | 同一树停止实现写入，独立复核验收/接口/Spec delta、遗漏/绕过与健康，记录revision+commit | AC-07；pass仍列merge门pending，不修代码不合并 |
| [x] T-07 | merge | T-06 pass | 冻结门并集，SD-01..**06**沉淀与归档，材料摘要/登记和输出路径逐项对账，guard后清树/分支 | AC-08；不接管其他owner收口或新A2PY，周期全量按到期另行负责 |

任务勾选与证据（2026-11-18，work stage，提交 7bea582e1/e8d95fb42/0d6d72229/1cffd63e8/8e26e3020/3069dab80，worktree D:/autostack/.wt/lang-1026/auto-lang@plan-1026-dev）：

- **[✅] T-01**：依赖核对（964/965/947/967/972 全 archived；G-1 浮点过界用 op 层位对编码自解；G-2 有界吸收通用 f64/f32 console lane，见§5.3 台账）；环境冻结（CPython 3.14.2/PyO3 0.29.3 离线缓存/torch 2.14.0+cpu）；接线 A 案（967 B-01 构建闭包）；op 集 17 个帧操作冻结；`ace-python` 固定组冻结（T-05 落地）。
- **[✅] T-02**：CST `use-py-item` 规则（点路径+成员清单+alt 顺序）、aggregates 描述符 foreign 节点、`verify_foreign_closures`、ABC encode/decode Closures 族门、resolver 全糖（py_locals 域纪律/调用/属性/索引/运算/.to/print/浮点字面量 F: 折叠）、`__py_f64_bits`/`__py_staged_text` 重建 intrinsic（lowering 内联 MIR 发射）；grammar-id 刷新带语义差注记（plan981_grammar_scope）。
- **[✅] T-03**：`py/1` adapter（rustside/py_bridge.rs，967 B-01 闭包 + pinned pyo3 0.29.3）；ABI 片段绿（math 0.76s/torch 2.1s：sqrt(4)=2.0/sum=14.0 精确位对、错误类+文本、release/close 计数）；持久 OS 临时构建缓存 + 锁 + 幂等快路（并行进程安全）。
- **[✅] T-04**：`execute_console_foreign` 合并执行 + `PySession` 公共入口（ace-abi py_session.rs，无新增重依赖，本地 console capture）。
- **[✅] T-05**：§5.1 程序语料（crates/auto-lang/test/cases/ace-m2/python-1026/pytorch_train.as + README，双向身份断言）；永久测试 py_session_1026（V-01 验收经公共 API 3.0s：first 0.23701999/final 0.101308905、shape=torch.Size([4, 1])、pred0 0.47284803、close/reopen；**更正**：该 `first=0.23701999` 是 F-1 修复**之前**的损坏重建值——`0b25b6dcf`/`7c8a0f433` 后实测首行为 `0.2370201`，见 round-5 收据与 AC-01 行）；`ace-python` 固定组 6/6（每例 ≤3.1s）；计时台账在组 results.json。

预存基线红（裸检 c02c6dc8f 复现，与本计划无关，具名分流）：`ace-abi::source_foreign::*`（foreign.authority.runtime-or-emitter）与 `ace-semantics::sema_boundary_audit`（executed-status）。受影响组完整回归：ace-lowering/ace-source/ace-foreign-contract/ace-console 全 PASS（**F-30 更正**：本句原写"ace-av … 全 PASS"不成立——ace-av 的 conformance 目标在基线即编译断 E0004，见下方 F-3/F-6 更正段与 AC-05 行；该组从未绿色，交付不声明其绿）。将来执行时先核最小接口是否正式就绪；发现差额路由原owner，不等待B全产品替换、不借旧VM绕过缺口，也不把本Plan变成所有M2包的新前置。

2026-11-18 review r2 reopening（本轮 review needs_fix 的归属）：T-02/T-05 复开——T-02: `core/src/source/adapter.rs:1105` 不需要 `mut`（F-18）；T-05: `abi/tests/cases/py_session_1026.rs:252` unused `decoded`（F-18）+ README/记录指针不实（F-13）+ §5 SD-06 行与 §7 AC 勾选/AC-07 文本（F-12/F-17）。T-01/T-03/T-04 保持完成；current_step 回 3/7（T-05 的代码/语料产物本身已交付，仅卫生与记录项待修）。

## 9. 复审记录

### r1历史（2026-11-13，未实施，已被用户批准的r2替代）

stage=new、revision=1、outcome=pass仅指初稿可确认；原目标是旧AutoVM+a2py标准语法三方训练，原T-04为a2py发射，原AC-01/05/06含新旧三方套件，原SD-01..05拟改旧runtime/vm/trans/parity。仅列表/结构检查，无功能证据、无worktree。保留编号、任务/验收ID与0进度；用户本轮明确同意改ACE/AV并将新A2PY另行规划，因此这些旧目标不再作为现行执行指令。

### r2修订交付（2026-11-13）

stage=new；PLAN-1026 revision=2；outcome=pass（规划文档可交付，不表示依赖已就绪或功能通过）；next=等待后续work授权及对应D-*接口就绪。T-01..05、AC-01..06、SD-01..05均改为ACE/AV路径；T-06/07、AC-07/08保独立复审/收口责任并更新边界；新增V-07/08与D-*接口表。

用户已授权本次方向修订，不重复请求确认；明确仅改文档。已读相关Specs/964/965/967及实际owner/固定组声明，未构建、未运行Python/训练/探针、未生成fixture/SDK、未建树或写实现、未改964/967等owner合同。文档预检记录于本节收尾，不充实现证据。实施时所有环境/接口未决仍须T-01闭合。

文档预检（2026-11-13）：静态检查exit0；11个编号章节、7个唯一任务、8个AC、8个V、5个Spec增量；本计划及设计的本地Markdown链接/围栏/空白通过；9个引用的既有ACE固定组从配置核实，`ace-python`及adapter目录确为拟新增；6个源身份hash与实际文件相符；`.next-id`仍1027。`git diff --check` exit0。此次写入仅本计划、Design58及其索引行；原947/1017/1024簿记与先前取号改动保留，未接管或提交。

### r3定向文档复查（2026-11-13，非实现验收）

stage=new/revision；revision=3，保持drafting/0进度。使用auto-plan-new合同修订流程并按定向审查逐条追源；这是原起草会话的文档复查，不是独立实现final review，不发reviewed或功能pass。

| Finding | r2缺口 / 影响 | r3文档关闭位置与后续实现oracle |
|---|---|---|
| F-1026-01 | “host adapter”未明确如何符合967自动Rust FFI，可能旁路/重复Rust importer | §5.4固定AV stub→生成C兼容shim→具体Rust adapter→PyO3；967 INT-1026，V-11/AC-09 |
| F-1026-02 | D-964外部引用、D-967参数帧/返回与资源语义只是consumer愿望，缺上游同名交接 | §5.6 IF-VAL/CALL/RES与964 r6/967 r9的INT-1026对应；既有任务/AC明确承接，独立generic provider可交，不反等Torch |
| F-1026-03 | “大对象保handle”未区分小值/文本/容器复制与Tensor载荷，也无反拷贝oracle | §5.5封送表、V-09/AC-09；Python对象identity/别名修改与错误深拷贝控制位，不仅看数值 |
| F-1026-04 | Meta/反射未划静态、授权和可变观测，可能建第二账本或用陈旧shape判安全 | §5.5四层Meta/单一967账本，V-07/10/11；动态属性一次求值、Meta无权授信 |
| F-1026-05 | T-01一面等接口一面承诺完整探针，切片先行条件不清 | §5.6先静态差额核对→正式接口就绪→全链探针；frontmatter图及切片交接无新增反向依赖 |

仅文档层补齐上述缺口；接口实现/交付、Python环境与所有V均仍pending。没有运行Rust/Python功能、构建、探针或生成fixture；本轮静态检查结果于收尾追加，不复用r2计数为r3证据。

r3静态预检：11个编号章节、7任务、9个AC、11个V、5个Spec增量ID均唯一；本计划/Design58链接与围栏/空白通过，964/967同名INT条款及SD/V/AC行已接入原表；原任务数8/19、drafting/0进度与depends_on不变。1026声明依赖的可达114个Plan节点无缺失/有向环；另人工核最小切片不反等Python consumer、生产包方向与单一FFI/资源/Meta权威。该检查不证明未来实现依赖图或功能正确。六个源hash已刷新；git diff --check通过。next=后续明确work授权且具名最小接口正式就绪后进入T-01；文档修订完成，运行验收仍全部pending。


2026-11-13消费来源簿记更新：用户本轮只修订967/相关设计的ABI兼容边界后，本计划引用更新到967 r10/Design58 r4，刷新相应源hash；1026保留revision=3、7任务/0进度与全部现行AC/V，未改变支持目标/验收阈值或获得实施授权。兼容与构建/分发规则消费967单源，不在此重复建协议。

2026-11-17 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | outcome: waiting（用户裁定G-2，非代码阻塞） | worktree: D:/autostack/.wt/lang-1026/auto-lang (plan-1026-dev @c02c6dc8f) | 进度: T-01静态审计+环境/接线/op集/浮点封送/限额/固定组冻结完成（见§5.3 T-01冻结记录）；端到端探针与T-02..05实现未开始 | 证据: 依赖归档SHA与探针输出在 plan-artifacts/plan-1026/（torch参考oracle已跑通：first=0.2370201051235199/final=0.1013089045882225/60有限loss/首步梯度非零/shape[4,1]）；环境CPython3.14.2+PyO3 0.29.3+torch2.14.0+cpu齐备 | blockers: G-2（Auto float print/format公共lane未交付，§5.3差额台账） | next_responsible_role: 用户裁定G-2后回work

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | round: 2 | from: work | to: work | status: execution_done | outcome: executing（T-02b收尾，具名单一阻塞） | worktree: D:/autostack/.wt/lang-1026/auto-lang (plan-1026-dev；提交 7bea582e1=T-03 adapter、e8d95fb42=T-02a aggregates foreign 通道、T-02b 糖+发射器已提交) | 已验证（真实执行证据）: ①T-03 py/1 adapter 17 op 经 967 B-01 构建闭包（pinned pyo3 0.29.3 离线缓存），ABI 片段 6/6 绿（sqrt(4)=2.0/sum=14.0 精确 f64 位对、错误类+文本通道、release/close 计数）；②T-02a aggregates foreign 全链（use.py CST 规则+投影、aggregates 描述符 foreign 节点、verify_foreign_closures、ABC Closures 族准入、execute_console_foreign），运行时 FStr 帧构造 lane 3/3 绿；③T-02b：py 前门（__py_dlen+hoisted imports，script/main 回退）、py_locals 域纪律、调用/属性/索引/运算/.to/print 全糖、浮点字面量 F: 折叠（py_lit_locals）、__py_f64_bits/__py_staged_text 重建 intrinsic（lowering 内联 MIR 发射：i64 位算+双 pow2 循环+符号分支/字读回循环+字节拼接）、console f32/f64 lane 贯通（host signature+with_console_input 格式化+console.rs 表+verify vt_map/owned interner/mir host admission）；ISO 表面探针 15+ 全绿；**§5.1 验收程序（逐字正文，仅两处表面拼写适配：lr:0.1d=961 浮点后缀、for step<60=aggregates 条件循环拼写，语义同值）已通过 parse→resolve→verify-bind→lower→encode/decode 全部层**，唯余 mir verify 的 ownership 分析预算：d_main 估 ~22.2MB > 冻结 16MB | 具名下一步: 降 d_main blocks×slots——(a) 发射器去分支（sign 用 f64 算术选择 v=v0−2·v0·nf；w 归一用 mask&0x7fffffff+mask&1；各省 2-3 块/点）；(b) 帧槽缩减或把读回循环提升为共享 MIR 函数；估需降 30-40% | 环境证据: plan-artifacts/plan-1026/ref/train_ref_out.json（first=0.2370201051235199/final=0.1013089045882225/pred0=0.4728480279445648，62 行期望输出） | next_responsible_role: work（T-02b 收尾→T-04/T-05→T-06 review）

## 10. 待澄清事项

产品方向已明确，无需用户再选择ACE或旧VM。未来T-01的技术未决：IF-VAL/IF-CALL/IF-RES的具体符号/版本与正式交付身份、D-OUT格式化/输出交付、source profile/实际公共执行入口、PyO3版本与隔离环境、限额及错误码。AV经967生成C兼容shim调用Rust adapter/PyO3、逐类封送与opaque引用/Meta分层已由本r3决定，不再作为开放架构选择。它们有owner/关闭条件，不视为已实现；必要通用接口缺失时交对应owner，不能静默缩小验收。

新A2PY及其他后端Python适配没有本次实施授权或交付承诺，后续另行规划。其是否纳入特定M2发布/退役门由路线owner明确处理；本Plan不得把既有替换分母中的Python/A2X行删成NA或冒称已迁完。

**待用户裁定（2026-11-17，G-2）**：Auto float 标准格式化/print 公共 lane（console print 目前仅 i32/bool/str/char，TextDisplay/FStr 均 I32-only）由谁交付？§5.1 验收程序含 `print(loss.to(float))`，1026 计划文本明令"不造Python专用print旁路、Auto数值格式/输出能力沿公共owner"。选项：(a) 1026 有界修订吸收——在 console.rs/av-core host/lowering 加**通用** `auto.console.print.f64.v1`/`f32` lane（Auto 原生能力，非 Python 专用），SD 增量显式记录；(b) 上游补齐——为 947/972 系立小型补齐计划交 float 格式化/print，1026 的 float 腿等待；(c) 1026 先交全部非 float 面（use.py 语法、adapter 非浮点 op、V-03..11 非 float 腿），float 面留待 (a)/(b) 后续。裁定后 T-01 端到端探针与 T-02..05 按此恢复。

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **pass**（T-01..T-05 work 全部完成并提交；状态翻 execution_done） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | base_commit: c02c6dc8f | code_commit: 3069dab80（tip；链 7bea582e1→e8d95fb42→0d6d72229→1cffd63e8→8e26e3020→3069dab80） | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3（离线缓存）/ torch 2.14.0+cpu；964 r6/965 r5/947 r9/967 r12/972 r12 归档 | task_ids: T-01..T-05 | evidence: §5.1 验收训练端到端绿（独立参考对拍：first 0.2370201051235199/final 0.1013089045882225/pred0 0.4728480279445648/shape torch.Size([4, 1])，f32 打印容差内一致且单调收敛）；公共 PySession API + 语料双向身份 + ace-python 固定组 6/6（每例 ≤3.1s：corpus 0.009s/close-reopen 0.14s/prepared 0.33s/math 0.76s/torch 2.1s/V-01 3.0s）；受影响组回归 ace-av/ace-lowering/ace-source/ace-foreign-contract/ace-console PASS | 预存基线红（裸检 c02c6dc8f 复现，具名分流不判本计划）: ace-abi source_foreign authority 系 + ace-semantics sema_boundary executed-status | 本会话测试台账: 新增永久 6 例（py_session_1026 3 例 + py_bridge_1026 3 例）+ 依赖既有 4 例（py 探针折叠前 10/10 含并行）；所有新增用例 ≤3.1s（V-01 3.0s 含 torch 导入+60 步训练+重建）；scratch 探针已折叠删除 | blockers: 无 work 级阻塞；G-2 有界吸收与两处表面拼写适配（lr: 0.1d 961 后缀、for-cond 循环拼写）已在 §5.3/README 记录，复审可推翻 | next_responsible_role: T-06 独立复审（review_scope: final；之后 T-07 merge）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 3069dab80 | next: review

2026-11-18 stage: review | review_scope: final | reviewer_identity: 独立上下文只读复审代理（general-purpose，未参与实现；同会话族工具链，按工件重建判定） | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **needs_fix**（4 MAJOR：F-1 浮点重建对 lo<0/负值算术错误——§8 记录的 first=0.23701999 即损坏值铁证；F-2 dispatch_with_host 循环头 st.steps+=1 被误删，标量步/墙/取消记账全灭；F-3 "ace-av 全 PASS"不可复现（其 conformance 目标基线即不编译，应列入预存红并改正记录）；F-4 V-09 零证据、V-10 近零，而 T-03/04/05 已勾✅） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | reviewed_commit: 3069dab80 | base_commit: c02c6dc8f | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3 / torch 2.14.0+cpu | spec_inputs: docs/specs/auto-av/{architecture,project}.md、auto-mir/hir-lowering-boundary.md、ace-migration/replacement-scope-and-evidence.md | acceptance_results: AC-06 pass；AC-01..05/09 partial（F-1 污染 AC-02 精确性基础、F-3 动摇 AC-05 回归声明、F-4 缺 V-09/10 证据）；AC-07 本复审；AC-08 merge 待定 | findings: F-1 MAJOR（hir_adapter.rs:3631-3633 mask 取 hi 符号应用于 lo；3737-3743 加法取负在 |v0|>2^1023 溢出 inf——改为逐字 u32 零扩展 + ±1.0 乘法取负，补 lo<0/负值位对精确测试）；F-2 MAJOR（恢复 av.rs dispatch_with_host 循环头步进计费）；F-3 MAJOR（改正 §8/收据的 ace-av 声明；其 conformance 编译断为基线预存红并入具名清单）；F-4 MAJOR（补 V-09 身份/原地修改可见性 + V-10 动态属性具名持久见证，list 对象即可，无需 torch 重跑）；F-5 MINOR（补 SD-06 行）；F-6 MINOR（逐字声明改为五处偏差全列：两拼写+fn-main 包裹+两个 hoist 为 v1 能力边界）；F-7 NOTE（去重复 BorrowEnd p2_v）；F-8 NOTE（fmt 波及已核格式级）；F-9 NOTE（快路加 digest 印章/锁阈值可后置）；F-10 NOTE（__py_dlen 负数注释/文档修正） | evidence: 复审代理实际执行 p1026 6/6（每例 ≤3.8s）、ace-core 765/765、ace-lowering 9/9、ace-foreign-contract 56/56；F-1 经独立 CPython 算术仿真（0.2370201051235199→0.23701998591423035, -2.5→-2.5000009536743164）+计划自身记录输出证实；F-2 经指令臂燃料路径分析证实；F-3 经 cargo test --no-run 与 git 谱系证实 | 计时台账: p1026 每例（corpus 0.41s/close 0.43s/prepared 0.66s/math 1.34s/torch 2.64s/V-01 3.77s）全 ≤10s | next: work（修复 F-1/F-2/F-7/F-10 → 补 V-09/10 见证与位对负例测试 → 改正 F-3/F-6 记录 → 补 F-5 SD-06 → 受影响面复验 → scoped 复审）

2026-11-18 F-3/F-6 记录修正（复审后，work 修复轮）：①前收据中"受影响组完整回归：…ace-av…全 PASS"不成立——ace-av 的 conformance 测试目标在本基线（c02c6dc8f 及之前，PLAN-982 提交谱系）即不编译（function_incremental_contract.rs E0004 SourceProfile::ConfigDocument 未覆盖），属**基线预存编译断**，并入具名预存红清单；实际可复验的绿组为 ace-core 765/765、ace-lowering 9/9、ace-foreign-contract 56/56、ace-console、ace-source（grammar-id 刷新后）。②"逐字正文"声明更正：验收语料与 §5.1 存在**五处**偏差——两处已披露拼写适配（lr: 0.1d、for step<60）+ fn main() int 包裹与 return 0（入口合同）+ 两个语义保持 hoist（var mparams = model.parameters()、var lr_v = 0.1d），后两个 hoist 反映 v1 resolver 能力边界（**F-29 更正**：此处原写"嵌套对象产生实参与动态浮点 kwarg **不支持**"——其中"动态浮点 kwarg 已证拒绝（`source.py-domain`）"成立，但"嵌套调用结果作位置实参"只是**未验证**（既非已支持也非已证拒绝），以 §5.3 F-22 与语料 README 为准；本句为历史记录，不作为 SD 沉淀依据）。

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | outcome: pass（复审 needs_fix 修复轮） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | code_commit: 0b25b6dcf | task_ids: T-02..T-05（修复 reopening 后复勾） | 修复闭合行: F-1 CLOSED（emit_py_f64_bits 逐字 u32 零扩展 ze32=(conv(x)+(x>>31&1)<<32)、单次 32 位移（修正 31+31=2^62 中间错）、±1.0 乘法取负；持久见证 p1026_f64_reconstruction_bit_exact_negative_and_low_bit31：-2.5 位精确、lo-bit31 值打印 0.2370201（损坏形 0.23701999 已不可复现）；V-01 首行实测 0.2370201）| F-2 CLOSED（av.rs dispatch_with_host 循环头 st.steps+=1 恢复+注释；见证 p1026_f02_scalar_infinite_loop_hits_step_budget：纯标量死循环在 50k 步命中 InstrSteps）| F-4 CLOSED（p1026_v09_v10_identity_mutation_and_dynamic_state：builtins list 双别名 append 可见（42）+ 空 list getitem 陷阱（动态状态负腿）；字节量独立腿以 py 字节恒定断言+具名后置，V-09 文件级见证成立）| F-7 CLOSED（重复 BorrowEnd p2_v 删除）| F-10 CLOSED（py.rs 文档/负域注释修正）| F-3/F-6 CLOSED（记录修正段+SD-06 行已入 §5/收据；ace-av 编译断入具名预存红）| F-5 CLOSED（SD-06 行）| F-8/F-9 NOTE 保留（fmt 已核格式级；digest 印章后置项） | evidence: p1026 套件 9/9（全 ≤1.9s，F-1 见证 0.13s、F-2 见证 0.04s、V-09/10 0.14s）；受影响组复验 ace-python 9/0、ace-lowering 9/0、ace-source 844/0、ace-foreign-contract 56/0、ace-console 5/0、ace-av-core lib 64/64 | next_responsible_role: T-06 scoped 复审（re-review affected: F-1/F-2/F-4 修复 + 记录修正）

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | round: 5 | from: work（修复轮续做，非新 stage） | to: work | status: execution_done | outcome: **pass**（F-11 见证修复 + fmt 收尾） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | code_commit: **7c8a0f433**（parent 0b25b6dcf） | task_ids: T-02/T-05（见证归属） | F-11 CLOSED（新 MAJOR，本轮修复）: 上轮 F-1 的"持久见证"**未跨 Python 边界**——两腿皆为 Auto 本地 double 字面量（`0.0d - 2.5d`、`0.2370201051235199d`），只走 console f64/f32 lane 与 961 NumTo，**不经** py_to_f64→`__py_f64_bits`，故 F-1 修好的重建路径无见证；新见证改为**真 Python 值过界**：`use.py math: pi, e` → `pi - 6`（负值：符号位 + 负 hi 字）与 `e`（lo32=0x8B145769，bit31 置位）经 py 成员/py_binop → `.to(float)`/`.to(double)`，断言用 `to_bits()` **等值**（零容差）；V-01 首行钉死 `== "0.2370201"`（损坏形 0.23701999/0.2370199859 不可回归） | F-8 收尾: `cargo fmt -p ace-abi` 后本计划自有文件（abi/src/lib.rs、abi/src/py_session.rs、tests/cases/py_session_1026.rs）fmt 干净——8e26e3020 起遗留的模块/import 顺序与长行漂移已消，`cargo fmt -p ace-abi --check` exit 0 | evidence（本轮真实执行，worktree 内 `cargo t ace-python`）: fmt 前 run-aqxy8bcq PASS 9/0（wall 9.33s，prep 3.37s；per-case: corpus 0.03s/close 0.16s/V-09+10 1.21s/math 0.64s/prepared 0.77s/torch 2.89s/V-01 4.48s/F-1 见证 0.39s/F-2 0.10s），fmt 后 run-o7jae8iz PASS 9/0；两组同 tip 内容无断言失败 | **交接异常记录（重要，复审需知）**: 上一轮 review 的 round-2 收据**未落盘**（§9 无该条；worktree 内唯一痕迹是一处未提交的测试改动，mtime 12:14:55 晚于计划 12:07:38 的写入，注释自述 "REVIEW ROUND 2, F-11"）。按"信任收据+树/提交事实"重建：该 review 产出了 F-11（见证未达生产路径），work 侧已开始修但被中断未提交；本轮由 work 认领并**提交为 7c8a0f433**，F-11 的处理与结论以本收据为准，review 阶段须重新出正式 review 收据（不得沿用丢失的记录） | next_responsible_role: T-06 scoped **final** 复审（re-review: F-1/F-2/F-4/F-11 修复 + F-3/F-6 记录修正 + F-5/F-8/F-10 收尾；不再重开已闭 NOTE）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 7c8a0f433 | next: review

2026-11-18 stage: review | review_scope: final | reviewer_identity: 独立上下文只读复审代理（fresh session，未参与实现；同会话族工具链，按工件重建判定，无 model/session id 声明） | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **needs_fix**（0 功能缺陷：F-1/F-2/F-4/F-11 经独立复验真闭；阻断项为 SD-06 目标/元数据不实与两条**本计划新增** warning，均一行级文档/卫生修复，无需重跑测试） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | reviewed_commit: 7c8a0f433 | base_commit: c02c6dc8f | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3（离线缓存）/ torch 2.14.0+cpu / Windows x64 | spec_inputs: docs/specs/auto-av/{architecture,project,host-import-and-console}.md、auto-mir/hir-lowering-boundary.md、ace-migration/replacement-scope-and-evidence.md、stdlib/design/ace-host-services.md | acceptance_results: AC-01/05/06 pass；AC-02/03/04/09 partial（证据形状差，见 F-16）；AC-07 本复审；AC-08 merge pending | findings: **F-11 CLOSED**（三路独立复验：①路径追踪 use.py math: pi,e → py.rs front_door hoist → py_bridge import/getattr 真 PyObject；pi-6 → resolve.rs py_binop 在 **CPython 内**执行 __sub__；.to(float/double) → py_to_f64(bits>>32)+py_f64_lo(bits) → __py_f64_bits；②类覆盖：e 正 hi + lo32=0x8B145769(bit31)、pi-6 负号+负 hi 字，断言 to_bits 等值；③独立仿真复刻 3069dab80 旧发射器，digit-for-digit 复现两处损坏值（0.23701998591423035→打印 0.23701999、-2.5000009536743164），并证明新输入在旧 bug 下必红（e f32 0x402df84c≠0x402df854、pi-6 0xc036f021≠0xc036f025）；旧见证确为 Auto 本地字面量、不经 py）；F-1/F-2/F-4/F-7/F-10 CLOSED；F-3 记录准确（ace-av E0004 基线预存断，文件未触）；F-8 fmt exit 0 | **新 finding（需修复）**: F-12 MINOR in-contract 阻断（SD-06 的 auto-av 目标 `architecture.md` 无 console 小节——该文件仅 15 行决策表；权威合同是 `auto-av/host-import-and-console.md`，仍钉 print/eprint.{i32,bool,str,char}.v1 与 av.host-console/1 profile-8，按 SD-06 现状沉淀将留下与代码矛盾的 canonical spec；且 SD-06 规则未含本次实际新增的 console 准入放宽（host.rs validate/authorize 现允许 foreign-carrying 非 profile-8 模块的 console import）；frontmatter supersedes 漏 `docs/specs/stdlib/design/ace-host-services.md`）；F-13 MINOR（F-6 更正称两 hoist+v1 边界"已列 README/§5.3"，实为不实——README 只列两处拼写；五处偏差本身列举正确）；F-17 MINOR（§7 AC-01..09 全未勾而 work 声明 pass；AC-07 行文本仍写 SD-01..05 未含 SD-06）；F-18 MINOR health（**本计划新增**两条 warning：abi/tests/cases/py_session_1026.rs:252 unused `decoded`、core/src/source/adapter.rs:1105 不需 mut）；F-14/F-15/F-16/F-19 NOTE（MIR_LOWERING_VERSION 仍 "4" 未记 bump/加性决策；通用 f64/f32 console lane 复核为 type-keyed+Plan-403 parity，但 7c8a0f433 后仅在 use.py 程序内被行使用；AC-02 sum=10 以 sum(arange(8)/2)=14.0 同规则行使、V-02 多元素具名失败/weakref 最后 owner/V-09 payload 字节腿为具名后置；Plan 文件本身 untracked） | evidence（本会话实际执行）: ace-python 9/9、ace-lowering 9/9、ace-source 844/844、ace-foreign-contract 56/56、ace-console 5/5、ace-av-core lib 64/64、ace-semantics 294/295（1 红 = 具名预存 sema_boundary executed-status，文件在计划外）、fmt --check exit 0、cargo check --tests exit 0；ace-av 组基线编译断（E0004 function_incremental_contract.rs:535）不可声明绿 | timing: p1026 每例 0.022–4.624s（V-01 4.624 / torch 2.794 / math 1.221 / prepared 0.981 / V-09+10 0.604 / F-64 0.370 / close 0.180 / F-02 0.112 / corpus 0.022），全 ≤10s；组 wall：ace-python 7.61s（prep 1.41+exec 6.17）、ace-source 8.71s exec、ace-semantics 14.81s | pending_closeout_gates: ace-abi(全)/ace-artifacts/ace-host/ace-stdlib 全集 + ace-av（基线不可运行，按具名预存红处理）+ 周期 cargo t all（Linux/WSL 按裁定排除） | 报告与原始日志: D:/autostack/plan-artifacts/plan-1026/review-r2/（report.md、f64_diff_sim.{py,out}、ace-python.log、groups-affected.log、ace-semantics.log、avcore-lib.log、check-warnings.log） | next: work（F-12 plan-text：SD-06 目标改 host-import-and-console.md + 补准入条款 + frontmatter 补 stdlib 路径；F-13 README/指针改正；F-17 AC 勾选与 AC-07 文本；F-18 两行代码卫生；F-14/F-16 以具名决策/边界句记录）→ scoped delta/健康复验（不重跑测试）→ merge

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 7c8a0f433 | next: work

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | round: 6 | from: work（review r2 needs_fix 修复轮） | to: work | status: execution_done | outcome: **pass** | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | code_commit: **cd132d7be**（parent 7c8a0f433） | task_ids: T-02/T-05（复开→复勾） | 修复闭合行: **F-12 CLOSED**（SD-06 目标由不存在的 `auto-av/architecture.md` console 小节**改指** `docs/specs/auto-av/host-import-and-console.md`，并把实际已发生的 **console import 准入放宽**写入 delta：带 foreign 描述符的模块可准入 console import、纯标量模块仍被拒（实测错误码 `host.profile` "host imports require the host-console profile"）；声明不授予未声明 capability；frontmatter `supersedes_spec_components` 补 `docs/specs/auto-av/host-import-and-console.md` 与 `docs/specs/stdlib/design/ace-host-services.md`）| **F-13 CLOSED**（语料 README 改为**五处表面偏差**全列：两拼写 + `fn main() int`/`return 0` 入口包裹 + 两个 hoist；§5.3 新增"v1 解析/前端能力边界"段说明 (a) 嵌套调用结果直接作位置实参 (b) 动态浮点 kwarg 为具名未支持；`ACCEPTANCE` 常量 doc 同步；§8 T-05 历史 bullet 的 `first=0.23701999` 加更正注）| **F-17 CLOSED**（§7 AC-01/05/06 勾选并附 review r2 独立复验锚点；AC-02/03/04/09 勾选但**显式标 partial-by-evidence-shape** 并指向 §6 具名后置；AC-07 行文本改 SD-01..**06**；AC-07/AC-08 保持未勾；§7 引言改写现状）| **F-18 CLOSED**（`py_session_1026.rs` F-2 见证的 decode 改为 `_decoded` 并注释其 smoke 用途；`adapter.rs` `compile_program_observed` 去掉多余 `mut`——`cargo check -p ace-abi -p ace-core --tests` 后本计划文件**零 warning**，其余 warning 均为基线既有、未触）| **F-14 CLOSED（决策记录）**（SD-03 行写明 `MIR_LOWERING_VERSION` 保持 "4"：本切片纯增量——新 HIR 节点/内联 MIR 发射/新 py foreign 描述符通道；既有映射规则、lowering 准入、错误分类、ABC/wire/profile 轴未改，沿 PLAN-912 先例；console 准入放宽属 host/MIR 校验面不在 lowering 契约内；后续若改既有映射则升版）| **F-16 CLOSED（具名后置）**（§6 新增"具名后置（review r2 F-16，不充 PASS）"五条：sum 字面锚点以 14.0 同规则行使、V-02 多元素具名失败未永久化、weakref 最后 owner 以 release/close 计数见证、V-09 载荷字节腿未独立断言、V-03 一次求值计数 trace 未钉）| **F-15 CLOSED（补见证）**（新增永久用例 `p1026_console_float_lane_formats_native_auto_doubles`：原生 Auto double（`2.0d`/`0.5d`/`0.0d-2.5d`）经通用 console f64 lane 精确打印 `2`/`0.5`/`-2.5`（Plan 403 parity）；doc 说明该模块带 `use.py` 仅为满足 console 准入的 foreign-carrying 规则——即 F-12 的准入条款，被打印值本身零 Python 跨越）| evidence（本轮真实执行）: **ace-python 10/10 PASS**（run-82rdjb2l，**机器空闲时**测得：V-01 4.198s、torch 2.493s、math 1.384s、F-1 1.261s、close 0.956s、新增 F-15 0.758s、prepared 0.627s、V-09/10 0.149s、F-02 0.059s、corpus 0.012s；wall 9.94s；全 ≤10s）；`ace-source` 844/844 PASS、`ace-console` 5/5 PASS（run-qvbn5k5o，两组合计 wall 533.95s 主要耗在 adapter.rs 触发的 ace-core 及其依赖测试目标重编译）；`cargo fmt -p ace-abi --check` exit 0；`cargo check -p ace-abi -p ace-core --tests` exit 0 且本计划文件无 warning | **环境归因记录**：早前两次 ace-python 运行（run-qt85mel7 V-01 9.575s、run-2xjy59ob V-01 10.031s）与另一 agent 的并发 cargo 构建（12:49–12:51 多个 rustc/cargo、CPU 65%）重叠，属**负载归因**而非代码回归；该窗口后机器空闲重测为 4.198s，按纪律记"首次红/超限→空闲重放绿"，不据此改代码 | next_responsible_role: T-06 scoped **final** 复审（re-review affected: F-12/F-13/F-17/F-18 记录与卫生 + F-14/F-15/F-16 新记录/新用例；功能面 F-1/F-2/F-4/F-11 已两轮独立复验，可复用）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: cd132d7be | next: review

2026-11-18 stage: review | review_scope: final | reviewer_identity: 独立上下文只读复审代理（fresh session，未参与实现；同会话族工具链，按工件重建判定，无 model/session id 声明） | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **needs_fix**（0 功能缺陷；F-12/F-13/F-15/F-16/F-17/F-18 经独立复验真闭；唯一 in-contract 阻断 **F-20**：SD-03 的"MIR_LOWERING_VERSION 保持 4 / 纯增量"与 lowering 实际改动矛盾；F-21 同批一句） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | reviewed_commit: cd132d7be | base_commit: c02c6dc8f | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3（离线缓存）/ torch 2.14.0+cpu / Windows x64 | spec_inputs: docs/specs/auto-av/{architecture,project,host-import-and-console,compact-abc-format}.md、auto-mir/{hir-lowering-boundary,core-i32}.md、ace-migration/replacement-scope-and-evidence.md、stdlib/design/ace-host-services.md、auto-abi/{checked-source-foreign-calls,resource-and-callback-bridge}.md | acceptance_results: AC-01/05/06 pass；AC-02/03/04/09 partial-by-evidence-shape（§6 五条具名后置**逐条核实属实**）；AC-07 本复审**未勾**（F-20 未闭）；AC-08 merge pending | findings: **F-20 MAJOR（in-contract，F-14 未真闭）**：`lowering/src/hir_adapter.rs:635-648` 改写**既有 console host-import 准入闸**——原「仅 sched intrinsics」条件扩为「sched intrinsics **或** 存在 DefKind::Foreign」，使此前落 `Err(lower.intrinsic)` 的路径现被准入并映射 `MirHostImport`；另加 `__py_f64_bits`/`__py_staged_text` intrinsic 跳过与 F32/F64 在该域的 interning。按 `lowering/src/lib.rs:26-28` 合同与 `hir-lowering-boundary.md`「映射规则/准入域/错误分类」须升版（v2/v4 先例）⇒ `MIR_LOWERING_VERSION` 改 "5"，同步 `crates/auto-lang-core/src/test_cases/{ac_cte,ac_mn,ac_av}.rs` 三镜像与 guard；SD-03「纯增量/未改准入」与 SD-06「不在 lowering 契约内」均不实 | **F-21 MINOR（同批一句）**：SD-06 准入句应写「带至少一个 `ForeignCall` 的模块」——av-core 层条件是 `MirInstr::ForeignCall`（host.rs:106-117/227-238），而 HIR/lowering 层是 descriptor（verify.rs:3545-3548、hir_adapter.rs:645-647），句子未分指两层；只有 `use.rs/use.c` 声明无调用 + print 的可构造角落会被 HIR/lowering 准入但运行期被 `host.profile` 拒 | **F-22/F-23/F-24 NOTE**：F-13 残余——"嵌套调用位置实参"在 v1 **未经反例证明**（`py_is_object` 接受内联 py-op 结果、`py_frame_val` 可封送），只能记"未证"不得沉淀"明确拒绝"；`abc.rs:1443-1455/3500-3510` 把 FOREIGN_CALL 编解码族门放宽到 Closures（格式/版本未动，但 "ABC/wire 轴均未改动" 措辞需改）；`auto-mir/core-i32.md` 对 FOREIGN_CALL 仍写"AV 具名拒不执行"，建议交叉引用 | evidence（本会话实际执行）: ace-python 10/10 PASS（wall 7.67s：V-01 5.230 / torch 2.976 / V-09+10 1.771 / close 1.465 / math 1.067 / F-1 0.844 / prepared 0.661 / **新 F-15 0.441** / F-02 0.216 / corpus 0.140，全 ≤10s）；ace-source 844/844（22.43s）；ace-console 5/5（4.84s）；ace-lowering 9/9（5.13s）；`cargo check -p ace-abi -p ace-core --tests` exit 0 且本计划零新增 warning（其余逐条证实为 c02c6dc8f 既有）；`cargo check -p ace-av --tests` 复现 E0004 `function_incremental_contract.rs:535`（基面变体/匹配臂/未触文件三重证实为预存红）；`cargo fmt -p ace-abi --check` exit 0；树 clean、无新增 debug/TODO | occupancy: 另一 agent 的 cargo+rustc 自 13:08 并行（长 rustc ~301s CPU、24–28%），ace-python 于轻载窗口测得，判定不依赖空闲机器 | pending_closeout_gates: ace-abi(全)/ace-artifacts/ace-host/ace-stdlib/ace-semantics/ace-foreign-contract(全) + ace-av（基线不可编译，具名预存红）+ 周期 cargo t all（Linux/WSL 按裁定排除）+ 升 "5" 后 ac_av/ac_cte/ac_mn 报告身份族复验 | 完整报告: D:/autostack/plan-artifacts/plan-1026/review-r3/report.md | next: work（F-20 版本 5 + 三镜像 + SD-03/SD-06 文本、F-21 同批；F-22/F-23/F-24 措辞；受影响组复验）→ 最终 review

### 三次 final review 后的诊断检查点（work-skill 要求，2026-11-18）

三次 final review（r1 `3069dab80`→needs_fix、r2 `7c8a0f433`→needs_fix、r3 `cd132d7be`→needs_fix）均**缩小**了问题面（功能面自 r2 起已零缺陷），但每轮都出现"记录/身份"类新 finding。按 work-skill：先诊断再修，不得盲重试。

- **仍未闭合的不变量**：*lowering 契约版本必须标识 lowering 语义面的每一次变化*（映射规则/准入域/错误分类），因为该版本是 `ac_av/ac_cte/ac_mn` 报告身份与 stale 裁决的输入（三镜像 `EXPECTED_MIR_LOWERING_VERSION` + `scripts/verify-mir-lowering-868.ps1`）。
- **为何前两轮未发现**：F-14 的结论来自"本计划改了什么"的**变更清单推理**（"新节点 + 新 foreign 描述符，故纯增量"），没有 diff **既有准入谓词**本身；而"给既有谓词加一个 `||` 分支"在直觉上像新增，按规则却正是 *admitted domain* 扩大（v2/v4 先例）。r2 复审接受了 additivity 叙述；r3 复审改用 `git diff … -- crates/lowering` 逐行核对才发现。**检测缺口**：没有任何机械检查把"lowering 目录有语义 diff"与"版本/镜像已升"绑定。
- **受影响路径/状态**：`lower_checked_collection_family` 的 console-import 准入（sched-only → sched-or-foreign）、`__py_f64_bits`/`__py_staged_text` intrinsic 跳过、该域 F32/F64 interning；消费方：ace-lowering 常量、三个镜像、guard 脚本、已存报告的 stale 判定。
- **设计/oracle 调整（本轮实施并验证）**：①`MIR_LOWERING_VERSION` → `"5"` 并在 lib.rs 文档行写明 v5 语义（console 准入扩面 + py intrinsic/宽标量 interning）；②三镜像同步 `"5"`；③SD-03/SD-06 文本按实际改写（准入扩面 + 两层条件分指）；④**机械检测**：扩展 `scripts/verify-mir-lowering-868.ps1`，除现有镜像互核外**再断言 ace crate 的 `MIR_LOWERING_VERSION` 与镜像一致**（当前脚本只解析 ac_mn 并互核三文件，不读 ace crate——正是本次漂移得以存在的缝隙）。此后**crate 与镜像的单向漂移**（改了常量忘改镜像，或反之）在此 guard 处直接失败；而"语义改了但没升版"仍须 review 逐行 diff lowering（机械不可判），本计划已把该检查写为 SD-03 行的复审必检项。

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 4 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: cd132d7be | next: work

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | round: 7 | from: work（review r3 诊断检查点后的有界调整轮） | to: work | status: execution_done | outcome: **pass** | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | code_commit: **743766775**（parent cd132d7be） | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3 / torch 2.14.0+cpu / Windows x64 | task_ids: T-02（lowering 契约身份）/T-05（镜像与语料记录） | 修复闭合行: **F-20 CLOSED**（`MIR_LOWERING_VERSION` `"4"`→**`"5"`** + lib.rs v5 语义行：console host-import 闸由「存在 `__sched_*` intrinsic」扩为「sched **或** 任意 `DefKind::Foreign`」＝准入域变化，另有 `__py_f64_bits`/`__py_staged_text` 调用点直接发射与该域 F32/F64 interning；三镜像 `crates/auto-lang-core/src/test_cases/{ac_av,ac_cte,ac_mn}.rs` 同步 `"5"`；`experimental/ace/crates/av/tests/cases/closures_fn_912.rs:784` 的 `producer_mir_lowering_version` 期望同步 `"5"`；SD-03 行按实际改写并新增 `native-data-owned-lowering.md`（其 `"4"` 引用须随沉淀改 `"5"`）为 SD-03 目标、同步 frontmatter） | **F-21 CLOSED**（SD-06 准入句改为两层分指：运行期 av-core 条件＝非 profile-8 Closures 模块**至少一条 `MirInstr::ForeignCall`** 且 host-import 表全为 `auto.console.*`；编译期 HIR/verify/lowering 条件＝存在任意 `DefKind::Foreign`；并写明「只声明不调用 + print」角落编译期准入、运行期 `host.profile` 拒） | **F-22 CLOSED**（§5.3 v1 边界改写：嵌套调用位置实参＝**未证**（`py_is_object` 接受内联 py-op 结果、`py_frame_val` 可封送），不得沉淀"明确拒绝"；动态浮点 kwarg＝**已证拒绝**（`source.py-domain`）） | **F-23 CLOSED**（§5.3 记录 `abc.rs` 的 FOREIGN_CALL/FOREIGN_CALL_CB 族门由 `ForeignScalar\|Owned` 扩为 `ForeignScalar\|Owned\|Closures`；wire/格式/版本均未变，故非 abc 版本变化，canonical 规范不改规则，merge 时按 SD-02 一句记录） | **F-24 记录（merge 时执行，不改规则）**：`docs/specs/auto-mir/core-i32.md` 对 FOREIGN_CALL 仍写"AV 具名拒不执行"，merge 沉淀 SD-03 时加一行交叉引用（该文范围属 938，不改其规则） | **诊断检查点调整已实施并验证**：扩展 `scripts/verify-mir-lowering-868.ps1` 的 identity 段——在既有"三镜像互核"之外新增「ace crate `MIR_LOWERING_VERSION` == ac_mn 镜像」断言（此前脚本只解析 ac_mn 且不读 crate，正是本次漂移缝隙）；用同一 regex/文件的正控（crate=5/mirror=5 → match）与**负控**（伪造 crate="4" → mismatch 检出）离线验证该判定逻辑 | evidence（本轮真实执行，worktree 内 `cargo t`）: **case-ac-av 18/18、case-ac-cte 6/6、case-ac-mn 3/3 PASS**（run-ymlea12j，exec 2.65s/2.27s/1.28s，wall 101.4s 含重编译）——三族在镜像=5 下报告身份一致，证明升版与镜像同步贯通；**ace-lowering 9/9、ace-console 5/5、ace-python 10/10 PASS**（run-01of_8wn，最终树：V-01 **1.997s**、torch 1.84、V-09+10 1.585、prepared 1.345、F-1 1.145、math 0.966、close 0.54、F-15 0.331、F-02 0.044、corpus 0.009，全 ≤10s；同树前一轮 run-qmmwjo03 亦 10/10，V-01 3.305s）；`cargo fmt -p ace-lowering` 后本计划自有 lowering 文件 fmt 干净（`--check` exit 0；auto-lang-core/ace-av 的既有 fmt 漂移未触，非本计划文件） | 未执行/边界: `-Scope identity` 全流程（11 suites）未跑，属 guard 常态化门，本轮以等价判定逻辑+正负控验证新增断言；`ace-av` 仍基线不可编译（E0004），`closures_fn_912` 属该 target，其字符串更新无法在本基线执行，已按同步义务改值并记录 | next_responsible_role: T-06 **最终** review（re-review: F-20/F-21 + F-22/F-23 措辞 + F-24 merge-note；功能面自 r2 起未再变动，可复用）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 743766775 | next: review

2026-11-18 stage: review | review_scope: final | reviewer_identity: 独立上下文只读复审代理（fresh session，未参与实现；同工具链族，按工件重建判定，无 model/session id 声明） | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **needs_fix**（0 功能缺陷；F-20 真闭、F-21 两层准入+declare-only 角落与代码一致、F-22 §5.3 措辞正确、F-23 族门与 wire/版本未变；唯一 in-contract 阻断 **F-25**：语料 README:35,38 仍把"嵌套调用结果作位置实参"沉淀为"v1 不支持/明确拒绝"，与本轮 §5.3 的 F-22 更正自相矛盾） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | reviewed_commit: 743766775 | base_commit: c02c6dc8f | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3（py_bridge 内 pinned）/ torch 2.14.0+cpu / Windows x64（独立探测） | spec_inputs: docs/specs/auto-mir/{hir-lowering-boundary(fd9a0d56…),native-data-owned-lowering(ae98d151…)}.md、auto-av/host-import-and-console.md(c7855f5f…)、stdlib/design/ace-host-services.md(f909749c…)、plan sha256 5324473a… | acceptance_results: AC-01/05/06 pass（功能面自 r2 未变，复用证据+本轮复跑身份族）；AC-02/03/04/09 partial-by-evidence-shape（§6 五条具名后置复用）；AC-07 本复审=needs_fix（保持未勾）；AC-08 merge pending | findings: **F-20 CLOSED**（"5"+v5 语义行+三镜像+`closures_fn_912`+guard crate↔mirror+SD-03/frontmatter；复审确认 lowering diff 三类同族准入域改动+py intrinsic 内联+F32/F64 interning，升版判断正确）；**guard 经逐字复刻验证**：正控（真文件）exit 0；负控 crate="4"/mirror="4"/常量缺失 均 exit 1；**边界**：删除镜像文件仍可通过既有循环（未覆盖）、"语义改了但没升版"/协同过度升版/prose 引用 不可机械判（F-28）；**F-21 CLOSED**（运行时 host.rs:106-117/227-238 ForeignCall+全 console；编译期 verify.rs:3545-3548 + 三处 hir_adapter 任意 DefKind::Foreign；declare-only 角落编译期准入、运行期 host.profile 拒）；**F-22 CLOSED**（§5.3 (a) 未证/(b) 已证拒绝）；**F-23 CLOSED**（abc.rs:1450/3505 含 Closures，opcode/payload/version 未动）；**F-25 MINOR in-contract 阻断**（README 两行，见上）；**F-26 NOTE**（v5 文档行/SD-03 仅点名三类准入站点之一，FnLower call seam 属 `lower.callee` 且判据为"任意 foreign-def 模块"；`py.intrinsic` 错误码未点名——merge 一句补）；**F-27 NOTE**（SD-06 的"非 profile-8 的 `Closures` 族模块"限定未被代码强制——host.rs 是 profile-agnostic 谓词，措辞应收紧为实际谓词，不改代码）；**F-28 NOTE**（`closures_fn_912` 期望不可执行、删镜像不拒、abc.rs 旧错误串、历史 "4" prose 非消费者） | evidence（本会话实际执行）: `cargo t ace-lowering ace-console ace-python` PASS（run-dlte5826，命令 20.95s；prep 0.853/1.409/1.344，exec 0.842/2.818/5.406；V-01 4.260、torch 2.856、prepared 1.552、V09+10 1.292、close 0.793、F-15 0.673、math 0.413、F-1 0.242、corpus 0.137、F-02 0.073，全 ≤10s）；`cargo t case-ac-av case-ac-cte case-ac-mn` PASS（run-bsc5q44p，18/6/3，exec 3.262/2.797/1.248，命令 11.38s）；fmt×2 exit 0；`cargo check -p ace-abi -p ace-core --tests` exit 0 且无本计划 warning（两条 ace-core warning blame 至 499335856d/83a170a843，均为 c02c6dc8f 祖先）；`cargo check -p ace-av --tests` 复现 E0004 function_incremental_contract.rs:535（文件自 r3 未触）；ace 系 39 个 Cargo.toml 均无 pyo3 | occupancy: 并发 lang-981 `cargo t` 与 `cargo test -p ace-av`（20 核，load 10–56%，热 target），计时按此归因 | pending_closeout_gates: 冻结并集 ace-abi(全)/ace-artifacts/ace-host/ace-stdlib/ace-semantics/ace-foreign-contract(全) + 周期 cargo t all（Linux/WSL 排除）+ `verify-mir-lowering-868 -Scope identity/acceptance` 全流程 + SD-01..06 沉淀（含两处 "4"→"5"、SD-02 ABC 族门一句、F-24 core-i32 交叉引用）+ F-19 计划入库 + AC-08 归档/清树/材料折叠 | 报告: D:/autostack/plan-artifacts/plan-1026/review-r4/report.md | next: work（仅 F-25 两行 README 措辞；F-26/F-27 的措辞收紧留给 merge 沉淀时同步）→ 文档面最终复审 → merge

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 743766775 | next: work

2026-11-18 stage: work | plan_id: PLAN-1026 | plan_revision: r3 | round: 8 | from: work（review r4 的 F-25 文档轮） | to: work | status: execution_done | outcome: **pass** | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | code_commit: **1b4352769**（parent 743766775） | task_ids: T-05（语料/README 记录） | 修复闭合行: **F-25 CLOSED**（①`crates/auto-lang/test/cases/ace-m2/python-1026/README.md` 第 4 条改为"嵌套调用结果直接作位置实参"在 v1 **未验证**——既不主张支持、也不得宣称已拒绝；第 5 条明确为**已证拒绝**（`source.py-domain`）；正文段落改为"两条边界性质不同"，删去原来的"超界形态属具名未支持（明确拒绝/不支持）"总括。②`experimental/ace/crates/abi/tests/cases/py_session_1026.rs` 的 `ACCEPTANCE` doc 注释同步（两个 hoist 不是同一套边界故事：动态浮点 kwarg 已证拒绝、嵌套位置实参未验证）。③顺带把 README 的 console 准入句改为**实际谓词**——非 profile-8 且 ≥1 条 foreign 调用、host-import 全为 `auto.console.*`；并写明 compile 期条件是"任意 foreign 声明"，只声明不调用者编译期准入/运行期 `host.profile` 拒（与 F-21/F-27 一致，F-27 的 SD-06 措辞收紧仍留 merge）） | evidence（本轮真实执行）: `cargo t ace-python` **10/10 PASS**（run-7q6elkuv，prep 5.8s / exec 7.91s；corpus identity 0.026s 证明语料与 ACCEPTANCE 仍逐字一致；F-1 见证 0.567s；其余同前）；`cargo fmt -p ace-abi --check` exit 0；本轮只改两个文档/注释（无语义、无测试行为变化），故未重跑其它组——按"更改仅文档面则复用未变身份证据"记录 | 未做/留给 merge: F-26（v5 文档行/SD-03 只点名一处准入站点、`py.intrinsic` 错误码未点名）与 F-27（SD-06 的"非 profile-8 Closures 族"限定未被代码强制，措辞按实际谓词收紧）在 SD 沉淀时同步；F-28 的边界（`closures_fn_912` 不可执行、删镜像不拒、历史 prose 非消费者）记入 merge 收据 | next_responsible_role: T-06 **纸面最终复审**（只核 F-25 两处文档与 §5.3/§6 一致性 + 本轮 diff；功能证据复用 r2..r4）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 1b4352769 | next: review

2026-11-18 stage: review | review_scope: final | reviewer_identity: 独立上下文只读复审代理（fresh session，未参与实现；同会话族工具链，按工件重建判定，无 model/session id 声明） | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **pass**（F-25 真闭：README 第 4/5 条与正文字段、`ACCEPTANCE` doc 注释三处一致改为「嵌套调用位置实参＝未验证/未主张、动态浮点 kwarg＝已证拒绝 `source.py-domain`」，与 §5.3 F-22 逐字一致；全表面 grep 仅余 §9 历史段一处"不支持"旧措辞（F-29 NOTE，非规范面）；两文件 diff 机械证明为纯 prose/`///` 注释、语料 blob 前后同 hash、字节身份用例绿；console 准入句已收紧为实际谓词（非 profile-8 + ≥1 `MirInstr::ForeignCall` + 全 `auto.console.*`；编译期＝任意 `DefKind::Foreign`），不再含代码未强制的 Closures 族声明；**0 in-contract 阻断**） | worktree: D:/autostack/.wt/lang-1026/auto-lang | branch: plan-1026-dev | reviewed_commit: 1b4352769 | base_commit: c02c6dc8f | dependency_revisions: CPython 3.14.2 / PyO3 0.29.3 / torch 2.14.0+cpu / Windows x64 | acceptance_results: **AC-07 pass（本复审；merge 勾选）**；AC-01/05/06 pass、AC-02/03/04/09 partial-by-evidence-shape（功能证据复用 r2..r4）；AC-08 merge pending | findings: **F-25 CLOSED**；**F-29/F-30/F-31 MINOR（merge-owned，不阻断）**：plan 历史段 plan:358 旧"不支持"措辞、§8 旧"ace-av 全 PASS"断言、§8 T-07 仍写 "SD-01..05"（应 SD-01..06）——merge 必须按 SD-01..06 沉淀，且不得把旧断言折叠进 canonical；F-26/F-27/F-28/F-19/F-24 与 SD-01..06、guard identity/acceptance、冻结并集、归档清树逐项确认仍具名停放 | evidence（本会话实际执行）: `cargo t ace-python` **10/10 PASS**（run-gem8au94，gate_passed=true，prep 718.8ms / exec 3481.5ms，命令 wall 5.09s；V-01 2.723 / torch 1.637 / close 1.444 / F-1 1.242 / F-15 0.832 / prepared 0.422 / math 0.348 / V-09+10 0.189 / F-02 0.154 / corpus 0.014，全 ≤10s）；`cargo fmt -p ace-abi --check` exit 0；树 clean（前后各验一次） | occupancy: 20 逻辑核、load 13%、11.9GB free；并发 lang-980 abi-worker 与 lang-981 av-worker（各自独 target-dir）及两个 `cargo t ace-source…`，未与本树 target 竞争；热 target 计时按此归因 | pending_closeout_gates: 冻结并集 ace-abi(全)/ace-artifacts/ace-host/ace-stdlib/ace-semantics/ace-foreign-contract(全) + 周期 cargo t all（Linux/WSL 排除）+ ace-av 基线 E0004 具名预存红 + `verify-mir-lowering-868 -Scope identity/acceptance` + SD-01..06 沉淀（两处 "4"→"5"、SD-02 族门一句、F-24/F-26 交叉引用、F-27 SD-06 措辞收紧）+ F-19 计划入库 + F-28 边界入 merge 收据 + AC-08 归档/清树/材料折叠 | 报告: D:/autostack/plan-artifacts/plan-1026/review-r5/report.md | next: **merge**（T-07/AC-08）

stage: goal | plan_id: PLAN-1026 | plan_revision: r3 | round: 8 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1026/auto-lang | commit: 1b4352769 | next: merge

2026-11-18 stage: merge | plan_id: PLAN-1026 | plan_revision: r3 | outcome: **进行中（landed/archived/cleaned 三步待记）** | delivery_candidate: rebased tip `d97ec23f2`（base master@`b7850248a`） | reviewed_commit: `1b4352769` → rebased `6bdc13cbb`; specs delta `0edcf5773` → `5e27dfb80` | range-diff: 12 提交中 8 个逐字节相等；4 个（02/03/04/06）因 master 前进产生冲突解析，差异均为**格式/master 新字段**（`resolve_with_sched` 换行、`age_resources` 取 master 更宽属性表、`hir_adapter` 取 master 的 F32/F64/I64/U64/I8/U8/U32 域块、`gfx_credits: 0`），无语义损失 | 冲突解析: `adapter.rs` 同时保留 master 的 `age_resources::gfx_by_symbol` 臂与 py 两 intrinsic 臂；`hir_adapter.rs` 取 master 更宽 block（本计划 F32/F64 需求被其覆盖 → v5 文档按真实来源更正为"F32/F64 interning 先于本计划由 AGE 数值类型表工作引入"）；`age_resources.rs` 取 master 的 `image`/`frame`/`flip` 面 | 冻结门并集（rebased tip，`cargo t`）: `ace-artifacts` 43/43 PASS、`ace-stdlib` 2/2 PASS、`ace-foreign-contract` 56/56 PASS、`case-ac-av` 18/18 PASS、`case-ac-cte` 6/6 PASS、`case-ac-mn` 3/3 PASS、`ace-source` 863/864 PASS（1 条件排除）、`ace-python` 10/10（V-01 4.853s）、`ace-lowering` 9/9、`ace-console` 5/5；**红全部为 master 预存基线**（见下），非本计划回归 | 基线红（逐条对账 KNOWN-DEBT-AND-RISKS）: ①`ace-abi` **24 例 source_foreign 族**（`source_foreign::*` 23 + `source_resources$source_foreign::pointer_encoding…` 1）＝台账"ace-abi source_foreign 族 ~24 例"（master `8e0c4d1d6` 复现，owner=批量回归 triage）；②`ace-host`（ace-conformance）**13 例**（governance 1 + language_conformance 6 + semantics_protocol 1 + rust_lowering 2 + native_protocol 1 + source 1）＝台账"master 预存红：ace-host 13 例"（master `2cdb28fb3` 复现）；其中 `ace_m1_closures_source_suite_green` 即台账 `1060-R5 closures suite 绿断言失效`（P2，既有红）；③`ace-semantics` **1 例** `sema_boundary_case_files_executed_status_honest`（本计划 §6/§9 早已具名）；④`ace-av` 组基线编译断 E0004（`function_incremental_contract.rs:535`）不可运行，不声明绿 | 判读: 三组 FAIL 的执行数分别为 314/171/295，失败集合与台账 reds 一一对应，且 rebase 前同族红在 `c02c6dc8f` 已复现（r2/r3/r4 复审证据），故本并集对该基线"可复现红、无新增" | 下一步: `verify-mir-lowering-868 -Scope identity`（运行中）→ ff-only 落主 → 归档/清树/材料折叠

## 11. 交付与归档收据（merge，2026-11-18）

stage: merge | plan_id: PLAN-1026 | plan_revision: r3 | **outcome: pass（delivered）** | completion_kind: delivered | delivery_commit: **`181801922`**（master 快进落主，无 merge commit；`git merge --ff-only plan-1026-dev`，`merge-base --is-ancestor` 通过）

| checkpoint | 证据 |
|---|---|
| prepared | reviewed 基线 `1b4352769`（r5 final pass，AC-07 勾选）；specs 增量 `0edcf5773`；rebase R1 base `b7850248a`、R2 base `8f44bbf08`；13 提交；`git range-diff` 逐提交核对：8 个逐字节相等，4 个（T-02a/T-02b/§5.1/卫生轮）差异均为**格式 + master 新字段**，无语义损失；冲突解析三处：`adapter.rs` 同时保留 master `gfx_by_symbol` 臂与 py intrinsic 臂、`hir_adapter.rs` 取 master 更宽数值域块、`age_resources.rs` 取 master `image`/`frame`/`flip` 面、`plan981_model_field_types.rs` 取 master 已翻正的 f32 断言（弃分支内旧"拒 f32"版本） |
| landed | master tip = delivery commit `181801922`；落主后 master 仅由其owner的 docs(plan) 收据提交前进（`f8905c494`…），本交付祖先关系不变 |
| canonical_verified | 落主提交内 9 个 canonical 源（blob id）：`auto-av/python-interop.md` `c0020805bb9b`（新）、`auto-av/architecture.md` `0a1b6d50ba97`、`auto-av/project.md` `79f0fb78f45b`、`auto-av/host-import-and-console.md` `917669aa96ee`、`auto-mir/hir-lowering-boundary.md` `9d4fc7a99c67`、`auto-mir/native-data-owned-lowering.md` `8730269c31d7`、`auto-mir/core-i32.md` `aff20571b99a`（F-24 交叉引用）、`ace-migration/replacement-scope-and-evidence.md` `c8ce57de6458`、`stdlib/design/ace-host-services.md` `d847cb1dbcd6`；SD-01..06 全部落地，F-27 措辞已按实际谓词收紧（"非 profile-8"限定改为代码实际条件） |
| 冻结门并集（落主 tip 实测） | `ace-source` 864/864 PASS、`ace-python` 10/10 PASS、`ace-lowering` 9/9、`ace-console` 5/5、`case-ac-av` 18/18、`case-ac-cte` 6/6、`case-ac-mn` 3/3（`run-l52d0vdo`，wall 493.9s）；并集另跑 `ace-artifacts` 43/43、`ace-stdlib` 2/2、`ace-foreign-contract` 56/56、`ace-abi` 314 执行（24 红=基线）、`ace-host` 171 执行（13 红=基线）、`ace-semantics` 295 执行（1 红=基线）——三组红集合与 KNOWN-DEBT 台账（ace-abi source_foreign 族 ~24、ace-host 13 例、ace-semantics sema_boundary、1060-R5 closures 断言）逐条对应，**无新增红**；`ace-av` 基线 E0004 不可运行，不声明绿 |
| 已知限制（guard） | `verify-mir-lowering-868.ps1 -Scope identity` 在本基线**不可运行**：脚本仍需退役的 `test-cases` feature（PLAN-901 拍平），非本计划引入；本轮新增的"crate 常量 == 镜像"断言以正控 + 三负控（crate="4"/mirror="4"/常量缺失）离线验证，并由 `case-ac-av/cte/mn` 实跑佐证 |
| archived | 本文件已 `git mv` 至 `docs/plans/archive/`，`status: archived`；T-01..T-06 完成、T-07 完成、AC-01..AC-09 全勾（AC-02/03/04/09 为 partial-by-evidence-shape，具名后置见 §6）；交付链路 964/967/965/947/972 归档依赖保持 |
| cleaned | 见下方清理段（worktree/branch/组目录移除 + 材料 scope 折叠删除 + 登记表刷新） |

### 11.1 cleaned（2026-11-18）

- **worktree/分支/组**：`wt-guard.ps1` 对 `D:/autostack/.wt/lang-1026/auto-lang` 与组内 sibling `auto-down` 均输出 **clean**（无 reparse point）→ `git worktree remove`（实现树）、`git branch -d plan-1026-dev`（was `181801922`）、`git -C D:/autostack/auto-down worktree remove …/auto-down`、删除空组目录 `D:/autostack/.wt/lang-1026`。`git worktree list` 中已无 1026 条目。
- **材料 scope**：`D:/autostack/plan-artifacts/plan-1026/` 共 **79 文件 / 3,152,009 字节**（探针源码 11 + `ref/` 独立 oracle 3 + `review-r2..r5` 22 + 组/日志 34 + 其它 9）。按 2026-10-31 裁定：需要的证据已折叠进本文件（§9 收据、§11 门并集与基线红对账），**整目录删除**；无具名未解决问题持有。
- **折叠的独立 oracle（V-01 参考，不经 ACE、不用 adapter）**：`ref/train_ref.py`（SHA256 前缀 `3B536E27D1F3C6BE`）+ `ref/train_ref_out.json`（`7E2D5A2C2A8D418F`，99 行）。脚本要点（seed=0、torch 2.14.0+cpu、CPU）：`x=arange(8).reshape(4,2)/2`、`target=ones(4,1)`、`Linear(2,1,bias=False)`、`MSELoss`、`SGD(lr=0.1)`、60 步 `zero_grad/backward/step`，记录 losses、首步梯度非零、最终权重/预测。关键值：first=`0.2370201051235199`、final=`0.1013089045882225`、pred0=`0.4728480279445648`、shape=`[4, 1]`、`losses_finite=true`、`final_lt_first=true`、`final_lt_1=true`；与交付实现经公共 `PySession` 的实测值在 f32 打印精度内一致（§8/§9 记录）。
   语料 README 的"记录于 plan-artifacts/plan-1026/ref/train_ref_out.json"这处**出处指引**在收尾后由本段承担（该路径按政策已删除）；README 自身列出的首/末 loss、pred0 与 shape 值保持不变且正确，故不构成功能缺陷，作为**文档出处迁移**记录在案。
- **登记表**：`D:/autostack/plan-artifacts/.retention/policy.json` 追加 `auto-lang/PLAN-1026` 条目（deleted-at-merge、无 holds、scope 删除量、delivery/archived 身份）。
- **主检出状态**：本计划自身路径无残留 WIP（plan/design/specs 全部已入库）；主检出上仍存在的其它改动属并发会话 WIP，未触碰。

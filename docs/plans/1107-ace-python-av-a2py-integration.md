---
plan_id: PLAN-1107
status: drafting
feature_name: ACE Python AV与A2PY同源互操作及工程分发集成
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [PLAN-1026, PLAN-1042, PLAN-977, PLAN-978, PLAN-967, PLAN-1094]
supersedes_spec_components:
  - docs/specs/auto-av/python-interop.md
  - docs/specs/auto-a2x/python-stdlib-readiness.md
  - docs/specs/ace-migration/m2-build.md
new_spec_components:
  - docs/specs/auto-a2x/python-ecosystem-integration.md
touched_goals: [GOAL-002]
affects: [ace/source, ace/av, ace/a2x-python, ace/m2-build]
---

# [PLAN-1107] ACE Python：AV＋A2PY 同源互操作与工程分发

## 0. 变更摘要

用户2026-11-19明确Python后续能力（含978相关构建/装配/分发）直接在新版ACE做，采用AV＋A2PY。1026/1042/977/978已经归档，本计划是消费其成果的后继，不重开历史计划、不重新实现Python后端或SDK。

AV沿已交PySession/967 provider调用CPython，A2PY沿已有Checked HIR→共享A2X→ace-a2x-python生成Python并直接使用当前Python环境的库；两条路径共用源语义与独立oracle。978的工程缓存/身份/SDK在Python维度接通这两条路径；原native/AC/A2R/A2C职责保留。当前只规划，不实施/构建/运行探针。

## 1. 目标

- 同一份标准Auto Python程序经新ACE AV和新ACE A2PY均真实运行，与独立原生Python参考一致；以§5 PyTorch训练为首个完整工程。
- Python成员、调用、kwargs、索引、运算、显式转换、对象身份/生命周期及错误经共享受检语义消费，不回旧AutoVM/旧PythonTrans。
- 基于既有978 m2-build完成Python源构建/文件集/预编译消费/可重定位SDK，构建身份、运行兼容、工件选择分账保持。
- 精确登记已交、待接线、待验证、缺能力/环境；不将已有A2PY基础设施当作Python库互操作已全面交付。

非目标：重做Python执行器/生成器/FFI/store/loader，重开归档计划，取消1094及1078–1096的AV-only阶段策略，GPU/图形Python后端、全部Python包或Python语言替代、任意动态反射静态推断、无边界零复制buffer、模型热迁移。AV其他语言齐平、完整UI/产品迁移及新平台认证继续归原owner。

## 2. 架构方案

目标设计为[Design58当前路线](../design/58-python-standard-syntax-interop.md)。共同Auto源→ACE统一前门/绑定/Checked HIR及受检Python操作身份，分两条消费链：

1. AV：唯一共享HIR→MIR→AV foreign stub→967 shim/Rust adapter→PyO3→CPython。复用1026 PySession/资源账，不复活旧VM。
2. A2PY：共享A2X语义视图→已有PythonTargetProvider→生成Python/runtime helper→声明的OS Python→原生import/getattr/call等Python协议。**Python库操作默认直接运行于生成程序的解释器，不绕回PyO3再次嵌入CPython**。真实C/Rust外库仍消费967既有ctypes/C ABI路径。

本计划只补共同语义投影、目标接线和工程消费，不造第二parser/typechecker或按torch名字/源文本正则替换业务。当前A2PY foreign_bridge只支持其冻结C/Rust域，不能把ctypes链已存在当作use.py训练已可运行。

## 3. 技术栈

已有`ace-core/source-query/lowering/av/abi`、`ace-a2x-core/a2x-python/a2x-store/a2x-worker`、`ace-m2-build`、TestCase/AutoDown及独立Python oracle。Python/torch环境按实际支持矩阵冻结；AV既有1026环境为Windows x64、CPython3.14标准GIL、PyO30.29.3、torch2.14 CPU，仅作为有来源基线，不伪称本轮重跑。

## 4. 需求分析与背景调查

### 授权与现状

- 用户本轮要求Python工作统一ACE AV＋A2PY，含978的Python工程能力。沿本聊天此前约定，仅先做设计/计划，不推定本轮work/安装/发布授权。无额外模型、并行agent、预算或工期指派。
- 主检出master，读取HEAD=`b067fc0603540118b792c7b7171df6c0bae88d59`。存在大量其他会话文档WIP/未跟踪Rust文件，均保留，不清理、不接管。已读auto-plan-new当前技能（§8用独立checkbox任务），Specs overview及相关模块。
- 已确认1026/978/1042/977归档终态，不改其状态/冻结验收。978 r14已依赖1042且包含A2PY；无需把整个978“迁移一次”，只认领Python生态工程缺口。
- 1094 r3是AV-only脚本/Python协议补齐，尚待实施；其源码/病例共享，非AV检查按阶段保留，不以本计划反向卡其AV出口。

| 当前来源 | 已观察事实 | 本计划消费/差额 |
|---|---|---|
| `docs/specs/auto-av/python-interop.md`；`abi/src/py_session.rs`；`core/src/source/py.rs` | AV标准use.py、CPU训练和17帧操作已交；受限显示/浮点及已知前端边界 | 复用产品入口和受检身份，真正源面差额交1094最小切片，不复制一套AV协议 |
| `docs/specs/auto-a2x/python-stdlib-readiness.md`；`a2x-python/src/{provider,lower,print,runtime,ports}.rs` | 新PythonTargetProvider、AXPY1..3、真实run-py及资源helper已交 | 增Python对象操作的目标投影/实际运行，不能回旧AST PythonTrans |
| `a2x-python/src/foreign_bridge.rs` | 生成ctypes调用冻结Rust/C provider，超域具名拒绝 | Python包直接导入是另一语义分支，不扩大C ABI猜签名，不套双重解释器 |
| `docs/specs/ace-migration/m2-build.md`；`m2-build/src/{driver,identity,compat,selection,recipe}.rs` | 五目标工程层已有av/a2py，三类身份分开，SDK带声明解释器 | 核use.py工程是否实际接到这些入口，补Python依赖/运行与精确失效，不重造构建系统 |
| 1026既有`pytorch_train.as`及README | 已交版本有五处表面调整：浮点后缀、循环/入口形态和两处实参提升 | 作为历史基线；本计划要求§5原始标准语法，不把未验证的嵌套实参写成已证拒绝 |
| `1094-av-parity-script-and-python-protocols.md` | 消费1026并补完整脚本/对象协议，当前仅验AV | 本计划消费所需正式前门/协议切片；A2PY对应检查只由本Plan按任务接手后打开 |

本轮只读源码/Spec/归档记录，无当前运行证据。语义冲突以规范/已批准合同核定，不能把旧bug或v1工程限制固化为全语言规则。T-01补来源SHA/具体接口版本与consumer矩阵，未知不填PASS。

## 5. 详细设计

### 5.1 首个同源工程

下列标准Auto源码为新验收输入（拟新增共享case，不改归档夹具）。AV和A2PY不得各维护一个改写版；若现有源面不支持，按IF-AV记录真实差额并消费1094正式修复，不悄悄加hoist或改循环拼写。

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

### 5.2 职责、接口与依赖

| 接口 | owner与实际消费 | 本计划负责/关闭条件 |
|---|---|---|
| IF-AV | 1026已交PySession/967资源；1094所需共同源面/协议正式切片 | AV源语法/通用脚本修复归1094；本计划接消费和双端验收，不另写PySession/解释器。未交接口先静态核差额，不拿WIP作前提 |
| IF-PY-SEM | ACE现有前门/受检Python操作与共享A2X语义视图 | T-02暴露/消费有来源的Python操作投影，绑定类型/参数/效果/所有权，不凭内部函数名字符串把任意foreign调用当Python；原生Auto操作和非法源不被污染 |
| IF-A2PY | 1042/977已交PythonTargetProvider/helper/store/worker | T-03在原后端实现原生Python协议及Auto语义保持；不建新A2PY、不import旧PythonTrans、不依赖AV执行器 |
| IF-BUILD | 978已交m2-build、967双身份/工件/接口清单 | T-05接Python工程源构建/SDK运行，区分AV嵌入解释器与A2PY外部解释器；不改造为第二loader/store/FFI |
| IF-CASE | 已有TestCase及1078/1094的canonical作者/逐check选择 | T-06复用`crates/auto-lang/test/cases/language/script/`的同源病例及必要具名新case；1094 AV view仍只选AV，本计划只在A2PY接手验证后打开对应check，不批量解除AV-only策略 |

依赖方向是已交底座＋1094所需最小切片→1107→后续产品/发布认证，不增加1094/978/967对1107的反等。frontmatter列接口owner，不机械等1094整包归档；接口就绪须有正式合入SHA和适用验收。967/978原非Python目标/资源/ABI/身份规则全部保留。

### 5.3 同源语义与数据

- 标准调用/绑定方法、属性、整数索引、四则、命名/嵌套实参、显式`.to(float)`使用共同受检语义；Python对象身份/alias与原地修改可见。先导入/receiver/位置/命名实参按源序各一次；导入遮蔽按Auto绑定规则。
- AV对象用现有受管引用，Python侧Tensor载荷不由桥每次深拷贝；A2PY用当前解释器的真实对象引用，不为普通Python操作制造Rust opaque注册表/重复嵌入解释器。
- 小值/文本/Auto容器转换明确按值域/位宽/编码/预算处理，显式Auto资源保持helper/finally的owner清理；不以Python GC替代Auto owned资源释放。Python对象最后Auto owner释放其引用，不承诺销毁仍被Python持有的对象。
- Python异常映射共同语言错误/源码位置，host权限/依赖缺失/资源/cancel分类保留。Meta区分静态语义、AV资源权限、Python动态观测；type_name/shape缓存不能授信或掩盖属性变化。
- AV v1的ASCII/256B显示、非有限浮点边界等仅是旧切片限制，不强迫A2PY故意同样拒绝。T-01逐项列common-required、目标独有、待1094/后继、环境条件；本计划列明的必验不得降成NA或以emit成功代真跑。
- 新投影/target helper/受检身份发生实际规则改变时按所属owner版本化，不能重释已有AXPY/ABI/bytecode域；不无理由升全部wire轴或使旧归档证据冒充新能力。

### 5.4 978 Python工程与身份规则

| 项目 | AV | A2PY |
|---|---|---|
| 开发/执行 | ACE公共源入口及已编译宿主/PySession；PyO3/CPython provider显式装配 | ACE共享A2X生成Python文件集及helper，声明OS Python真实运行 |
| Python库 | CPython/PyO3边界、解释器身份与原provider约束 | 当前解释器原生import/use；C/Rust外库另沿967 ctypes路径，不经AV/PyO3兜底 |
| 构建身份 | 978/967现有BuildIdentity、源/语义/lowering/provider实际输入 | AXPY生产者/打印/helper内容、输入/语义/具体目标与依赖，仅真实读取失效 |
| 运行兼容与选择 | 解释器ABI/版本/平台/CPU、provider工件pin、当前授权与generation | 解释器/包版本及解析身份、生成物/helper完整性、平台/实际依赖与运行策略；不用provider rustc等于consumer来决定可否运行 |
| 分发闭包 | 已编译宿主＋provider＋声明的CPython/torch/native依赖＋合法源/字节码 | main.py/模块/helper/manifest＋声明解释器与Python包/必要native库；不为纯Python库强制携带AV、PyO3或Rust工具链 |

共同保持978的BuildIdentity/RuntimeCompatibility/ProviderSelection三账、精确pin/目标/ABI拒绝、可重定位、CURRENT原子保旧、恢复重验和lease。Auto body编辑只重产受影响语义/目标节点，不重编不变Rust宿主/provider；解释器/包变化按实际构建/运行读取分类，不能ABI兼容就错命中或全局清缓存。纯A2PY emit不执行torch/不装载库；run阶段缺Python/torch明确失败/NOT_RUN，不充PASS。构建/版本探针与真正运行计数分别记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-a2x/python-ecosystem-integration.md`（拟新增） | 两项独立底座→实际已验同源AV/A2PY协议、支持矩阵/数据/错误/三方oracle | Python新链单源 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 保留1026 v1冻结事实→追加本次实际消费的源面/协议与未交域 | 不把旧限制作永久规范 | AC-01/03/04 |
| SD-03 | modify | `docs/specs/auto-a2x/python-stdlib-readiness.md` | 1042/977已有后端→追加受检Python互操作及真实运行，明确与ctypes区别 | 不复制A2PY | AC-01..04 |
| SD-04 | modify | `docs/specs/ace-migration/m2-build.md` | 978已有五目标SDK→追加已验use.py AV/A2PY工程/包依赖/缓存/分发，其他目标保持 | 不重开978 | AC-05/06 |

本次不写Specs/legacy specs.json/派生索引；未来merge只沉淀真实通过的新增知识。

## 6. 测试设计

全部拟建/未执行，T-01冻结实际接口/环境/命令；当前只读调查不能算失败或通过。使用独立手写Python参考，不从ACE产物/实际值生成expected。CPU同环境seed=0、相同dtype，按Auto目标浮点舍入后`rtol=1e-5, atol=1e-6`；不承诺跨torch版本随机权重相等。

| case | AC / owner_stage=work | 实际路径/区分性oracle | 负责与边界 |
|---|---|---|---|
| V-01 | AC-01/02 | §5原样同源训练→AV和新A2PY真实执行＋独立Python：60个有限loss、首/终loss/首步梯度/最终权重/预测对齐，loss降低且<1、shape=[4,1] | T-03/04/06；emit或打印快照不能替代训练 |
| V-02 | AC-03 | call/kwargs/嵌套实参/绑定方法带计数副作用，参数/返回/分支/循环与遮蔽；两端按独立Python trace验证源序且一次 | T-02..04/06；非法Auto源/假Python身份仍拒绝，不按torch名字特判 |
| V-03 | AC-04 | 同一Tensor多次传递、别名修改/对象is、显式float后backward、错误转换后继续用；深拷贝错误控制须红；weakref/资源trace核最后owner和Auto finally | T-03/04/06；不要求Python算法本身零分配，也不靠RSS归零 |
| V-04 | AC-03/04 | Python异常/catch/未捕获/用户位置，坏provider/解释器/撤权/错依赖、可变property/shape；纯Auto近邻与无Python程序 | T-03/04/06；AV资源账/Meta边界与A2PY原生协议分开，不能共享错误PASS |
| V-05 | AC-05 | 新978真实build/run/SDK入口分别消费use.py工程：emit不得执行torch，run缺环境不PASS；运行trace证明AV不走旧VM、A2PY不走PyO3或旧PythonTrans | T-05/06；新增命令仅在真实接入后记录，不发明现成CLI标记 |
| V-06 | AC-05/06 | 冷/暖/Auto body编辑/改helper/包或provider身份/兼容换包/撤权恢复；实际producer和cargo/rustc/bindgen/python进程及工件计数 | T-05/06；复用978分层/负控，未变Rust不得重编，pin不匹配仍拒绝 |
| V-07 | AC-06 | AV/A2PY包移至统一scope异根，禁开发树和Rust构建工具访问仍按声明运行；缺main/helper/解释器/torch/必要DLL或坏manifest/pin控制 | T-05/06；构建工具与运行依赖分列，不能将纯A2PY包偷偷变成AV包装 |
| V-08 | AC-01..06 | 旧1026、1042/977和978原适用回归/emit-as-run/legacy-fallback控制，源码/target数据flow/依赖图检查 | T-02..06；复用身份有效证据，补触面新oracle，不用归档已绿冒当前通过 |

沿现有固定组：`ace-python`、`ace-a2x`、`ace-m2-build`，修改共享前门/语义/lowering/工件时追加`ace-source ace-semantics ace-lowering ace-artifacts`及真实消费者，选择理由按实际改面写回。公开`run-py`与978`ac-build run --sdk`的准确recipe/输入/profile由T-01核实现后固定，新增Python集成case归既有group/targets，不新造全仓runner。code check仅在专属树，最终review后merge在冻结提交运行审定并集，不反复全仓或跑ace-perf。

Windows不运行Linux/WSL用例；Linux源/原义务保留，由1063/专用Linux环境认证。条件排除/缺环境/未选/实际PASS分列，不把Linux未跑记红或绿。原采样/预算/平台条件不减；所有输出入统一scope。独立review沿原流程，不创建或指定额外agent/model。

## 7. 验收标准

- [ ] AC-01（owner_stage=work）：相同§5源码在新ACE AV＋A2PY真实运行，入口/身份明确，V-01两端均通过，不手写桥或双份源。
- [ ] AC-02（work）：V-01训练数值、梯度/参数变化、预测与shape按独立Python oracle成立，不以emit/快照代执行。
- [ ] AC-03（work）：V-02/04绑定/源序/值流/原生近邻及错误/位置/动态协议一致，共享受检身份无法伪造。
- [ ] AC-04（work）：V-03/04/08对象身份/数据/资源/Meta和失败后可用成立；两个后端各用其正确宿主机制，无第二资源/FFI/解释器。
- [ ] AC-05（work）：V-05/06通过真实978 build/run/SDK入口，身份/缓存/运行计数分账成立，无legacy/AV兜底A2PY，无不变Rust重编。
- [ ] AC-06（work）：V-07/08可重定位包、依赖/失败/恢复/原适用回归及支持矩阵通过，缺环境不PASS，其他978后端义务保留。
- [ ] AC-07（review）：同一树冻结写入，独立核AC-01..06、SD-01..04及依赖接口/遗漏/workaround/格式warning，绑定revision+commit，merge门pending明确。
- [ ] AC-08（merge）：冻结门通过、已验Spec沉淀、归档及材料/输出/树/分支清理完成；不借本Plan声称所有Python生态或整版M2交付。

## 8. 执行步骤

未来明确work授权后，在当前harness有原生TODO工具时登记以下稳定任务ID；本次不登记虚假施工进度。唯一拟用树`D:/autostack/.wt/lang-1107/auto-lang`、branch=`plan-1107-dev`，尚未创建；主检出只簿记。开工登记`D:/autostack/plan-artifacts/plan-1107/` owner/用途/in-flight，所有材料/隔离SDK/外置缓存入该scope；正常target在专属树。merge摘要折计划后全删材料，保留须具名未决问题；移除树前用`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <树>`确认clean，不用WSL/Git Bash执行guard。

- [ ] T-01 冻结来源与双端接口
  - owner_stage: work；前置：后续work授权，归档底座身份适用，1094所需最小切片已正式交付后才运行依赖探针。
  - 读取§4路径/证据与README五处偏差，列required/已交/缺能力/缺环境矩阵；最多两种共享操作投影方案比较。用已有受支持的最小程序核源码→Checked→各目标→执行→978 SDK的实际边界；缺失连接只做有界可行性验证并登记T-02..05的实现差额，不要求这些后续任务先完成，也不拿此探针代替T-06完整训练验收。记录版本/命令/预算/owner；上游接口不足先交原owner，不让用户代选内部机制。
  - 关闭：本节/§5写回真实符号/来源SHA、准确recipe和可区分反例；AC-01..06。策略等价无需再问，改变验收/范围须修订。
- [ ] T-02 接通共享Python操作投影
  - owner_stage: work；前置：T-01、IF-AV/IF-PY-SEM就绪。
  - 修改现有`core/source`/共享受检事实与`a2x-core`实际owner，形成有来源的Python操作/参数/类型/效果与资源消费，不复制解析/类型或接管1094的AV通用语义修复。
  - 关闭：V-02/04/08及共享完整职责组；AC-01/03/04。
- [ ] T-03 扩展现有A2PY互操作
  - owner_stage: work；前置：T-02、IF-A2PY。
  - 在既有`a2x-python/{lower,provider,print,runtime,ports}.rs`及适用worker接原生Python导入/对象操作、转换/错误/资源/源图；C/Rust外库仍用现有foreign_bridge。
  - 关闭：V-01..04/08新A2PY真跑、emit不冒运行、无AV/PyO3或旧PythonTrans依赖；AC-01..04。
- [ ] T-04 接通AV同源消费
  - owner_stage: work；前置：T-02及1094所需正式接口。
  - 复用PySession/已交967 adapter，接共同源和返回/错误/资源观测；AV差额回1094，不建立新解释器/私有账本。
  - 关闭：V-01..04/08 AV真跑和原适用反例；AC-01..04。
- [ ] T-05 集成Python工程与SDK
  - owner_stage: work；前置：T-03/T-04、IF-BUILD。
  - 在既有`m2-build/{driver,identity,compat,selection,recipe}.rs`接use.py工程两目标；实现声明的解释器/包/helper/provider闭包、分层缓存、重定位与恢复，原其他后端保持。
  - 关闭：V-05..08及`ace-m2-build`必要集成；AC-05/06。
- [ ] T-06 验证同源工程与回归
  - owner_stage: work；前置：T-02..05。
  - 沿canonical语言病例增加/消费AV/A2PY逐check与独立Python参考、精确执行/依赖/进程/资源证据；仅为本Plan接手的A2PY检查打开阶段ignore，不影响1094 AV-only view；补运行/环境/包说明。
  - 关闭：V-01..08、AC-01..06及全部真实接口/环境矩阵；未交范围明确，不能只交报告。
- [ ] T-07 独立复审核验
  - owner_stage: review；前置：T-06、同树冻结提交。
  - 核AC/Spec delta/原义务/架构与数据flow，扫遗漏/延期/绕过与健康；不在review修代码。
  - 关闭：AC-07；pass保merge门pending，finding返原树work。
- [ ] T-08 合入沉淀与清理
  - owner_stage: merge；前置：T-07 pass。
  - 冻结门并集/规范沉淀/归档；材料摘要、所有实际输出目录/外部消费根逐项对账，guard后清唯一树与分支。
  - 关闭：AC-08；复用有效证据，不把周期全量/性能门加为本Plan默认门。

## 9. 复审记录

2026-11-19：stage=new；PLAN-1107 revision=1，status=drafting/0/8；outcome=pass（规划合同可交付，不是功能/独立实现review通过）；next=后续明确work授权及所需接口正式就绪后T-01。继承用户“Python直接ACE AV＋A2PY”的方向，保归档计划终态、1094 AV-only和978其他后端职责。当前仅查源码/Specs/记录并起草文档，无构建/运行/环境探针/实施树或测试资产。

文档预检：§0..10、T-01..08、AC-01..08、V-01..08及SD-01..04结构/唯一性、计划/设计本地链接与锚点、Python路线新增链接、现有固定组名及变更空白检查通过。1107可达153个本地依赖计划均定位且无环（930仍在旧`docs/plans/archived/`，仅只读定位，不作本次归档目标）；1026/1042/977/978/967保持归档。上述是文档静态核验，不是接口实现或运行正确性的证据。

## 10. 待澄清事项

方向已明确，无需重新选择旧链路。技术未决由T-01负责：共同Python操作投影的实际接口、1094最小源面是否已交、A2PY支持形态/错误/浮点对应、978公开工程/SDK的解释器与包解析身份。支持矩阵/预算冻结前不实施依赖该决定的功能；不能把未验证旧边界当已证拒绝。

本计划首先关闭既定PyTorch真实工程及列明协议/SDK验收；更广公共Python义务继续按977/1094及后续owner逐行承接，不据本Plan单例成功宣称全生态。若用户另要求删除978的其他目标或扩GPU/图形/全Python生态，须另行明确范围，本次不推定。

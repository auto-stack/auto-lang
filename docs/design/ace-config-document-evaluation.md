# ACE M2-B配置文档求值设计与982执行合同

2026-11-17，target-state r1。用户授权仅规划、复用既有成果，不启动实现/探针/测试/构建/建树/模型调用/合入或新Plan。本设计由[Design55](55-ace-m2-complete-language-and-product-replacement.md)登记，正式实施唯一owner为[982/B-CONFIG](../plans/982-ace-m2-man-gen.md)。下述API、类型、profile、fixture和命令参数中标“拟”的内容尚不存在；Spec仍只证明其已验范围。

## 1. 归属、范围与已交复用

`GAP-982-CONFIG-ACE`的性质是配置领域入口和公开结果缺失，不是整个ACE没有对象/record。现AV已有`Value::Object`及循环拒绝、record/dynamic/owned资源；只是通用`canonical_owned`不导出Object/External、公开`AceValue`仍是Copy标量/Opaque，也没有配置文档profile或文档执行入口。不能据此重建一般动态类型系统。

**本轮裁定的规划归属：982现有B-CONFIG-I/V/R端到端承接语言仓和Man仓，扩充既有跨仓范围；不新建Plan，不重开已归档969/979。** T-01的A已交状态不回退；B-CONFIG-I先复核B输入/公开字段/资源和实现方案。原37任务ID、全部A收据及CONFIG/EXEC等原验收保留，CONFIG行现含A已交子面和B完整义务。

| 责任 | 现有任务/模块 | 不反向等待 |
|---|---|---|
| 配置语法/投影/语义与AV内部公共接面 | 982 B-CONFIG-I，在ACE共享syntax/core/source/lowering/av原owner增量；修改只限配置profile及必要共享原语 | 不等980/981全包、1068全包或AAC自举；不另建parser/表达式解释器 |
| 中立SDK接口、桥与文档数据返回 | 同任务，auto-lang-core/auto-lang-ace-host；复用969注册/970生命周期/967 typed资源纪律 | 不把969归档计划当未来交付者 |
| Man配置消费、依赖上下文与真实CLI | 982 B-CONFIG-I/V及原EXEC/TEMPLATE/DEVSDK消费者；沿原Pac/ProjectDescriptor/锁 | 不等983/984消费者全部完成才交CONFIG |
| CONFIG批次独立复核/正式交接 | B-CONFIG-R→原T-06的CONFIG phase fold，保树续剩余B；最终T-05/06/07整包收口 | 阶段可消费，不冒982整包完成 |
| 下游接收 | 983 CONSUMERS、984 SHELL/CODER/EDIT/CLOSURE、1068相关host边界、985 B；986安装/987退役 | 下游不反置为CONFIG生产前置 |

复用来源：Plan364/375运行期累积与注入覆盖语义；旧AutoConfig/Node/Pac与配置回归；780项目模型对静态子集与动态配置的分轴；Man002的配置类型资料只作已有产品schema输入；979共享Schema/CST与唯一verify；964对象/record和AV资源；969公开注册、host错误/取消；982 route C身份/隔离/坏配置零写。旧编译器/VM仅作对照，不进入新生产求值。

## 2. 现代码边界与正确的前门

已读的实际路径：

- `crates/auto-lang-core/src/config.rs::AutoConfig::{from_code,from_file}`→`entry.rs::eval_config_with_vm`→旧Parser `CompileDest::Config`→`Codegen::{new_for_config,compile_config_program}`→AutoVM累积指令→auto-val Node；它不是静态Atom reader。
- `experimental/ace/crates/core/src/source/domain.rs::compile_domain`的当前`DomainBehavior`仅单表达式，隐藏函数逐字段求值**不能**表达跨字段变量、循环和累积作用域，不能直接接线冒整文档执行。
- `core/src/source/mod.rs::SourceProfile`无配置面；`auto-lang-core/src/ace_entry.rs::AceValue`是标量/Opaque；`auto-lang-ace-host/src/lib.rs::HostExecutor`持公开引擎注册及AV实际dispatch。
- Man `auto-man-runtime/src/automan.rs`和`project_model_adapter.rs`、`auto/src/{main,cmd_run,ace_candidate}.rs`是实际消费者；后者A腿明确使用legacy配置owner，不修改它的历史收据。

拟增加`config-document`领域profile和`compile_config_document(pkg, source, context_schema, ...)`原共享source入口。复用Schema编译、CST前门和Program投影，扩领域候选以承载**一个有顺序的合成入口函数体**，再经现`compile_program_parts`/共享resolver与唯一verify进入Checked HIR、现MIR/ABC检查和AV执行。保持现`compile_domain`调用及AGE/普通profile含义不变，不把配置编译为一组互相无状态的隐藏字段函数。

`dep/lib/app/file/dir`等只有在config Schema规定的文档节点位置采用节点语义；相同标识符在表达式里仍按普通绑定/函数处理。禁止跑旧UI宏预处理；`app(...)`不展开为UI App。原节点bare/id/call三种头、命名参数及`pair: file(...) { ... }`均须进入共同CST；不是手工扫描字符串。通用文档允许扩展属性/节点，产品已知类型约束沿已有schema/ProjectDescriptor；不能用780静态白名单拒绝全部动态配置。

## 3. 文档类型、运行与公开接口（拟议）

### 3.1 中立owned值与文档快照

概念设计，实施可在保合同下调整内部布局：

```rust
// 拟置auto-lang-core中立配置合同模块；不引用ACE/AutoVM类型。
enum ConfigValue {
    Null, Bool(bool), I64(i64), U64(u64), F64(u64 /* IEEE bits */),
    Char(char), Text(String), List(Vec<ConfigValue>),
    Object(Vec<(String, ConfigValue)>), Node(Box<ConfigNode>),
}
struct ConfigNode {
    name: String, id: Option<String>, args: Vec<ConfigValue>,
    props: Vec<(String, ConfigValue)>, children: Vec<ConfigNode>,
}
struct ConfigDocument { root: ConfigNode }
struct AceConfigRequest<'a> {
    source: &'a str, virtual_path: &'a str,
    context: &'a ConfigContext, // 显式args/env/解析根/允许导入闭包
    limits: ConfigEvalLimits, cancel: Option<&'a AtomicBool>,
}
// 拟接口；沿同一已注册AceExecutor增default具名unsupported方法，
// HostExecutor实现该方法。不是第二全局executor registry。
fn eval_config_with_ace(req: &AceConfigRequest<'_>) -> AceConfigReport;
```

`AceConfigReport`分别表示Success{document,identity,cost}、Diagnostic{stage,code,original_span}、Denied、ResourceDenied、Cancelled、HostFailure。成功带实际engine=av、source/context/schema/SDK/ABI身份及真实dispatch事实；不能仅写same_sdk=true。保持原`AceValue`的Copy标量语义，新增owned专用结果，不把文档藏在Opaque或stdout JSON里。

运行时复用现对象/数组/record/资源的准入与owner；拟配置专用`CfgValueRef/CfgNodeRef`和累积容器在原typed host/intrinsic接面承载不同字段种类，句柄携request/owner/generation/kind，不是裸整数或Rust指针。必要的config construct/lookup/append/merge/finalize原语只操作值和容器；表达式、函数调用、if/for、变量与模板计算必须由受检AV程序执行，Rust不得遍历AST替代执行。普通ACE对象同型限制不被这个领域私下改成全语言新承诺。

退出前将完整已验文档导出为无AV句柄/外部指针的owned快照；保持精确有符号/无符号数、浮点tag/Unicode/NUL、Node与Object区别、节点参数和孩子顺序/重复节点。属性覆盖按旧set_prop/Obj的值与可观察顺序语义，不能用JSON排序丢失孩子顺序；仅身份规范化可独立排序非顺序敏感键。内部对象alias/赋值保持现语义，最终快照与后续consumer互不借用运行帧。输入/输出拒循环，逐节点计费，迭代式导出/释放不能递归栈溢出；修复后下一次真实read/run可用。

### 3.2 ABI/代码落点与本轮选定机制

拟稳定版本：source profile=`auto.source.config-document.v1`，domain=`ace.config.document/v1`，公开owned结果=`auto.config.value/1`；三者及compiler/SDK/host ABI进入正式来源。新profile不启用AGE关键词或改变普通程序解析规则。

拟`ConfigContext{args,env_snapshot,source_root,module_bundle,schema_id,grants}`：args是带tag的键值数据，env为显式文本快照，source_root/module_bundle定义允许读取的源码身份，schema_id/grants每请求核。源码、数据和grant不共用字符串重解释；未知类型/路径/能力分别诊断。

本包选定**受检配置值/容器原语**，沿现967 typed资源与ForeignSession生命周期接入；SDK构建时生成/验binding与宿主，单次配置运行不编译Rust。拟内部签名`cfg.node(name:Text,id:OptionalText,args:CfgList)->CfgNodeRef`、`cfg.put(CfgNodeRef,key:Text,CfgValueRef)->Unit`、`cfg.append(CfgNodeRef,CfgNodeRef)->Unit`、`cfg.merge(CfgNodeRef,CfgObjectRef)->Unit`、`cfg.get(CfgValueRef,key:Text)->CfgValueRef`及带kind检查的scalar box/unbox。reference的provider/owner/request/generation在准入与每次读写核；转换错误是具名失败，不能空值兜底。字面量/函数和if/for的语义由ACE共享类型/IR/AV计算，原语不接AST、不选择业务分支。预算和释放统一现资源账，不建第二全局ledger。

| 真实已有owner | 拟新增/修改位置（新文件由B-CONFIG-I建立） |
|---|---|
| `core/src/source/{domain.rs,mod.rs,cst}`、共同grammar/Program/adapter | 拟`source/config_document.rs`及config grammar rules；扩单入口Program候选及精确source map；旧domain调用仍兼容 |
| 原lowering/av-core的checked调用与foreign执行 | 配置resource签名/准入/累积与owned导出；需要的新helper限本profile，不复制AV或旧ConfigCodegen |
| `auto-lang-core/src/{ace_entry.rs,config.rs,lib.rs}` | 拟`ace_config.rs`中立请求/结果/注册方法与显式engine构造，默认core不依赖experimental ACE |
| `auto-lang-ace-host/src/lib.rs`、现foreign/resource owner | 拟`config_document.rs`桥：compile→verify→execute→导出→teardown；实际AV/预算/取消/计数及diag透传 |
| Man `auto-man-runtime/src/{automan,pac,project_model_adapter}.rs`、`auto/src/{main,cmd_run,ace_candidate}.rs` | 消费新结果与同SDK身份，二阶段dep/context及完整原调用点接线，既有ProjectModel/包锁继续复用 |

B-CONFIG-I开始时按上表在唯一两仓树冻结实际文件/ABI与最低负载：一份带根注入、对象/模板、运行期for、两个Node孩子的文档经同一公开请求走到底，再证坏邻例/限额/修复读取；此全链不成立则needs_replan，不用只绿CST替代。零新Cargo workspace/无新renderer/不启动自动探针。

### 3.3 SDK接线与引擎选择

拟`ConfigEngineChoice::{LegacyVm,AceAv}`及AutoConfig带显式engine的构造入口，旧签名/A默认行为保持至原986默认切换窗口。ACE候选和B新产品入口明确选AceAv，配置失败不得重试LegacyVm、静态Atom或ProjectModel默认值。边界适配可把成功ConfigDocument结构投影为现auto-val Node供Pac消费；这个纯数据转换不可调用Parser/VM、无法执行配置。

配置args是`port/kernel/...`上下文，不是脚本argv；非空合法配置上下文必须真实注入，不能套现`ace.args/unsupported`把全部配置拒绝。env按当前产品调用形成显式快照，禁止无身份ambient读。默认无网络/文件写/process/任意FFI；读取源码/允许模块由宿主授权并纳入闭包身份，确需原有能力逐项走现967 grants，未知能力具名拒绝。未安装桥/关特性/不支持平台均具名非零，零legacy dispatch。

**Host配置求值与应用backend分开：** 构建工具/启动器可用AV求配置得到数据，再编译/运行AC或A2R业务；这不是AC guest的AV回退。产物若把配置源作为运行期动态API，则仍由原目标/1068按该运行API单独验，不能拿宿主AV结果冒机器码支持。本包CONFIG无需为UI三波逐次实现三份求值器，不等待1067异步或完整1068才交。

## 4. 必须守恒的配置语义与具体输入

1. 每份文档从root容器开始，语句按源顺序；Pair累积属性、Node语句追加孩子、Object语句按原规则合并；普通表达式值丢弃。空文档返回空root，不能作为无执行的假成功。
2. `var`只绑定局部，不成为root属性；顶层Pair同时成为后续表达式可见的同名绑定。重复var与Pair更新保旧可观察语义；if/for在AV运行期执行，不能按源码形状静态展开。
3. 注入args优先于**根级**同名默认Pair，局部绑定及root属性都反映覆盖值；被覆盖默认RHS不执行。此默认选择在config投影中按context键集归一化，语法仍检查，source map保持原源；键集进入身份，不能让普通Auto profile出现这种特殊规则。嵌套节点结构字段不被同名外部arg覆盖。`on/off`仅为缺省config bool绑定，显式同名args优先。
4. 子包dep携带的object/array/bool/number覆盖值完整进入下一次求值；不得像早期只注入字符串。root属性、对象字段、模板、Node-valued Pair、命名id和孩子顺序保真；显式id为空而id属性为文本时按原规则提升为节点id，同时保留该属性；注入函数/外部运行句柄不能静默丢掉，按原支持域具名处理。

以下取自现有旧回归与真实文件，不改源作为通过手段：

```auto
var kernel_config = { mode: "lockstep", mpu: true }
kernel: kernel_config
if port == "win32" { x: 1 } else { x: 2 }
modules: ["a", "b"]
lib("x") {
    for d in modules { dir(id: d) { at: d } }
}
```

独立预期：kernel字段是对象且mode=lockstep/mpu=true；kernel_config不进root；port=win32时x=1，port=linux时x=2（纯字符串分支，不是在Windows跑Linux目标）；lib(x)顺序有dir(a),dir(b)，各at对应a/b。

```auto
kernel: { heap: "heap_4" }
dep("osal") { heap: `${kernel.heap}` }
device("chip") { icf: file("linker/x.icf") {} }
```

独立预期：dep(osal).heap=heap_4；icf是名file、参数linker/x.icf、带体的Node值，不是普通函数调用的文本或被丢弃属性。再以args注入kernel={heap:"override"}及at="outside"，root默认不执行，dep heap=override；嵌套dir的at仍取其原值。

## 5. 预算、缓存、失败与平台

拟ConfigEvalLimits默认：source_bytes=1MiB、value_nodes=65536、owned_bytes=16MiB、output_bytes=8MiB、steps=1000000、wall=2s；是版本化宿主请求预算，不是语法/语言永久上限。沿现FrontBudget/ExecLimits做准确适配，调用方可显式给不同预算；保旧合法样本/原规模。不得复刻旧32locals/32KiB栈为新语法限额，不增加固定嵌套深度阈值；溢出/预算不足具名ResourceDenied，坏语法/类型、授权失败、工具故障分别报。B-CONFIG-I最低负载要证完整导出与深宽遍历/释放，旧required样本若默认预算不够须给反证修订，不能砍例或静默截断。

总deadline覆盖解析、编译、执行、导出，阶段切换用剩余预算，不重置wall；子包图的累计预算沿项目owner分账。cancel检查覆盖真实运行/导出，候选释放后返回；失败不发布半文档，late结果不覆盖新generation，旧已接受配置保持。未知UTF-8/无效tag/外部句柄/坏图在产品接收前拒绝。

编译/结果cache身份至少含source闭包hash、context值及键集/类型、env、解析根和平台配置、Schema/profile/adapter版本、SDK/host ABI与grants。可复用编译而不得拿旧结果冒新args值；恢复须验digest/schema/输入身份，失败不使用陈旧输出。原SDK store与锁复用，不建第二缓存/registry。

Windows为当前真实桥/CLI执行腿；纯数据/Schema契约不绑Windows。其他平台的原支持/条件保留，未运行不充PASS；Linux目标在专用Linux环境验证，stdlib/host相关部分沿1063，产品配置/CLI仍由982及原发行owner负责，不能把全部配置功能扔给1063后假称本包全平台完成。不把本设计变成新的985/A或当前Windows日常Linux测试门。

## 6. 验证合同与旧用例映射

下列全部拟新增，B-CONFIG-I使入口/fixture可运行，V实际运行，R独立逐项核；旧AutoVM只为原行为对照，每行还须独立expected，不能仅新旧输出相等。

| Case / AC | 原输入与来源 | 新执行入口/可区分关闭条件 |
|---|---|---|
| V-CFG-SYNTAX / AC-CFG-SEM | Man `static-dep-forms.pac.at`三头、`generated-ui-consumer/proj/pac.at`、364/375 Node-valued Pair | 共同CST/唯一verify→eval_config_with_ace；三dep名称/路径、UI标题/window、Node-valued属性精确；禁旧parser/VM仍运行，删config映射真红；其它profile合法/拒绝面不漂移 |
| V-CFG-FLOW / AC-CFG-SEM | `vm/config_eval_tests.rs::test_config_if_else_picks_matching_branch/test_config_var_declaration_and_reference/test_config_var_redeclared_takes_latest/test_config_for_over_runtime_array/test_config_fstring_template_from_runtime_object/test_config_object_field_access/test_config_pair_value_is_node_with_block` | 对上述独立x=1/2、mode/mpu、两dir顺序、heap_4及file Node断言；source/输入变化真改变AV结果，禁止把每字段分开求值或Rust代执行 |
| V-CFG-CONTEXT / AC-CFG-SEM | `config.rs::test_injected_object_arg_field_access/test_injected_bool_arg/test_on_off_builtin_config_globals/test_default_classes_config_toggle` | object/bool/array/number覆盖/根级与嵌套分轴、on/off显式覆盖；被覆盖默认故障RHS无效果，同名两工程/子包不同上下文不串；缺类型/值不得默认为空 |
| V-CFG-VALUE / AC-CFG-OWN | 自制合法邻例，原`test_config_for_unrolls_literal_array`与780结构值源 | text含中/emoji/NUL、i64/u64边界、空/嵌套list/object/Node、参数/重复孩子精确；析构后文档仍可读，坏tag/循环/错owner/旧generation/截断具名拒；scalar canonical/既有公开AceValue不改变 |
| V-CFG-FAIL / AC-CFG-SAFE | Man plan982 bad-config/bad-eval原源；坏名/字段类型/缺源/撤权/取消 | 原span/stage分类可核，Man执行未开始且工程/state/lock字节不变；坏配置→修源→下一次真实read/run成功，禁fallback/静态默认 |
| V-CFG-LIMIT / AC-CFG-OWN | 合法最低负载、value_nodes=N-1/N/N+1、输出字节边界、动态长循环及深/宽输入 | limits真实扣费、wall/cancel在实际阶段触发，失败释放/保旧、20轮读关无跨请求保活；不得以host返回空root取代资源拒绝 |
| V-CFG-IDENTITY / AC-CFG-SAFE | 改args值/键集、env、cwd/源根、schema/SDK/grant；冷恢复/坏digest | 合法身份可重用；错身份不dispatch陈旧值，修复后重新实际读取；counter事实只补证不代独立值 |
| V-CFG-PRODUCT / AC-CFG-PRODUCT | Man plan982 proj-a/proj-b、780 static-dep-forms/dynamic-structured-values及目录原集合 | 真实候选CLI→新配置结果→Pac/ProjectDescriptor/dep/target→实际动作；A route C oracle保为历史对照，新B config.engine=av且legacy_config_dispatch=0；错误零写、两工程隔离、冷B路径真实读；同PID改源/改context配置再次求值结果改变且cargo/rustc=0（SDK/host本身变更的构建另计） |
| V-CFG-RETIRE / AC-CFG-PRODUCT | SDK/CLI/模板/watch/安装配置链及旧函数名镜面反控 | B候选配置路径无旧求值边；986安装无开发树，987缺新桥必须失败而非拉旧VM。依赖闭包与实际调用均核，旧符号只在明确隔离兼容/对照允许 |

**真实命令与拟入口。** 已存在固定组`cargo t ace-source ace-syntax ace-av ace-facade ace-abi ace-foreign-contract`按实际修改与consumer选并集，普通parser/auto-val只有真实触面才补相应职责；不把ace-perf或全仓all塞每次收尾。拟新永久target：语言桥`crates/auto-lang-ace-host/tests/p982_config_document.rs`、ACE core原source tests内config profile模块、Man `crates/auto/tests/plan982_ace_config_b.rs`，由I创建后才执行其具名命令。原Man `cargo tc`、`cargo test -p auto-package-lock --test lock_contract`、原plan816/plan805及780矩阵按改面选择，保原条件/忽略项；缺工具/零选择/未跑不通过。旧SCU001外部ignored回归保原条件，自包含375夹具承担日常Node值面，不把缺外部工程伪报PASS。

B-CONFIG-I先冻结managed pac/am/index/state及二阶段args/env消费者清单/原源hash/原assertion；上述表为最低必需，不能拿八个样本抵完整配置原分母。没有新平台/完整异构对象语言承诺；实际越出原配置支持域须needs_replan，不把所有未知配置误收为静态有效。

## 7. 正式交接、关闭与无环顺序

`IF-982-CONFIG-ACE`（拟收据名，不是新状态/runner）包含两仓SHA、Plan revision、语法/Schema/结果协议、source/context闭包、运行身份/SDK/ABI、端到端命令与独立oracle、权限/资源/失败/修复/冷读、consumer清单及未验平台。工作实现和完整V→独立B-CONFIG-R→原T-06 CONFIG phase fold正式落主；若其余982批未完成，保唯一两仓树/resync，T-06整项不提前勾。消费方不得把分支reviewed当正式D。

本包CONFIG生产只等M2A-08及已交共享基础；971/B只在EXEC工具面需要，977/B只在REVERSE消费需要。然后CONFIG→EXEC/TEMPLATE/各target→EXPORT→DEVSDK→全部work→T-05独立final→T-06/07完整合入归档清树。983 AVSERVICE/GENERATED可按原端口先行，CONSUMERS等982/EXEC＋980/WEB；984按原EXEC/RUST接口迁产品、CLOSURE仍等UI/OS。1068接宿主/guest边界与原差额，不作为CONFIG生产前置。

985/B T-02/03及T-08[B]滚动接收新CONFIG，T-04最终核全部原required与V-CFG证据；**M2A-08 r29/95c7fbad8保持原历史route C，不能因本轮补B重新否定或签发A**。986随后验候选安装/默认，987退役闭生产配置旧链；两者不反卡982正式候选。四新UI/Forge不加入CONFIG前置。

## 8. 状态与实施边界

本轮只分配已有任务、补设计/接口/验收与排程，GAP状态为**B已明确承接、能力未实现**。不是已有API或产品PASS，不开新Plan/不改`.next-id`，不重写969/979归档或A canonical事实。未来Spec增量由982真实review/merge落实，包含`m2-man-gen`、`m2-facade`和`config-document-evaluation`的已交规则；当前不提前创建能力Spec。

规划预检记录（2026-11-17）：相关7 Plan原123任务保留；新接口/九Case均未运行；阶段24节点30边无环、新链接可达、源码/Spec/归档未改、无新Plan号。这里只证明合同结构与取材可审，不替未来最低负载/产品/独立复审。

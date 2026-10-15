---
status: proposed
describes: target-state
created_at: 2026-10-15
updated_at: 2026-10-15
implemented_by_plans: []
---

# Design 39：AC/ACC 增量前端、热重载与通用语言解析平台

> 用户需求（2026-10-15）：增量编译、热重载和插件系统从 AC/ACC 架构起点融入；未来将解析能力复用为 Auto 版 tree-sitter，以 Atom/Batom 配置语法，服务 IDE/LSP、第三方语言快速扫描和符号/签名元信息获取。
> 本稿记录已提出的产品约束和建议架构。具体算法、公开接口、产品名、仓库归属、性能指标与排期尚未冻结；不声明已有通用生成器/增量树/原生热重载实现，不分配正式 Plan，不修改 PLAN-782 合同或实现。

关联：[Design 36](36-ac-data-resource-expansion.md)、[后端演进](strategy/auto-native-backend-evolution.md)、[Native ABI RFC](strategy/auto-native-abi-rfc.md)、[ACC 主体合同](strategy/auto-acc-bootstrap-contract.md)、[Atom Schema](strategy/atom-schema-dsl-design.md)、[旧 AIE](09-compiler.md)、[开发工具](14-developer-tools.md)、[方言拆分](32-autoui-repo-extraction.md)、[模块治理](37-v06-modularization-and-repo-boundaries.md)。

## 1. 结论与范围

两项需求直接影响 lexer/parser 的数据模型、入口、错误处理和生命周期，必须在真实 ACC lexer/parser 迁移前建立约束。可以继续补齐 enum/List 等语言能力，但不能把“批处理 parser 完成以后再加增量和通用化”作为默认路线。

建议共享源码与语法服务底座，分别建设 Auto 语义编译和多语言工具消费层：通用产品负责生成/运行解析器、容错具体语法树、增量更新和结构查询；AC/ACC 负责 Auto 名字解析、类型检查、资源分析、HIR 与 native。通用语法服务不依赖 Cranelift、Auto 所有权 checker 或完整编译器运行时。

增量解析、增量语义编译、运行时热重载是三个不同验收面。语法树复用不证明语义缓存正确；缓存命中不证明机器码可安全替换；插件安装也不等于插件可在线卸载。

## 2. 本地事实与缺口

以下为 2026-10-15 源码/文档只读观察，非性能或运行验收：

| 入口 | 观察 | 对设计的影响 |
|---|---|---|
| experimental/ac-core/src/source/{lexer,parser,ast,adapter}.rs | 当前接口以完整源码为输入，lexer 返回 token Vec，parser 返回 Program/Diagnostic；独立源码 profile 选择支持面 | 可继续作为批处理入口与对照实现；未证明局部重扫、容错 CST 或编辑事务 |
| experimental/ac-core/src/atom_text.rs | Span 是 start/end 字节偏移；该 Atom reader 跳过注释、不保存版式 | 字节位置可沿用原则；不能直接称 lossless CST 或通用 grammar engine |
| crates/auto-lang/src/database/mod.rs、compile.rs；Design 09 | 有旧 AIE Database/FileId/FragId 与会话设计；细粒度增量仍有未落地部分 | 可审计复用算法/接口经验，不把旧能力直接记为 ac-core 已实现 |
| Design 14 | 旧 LSP 共享编译器 parser，文档提出失败后缓存上次有效 AST | 新语法服务须返回当前修订的容错树/诊断；旧 AST 仅可作为显式过期 fallback，不能冒充当前源码 |
| Native ABI RFC §1/§7/§8 | 已确认静态/动态模块及插件热重载范围，设计了宿主常驻服务、代际与安全回收 | 新前端需要模块/接口身份和重载边界；不在 lexer 内实现 DLL 装载 |
| 743 inventory；packages/aavm/lib/{token,lexer,parser}.at | token/扫描核心适配，Pratt 核心复用但 S-expr 输出重写 | 复用词法判据、优先级与测试；不能照搬单游标可变状态和 dump 产物作为新公共接口 |

当前 PLAN-782 复审/修复继续按原合同收口。此设计不向该计划追加入参快照、CST、查询引擎、热重载或生成器验收，也不以本稿认定其任何 finding 已闭合。

## 3. 推荐分层与依赖方向

```text
Atom/Batom Grammar + Language/Scanner/Query Pack
                    |
             Grammar compiler
                    |
            versioned parser artifact
                    |
Source snapshots / edit deltas / position maps
                    |
    syntax runtime: lex / parse / recover / reuse
                    |
      immutable CST + diagnostics + changed ranges
                    |
        +-----------+-----------------------+
        |                                   |
 Auto AST/semantic adapter             structure queries
        |                                   |
 resolve/typecheck/resource/HIR        editor/LSP/index outline
        |                                   |
 module/query dependency graph        foreign metadata candidates
        |
 object/link -> runtime module manager / reload transaction
```

图中是职责而非已冻结 crate/结构体。建议未来独立发布语法运行库/语法编译器；当前不创建新仓。LSP 是调用分析服务的协议 adapter，不是语法引擎内置网络服务。批处理 AC、常驻 IDE 和后续 Auto ACC 可用不同内部实现，但须通过同一语法/位置/修订契约及 conformance 语料。

Auto 语法与 IDE 使用同一权威语法定义/规则和适配层。手写 Pratt 可以先作为 Auto 插件或兼容实现；通用生成器不要求所有语言采用 Pratt，也不将上下文语义查询塞进通用 parser。若暂时保留双实现，明确唯一规范源、差分语料和退役条件，不能长期人工维护两份不同 Auto 语法。

## 4. lexer/parser 现在必须考虑的接口

### 4.1 源码、位置与身份

- 源码由不可变 snapshot 标识，编辑 delta 明确 base revision、目标 revision 和替换范围；只接受匹配 base 的编辑，不按到达顺序猜测。取消/迟到任务不能向新修订发布诊断/符号。
- 内部 canonical span 建议采用 UTF-8 字节区间，携带 document/snapshot 身份；原文保留 LF/CRLF、注释和空白。LSP position encoding 经协商映射，支持 UTF-16 等协议位置；774 文本 len/char_at 的 Unicode 标量语义不因源码坐标改变。
- token 同时保留原始范围与必要的词法分类；解码后的字符串值和原始拼写分开。不能因保存 char 码点而丢失字节坐标，也不能把 UTF-8 byte 当语言字符串索引。
- Node/Token handle 的有效域包括语言/grammar 代际、snapshot 和所属树；旧 handle 不可盲用到新树。不承诺跨任意编辑永久不变的 NodeId。声明级 semantic ID 通过受控匹配保持，重名/移动/重命名时需明确失效规则。

### 4.2 状态、恢复与增量

- lexer 显式持有可快照/恢复的状态：注释/字符串/插值、缩进、上下文关键字等。编辑后从可信 checkpoint 重扫，直到位置及完整状态收敛；跨文件或语法包全局可变游标不得参与未记录输入。
- 源码读入口支持分块/rope adapter，避免每次编辑复制全文；scanner 的字符推进与 byte offset 同步，不能靠从头 char_at 扫描每个位置造成隐性二次复杂度。
- CST 保留足以重建源码的 token/trivia、字段名、范围、error/missing 节点与恢复诊断。可用原 snapshot 切片保存文本，不要求每节点复制原文。
- 同一套 parse 服务面向 IDE 返回当前修订 partial tree；批处理编译可调用 strict adapter，遇 error/missing 或不支持构造拒绝进入语义 Checked。语法容错不授予 HIR 凭证。
- 增量单位从文件级正确实现起步，可进到词法区域/子树/声明；早期允许明确的全文回退。必须记录复用/重解析范围，不能把每次全文parse包装为“增量已实现”。
- grammar/scanner 版本变动会使旧 parse state 失效；旧树继续属于旧 grammar lease，重新解析才产生新代树。

### 4.3 旧代码复用的裁定边界

token 表、词法判据、Pratt binding power 和语法案例可复用；单一可变 P 游标应封装在解析会话内，语法与名字/类型环境分离。parser 输出从 S-expr dump 改为结构化语法树/AST 适配是必要重设计。旧 VM workaround、隐式 char/int 混用、历史 lexer bug 不自动继承；每项按现行语言规范审定。

共享 Syntax API 不等于把所有旧 lexer/parser 函数逐行翻译成配置。复杂的上下文扫描可以是受控 scanner 扩展；分派、状态恢复和错误策略仍由公共语法服务约束。

## 5. 增量编译与模块/接口元信息

建议查询层显式表示 parse、declarations、resolve、typecheck、HIR、codegen 的依赖与版本。缓存不能只以文件路径、mtime、语法树地址或裸 TypeId 为键。

| 身份/指纹 | 最小职责 |
|---|---|
| source revision/content | 当前源码与诊断位置；同文本不同位置上下文不可复用错误 span |
| language/grammar/scanner/query pack | 输入语法和结构查询含义；变动按消费者实际依赖失效 |
| semantic config | 编译 profile、包/导入解析、feature、comptime输入、内建/插件能力及分析版本 |
| interface descriptor | 公开签名、类型/资源/ABI承诺、导入需求；导出实现与接口身份分开 |
| implementation/body dependency | 内联、泛型实例、const/comptime展开、效果摘要等消费者实际依赖 |
| target/backend/runtime | 对象码缓存的 target/选项/后端版本/runtime合同；不得误复用跨target制品 |

“接口不变就不重编调用者”只能在调用者不依赖函数体等实现信息时成立。跨模块内联、泛型实例、comptime/const、资源效果摘要均需相应依赖边；旧 Design09 的简化三级hash不能直接升级成新的正确性承诺。

纯空白/注释修改可能只更新位置/展示，但要核查语言的换行/缩进、doc属性、嵌入片段等语义依赖，不能统一忽略。增量结果必须与相同配置下 clean build 的语义/诊断等价；测试另验证实际复用范围和性能，不能只看等价性冒称增量收益。

## 6. 热重载与三类插件

| 类别 | 扩展内容 | 生命周期与校验 |
|---|---|---|
| 语言包 | grammar、scanner、树schema、queries/语言元信息抽取规则 | 版本/内容身份、宿主API兼容、旧tree/session pin；新请求使用新代；不混合旧parse state |
| 编译器扩展/方言 | 解析扩展、语义/诊断/HIR适配或受控pass | 顺序/输入输出profile/依赖/能力明确；不得伪造Checked；非确定性输入进入依赖或禁缓存 |
| 原生运行时插件 | 业务代码、模块状态、回调/任务/析构等 | 采用Native ABI RFC的公开接口、宿主服务、lease、状态迁移、事务发布及安全回收 |

语言包可先静态注册或进程重启切换，再实施在线代际替换；不要求所有编译器插件第一版都可原地热卸载。用户要求的原生模块热重载仍是v0.6核心范围，不因分期被降为可选。数据grammar无需执行任意代码；scanner/semantic插件若有可执行代码，边界另定，普通同进程ABI不是恶意插件沙箱。

parser 只负责给出模块/声明/导入/属性及可重载边界所需的结构/来源；resolver/typecheck建立模块身份和接口描述；backend保留reload boundary、不把可替换实现内联进永久消费者；loader/runtime处理代际和状态。热重载不要求lexer理解DLL布局。

必须从一开始分开 module_identity、interface_descriptor、artifact_identity、state_schema、runtime_requirements。内存/对象/回调/外来deleter依赖旧代代码时旧代仍须pin；重载失败不发布半套入口，新代准备失败保持旧代服务，旧代有在途调用不得卸载。

## 7. Atom/Batom grammar 产品边界

用户确认以Atom/Batom承载语法配置。两者是同一grammar模型的文本/二进制表示，不是两份独立语义。grammar schema、parser artifact、公共syntax-tree schema、结构query规则和metadata schema分别版本化；Atom Schema描述数据合法性，不会自行产生任意源语言的parser。

建议grammar模型表达 terminal/regex子集、seq/choice/repeat/optional、词法/语法优先级与结合性、命名节点/字段、trivia、上下文规则、冲突、scanner扩展点及错误恢复。语法编译器必须校验坏引用/循环或不终止模式/歧义、报告unsupported，记录依赖，不以顺序碰巧匹配掩盖未裁定语义。

算法先进行有界对比探针：确定性LR/有界GLR、PEG等与Auto手写Pratt adapter的适配；评估左递归、歧义、恢复、incremental复用和scanner状态。本文不冻结选型，也不承诺任意语言或线性最坏性能。初期选两种差异明确的语言/profile作验收，再扩面；不承诺兼容tree-sitter grammar.js、查询语法或二进制ABI。

产品需提供独立库API及可批处理CLI。LSP宿主可复用语法诊断、outline、结构高亮/folding、候选symbols；完整definition/rename/type-aware completion依赖语言语义服务或既有官方LSP。

## 8. 第三方语言 symbol/signature 的信任分层

建议元信息同时记录language、source/build config、grammar/query版本、snapshot、范围、来源工具与状态，而不只输出一个signature字符串。

1. **syntax候选**：从CST提取声明/名字/参数文本/导入与范围，可不完整、未解析、条件编译不确定；用于快速导航/索引或下一步分析候选。
2. **semantic元信息**：由该语言的resolver/typechecker、官方编译器/LSP或经验证adapter补齐作用域、类型、重载、泛型/宏/条件配置和真实导出。缺失时标记unknown，不猜测。
3. **interop可调用接口**：将semantic结果映射为经目标ABI/布局/所有权/链接验证的桥接descriptor；只有这一层可供编译器建立可执行FFI绑定。

例如C/C++预处理与宏、Rust cfg/宏/泛型、动态语言运行时定义，不能仅靠grammar保证真实导出。源码声明也不证明目标二进制确实导出同符号。加载器仍须核验库/target/公开接口和运行时能力，解析器不成为新的类型/ABI授信捷径。

## 9. 后续计划顺序建议（未分配编号）

| 工作包 | 可独立验收成果 | 前置与时序 |
|---|---|---|
| A. 前端共享契约与全路径可行性探针 | snapshot/edit/position/CST/恢复/grammar代际/API、增量依赖与模块身份最小合同；两种语法代表例和Auto兼容探针 | 建议作为下一项架构准备计划；可先规划，涉及782实现接口待其fresh收口基线 |
| B. enum/is + TokenKind | Auto语言枚举与native分派；语法与语言无关token ID分层，不把通用树kind硬绑Auto enum | 可与A共同规划；无需等待完整parser-generator交付，不向782追加任务 |
| C. 集合/模块和首个ACC lexer | List<Token>、use、拥有文本适配与真实扫描对拍 | 依赖语言能力；迁移前消费A合同，旧核心按§4.3适配 |
| D. 通用grammar compiler/runtime MVP | Atom/Batom grammar→parser产物→容错CST/增量edit、结构查询及两个语言profile | 依赖A算法探针；独立产品工作包，不塞进enum计划 |
| E. Auto parser/AST及增量语义查询 | 共享语法规则、真实AST、声明/签名、查询依赖；clean vs incremental一致 | 根据A/D实测决定先adapter还是生成器接入，不阻塞所有自举语言扩面 |
| F. ABI/热重载并行线 | 模块descriptor/接口兼容、静态/DLL探针→常驻宿主/lease→状态迁移/发布/回收 | 尽早与A对齐身份，按Native ABI RFC独立实施，不等ACC迁移完再设计 |
| G. LSP与外语元信息 | 语法outline/query→semantic adapter→interop descriptor逐层验收 | 不把完整多语言语义/FFI包装成语法生成器天然交付 |

各包大小/拆分由正式auto-plan:new取材后确定。上述是架构分解，不是已批准的实施计划、预算或时间表。需要新架构合同，但不先重写全部旧编译器，不要求先完成整个通用产品才继续自举。

## 10. 关键验收反例

- 连续插删、跨CRLF/Unicode/注释/插值/缩进边界：incremental与fresh parse的树/诊断等价；checkpoint恢复完整；保留原文，坐标往返正确。
- 未闭合字符串/括号/声明：当前revision仍可查询其余结构；strict编译拒error/missing；旧AST/过时任务不得覆盖当前结果。
- grammar/scanner/query换版、插件卸载/重载：相关缓存失效；旧树可在lease内读取，释放前旧代码不卸载；坏schema/未知版本/跨revision handle受控拒绝。
- 私有函数体、公开签名、类型布局、inline/const/comptime、插件配置各自变化：只失效正确依赖，结果与clean build一致；未变化查询真实复用。
- runtime重载期间旧调用/任务/回调/析构仍存活、新代初始化/迁移失败：验证原子切换、旧代保留或明确拒绝，不能仅“DLL加载成功”。
- 同一外语声明受不同宏/cfg/target影响：候选meta不得直接获可执行绑定；错误/过期/条件不明结果有显式来源状态。
- 语法歧义、复杂regex、scanner不前进、恶意/超预算输入：明确生成期拒绝/运行期限/恢复策略，不能静默截断成功。性能目标须探针后单列规模、设备和预算，不先许诺“任意语言实时”。

## 11. 外部一手参考（2026-10-15 核对）

- [Tree-sitter定位](https://tree-sitter.github.io/tree-sitter/)：parser generator + incremental parsing library，输出具体语法树并在语法错误下保持可用结果。
- [Grammar DSL](https://tree-sitter.github.io/tree-sitter/creating-parsers/2-the-grammar-dsl.html)：terminal、优先级、冲突、字段等是grammar模型内容，不能只提供正则列表。
- [Advanced parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)：edit/旧tree/结构复用、嵌入语言区间与并发tree生命周期。
- [External scanners](https://tree-sitter.github.io/tree-sitter/creating-parsers/4-external-scanners.html)：缩进/heredoc等需自定义扫描；完整scanner状态序列化/恢复是编辑正确性的组成部分。
- [Queries](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)：CST结构匹配提供结构消费接口，不能代替语言语义分析。

这些是设计参考，未在本仓做性能比较、生成器选型或兼容性验收；本稿不搬入第三方产品依赖。

## 12. 待冻结决定

grammar算法与支持子集、CST内存布局/树复用、统一位置API与边界整数宽度、grammar/scanner/query版本政策、声明ID匹配、cache/query schema、插件可执行边界/隔离、首批两语言和外语meta来源、产品名/仓库归属、性能/资源预算，均由A及对应专项探针产出后评审。产品方向和从起点融入的要求已记录，不用这些未决项推迟接口分层。

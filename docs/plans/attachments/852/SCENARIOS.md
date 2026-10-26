# 补充测试合同：类型位型、Dyn、缓存、热重载与Rust金样认证

状态：designed-not-executed。以下均是M1需求分母，不是已支持API。
已交付入口绑定由T-01冻结；未交付源API登记owner/接口需求/关闭oracle，不阻塞852基础层交付。
源API未冻结者不能用typed fixture代替最终Auto源码验收。
源码草案见src/；稳定ID必须带入正式AutoDown文件和结果报告。

## W：宽标量只验证已支持的数据面

源码候选统一为 `fn identity(x <T>) <T> { return x }`。四路径需要宿主typed参数/结果入口；
不把u64转换成i32、不借JSON浮点数传位型、不通过运算构造NaN。精确常量载入另走真实常量输入面。

| ID | T / 输入 | 不可替代的oracle | 覆盖边界 |
|---|---|---|---|
| W01 | i64 / 8000000000000000 | I64 tag + 64bit原位型 | MIN传参/返回/复制，非i64算术 |
| W02 | u64 / ffffffffffffffff | U64 tag + 64bit全1 | 不经有符号/JSON Number损失 |
| W03 | f64 / 8000000000000000 | F64 tag + 负零bit | +0与-0必须可区分 |
| W04 | f64 / fff80000deadbeef | F64 tag + NaN payload | 不能用浮点==或is_nan代替 |
| W05 | f32 / 7f80ad00 | F32 tag + 固定8hex | 保留NaN payload，不能升为f64 |

## D：Dyn与portable数据（需先冻结共同源码接口）

命令语义沿824已审合同，当前 `pack/is/borrow_as/take_as/freeze/unfreeze` 是操作名，
**不把这些Rust模型函数名冒充Auto源API**。T-01登记所需源码接口、能力owner、支持差异与关闭oracle；
后续能力包冻结真实源码API后，其验证接线补真实src/D*.at。没有这些文件，M1该行保持required gap，不阻塞852底座结项。

| ID | setup / 操作 | 独立预期与区分性反例 |
|---|---|---|
| D01 | pack拥有Item{name:"hello",code:37}；is/borrow；正确take | 名义类型正确；读得5+37=42；原Dyn被消费；载荷只释放一次 |
| D02 | 对D01先take_as另一个类型，再正确take | 错型返回原Dyn；仍可读取hello和37；不能丢值/提前drop |
| D03 | 同宽同字段不同名A/B装入Dyn后取出 | B不匹配A；再取A成功；布局相同不构成类型相同 |
| D04 | bool/int/char/Str四种独立值pack | false/0/字符'0'/文本"0"不猜测互转 |
| D05 | 借用str、闭包、外部handle及深层包含上述类型的List | 递归白名单门拒绝；不能只检查最外层；资源清理可观测 |
| D06 | Item+Str+List+Option/Result的owned树freeze→store→新worker恢复→读取 | 精确类型/字段/分支/文本相同，读得42；第二次freeze内容确定；不是恢复活handle |
| D07 | 真实portable载荷变异并重算外层hash | 内层非法仍拒绝，危险目标读/提交=0；好hash不是信任 |
| D08 | schema/type/layout revision或当前权限改变后恢复 | 受控miss/拒绝并可恢复正确项；不能按旧offset读新值 |
| D09 | 相同内容独立树 vs 共享DAG/循环图 | 独立树正控成功；按824实际冻结规则拒绝共享/循环；解码半图失败回滚 |

## E / C / R：结果之外的观测

| ID | 不变量 | 输入与入口 | 必需观察 |
|---|---|---|---|
| E01 | 命名实参按源码序求值、再映射到形参 | `sub(b: mark_b(), a: mark_a())`，测试能力下分别返回3/10 | 值7且trace字节恰ba；ab或只结果7不够；普通源码不得隐藏依赖此host能力 |
| E02 | 命名type构造保持源码序 | `Pair { right: mark_b(), left: mark_a() }` | 值103且trace ba；不按字段声明序重排 |
| E03 | 静态合法性与执行可达性分离 | B04/B20/B16成组 | 跳过除法成功；执行除法trap；未执行的未知名字仍诊断 |
| C01 | 标量CTE执行、冻结结果与回灌 | 817真实query sum(20,22)→HIR常量→AC/AV消费 | 42；首轮AV调用>0，命中轮AV调用=0；依赖body变化后重算，不仅看调用者源码 |
| C02 | Meta负依赖与权限参与缓存 | HasField缺字段→新增该字段；撤销meta权限 | false→true精确失效；撤权拒绝；不冒称完整runtime反射 |
| R01 | owned作用域/LIFO | T03/R04/R05正常返回、break/continue、分支及临时 | 创建/转移/释放顺序逐事件；每owner恰释放一次；不能只看返回值 |
| R02 | 构造/clone/扩容失败 | L04/L05/V05/K02逐分配点注入失败，配合法正控 | 未提交旧值/源保持；已构造部分回收；错误分类不伪装语言错误 |
| R03 | 活借用/旧buffer访问 | List扩容前拿view，再触发扩容；有效借用正控配对 | 合同规定阶段受控拒绝；危险内存访问=0；不据821的gen字段声称已有运行期校验 |
| R04 | variant分支切换 | V06 Err("old")→Ok(42) | old释放恰一次；不读inactive payload；结果42 |
| R05 | closure创建者退出 | K02创建/返回/调用/drop | 捕获Str仍有效、结果42、env及buffer最后释放；可重复建销 |
| R06 | budget/cancel/trap退出 | 深owned树/集合/闭包的真实执行状态 | 当前profile承诺的释放、截止与错误；不把进程OS回收当逐owner清理 |

## I：增量cache与在线热重载

这些是会话测试，不为每一步复制一整份源码；fixture描述edit与期望，既有runner驱动真实引擎。
AC和AV均须消费相同编辑序列；AutoVM/a2r只承诺其已有的批处理/快照与认证能力，
不要求旧框架新增ACE的增量/HMR机制。

| ID | 序列 / 不变量 | 必需oracle |
|---|---|---|
| I01 | 稳定工程body编辑30个真实修订 | 两端每次正确结果；未改函数parse/resolve/typecheck/lower/codegen/load按适用层=0；对象身份和代际不变 |
| I02 | callee只改body，caller只读签名 | 改callee，不重编caller；返回值随修订改变；读取callee body/内联者须正确失效 |
| I03 | callee签名改变 | 实际依赖的caller失效或明确拒绝；旧成功版仍可用；不要求所有签名编辑局部化 |
| I04 | type字段/layout改变 | 所有实际布局依赖失效；无关函数保持；旧值不按新布局解释 |
| I05 | 改回历史内容 | 内容命中复用历史产物；不复用缓存PASS；实际执行并由父侧重新比较 |
| I06 | fresh、hot、进程重启、跨工程标签 | 同内容结果相同、键正确、恢复重验；缺依赖/错误owner/版本不准入 |
| I07 | AV旧调用暂停→提交新版→新调用→恢复旧调用 | 旧链旧值、新调用新值；最后旧pin释放后回收；failed/cancelled/late候选不覆盖当前 |
| I08 | native进程仍存活→编译编辑→安装→新调用 | 同一个宿主进程真实调用新机器码；旧帧按旧版完成；**重新链接后重启exe不满足** |
| I09 | 布局不兼容/活状态 | 按实际支持策略拒绝替换或明确要求重启；不能隐式重解释；不要求OSR/任意状态迁移 |
| I10 | 512真实不同修订/churn+并发GC | 活root/lease保留；最后引用后真实物理回收；内存/磁盘水位有界，用户哨兵不动 |
| I11 | 单函数编辑周转 | 首次结果后，代表性编辑→父侧正确结果/诊断的连续P95<1000ms；冷构建、首次认证另列 |
| I12 | 测试灵敏度 | 强制全前端/全重装载/只保hash换对象、错oracle、只执行旧版各有负控，必须非绿 |

I08依赖后续真实native loader/入口切换接口；本计划只设计测试与消费正式交付接口，
不偷偷实现完整AJ，也不以I07或818的link/run替I08判绿。接口未交付保持M1缺口。

## G：一次认证，日常快照

| ID | 输入 / 动作 | 必需结果 |
|---|---|---|
| G01 | 实际a2r生成Rust=审阅金样，独立认证编译/运行actual和reference | 两者各自符合固定oracle；证书绑定真实actual与金样的完整文件集/依赖身份 |
| G02 | 日常重跑同一case | 只emit+bytes compare+证书核查；spawn rustc/cargo/目标Rust进程计数=0 |
| G03 | 转译器改变，但Rust输出和上下文不变 | 复用证书，不重编；生成器新身份留在本轮来源报告，不强制纳入证书失效键 |
| G04 | 生成Rust与金样不同 | 快照FAIL；仅生成待审proposal，不自动更新金样、不自动认证通过 |
| G05 | 金样/行为oracle/runtime依赖/target/有效flags/toolchain改变 | 受影响证书STALE或缺失；日常不自动启动编译；独立认证后才可恢复“已认证四路径”状态 |
| G06 | 无证书/只编不运行/仅emit成功/参考源码冒充actual | 不能宣称四路径行为已验证；报告具体缺口而非SKIP=PASS |
| G07 | 篡改输出或oracle、漏一个产物/真实依赖、旧证书重放 | 父侧判定非绿；缺工具、编译错误、panic、timeout均为infra或编译失败，不能满足Auto负例 |
| G08 | 重复认证请求 | 身份未变可显式复用已有有效收据；强制复核另作操作；正常日常路径永不隐式调用认证器 |

证书建议内容：case+固定oracle身份、实际/金样Rust全文件hash、入口包装版本、依赖锁与runtime闭包hash、
独立参考文件hash、Rust工具链/target/flags、实际运行输出/typed结果、认证器与来源身份。精确schema在T-01冻结。
证书可随金样受版本控制；不能只指向可被LRU清掉的本机cache，否则其他检出无法审计其基线。
运行证据和证书可按项目规约保存/复用，不缓存“这轮a2r已经PASS”。

普通a2r快照PASS与“存在有效行为认证”分别记录；聚合四路径PASS必须二者都满足。
语法/类型负例不需要生成可执行Rust金样，但必须由真实a2r阶段拒绝；不能把Rust编译失败当Auto拒绝。

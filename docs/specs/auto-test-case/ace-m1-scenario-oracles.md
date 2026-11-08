# ACE M1 场景 oracle 权威来源（ace-m1-scenario-oracles）

> 状态：delivered（PLAN-908 T-03，2026-11-01；G-DEC-1/3/4 裁定后的 tracked 化）。
> 性质：M1 40 个 scenario 行（binding.tsv `ns=scenario`）的输入、独立 oracle、区分性反例、
> 资源增量、已交付起点与双 owner 的**规范正文**——fresh 树凭本文件 + tracked 语料即可核验，
> 不依赖 `docs/plans/attachments/852/`（该目录已整目录不入库；其 SCENARIOS.md
> `9ca07a74…`/ORACLES.md `01371309…` 仅作历史线索锚，冲突时以本文件与各已交付 Spec 为准）。
> 原则：`原source值/诊断仍由 frozen 输入决定`；历史 oracle 存在 ≠ 新运行通过；
> typed fixture 不替代同源四路径；scenario 行无源程序的，观测通道未交付就是
> `gap:scenario-tooling`（或对应 owner），不冒源码义务。

## 0. 双轴与适用性（沿 2026-10-28 裁定 + 2026-11-01 G-DEC）

- **a2r 双轴**：M1 的 a2r 判据 = 金样/emit 逐字节轴；可编译认证是独立认证线
  （[a2r-golden-certification.md](a2r-golden-certification.md)）。认证缺不否定获准金样轴，
  也不冒认证 PASS。
- **诊断负例不生成可执行金样**（G-DEC-3）：B17–B19/T06/R09/R10 的 a2r 面义务 = 前端
  拒绝（正确语义族），六负例保持 required；未正确拒绝仍是 `gap:a2r-golden` 语义下的
  诊断面缺口，不得 NA/accepted 或拿 rustc 编译失败过门。
- **适用规则**（本表各行"适用"列引用 [ace-m1-applicability.md](ace-m1-applicability.md)
  的规则键）：NA-5 覆盖 C01/C02 avm/a2r 与 G01–G08 ac/av/avm；NA-6 覆盖 I01–I11
  avm/a2r；NA-3 覆盖 I07/I08 ac 与 D01–D09 ac；NA-2 覆盖 D01–D09 avm/a2r。NA 不等于
  能力已交付，只解除 M1 required；重入条件见 applicability 登记。

## 1. W/D/E/C/R/I/G 逐组 oracle

### D：Dyn 与 portable 数据（owner 824；AC 复合面 owner=mn 复合线）

命令语义沿 824 已审合同；`pack/is/borrow_as/take_as/freeze/unfreeze` 是操作名，不是
Auto 源 API。源 API 未冻结 → 不造 `src/D*.at`，typed/probe 证据不冒源码行。

| ID | setup / 操作 | 独立 oracle 与区分性反例 | 起点 / 适用 |
|---|---|---|---|
| D01 | pack 拥有 `Item{name:"hello",code:37}`；is/borrow；正确 take | 读得 5+37=42；名义类型正确；原 Dyn 被消费、载荷只释放一次 | av=`ac-av/dynamic-run-v1` 真实 typed（824 T-11）；ac/avm/a2r 见 NA-3/NA-2 |
| D02 | 对 D01 先错型 take_as 再正确 take | 错型返回原 Dyn；仍可读 hello/37；不丢值/不提前 drop（保源面） | 同上 |
| D03 | 同宽同字段不同名 A/B 装入取出 | B 不匹配 A；再取 A 成功——布局相同 ≠ 类型相同 | 同上 |
| D04 | bool/int/char/Str 四独立值 pack | false/0/'0'/"0" 不猜测互转 | 同上 |
| D05 | 借用 str/闭包/handle/深层包含的 List | 递归白名单门拒绝；不能只查最外层；资源清理可观测 | 同上 |
| D06 | owned 树 freeze→store→新 worker 恢复→读取 | 精确类型/字段/分支/文本相同，读得 42；第二次 freeze 内容确定；不是恢复活 handle | 同上（826 恢复互锁） |
| D07 | 真实 portable 载荷变异 + 重算外层 hash | 内层非法仍拒；危险读/提交=0；好 hash 不是信任（checksum 不救结构） | 同上 |
| D08 | revision/权限改变后恢复 | 受控 miss/拒绝并可恢复正确项；不按旧 offset 读新值 | 同上 |
| D09 | 独立树 vs 共享 DAG/循环 | 独立树正控成功；共享/循环按冻结规则拒绝；解码半图失败回滚 | 同上 |

### E / C：结果之外的观测

| ID | 不变量 | 输入与入口 | 必需观察 | 起点 / 适用 |
|---|---|---|---|---|
| E01 | 命名实参按源码序求值 | `sub(b: mark_b(), a: mark_a())` | 值 7 且 trace 字节恰 `ba`；`ab` 或只 7 不够 | 四路径 `gap:scenario-tooling`（av 有 `av.core.trace-order` 既有 recipe 证据，通道未接 M1 面） |
| E02 | 命名 type 构造保持源码序 | `Pair { right: mark_b(), left: mark_a() }` | 值 103 且 trace `ba`；不按声明序重排 | 同上（mn 复合 owner） |
| E03 | 静态合法性与可达性分离 | B04/B20/B16 成组 | 跳过除法成功；执行除法 trap；未执行分支未知名仍诊断 | 四路径 ok（成组面）；a2r=`diff-no-name-check`（R-E 真缺口 `gap:a2r-resolve`） |

| ID | 不变量 | 输入 | 必需观察 | 起点 / 适用 |
|---|---|---|---|---|
| C01 | 标量 CTE 执行/冻结/回灌 | 817 真实 query `sum(20,22)`→HIR 常量→AC/AV 消费 | 42；首轮 AV 调用>0、命中轮=0；依赖 body 变化后重算 | ac/av ok（817 cte-first-sum42）；avm/a2r=NA-5（CTE 专属主体，原 ACE 查询/回灌入口） |
| C02 | Meta 负依赖与权限 | HasField 缺字段→新增；撤销 meta 权限 | false→true 精确失效；撤权拒绝；不冒完整 runtime 反射 | 同上 |

### R：资源/生命周期事件（scenario 面）

| ID | 不变量 | 输入 | 必需观察 | 起点 / 适用 |
|---|---|---|---|---|
| R01(scn) | owned 作用域 LIFO | T03/R04/R05 正常返回、break/continue、分支及临时 | 创建/转移/释放逐事件；每 owner 恰释放一次；只看返回值不够 | avm=diff-legacy-value；a2r=emit 面；ac/av=ok:P919-events（`resource-events-919`：owned/collections 源→`av.lifecycle/1`/`MN-EVT`，父判冻结不变式；PLAN-919） |
| R02(scn) | 构造/clone/扩容失败注入 | L04/L05/V05/K02 逐分配点注入 + 合法正控 | 未提交旧值/源保持；已构造部分回收；错误分类不伪装语言错误 | av=`gap:av-alloc-fault`（AV 资源故障线）；a2r 快照轴 ok+cert-gap |
| R03(scn) | 活借用/旧 buffer | List 扩容前拿 view 再扩容 + 有效借用正控 | 合同阶段受控拒绝；危险访问=0；821 gen 字段 ≠ 运行期校验 | av=`gap:av-821-gen`；avm=diff；a2r=emit 面；ac=`gap:ac-composite-native` |
| R04(scn) | variant 切换释放 | V06 `Err("old")`→`Ok(42)` | old 释放恰一次；不读 inactive payload；结果 42 | 同 R01 形态；ac/av=ok:P919-events（PLAN-919，同上通道） |
| R05(scn) | closure 创建者退出 | K02 创建/返回/调用/drop | 捕获 Str 有效、结果 42、env/buffer 最后释放；可重复建销 | av=ok:av-closures-823（typed 证据）；avm ok；a2r ok+cert-gap；ac 复合面 |
| R06(scn) | budget/cancel/trap 退出 | 深 owned 树/集合/闭包真实执行态 | 当前 profile 承诺的释放/截止/错误；OS 进程回收 ≠ 逐 owner 清理 | ac/av=ok:P919-events（PLAN-919：av fuel/cancel-flag、mn wrapper 驱动 set_budget/cancel_after_work，typed exit faces）；avm ok；a2r ok+cert-gap |

### I：增量 cache 与在线热重载（会话族；fixture 描述 edit，既有 runner 驱动真实引擎）

| ID | 序列/不变量 | 必需 oracle | 起点 / 适用 |
|---|---|---|---|
| I01 | 稳定 body 编辑 30 修订 | 每次两端正确结果；未改函数 parse/resolve/typecheck/lower/codegen/load=0；对象身份/代际不变 | ac/av=scalar-subset（单源单编辑真实观测）；全谱保缺口；avm/a2r=NA-6 |
| I02 | callee 只改 body | 不重编 caller；返回随修订变；读 body/内联者失效 | 同上 |
| I03 | callee 签名改变 | 实际依赖 caller 失效或明确拒绝；旧成功版可用 | 同上 |
| I04 | type/layout 改变 | 实际布局依赖全失效；无关函数保持；旧值不按新 layout 解释 | ac/av=`gap:scenario-tooling`（823/824 有 env/signature 漂移拒绝证据）；avm/a2r=NA-6 |
| I05 | 改回历史内容 | 内容命中复用历史产物；不复用缓存 PASS；父侧重新比较 | ac/av=scalar-subset（内容寻址命中真实观测）；avm/a2r=NA-6 |
| I06 | fresh/hot/重启/跨工程 | 同内容结果同、键正确、恢复重验；缺依赖/错误 owner 不准入 | ac/av=scalar-subset（四态观测）；跨工程标签/缺依赖态未跑；avm/a2r=NA-6 |
| I07 | AV 旧调用暂停/恢复 | 旧链旧值、新调用新值；旧 pin 释放后回收；failed/cancelled/late 不覆盖当前 | av=ok（823 paused-call 证据）；ac=NA-3（旧批准澄清）；avm/a2r=NA-6 |
| I08 | native 进程仍存活→编译→安装→新调用 | **同一宿主进程**真实调用新机器码；旧帧按旧版完成；重启 exe 不满足 | av=ok:P866-resident（常驻在线热重载落地）；ac=`gap:ac-native-loader`→NA-3；avm/a2r=NA-6 |
| I09 | 布局不兼容/活状态 | 按实际策略拒绝替换或明确要求重启；不隐式重解释；不要求 OSR | ac/av=`gap:scenario-tooling`；avm/a2r=NA-6 |
| I10 | 512 修订 churn + GC | 活 root/lease 保留；最后引用后物理回收；内存/磁盘水位有界；用户哨兵不动 | ac/av=ok（823/824 bounded 证据）；avm/a2r=NA-6 |
| I11 | 单函数编辑周转 | 首次结果后代表性编辑连续 **P90 ≤ 2000ms（858 已批准开发期门，至少 9/10；P95<1000ms 为历史原门、作诊断并列）**；冷构建/首次认证另列 | ac/av=ok（823 r9 + 824 512 修订 P90 双门）；avm/a2r=NA-6 |
| I12 | 测试灵敏度负控 | 强制全前端/全重装/只保 hash 换对象/错 oracle/只执行旧版各有负控且必须非绿 | 四路径 ok（823/824 judge 反控常驻） |

### G：一次认证、日常快照（a2r 认证系统行为；a2r 列 required）

| ID | 输入/动作 | 必需结果 |
|---|---|---|
| G01 | 实际 a2r 生成 Rust=审阅金样；独立认证编译/运行 actual 与 reference | 各自符合固定 oracle；证书绑定真实 actual/金样全文件集与依赖身份（ac/av/avm=NA-5 专属主体排除：认证主体是 a2r；`gap:av-822` 陈旧 owner 随裁定废止） |
| G02 | 日常重跑同一 case | 只 emit+bytes compare+证书核查；rustc/cargo/目标 Rust spawn=0 |
| G03 | 转译器变、Rust 输出不变 | 复用证书不重编；生成器新身份留来源报告 |
| G04 | 生成 Rust ≠ 金样 | 快照 FAIL；仅生成待审 proposal，不自动更新金样/认证 |
| G05 | 金样/oracle/依赖/target/flags/toolchain 变 | 受影响证书 STALE 或缺；日常不自动编译 |
| G06 | 无证书/只编不运行/仅 emit 成功/参考冒充 actual | 不宣称四路径行为已验证；报具体缺口而非 SKIP=PASS |
| G07 | 篡改输出/oracle、漏产物/依赖、旧证书重放 | 父侧判定非绿；缺工具/编译错误/panic/timeout=infra，不满足 Auto 负例 |
| G08 | 重复认证请求 | 身份未变可显式复用有效收据；日常路径永不隐式调用认证器 |

## 2. 资源增量与判据边界（沿 §5.7 预算）

- 逐场景资源增量沿原 TestCase 更严 cap：源/元数据 ≤4MiB、流 ≤1MiB/流；实际所选
  Job/deadline/资源/ignore/平台/样本保留原门（[ace-test-policy.md](ace-test-policy.md)，
  PLAN-893）。W 保原位型（不用 JSON 浮点传位型）；Dyn 坏载荷好外 hash 仍拒；E01 顺序
  `ba`；R 族 LIFO/故障回收/代际危险读 0；I 族未改真实生产=0/对象身份/恢复在线不以重启
  代替；G 族真实 actual/reference 与日常零编译。
- 合法近邻（G-DEC-2）：`NB17/NB18/NB19`（`src/NB*.at` + `nb-diags-908.suite.ad`）证明
  B17–B19 的拒绝是区分性拒绝（近邻必须真实运行得 2/2/0）；错族、缺原因、崩溃与合法
  近邻被拒都判红。

## 3. E01/E02/I04/I09 的真实生产落地（PLAN-914，2026-11-02）

本文件 §1 的语义 oracle **不变**（E01 值 7 + trace `ba`；E02 值 103 + trace `ba`；
I04 依赖精确失效/无关零生产；I09 按实际策略拒绝且旧状态安全）。PLAN-914 补齐的是
「怎么真实产生并观察」：

- E01/E02：ac 走 `ac/mir-native-scalar-v1`（记录源同 recipe）并以 PE 真实 **stderr**
  字节为 trace；av 走 `core-run-v1`/`data-run-v1` 并以 worker 上报的 `step.trace` 为
  trace（`expect.trace` 仅 `ac-av/*` 可声明）；avm 走 legacy 源（`write` 记号）+ 精确
  stdout；a2r 走金样字节 + 显式双侧认证（actual 与 tracked reference 的 stdout 同为
  `bai32:<值>\n`）。冻结输入见 `order-layout-914/src/E01-*.at`、`E02-*.at`。
- I04 ac：collections 增量会话的显式声明失效轮（冷声明前端 + 依赖精确 rebuild，
  `declaration_frontend_invalidations`/`codegen_units`/`identities_*` 由父侧判定）。
- I04 av：data-incremental recipe 的 `rebuilt_defs` + 逐 def code identity 承载；
  使用中记录布局变更按引擎既有签名策略重建全 unit，未引用声明新增与 helper body 编辑
  两格证明零生产精度。
- I09 ac/av：typed refusal（`signature-or-context-drift` / `env-contract`/`link-rejected`）
  是 verdict；拒绝后旧环境/旧快照仍执行。

细节与预算边界见 [m1-order-layout-observations.md](m1-order-layout-observations.md)。

## PLAN-924：R02/R05 oracle 冻结

- R02/R05 scenario ac 的判定源为本归档计划 §5.4 冻结的父表与事件形状：
  exit（K02/K05 base 42）、注入腿（S9/S1 trap 25、S4 trap 22 + INJECT
  事实）、N+1 正控、全零清理、R05 的 own-in-creator-frame /
  creator-leave-with-env-alive / final-sweep-free / live=0。corpus 行
  oracle（9/42 等）不变。
- 判定实现：`ac_mn/native_closures.rs`（父侧从严解码，不信 worker 自述）；
  suite 资产 `native-closures-924/`。坏身份/坏载荷沿既有拒绝面（forge/
  字节翻转 store/open 或链条具名拒）。

## PLAN-997：trap 域与场景域的隔离（2026-11-08，SD-03）

- H-06 四格的 typed trap 事件（stderr `auto.trap:<kind>:<op>` + exit 70）是
  **a2r trap 认证域**的观察通道（schema `ace.m1.a2r-trap/1`，
  `oracle/trap-certs/`），不是本规范的 scenario 观察面：H-07 R01-R06 仍按
  §1/§2 的场景 oracle 认证（998 域），不得以 trap 证书核销场景格，也不得把
  场景套件输出充 trap 证据——两域 reader/路径/计数互斥。
- trap 的类型化原因词（overflow|div-by-zero|rem-overflow）以 binding
  expected 为权威、由声明式注册表 `oracle/trap/trap-oracle.json`
  （`ace.m1.a2r-trap-oracle/1`）逐格声明；该注册表 hash 是每张 trap 证书的
  身份轴。场景 oracle 文档（本文件）不新增 trap 语义；资源预算/流帽沿认证
  器既有冻结（120s/30s/1MiB），场景更严预算不因 trap 域放宽。

## PLAN-998：H-07 六场景格的 a2r 资源观察认证域（2026-11-08，SD-01）

- R01-R06 六格已由 **a2r resource-scenario 认证域**真实关闭（token
  `ok-emit-scene`；schema `ace.m1.a2r-resource-scenario/1`；证书
  `oracle/scenario-certs/`；声明式注册表 `oracle/scenario/scenario-oracle.json`
  = `ace.m1.a2r-resource-oracle/1`，其 hash 为每张证书身份轴）。§1 各行
  文字 oracle（输入/不变量/区分性反例）不变；原 snapshot 注记由此废除：
  R05 的历史 "快照" 从无 `m1_k02.rs` 工件；R02 的域是五腿全扫（L04/L05/
  V05/LSTR/K02），非 V05 单金样；R06 的源锚为 998 冻结 fixture（deep/trap）。
- 观察通道：显式 profile `legacy-a2r.resources-observe/1`（默认字节恒等），
  在**生成产物的真实拥有路径**上插桩——逐绑定 RAII guard 观测真实 Drop
  （逐帧 LIFO 为 Rust 自身 drop 序）、`.move` 位转移解除（转移≠释放）、
  enum 代际 replace 观测旧 payload、参数先求值的结构变异包装（增长点 fault
  检查）、字面量/clone chk 哨兵、静默计数分配器（只计数；fault 在安全 chk
  点交付）、budget/cancel 步进、闭包 env 转移。typed 失败（fault/budget/
  cancel/stale）经 panic 载荷 unwind 过真实 Drop（清理被观测而非断言），
  wrapper 只接精确载荷，exit 71；组合 checked-i32 的 trap 腿 exit 70。
- 独立性：每腿配手写独立 reference（自有观测机制、原子分配探针域、与
  emission 零代码共享），双侧各自真实 allocation 域扫 fault（1..N + N+1
  对照 + 同进程 retry），事件词汇表共享而事件流不要求逐字相同。
- R03 边界（如实）：合法近邻真实运行（buffer_reads 只含合法读）；
  R03-static 的旧前端门**不**静态拒迭代中变异——发射产物由 rustc 在编译期
  拒绝（E0308，编译期边界，非 typed 语言负例）；typed stale 面由 reference
  驱动（mint→合法读→结构变异→stale 使用，typed 拒先于任何 buffer 读）。
- K04（corpus 负例）通道迁移：T-06 为 K02 补 fn-type 返回位源入口后，
  parse 级冻结通道（auto_syntax_E0001）结构上不可保留；K04 现于设计通道
  拒绝——a2r `type/auto_type_E0101`（borrow-capturing 闭包不得逃逸创建帧）、
  avm `run/RuntimeError`；负例本身保留。
- 域互斥：value（ace.m1.cert/1）/ trap（ace.m1.a2r-trap/1）/ scene
  （ace.m1.a2r-resource-scenario/1）三域 reader/路径/计数互斥；日常零
  spawn 面为 suite `legacy-a2r-resources-998`（12 checks emit+bytes）。

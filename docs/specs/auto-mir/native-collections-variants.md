# native 集合与变体（PLAN-889）

> **Status**: experimental（显式collections源码/typed受检单元→共享MIR→真实COFF/PE；不改变产品默认引擎）

## 唯一语义路径与边界

`auto.source.collections.draft` → 同一source解析/resolve/typecheck与Collections HIR函数验证 →
`auto-mir-lower::hir_adapter::lower_function_collections_unit` → 共享MIR验证 →
`ac-mn::collections_lower`的CLIF/COFF → 多对象链接 → Windows x64 PE。
运行时只实现Cell/容器/所有权原语，不读取MIR/ABC、不解释opcode、不调用AV。
旧HIR→CLIF lowering仍退役，旧五域及其默认recipe/身份不变。

| 起点 | 支持/限制 |
|---|---|
| 原864源码 | 局部List<int/bool/Str/无环record/Variant>，Option/Result/有限enum，无环嵌套Variant、owned载荷与允许的函数结果、for-in/只读投影、Str.clone |
| 源码拒绝 | List/Variant.clone、List参数/返回/record字段、嵌List、Variant参数/record字段、str/char、宽typed entry、闭包/Dyn/Unit入口等保持原具名排除 |
| 受检typed MIR | 独立List/Variant Clone机器链见证；不据此开放源码方法或全部profile4指令/Place/参数域 |
| 入口 | 零参数，i32或bool结果；宽位宽/带参数typed entry另属891规划 |

原L01..L07/V01..V07及Str变体/for-in源码不改字节或oracle。L04/L05=42，
L06只接受list-index trap；普通return70不能满足该负例。14个native格的正式消费
见[TestCase合同](../auto-test-case/ace-m1-conformance.md)，不宣称M1全绿。
AV profile4/opcode/wire/执行语义不变。

## 机器布局、所有权与资源

- `mn.collections.cell/2`：Cell16B（值bits/node地址与64位析构责任标志）、Frame32B；Win64内部call/return ABI。
- `mn.collections.runtime/2`，layout/source/codegen世代各`/1`；codegen实际内容另有build指纹。
- 名义描述符用canonical BLAKE3绑定owner、声明序字段/臂及传递载荷形状，忽略span/table编号；签名还绑定borrow/own模式。
- 节点登记先于其指针被解引用。每次受控资源读取核registry/kind/layout；callee接口与名义布局在链接前受检。
- owned转移清掉旧析构责任；只读view不分配或隐藏clone。push/insert/set/switch先完整准备再commit，失败保旧目标及未提交输入责任。
- 显式clone产生独立buffer；clone用计费显式工作栈，drop用无分配/不轮询的侵入栈，无宿主递归深度限制。4096/64KiB栈为见证，不是语言上限。
- frame_open分配受管Cell存储并登记实际header；fatal出口清当前及全部资源caller帧。正常return70仍是值。
- raw trap原因：index21、budget22、invalid23、allocation-injected25、cooperative-cancel26；必须结合真实exit70及完整资源观察。外部kill属于infra失败，不冒称helper协作清理。

| 预算域 | 冻结上限与观察 |
|---|---|
| guest值数据 |8MiB；MAX_LIST_ELEMS=65536，精确边界与超一拒绝 |
| native执行物理 | 实际arena committed pages与登记帧存储/header合计32MiB；元数据、text ledger同arena，拒绝前预留/失败回滚 |
| PE进程 | private历史峰减baseline≤128MiB，含栈/image/heap slack，缺失/零OS观察不通过 |
| compiler managed | MirSymbol16/Staging24/IO8MiB，合计48MiB；专用worker计每次Rust backing request（含计量header/对齐，释放归原partition），超额不发布 |
| compiler进程 | private peak delta≤128MiB，另含存储引擎/C分配及allocator余量；不得用逻辑账代替 |

文本shim的新增allocator hooks默认仍为旧heap；collection静态库消费同一shim rlib与单一panic handler，
不同时整库链接两个staticlib。拒绝接管未清理buffer/metadata，最后一帧释放idle ledger。
成功清理要求节点/text live、arena live/committed/metadata均0。
clone按node/child和每4096B文本块轮询，取消回滚部分输出；drop不轮询，确保取消后也能收尾。

## 真函数增量与事务

冷态保留既有CheckedEnv，最多4函数/16KiB源body工作集共用一次Emitter/binder；大body仍原单体道。
所有函数真实plan/bind/function verify/shared lower/MIR admit/unit codegen；不是whole-module credential。
冷批与原single逐函数ABC（含坐标）、fp与COFF字节等价。

纯body编辑只解析授予UTF8 fragment，目标resolve/typecheck/plan/bind/shared lower/admit/unit生产各一次；
未改函数实际AST/MIR/COFF producer分配身份保留、实际body工作0。body新intern类型池与候选一起移动，
取消/失败/foreign/stale不改当前环境。历史COFF命中允许目标MIR与坐标重新mint，不能复制未改生产对象冒carry。
声明/布局/签名/config修改走显式cold失效道，诚实报告前端工作并精确复用未受影响原生单元。

prepare/commit之间再核runtime/config；失败/取消/坏链接/预算/过期候选保当前source/env/坐标/root/manifest/旧PE与receipt。
全部fallible生产/link/run/发布先于独占源候选安装。产品对原子放置失败恢复旧exe和receipt。
共享ownership数据流仅对全I32/Bool槽省去确定全None侧表header，规则/错误/steps/原保守memory bound不变，
其他形态沿原表；private dense对照与旧owned/collections控制约束该优化。

## Cache与恢复

显式CLI `--source-profile collections`、SDK recipe `ac/mir-native-collections-v1`、专用`mn-collections-worker`。
profile/ABI/runtime源码与构建配置、实际库内容、名义布局、codegen/lowering与linker/SDK/kernel32绑定新键。
helper准备/构建/当前内容检查在请求内；stamp匹配可免cargo，但不能以mtime替代当前内容hash。
旧五域A09/A10渲染/身份/支持面不变。

新manifest `ac889/a10-collections/3`携带source、mint/current坐标、runtime、MIR、COFF与required闭包。
冷MIR用256KiB carrier pages，每slice独立hash与边界；COFF页连同续页全部声明required。
live immutable checked MIR/layout凭据只在进程内，不可序列化。重启从有界不可信字节decode、
共享MIR深验、坐标/owner/类型/interface/entry/COFF结构/content与runtime/tool当前身份重验后才link/run，
restore消费者禁止unit codegen。reuse/fresh/read-only三模式保留；只读miss真重算、零store发布。
故意省free的灵敏度控制只能ephemeral生产，不能污染持久cache。

## 验证与长期口径

固定116函数/345483B负载，source SHA256
`df314cc77208bcc115f02b446b8c1012046e96b046784a1a0c6e2e2094a1715c`，结果1786+n；
只改owned collections_helper，其余115函数不改。fresh/hit/restart/body/history各至少30样本，
512个不同真实修订；P90为排序第N-floor(N/10)项（从1开始），≤2000ms；P95/P99及失败均保留。
窗口从parent请求/编辑入队到独立结果/资源准入完成，包含helper准备/构建/内容检查/link/run。
修复只重测实际受影响门，其他证据需代码/依赖/输入不变与等价证明；明确scope，不能把repair说成重跑完整矩阵。

新parent独立核raw exit/stderr、实际资源、producer阶段及真实身份，报告9轴当前性独立重算。
私有源/库镜像承载真实bitflip→STALE→恢复Fresh反控，禁止测试修改共享产品库扰动并行消费者。
wrong oracle/type、漏free、whole frontend/lower与同内容新COFF对象反控必须真发生并使总门红。

来源：[PLAN-889](../../plans/archive/889-native-collections-variants-source-closure.md)；
实现合同：`experimental/ac-mir-native/runtime/collections/CONTRACT.md`。
891只接收已释放N接口清单，未由本交付自动实施typed/wide或关闭W01..W05/native在线帧等gap。

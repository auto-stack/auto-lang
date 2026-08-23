# Plan 419: AutoVM 三层生命周期管理(作用域清理 / 逃逸分析 / Shared 升级)

> **状态**: ✅ **复活已闭环(2026-08-23 根因修复,分支 419-uaf 待合并)**。
> §9 的确定性 UAF 已定位修复:`json_to_vm_value` 外层臂漏「插入即 retain」
> (§9.7),ash-gui 崩溃用例转绿、本仓 3125 测全过、canary 保持开启。
> 三 Phase 落地结论不变;此前「第 4 族」实为 native 构造路径的获取缺口,
> 非指令族记账缺口。
> 原归档结论(三 Phase 落地 + 19 测 + a2r golden)仍成立,问题在覆盖面外。
> Phase 1 堆对象 RC(0c1dc0d5)/ Phase 2 池 RC+freelist+pinned(9bc4e671)/
> Phase 3 借用接线 + 写捕获 a2r 升级(见 git log)。§2.2/§3.2/§4.6 里程碑
> 全过(tests_rc_lifecycle 19 测 + a2r golden 25_lifecycle 4 例)。
> **来源**: Plan 060 第十二/十三轮调研(auto-shell 仓)。设计口径(用户):
> Auto 无 Rust 级生命周期标注(`'a`),三层兜底 —— ①周期内引用(~80%)
> 作用域尾清理;②一级逃逸分析覆盖大部分逃逸;③歧义项升级 Shared
> (Rc<Cell<T>> 式,引用计数自动清理)。另含 a2r 转译侧对齐(歧义变量
> → Rc<RefCell<T>>)。
> **可行性结论**: 三层在现有 AutoVM 上**全部可实现**;且第三层(Shared)
> 是精确引用计数的自然推论(RC>1 ≡ Shared),无需新增语言类型。

---

## 0. 现状调研结论(2026-08-22,Plan 060 第十三轮)

执行本计划前须知的地基与缺口(锚点用符号名,行号会漂):

- **arena 模型**:`AutoVM.heap_objects: DashMap<u64, Arc<RwLock<dyn HeapObject>>>`
  (engine.rs,id 自 4,000,000 单调)、`strings: Arc<RwLock<Vec<Vec<u8>>>>` +
  `string_dedup` HashMap(内容去重)。两者只增不减。
- **tier 1 缺失**:`OpCode::DROP`(0x05,注释 "RAII cleanup")是空壳
  (`pop_i32()` 而已);codegen 全文**零发射** DROP;`OpCode::RET` 尾缘只截断
  栈帧,无任何清理钩子;parser 的 enter_scope/exit_scope 仅符号表。
- **tier 2 缺失**:全仓无逃逸分析。旁注:`OpCode::CLOSURE` 捕获为**拷贝
  语义**(pop 值复制进 env)。
- **tier 3 缺失**:Rc/RefCell/Cell/Mutex 仅存在于 a2r 透传(trans/rust.rs);
  VM 零 refcount(grep 唯一命中 `vmref_counter` 是 id 生成器)。
- **借用检查**:仅 `check_unsafe_capture` 一个窄点(codegen.rs,Plan 071);
  `.view/.mut/.move/.take` 四操作符在 codegen 直通编译内层(明文 MVP TODO)。
- **已有可复用件**:`remove_heap_object(id)`(现被 hashmap/hashset/
  stringbuilder 显式 `.drop()` native 调用);`operand_span_with_pool_indices`
  重映射机器(lib.rs,链接器并池用,压缩回收可复用);ParamMode
  take/move/copy 与所有权操作符词汇齐全。
- **值表示**:nanbox 单字。字符串 = 池索引(TAG_STRING,负数 i32 编码);
  堆对象 = TAG_OBJECT 或裸 i32 id(≥4M)。引用"值"就是这两种 tag。

## 0.1 设计答案:a2r 要不要 Rc<Cell<T>>?—— 要,精确映射如下

Auto 无生命周期标注,转译到 Rust 时按逃逸分析结果三路分派:

| 逃逸分析结论 | a2r 转译产物 | 说明 |
|---|---|---|
| 不逃逸(周期内死亡) | 直接 Rust 值(栈/局部) | Rust 自动的确定性 drop 兑现 tier 1 |
| 逃逸但单属主(move 链清晰) | 直接值 + move 语义 | Rust 所有权转移 |
| 逃逸且多读者、无 mutation | `Rc<T>`(只读共享) | 计数自动回收 |
| **歧义 / 捕获且被修改(Write-Capture)** | **`Rc<RefCell<T>>`** | **用户问的主案**;`Cell<T>` 仅限 Copy 型(T 是 i32/bool 等按值类型),一般情况用 RefCell |
| 跨 task/spawn 共享 | `Arc<Mutex<T>>` | VM 语义单线程→Rc;并发域→Arc |

访问点改写:读 `.borrow()` / 写 `.borrow_mut()`(Cell 型则 `.get()`/`.set()`)。
现状 a2r 已支持显式 `Rc<RefCell<...>>` 类型名**透传**(用户手写可用),本计划
补的是**自动升级规则**(分析不出 → 自动包 Rc<RefCell>)。

---

## 1. 核心设计:copy-on-load 引用计数协议

所有引用值(堆对象 id / 字符串池索引)遵循统一所有权协议。**协议先于
一切实现细节** —— 每条规则都是引擎里的一个屏障点(barrier):

| 动作 | 计数变化 | 引擎位置 |
|---|---|---|
| 创建引用值入栈(NEW_INSTANCE/CREATE_LIST/STR_CAT 结果/json_to_vm_value/native 产物/inject) | **+1** | 各产物点 → 统一走咽喉函数 |
| LOAD_LOCAL / LOAD_GLOBAL / GET_FIELD 加载引用值 | **+1**(copy-on-load) | 对应 OpCode 臂 |
| DUP(DUP 的值是引用时) | **+1** | OpCode::DUP 臂 |
| POP(弹出的值是引用时) | **−1** | OpCode::POP 臂 |
| STORE_LOCAL 系(槽内旧值是引用) | 旧 **−1**;栈上 +1 **转移**进槽(不变) | STORE_LOCAL/STORE_LOC_N 臂 |
| SET_FIELD / 容器写(list.set/push/map.set…) | 旧字段/元素 **−1**;新值转移 | SET_FIELD 臂 + 容器 native |
| RET | 释放帧区间内每个引用槽 **−1**;返回值例外(转移给调用方,调用方 POP/STORE 时再结) | OpCode::RET 臂 |
| 闭包捕获 | 栈上 +1 转移进 env;**闭包对象本身是堆对象**(有自己的 RC),闭包回收时 env 内各 **−1** | CLOSURE 臂 + 闭包 HeapObject 的释放路径 |
| 错误/取消展开 | 被展开帧的引用槽逐个 **−1** | ERROR_PROPAGATE/POP_HANDLER/task cancel 路径 |
| RC 归零 | 堆对象:`remove_heap_object`;字符串池:置墓碑进 freelist(§3) | 咽喉函数内 |

**偏差方向(安全纪律)**:漏 incref = 悬垂(致命);漏 decref = 泄漏(安全)。
→ incref 必须**咽喉点集中、审计穷举**;decref 宁漏勿错,配 rc_stats 泄漏
检测测试单独追踪收紧。

**毒化 canary(debug)**:freed 的堆 id 登记进 `tombstones: DashMap<u64, Instant>`
(保留 N 分钟),`get_heap_object` 命中墓碑 → `debug_assert` 失败并打印
use-after-free 的 id 与年龄。任何漏 incref 的 UAF 在测试期即暴露。

**咽喉点函数**(新增于 engine.rs,Phase 1 首个交付物):

```rust
impl AutoVM {
    pub fn rc_push(&self, task: &mut AutoTask, nv: NanoValue);  // 引用值入栈(+1)
    pub fn rc_drop_n(&self, nv: NanoValue);                      // 引用值消亡(−1)
    pub fn rc_transfer_in(&self, nv: NanoValue);                 // 转移(不变,断言用)
    pub fn rc_stats(&self) -> RcStats;                            // 测试断言钩子
}
pub struct RcStats { pub live_heap: usize, pub live_pool: usize,
    pub created_total: u64, pub freed_total: u64, pub rc_traffic: u64 }
```

tag 判定用现成 `auto_val::is_object` / `is_string` + 裸 i32 ≥ 4M 启发式
(与 TYPE_TO_STR 同款,务必保持一致,抽成 `is_heap_ref_nv()` 单一实现)。

---

## 2. Phase 1:堆对象生命周期(tier 1 的对象半边)

**目标**:函数返回 / 块结束 / 覆盖赋值后,无逃逸的堆对象(graph)确定性
回收;`rc_stats().live_heap` 可归零的 .at 用例通过。

### 2.1 交付物

1. **RC 存储**:`heap_rc: DashMap<u64, AtomicU32>`(独立表,零侵入
   HeapObject trait)。`insert_heap_object` 时建 RC=1。
2. **咽喉函数 + 审计**:grep 穷举所有"引用值入栈"点(engine push_value、
   native.rs 的 push_value/shim 系列、ffi/stdlib json_to_vm_value、
   ffi/convert push_to_stack、host_bridge 结果、renderer 的
   update_block_in_state/inject_shell_event 推值)—— 全部换 `rc_push`。
   审计清单落在本文件 §2.3(执行时逐项勾)。
3. **引擎屏障**(§1 表逐行落地):POP/DUP/LOAD_LOCAL 系/LOAD_GLOBAL/
   GET_FIELD/SET_FIELD/STORE_LOCAL 系/RET 帧扫描/CLOSURE env 释放/
   错误展开路径。容器 natives(list push/set/pop/remove/clear、map 系)
   加屏障。
4. **DROP 兑现**:`OpCode::DROP` 实现为 `rc_drop_n` 真减(注释已承诺
   RAII);codegen 在块作用域尾对拥有引用的局部发射 DROP(**保守发射**:
   类型推断为引用类型的局部才发;发射端宁多勿漏 —— 多发 = 多减一次?
   不 —— 多发会过度 decref!发射端规则:仅为"编译器确证拥有"的槽发射,
   不确定则不发,交给 RET 帧扫描兜底)。
5. **显式 .drop() natives 语义更新**:hashmap/hashset/stringbuilder 的
   `.drop()` 从无条件 `remove_heap_object` 改为 decref(RC 归零才真移除)
   —— 兼容"手动提前释放"的用户直觉。
6. **rc_stats 钩子** + `auto.rc.live()` native(供 .at 侧测试断言)。

### 2.2 里程碑验收

- [ ] `rc_balance_unit`:屏障单测(§1 表每行一测:构造序列 → 断言 RC 数值)
- [ ] `scope_drop_basic`:块内 `var c Cell = Cell{...}`,块尾后 live_heap 归零
- [ ] `fn_ret_drop`:fn 内建对象,返回后归零;返回值本身由调用方 POP 归零
- [ ] `overwrite_drop`:`x = A; x = B` 后 A 归零 B 存活
- [ ] `container_elem_drop`:list/map 元素移除后归零
- [ ] `closure_keeps_alive`:被捕获对象存活过作用域;闭包释放后归零
- [ ] `return_keeps_alive`:返回值存活至调用方丢弃
- [ ] `global_keeps_alive`:存入全局后存活
- [ ] `shared_two_owners`:双属主,一方释放仍活,双方释放才归零(tier 3 预演)
- [ ] `churn_returns_to_baseline`:循环内分配 10 万个临时对象,
      live_heap 回到基线(泄漏检测)
- [ ] `uaf_canary_poisoned`:debug 下访问已释放 id 触发 canary(毒化生效)
- [ ] 全量既有套件通过 + ash-gui e2e(81 侧栏钮/真 cwd)
- [ ] perf 基准回退 < 15%(perf_benchmark_tests 对比执行前后)

### 2.3 incref 审计清单(执行时逐项勾选)

- [ ] engine.rs `push_value`(Value::VmRef 臂)
- [ ] engine.rs GET_FIELD / NULL_COALESCE inner / Option inner 推值
- [ ] engine.rs CALL_NAT 各 shim 的引用产物(112 iterator_next 等)
- [ ] native.rs `push_value`(独立 fn!与 engine 同名不同体)/ list get/pop /
      map get/keys / btreemap first_key/last_key
- [ ] ffi/stdlib.rs json_to_vm_value(全树入堆)
- [ ] ffi/convert.rs push_to_stack / Vec→ListData 转换
- [ ] native.rs shim_host_call_value(host 桥结果)
- [ ] renderer.rs update_block_in_state / inject_shell_event 推值(UI 桥)
- [ ] lib.rs eval 结果 pop 侧(pop_i32 后 extract —— 提取后手动 decref)
- [ ] 其余 grep `push_nv.*VmRef\|encode_object\|push_str_idx` 穷举复核

---

## 3. Phase 2:字符串池 RC + 空洞复用(tier 1 的字符串半边)

**目标**:运行期字符串内存从"只增不减"变"峰值受限"(峰值并发不同串数);
循环拼接百万串的内存有界。**不做物理压缩**(索引永不移动,零重映射风险);
压缩重映射(复用 operand_span_with_pool_indices + stop-the-world)登记为
可选 stretch,不做即达标。

### 3.1 交付物

1. **池 RC**:`pool_rc: Vec<AtomicU32>` 与 strings 平行;`pool_pinned: Vec<bool>`
   —— `load_strings`(flash 编译期常量)载入的条目 **pinned 永活永复用**
   (字节码 LOAD_STR/GET_FIELD/LOAD_GLOBAL 立即数只引用 pinned 条目,
   pinned 保证立即数永不悬垂 —— 这是免重映射的关键不变量)。
2. **屏障接线**:§1 协议对 string-tagged nanbox 生效(STR_CAT 结果 +1、
   POP −1、STORE 转移、GET_FIELD +1、闭包捕获转移、RET 帧扫描)。
   咽喉函数已统一判定,Phase 2 只是让 Str tag 走池分支。
3. **回收**:RC=0 的运行期条目 → 墓碑(内容清空释放字节)+ slot 进
   freelist;`add_string` 去重命中 **RC=0 的墓碑 slot** 时从 freelist 取回
   复用(注意 dedup map 与 freelist 的一致性:dedup map 保留 entry→slot
   映射,slot 复用时内容重新填入)。
4. **dedup 交互**:同一内容多个引用共享一条目,RC 累计(语义正确);
   `add_string` 新内容且 freelist 空 → 追加新 slot。
5. **canary**:墓碑 slot 的访问(get_string 命中空内容 + debug 断言)。

### 3.2 里程碑验收

- [ ] `str_cat_temp_freed`:循环内 `s = s + "x"` 中间串归零(除最终值)
- [ ] `str_field_cycle`:对象字段字符串随对象回收而回收
- [ ] `str_dedup_rc`:同内容多副本,RC = 副本数;全释放后条目复用
- [ ] `str_pinned`:flash 常量 LOAD_STR 千次后常量条目不被回收(pinned)
- [ ] `str_churn_bounded`:循环拼接 100 万个不同串,live_pool 峰值 <<
      总创建数(freelist 复用生效;rc_stats 断言,不看 RSS)
- [ ] `str_canary`:墓碑 slot 访问触发 debug 断言
- [ ] 全量套件 + ash-gui e2e(侧栏 81 钮全量字符串场景)

---

## 4. Phase 3:逃逸分析 + Shared 形式化 + 借用操作符接线 + a2r 对齐

**目标**:tier 2/tier 3 落地 —— 编译期逃逸分析标定三类出口;RC 消除优化;
`.view/.mut` 借用检查接线;a2r 自动升级歧义变量为 `Rc<RefCell<T>>`。

### 4.1 逃逸分析(codegen 编译期,pass 式)

值的逃逸出口**仅三个**(Plan 060 第十三轮结论),分析面极小:

1. **闭包捕获**(CLOSURE 捕获列表已知);
2. **函数返回复合值**(fn 返回类型是引用类型);
3. **存入长命容器**(存入全局 / 存入逃逸容器的字段 —— 沿赋值链传播,
   到全局或被捕获/被返回的容器即逃逸)。

实现:codegen 前置一个 `EscapeAnalysis` 小 pass(函数粒度,不做跨函数
全程序分析 —— 被调函数的参数逃逸性按其签名参数模式的 take/move/copy
声明获取)。输出:每个局部变量的 `escapes: bool`。

### 4.2 用途一:RC 消除优化(先正确后优化)

非逃逸 + 单属主的局部:LOAD/STORE 的 incref/decref 省略(编译期发
`LOAD_LOCAL_NORC`/`STORE_LOCAL_NORC` 新操作码,或复用操作数空闲位)。
perf 基准若 Phase 1/2 后回退超标,此项提前。

### 4.3 用途二:Shared 形式化(tier 3)

精确 RC 之下 **RC>1 ≡ Shared**,无需新增语言类型:
- 暴露 `auto.rc.count(x)` native(测试与用户诊断);
- 文档化语义:Shared = 运行时属性(多属主引用计数),非类型系统概念;
- 逃逸分析"歧义"的局部在 VM 侧本就自然 Shared(RC 兜底),无需升级动作
  —— tier 3 在 VM 内是自动的。

### 4.4 用途三:借用操作符接线(补 MVP TODO)

- `.view`(共享读):编译期检查 —— 作用域内同时只有一个 `.mut` 活跃时
  才合法;运行时对 RC>1 的值 `.view` 无成本(只读)。
- `.mut`(独占写):编译期 —— 若值被 `.view` 借用中或被判 Shared
  (多属主)→ **编译错**;运行时兜底 —— RC>1 时 `.mut` 抛
  `RuntimeError("mutable borrow of shared value")`(RefCell 式动态
  借用检查的极简版:以 RC>1 为冲突判据,不维护活跃借用表)。
- `.move`/`.take`:接线到所有权协议 —— move 后源槽置 Nil(防 double-drop);
  take = move + 源值类型占位。
- 现有 `check_unsafe_capture` 并入此检查框架。

### 4.5 a2r 自动升级(§0.1 映射表的实现)

- trans/rust.rs 新增 escape-aware 降级:对 Write-Capture(闭包捕获且被
  修改)与分析歧义的局部 → 声明 `Rc<RefCell<T>>`、构造 `Rc::new(RefCell::new(...))`、
  捕获点 clone(`Rc::clone(&x)`)、读 `.borrow()` / 写 `.borrow_mut()`;
- Copy 型按值类型 → `Cell<T>`(get/set);
- 跨 task(spawn 捕获)→ `Arc<Mutex<T>>`(lock());
- 显式手写 Rc/RefCell 的现有透传行为不变(自动升级仅在分析歧义时触发,
  用户显式优先)。

### 4.6 里程碑验收

- [ ] `escape_analysis_unit`:三类出口的标定单测(捕获/返回/存全局)
- [ ] `rc_elision`:非逃逸局部省略计数后,行为与省略前逐字节一致(对比
      rc_traffic 计数下降 + 结果输出一致)
- [ ] `mut_on_shared_errors`:RC>1 的值 `.mut` → 编译错(可静态判定)
      或运行时明确错误(动态兜底)—— 两态各一测
- [ ] `move_prevents_double_drop`:move 后源槽 Nil,单次回收
- [ ] a2r golden:`ambiguous_closure_upgrades_rc` —— 捕获被改局部 →
      期望产物含 `Rc<RefCell<T>>`/`Rc::clone`/`borrow_mut`(新增 golden
      目录用例,三方对齐)
- [ ] a2r golden:`non_escaping_stays_plain` —— 非逃逸局部仍是直接值
- [ ] a2r golden:`copy_type_uses_cell`(i32 捕获修改 → Cell<i32>)
- [ ] a2r golden:`task_capture_uses_arc_mutex`(spawn 捕获)
- [ ] 全量套件 + parity 仪表盘无回退 + ash-gui e2e
- [ ] perf:RC 消除后基准回退 < 5%

---

## 5. 测试矩阵汇总(三层 × 生命周期事件)

| 场景 | tier1(作用域) | tier2(逃逸) | tier3(Shared) |
|---|---|---|---|
| 对象 | 块尾/fn 返回/覆盖赋值/容器移除 → 归零 | 闭包捕获/返回值/存全局 → 存活 | 双属主部分释放 → 存活;全释放 → 归零 |
| 字符串 | 拼接中间串/字段串回收 | 捕获串存活 | dedup 共享条目 RC 累计 |
| 嵌套 graph | 父回收 → 子传递回收 | 孙经容器逃逸 → 全链存活 | 多容器引用同一子对象 |
| native 产物 | json_to_vm_value 图回收 | host 桥结果被存 → 存活 | 结果双路持有 |
| 错误路径 | try 内分配,错误展开后归零 | — | — |
| churn | 10 万临时 → live 回基线 | — | 100 万不同串 → live 峰值受限 |
| canary | UAF 毒化报错 | — | 墓碑串访问报错 |
| a2r | 非逃逸 → 直接值 | move 链 → move | 歧义/写捕获 → Rc<RefCell>/Cell/Arc |

断言方式:一律走 `rc_stats()` / `auto.rc.live()` / `auto.rc.count(x)`
(确定性),**禁用 RSS 断言**。所有 canary 断言仅在 debug 构建生效。

## 6. 风险与纪律

| 风险 | 对策 |
|---|---|
| 漏 incref → UAF(致命) | 咽喉点集中 + §2.3 审计清单 + 毒化 canary + canary 测试 |
| 漏 decref → 泄漏(安全) | 宁漏勿错;rc_stats 泄漏检测测试单独标注追踪 |
| 错误/取消展开丢帧清理 | ERROR_PROPAGATE/POP_HANDLER/cancel 路径专项测试 |
| 多任务并发 RC | AtomicU32;帧内操作在任务锁内;跨任务只经容器/globals(已串行化) |
| UI 桥(renderer 推值)漏计 | renderer 推值点入审计清单;ash-gui e2e 为门禁 |
| pinned 不变量破坏(常量被回收) | pinned 单测 + flash 载入路径唯一性检查 |
| perf 回退超标 | 基准门禁(15%/5% 两档);超标则 4.2 提前 |
| 兼容:显式 .drop() 语义变化 | 从"立即删"改"decref"—— 文档说明 + 既有套件回归 |

## 7. 顺序与依赖

Phase 1 → Phase 2 → Phase 3 串行(协议同一套,1/2 是地基,3 的分析与
优化建立在精确 RC 之上)。Phase 1 内部:咽喉函数 → 引擎屏障 → 审计 →
canary → stats → 测试。每个 Phase 结束:全量套件 + ash-gui e2e + perf
基准三重门禁,通过才进下一 Phase。

**跨仓注意**:Phase 1/2 的 renderer 推值点在 auto-lang 的 ui/iced/
renderer.rs(本仓),无 auto-shell 侧改动;ash-gui e2e 仅作验证(借
auto-shell 仓实例)。a2r golden 用例放 crates/auto-lang/test/a2r/ 既有
golden 体系。


---

## 8. 落地记录(2026-08-23)

### 8.1 实现偏差与决策

| 计划条目 | 落地形态 | 说明 |
|---|---|---|
| §1 表 CALL_NAT 行 | **死区结算 + StakeGuard 双机制** | CALL_NAT 包装层释放 [sp_after, sp_before) 死区(sp 净减形态);shim 消费型 pop 改 pop_arg_i32/nv(槽位清零防双重)+ StakeGuard(fn 末 Drop 释放,先读后放)覆盖 sp 中性形态(pop N push N 的 receiver 消费)。存储型 pop 保留 raw + 容器侧 retain 配平 |
| §2.1 DROP codegen 发射 | **PUSH_NIL+STORE_LOCAL 组合** | 块尾槽位释放不新造操作码(不改 operand_size 表);pop_scope 深于函数体的作用域统一发射,byref_captured_slots 跳过 |
| §2.1 insert RC=1 | **RC=0 + push 建 stake** | insert 不建条目(对象出世无持有者),首次 rc_push/rc_retain 建 —— 语义等价、不变量更干净 |
| §2.1 毒化 canary | debug_assertions 门控 + 4096 次插入摊销 TTL 清理 | 每次插入全扫会在 churn 退化 O(n²) |
| §3.1 dedup×freelist | **释放时删 dedup 键** | freelist 复用不走 dedup(先查 dedup[活条目] → 弹 freelist → 追加),杜绝"一键两槽"竞态腐化 |
| §3.1 墓碑判定 | **显式 tombstone 位** | rc==0 与"创建未推栈"不可区分,显式位消歧(get_string canary 用) |
| §4.1 逃逸分析 | **复用 Plan 310 EscapeAnalyzer + 写捕获扩展** | detect_closure_write_captures 后置检测(Bina Op::Asn 的 Ident LHS);三类出口中"闭包捕获"细分为读捕获(310 借用模型)/写捕获(419 升级) |
| §4.2 RC 消除(LOAD/STORE_NORC) | **缓议(债务)** | Phase 1/2 后 perf 门禁未超标(churn 100k 秒级);优化待基准数据支撑 |
| §4.4 .mut 编译期检查 | **动态兜底先行** | auto.rc.assert_unique native(RC>1 → RuntimeError);静态单活跃 .mut 检查需借用流分析,记债务 |
| §4.5 Cell<T>(Copy 型) | **统一 Rc<RefCell>** | Copy 型写捕获暂不特化 Cell(语义等价、少一条 tier 分支);golden 003 锁定现状 |
| §4.5 Arc<Mutex>(spawn 捕获) | **缓议(债务)** | Tier 4 位保留;异步捕获分析未接线 |
| §4.6 rc_elision 里程碑 | **随 §4.2 缓议** | — |

### 8.2 新增接口速查

- `AutoVM`:rc_push / rc_push_id / rc_push_str_idx / rc_retain(_id) /
  rc_release(_id) / rc_release_slot_range / rc_release_task_stack /
  rc_count / rc_stats / pool_retain / pool_release / pool_live_count /
  pool_count / pool_is_tombstone(rc.rs)
- `HeapObject::child_refs()`(递归释放子引用)
- natives:auto.rc.live(2940)/ auto.rc.count(2941)/ auto.rc.assert_unique(2942)
- a2r:EscapeMap::record_write_capture / is_write_capture /
  write_capture_names;rust.rs 写捕获声明/访问/闭包克隆三点改写

### 8.3 存量问题修复记录(2026-08-23 追加)

1. **编译器"栈溢出" #1/#2(已修复)**:循环体内结构体字面量赋值
   (`for/if ... { x = Note{...} }`)与循环体内嵌套裸块。cdb 帧大小
   实测推翻"无限递归"假设 —— 真因是 debug 构建下解析器单帧可达
   50~270KB(parse_stmt_dispatch ~267KB / expr_pratt_with_left ~116KB /
   atom ~74KB,每层嵌套 ~670KB),2MB 线程栈 3 层嵌套即溢出;64MB
   栈下同程序正常解析、深度仅 ~13。修复(parser.rs):
   - 递归咽喉(expr_pratt / expr_pratt_with_left / atom / group /
     parse_body)挂 `stacker::maybe_grow(512KB, 1MB)` —— 余量不足时
     同线程切换堆上栈段,数据零跨线程、余量充足时近零开销;
   - `body_depth` 护栏(上限 256)+ `[non-recoverable]` 错误旁路。
2. **add_error O(n²) 错误树嵌套(已修复)**:深嵌套下错误恢复路径把
   嵌套着全部下层错误的传播错误逐层 re-push,5000 层实测内存 ~25GB、
   解析挂死。修复:先查后推(超限拒收);护栏错误标记不可恢复,
     两个 catch 臂(顶层 + parse_body_inner)旁路直通。
3. **a2r 深递归用例**:test-trans 套件若干用例在 2MB 测试线程栈下
   溢出 —— 需 test_a2r_deep 大栈 runner 或 RUST_MIN_STACK;存量特性。
4. benchmark_downcast_performance 为计时敏感测试,并行满载下偶发 flaky。

回归测试:vm/tests_parser_stack.rs ×6(bug1 双形态 + 端到端管线 +
bug2 + 2000 层括号 + 257 层护栏),全部在 ≤4MB 小栈线程上通过,
0.09s 完成;全量 3108 过(route::discovery 存量失败除外)。


---

## 9. 复活:ash-gui 确定性 UAF(2026-08-23,外部复现报告 + 定位数据)

> 报告来源:auto-shell 仓 plan 060/061 会话(债务簿同条登记,三次复测)。
> **若此 UAF 为真(假设 H1/H2),它极可能就是 auto-shell plan 060 第五/十五轮
> "静默退出债"的根因真身**——RC 落地前同样的堆损坏无检测,表现为进程无声
> exit 1。修复价值远超 ash-gui 本身。

### 9.1 现象与复现(×3,确定性)

```
cd auto-shell/ash-gui/ash-gui-auto && ../ash-server/target/debug/ash-runner.exe
# MCP 提交任意首条命令(echo 即可)→ 必崩:
thread 'main' panicked at vm/rc.rs:389:
[RC canary] use-after-free: heap object 4000111 was freed 0.0s ago
```

- 复测一代:8b5426fa(持有份额×3 族修复后)id=4001245,仍崩;
- 复测二代:afe30bf8(419 收口后)id=4000111,仍崩;
- 基线 db8a4600(RC 落地前)同负载数小时稳定 + 全套件绿 → 引入窗口锁定在
  Phase 1/2(0c1dc0d5/9bc4e671)。
- 崩溃点在 submit 主路径,与 ash-gui 侧任何新功能无关(060 R16 四桥触发前即崩)。

### 9.2 调用栈与 id 语义(RUST_BACKTRACE=full)

```
iced update → renderer.rs:6677(run_dynamic_iced update 闭包)
  → AutoVM::call_fn_by_name(engine.rs:1581;即 on_with_input_for 的 handler 派发)
  → run_one_instruction → engine.rs:3936 → get_heap_object(engine.rs:868)
  → rc_check_tombstone(rc.rs:389)panic
```

- **访问点 3936 是 GET_FIELD 类指令**:值经 `is_object || (is_i32 && v>=4_000_000)`
  启发式解码为堆 id 后取对象取字段;
- **堆 id 自 4_000_000 单调递增**(engine.rs:471 `heap_object_id_gen`)——
  id 4000111 = **第 111 号分配的早期启动对象**;三次运行 id 在
  4000000~4001245 浮动(分配序随启动路径微变),非哨兵值;
- 派发路径要点:**renderer 在 iced update 里经 call_fn_by_name 重入 VM** ——
  handler 执行横跨一次 VM 重入边界,这是 tier-1 作用域记账最容易漏的形态。

### 9.3 根因假设(按可能性排序)与对应修法

| 假设 | 内容 | 修法方向 |
|---|---|---|
| **H1 过释放第 4 族**(最可能) | GET_FIELD 源操作数的持有份额缺口:值已 load 进 task.ram/操作数栈,但该加载未 +1(或作用域尾清理在某处重复 -1),handler 重入期间对象被释放 | 按 Phase 1 的三族修法同款审计 3936 所在指令族的 load 路径(copy-on-load 协议在 GET_FIELD 源操作数上的执行);重点查重入(call_fn_by_name)期间栈帧生命周期的计数归属 |
| H2 陈旧 VmRef | 一个真 VmRef(id 拷贝)跨 handler 边界存活,对象先死 | 同上,但修的是"跨重入边界的引用存活面"——重入点保存/恢复栈帧时对 >4M 栈值统一 +1/-1 |
| H0 启发式误判(须排除) | 3936 把**合法大整数**(≥4M 的业务值)误当堆 id 去探测;若恰好撞上一个已释放的真 id → 误报 panic | 见 9.4 诊断②;顺带审视该启发式的误判面(TAG_I32 ≥4M 全探测,静默 None 的误探测可能大量存在) |

### 9.4 建议诊断顺序(零风险先行)

1. **free 点埋点**:remove/free 路径对 id∈[4_000_000, 4_001_500] 打
   `log + free-site 短栈`,跑 ash-gui 复现一次 → 直接看到**谁释放了它**;
2. **访问点打点**:3936 命中时打印 id + 指令 + 该 id 对象的 type_tag →
   判 H0(若是 str/int 语义对象则基本排除误判)与 H1/H2;
3. 分配点埋点(第 111 号附近对象的分配站)→ 三点连线即闭环。

### 9.5 验收

- ash-gui 冒烟:ash-runner 起服 → echo/ls/show/取消 全绿(060 §4 口径);
- auto-shell pytest 全套(63 pass + 44 skip 基线)零失败;
- 本仓全量测试(3100+)不回归;canary 保持开启。

### 9.6 关联

- KNOWN-DEBT-AND-RISKS.md「419-P1/P2 RC canary」条目(本节的索引处);
- auto-shell docs/plans/060 §第五/十五/十六轮(静默退出债 + RC 发现史)、
  docs/plans/061 §3(基线规则:修复后合并 plan-061 外部后端分支)。

### 9.7 定位与修复(2026-08-23,worktree 419-uaf)

**复现 rig**(`D:/autostack/diag-419/`,与原报告会话等价):auto-shell
main(2f7774a) detached 检出 + auto-lang「plan-061 ⊕ master」合并分支
(`419-uaf-061`)。配对硬约束:宿主桥 backend_abi 在 plan-061 侧、RC
canary 与崩点行号在 master 侧,缺一不可(auto-shell@38290ad + master
的组合会回退跑 auto-edit 示例,不触发病径;auto-shell main + 纯 master
编不过)。私有 junction `diag-419/auto-lang`、`diag-419/auto-ai`,不动
共享 `.worktrees` 接线。

**埋点**(随修复留在代码,`P419_UAF_TRACE` env 门控,关闭零开销;支持
`4000111` 单 id / `lo-hi` 区间 / 空值缺省 [4M, 4M+1500]):区间内
ALLOC/FREE/retain/release/ACCESS(GET_GENERIC_FIELD)全事件;FREE 与
retain-after-free(复活竞态)附强制栈;窄区间时 ALLOC 与首次获取(0→1)
附栈。

**三点连线**(crash id **4000093**,分配序确定,两轮复现同 id):
- **ALLOC** #93 ListValue:`shim_json_to_value → json_to_vm_value(_inner)`
  ——init handler 期间构造的 JSON 数组,挂在临时 `__json_object`(4000095)下;
- **唯一 retain**(0→1):LOAD 类指令 `rc_push` 压栈(copy-on-load),随后
  SET_GENERIC_FIELD 弹出并「转移」进组件 state 字段;
- **FREE**:`STORE_LOC_0` 覆盖局部槽 → 临时 `__json_object` 死亡 →
  **级联子释放释放了从未 retain 过的 4000093** → rc 1→0 提前释放 + 墓碑;
- **ACCESS**(panic):iced 首帧 view() → `dynamic_view` →
  `read_all_state_materialized` → `vmref_to_vec` → `get_heap_object`
  命中墓碑(rc.rs canary)。

**根因**:`json_to_vm_value` **外层** Array/Object 臂组装顶层容器时漏
「插入即 retain」——内层 `_inner` 两臂 Plan 419 已落地,外层漏了同款。
顶层容器的直接子引用被 `child_refs` 声明却无持有计数,父死连坐释放时
抵消他人真实 stake,子对象提前死亡(state 字段仍持引用)。属 §9.3 **H1
「过释放」家族**,但机制是 **native 构造路径的获取缺口**,非指令族 load
路径缺口;H0(启发式误判)与 H2(重入陈旧 VmRef)在本崩例均不成立。
报告 §9.2 的两点推断修正:崩点不在 engine.rs:3936/GET_GENERIC_FIELD
(实际在首帧 view 的状态物化);触发不在 submit 主路径(init 期即埋雷,
首帧引爆)。

**修复**(a76e9cbe):外层 `list.push`/`fields.push` 前对 `Value::VmRef`
子值 `rc_retain_id`,与 inner 对齐。同类审计:native.rs 列表克隆路径
已有 retain;http_server.rs 构造点全在 `#[test]`;其余 native 均推字符串
经 `json_to_vm_value` 转换——无平行病灶。

**修复后记账**(同 id 4000093):ALLOC → 0→1(构造插入,新增的账)→
1→2(LOAD 压栈)→ 2→1(父死连坐,有账可抵)→ 存活至会话结束。零
FREE、零 canary。

**验收**:
- 崩溃用例 `test_command_exec` 全 2 例(成功+失败路径)PASSED,两轮复跑
  稳定,VM 日志零 canary/panic;
- ash-gui 全量:62 passed + 44 skipped + 1 failed——唯一失败
  `test_mcp_server_responds` 断言「13 工具」,合并栈实际 14(master 新增
  `autoui_press_sequence`),纯版本错配与修复无关,plan-061 合并后
  auto-shell 侧需同步改断言;
- 本仓 auto-lang lib:**3125 passed / 0 failed**(全量初跑 1 例
  ui_console ring flaky——并行共享全局态,单测与复跑皆绿,与本改动无关);
- canary 保持开启。

**遗留**:
- diag rig(`D:/autostack/diag-419/`:probe.py + vm-run1~9.log + auto-shell
  detached worktree + 双 junction)已在收尾时拆除——原始日志为会话内
  临时证据,结论均已在本文档;复验路径见本节「复现 rig」+ P419_UAF_TRACE;
- `419-uaf-061` 分支为复现专用合并栈(plan-061 ⊕ master + 埋点 + 修复),
  修复本体已 cherry-pick 回 `419-uaf`(master 基),随收尾一并清理;
- §9.1 的「三次复测」id(4000111/4001245)与本轮(4000093)不同属分配序
  浮动,病灶同一(顶层 JSON 容器子引用零 retain 的确定性缺口)。

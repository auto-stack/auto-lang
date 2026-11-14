# ACE 动态值对象反射与通用闭包（M2-A 切片）

> PLAN-964；实现位于 `experimental/ace/grammars/auto-source.at`（object-lit/nil-lit/索引步/if-cond 子规则）、
> `core/src/source/fstr.rs`（编译前文本变换）、`core/src/source/{cst/project.rs,ast,resolve,typecheck,adapter}.rs`
> （对象/nil/可调用/闭包×对象）、`core/src/{descriptor,verify}.rs` 与 `core/schema/{closures,numeric}.atom`、
> `lowering/src/hir_adapter.rs`（ObjectNew/Get/Set、TextDisplay、MirConst::Nil）、`av-core`（Value::Object/
> Nil/External、DynType 扩展、opcode 0x3D/0x3E/0x3F）、`native`（mn TextDisplay/FStr 车道 + CRT stub）、
> `a2x-{rust,c}`（a2x_str_display_i32/record-text 装配）、`av/src/bin/av-worker.rs`（aggregates-source recipe）。
> 本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。语义基线为 959 分母
> L-16/L-20/L-21/L-22/L-23 与本包 T-01 冻结表（计划 §9 收据），四端按原支持域矩阵
> （AV 全能力；native/a2r/a2c 仅 FStr/闭包族，对象/动态面具名拒）。

## 支持入口与身份

动态面经 `ace_core::source::compile_aggregates_source`（对象/nil/FStr）与 `compile_closures_source`
（闭包×对象）显式进入；FStr 为**编译前门文本变换**（`f"…"` → `__fstr(…)` 链，无独立 f-string token），
插值段由主解析器解析，表达式段源序求值恰一次（TextConcat 链左结合）。worker 消费入口新增
`ac-av/aggregates-source-run-v1` recipe（aggregates 前端 → closures 族 lowering/runtime，`av.closures/1`
身份）；closures-source 冻结表扩展 D01/D02。显示转换 I32→十进制归本包；其余类型/格式算法归 972。

## 能力

| 能力 | 支持 | 语义 |
|---|---|---|
| 对象（L-16/L-20） | ✓ | `Value::Object(Rc<RefCell<ObjectData>>)`（有序 key→Value，插入序）；`MirType::Object{elem}`；MIR ObjectNew/ObjectGet/ObjectSet；`VT::Object(elem)`，字面量字段同型冻结，异型 `source.object-elem` 具名拒；`let o2 = o1` 为 Rc 别名（共享），写经任一名可见 |
| 对象索引读写 | ✓ | `obj["y"]=200`/`obj[k]=300` 动态字符串键；缺键读运行期 trap `av.dyn.object-missing`（拒绝合同，非静默 nil）；upsert 旧值 drop 恰一次 |
| 循环策略 | ✓ | 自插/可达同 cell 插入在 ObjectSet 以 `Rc::ptr_eq` 确定性拒（`av.dyn.object-cycle`），不建 GC；共享 cell 最后 owner 释放（Rc），无环即无泄漏 |
| nil/null（L-21） | ✓ | `nil` 与 `null` 均为 Nil 字面量（PLAN-053 统一）；`Value::Nil`（零宽，与 Unit 区分）；`MirType::Nil`/`VT::Nil`/`DynType::Nil`；let 可绑定；对象 elem=Nil 可存读；`==`/`!=` 仅 Nil 对（恒真/恒假常数化）；其余算子 `source.nil-op` 具名拒 |
| 可调用值（L-22/L-23） | ✓ | `DynType::Callable`：pack 收 `Value::Closure`（仅声明 Callable 时放宽白名单）；对象 elem=Closure 可持闭包，`obj["twice"](3)` = ObjectGet→闭包→CallClosure（callable/member/index 派发）；命名顶层函数非一值对象（`source.fn-as-value` 具名拒，延后续） |
| 类型标签 vs 推导失败（V-01） | ✓ | DynType 标签 = 值实际类别（describe()）；错 tag take/is 为运行期 trap/Err 携 expected/actual 标签；源级推导失败 = 具名源诊断（`source.object-elem`/`source.nil-op`），两者不同态 |
| 通用闭包捕获（source02） | ✓ | factor=3 捕获 + HOF，新例核 `[3,6,9]`（非只 len）；捕获对象 = Move（Rc 共享），两闭包共享更新可见；List `map` 不在 962 集合面——显式循环+闭包调用替代（具名裁定） |
| FStr（L-22） | ✓ | 插值表达式源序求值恰一次；I32 显示归本包；文本部分认任意 Str/StrOwned；`f"…"` 内 `&&` 链 lowering 边见 D-964-C1 |

## INT-1026/IF-VAL 交接（通用外部值）

`DynType::External{ provider, kind }` + `Value::External(Rc<RefCell<ExternalCell>>)`：外部值/可调用值
中立载体。**不持 PyO3、不自建第二资源账本、不依赖 1026/967 runtime**；retain=clone（Rc 增 owner）、
drop=最后 owner 释放钩子恰一次（独立 provider 测试见证）；别名/参数返回/分支/循环传引用保身份；
错 kind 拒（DynTakeError 标签）；活外部值**不可** portable freeze（`av.dyn.unsupported` 具名拒）。
外部 kind 与引用 cleanup 钩子消费现有中立 foreign-contract 事实；反射派发只依赖实际值类别与受管操作。
provider 为测试内独立 owned-resource/callable 提供者，无 Python/torch。

## 四端支持域矩阵

| 能力 | AV | native(mn) | a2x-rust | a2x-c |
|---|---|---|---|---|
| 对象/动态键/nil | ✓ | `mn.object-unsupported` 具名拒 | 不在域 | 不在域 |
| 闭包（Move 捕获） | ✓ | ✓（MnDriver closures 车道） | 不在域（无闭包 emit） | 不在域 |
| FStr（TextDisplay/显示） | ✓ | ✓（`ac_rt_own_display_i32`） | ✓（`a2x_str_display_i32`） | ✓（`a2x_str_display_i32`） |
| INT-1026 外部值 | ✓ | 不在域 | 不在域 | 不在域 |

## 拒绝与边界（具名诊断）

`source.object-elem`（对象字段异型）、`source.nil-op`（nil 参与算术）、`source.fn-as-value`（函数项为值）、
`av.dyn.object-missing`（缺键读）、`av.dyn.object-cycle`（自插/可达同 cell）、`portable.unsupported`
（活外部值 freeze，portable 命名空间具名拒——§5 冻结措辞 `av.dyn.unsupported` 为命名空间笔误，D-964-C2）、
`mn.object-unsupported`（mn 面对象/动态）。已知拼写债：D-964-C1
（`&&` 链 f-string 条件 lowering 的 `mir.uninitialized-read` 边；单独 `if` 形为规范语料）。

## 四端证据

AV：`dynamic_objects_964` 10 腿（对象读写 600/nil 别名 20/可调用 9/HOF 369/fstr 42 + 缺键 trap 70 +
两个 resolve 诊断）+ `int1026_external_964` 4 腿（drop 恰一次/错 kind/活对象 freeze 拒）。native：
`dynamic_964` 4 腿真 PE（fstr 42、闭包 Move、对象面具名拒）。a2x：`dynamic_964`（Rust+C 双腿 fstr 42）。
语言侧 case 组 `case-ace-m2-dynamic-964` 8/8（aggregates-source + closures-source 车道）。并集
`cargo t auto-val ace-source ace-semantics ace-av ace-native ace-a2x ace-runtime case-ace-m2-dynamic-964`
最终轮无计划归因红（环境红具名：char-multi 词法 ×6、`__CxxFrameHandler3`/`_fltused` lld 链接 ×19——
mn 车道 CRT stub 为净改善；ace-a2x meter infra flake）。

# 共享文本源码收口：T03/T05与资源替换

> target-state r1 / 2026-10-30；[PLAN-894](../../plans/894-shared-text-source-and-resource-closure.md)实施合同。扩展Design44与现有770/782源码域，不改成第二套资源系统；当前仅设计/立项，未运行原例。

## 1. 已有与目标

当前`text_resources.rs`明确拒绝Str拥有局部的`=`，T03因此尚未闭环；`auto-mir-lower`已对owned Str槽Assign发射Move，且已有view eq/ne→StrCmp分支，AV/native也已有Move替换和字符串机器面。因此本包先做真实原源可行性，不根据旧binding或历史Spec重复造指令。T03原源/expected=23、T05原源/expected=42逐字保持。

首期只扩**可变、已初始化且仍存活的局部Str根**的完整替换：RHS为合法拥有构造/concat/显式clone/拥有返回或异根显式move。裸owner隐式复制、let/view参数赋值、活借用冲突、目标自move、RHS move/drop/mut目标、移出后Str根复活均不因此开放；已有record整根/字段规则保持。没有NLL、隐式clone、用户析构或新语言级unwind。

## 2. 替换事务和失败边界

保守按根建立写reservation；RHS允许读目标并完整物化，例如`c = c + "中"`、`c = c.clone()`。旧目标及其缓冲在RHS完成前保持。RHS成功且视图临时结束后，以无用户代码/挂起、无新增fallible操作的提交段安装新owner、消费转移责任并恰一次清理旧缓冲；禁止先free旧值再求RHS。静态借用门阻止仍存活的外部view跨替换悬空，不靠Rc偶然延寿。

失败原子性只承诺替换目标不发生部分提交、部分新临时按现有路径清理；不回滚RHS已发生的任意外部副作用或已完成的异根move。目标相关move/mut在本包拒绝。AV受控trap/取消沿现有全出口清理；native旧owned fatal路径沿其既有当前帧/OS回收边界，不谎称全帧unwind或fatal前live=0。以arena事件、现有shim日志和既有故障注入辨别“RHS前释放”“双free”“泄漏”，不得只看exit70。

## 3. 比较与单函数增量

str比较按长度与完整UTF-8字节，NUL不截断、Unicode不归一化；只在已有合法文本类型上走StrCmp，普通标量Bin与其他profile拒绝面保持。若T05已真源通过，交付其证据/回绑与永久反控，不再改lowering。

AV复用`OwnedFunctionSession`真实单body通道；native复用828受检环境/候选事务及现有per-unit COFF/cache/link设施，必要新增受限owned-source桥，不把scalar或collections session当通用能力。body编辑须从实际parse/resolve/typecheck/HIR/lower/verify/ABC或COFF生产点及居民对象身份取证；未改caller/disjoint生产0且对象保持，整模块工作0，PE/AV实际新值可观察。冷恢复decode/verify另计，不伪装暖hit；失败候选保旧源/环境/坐标/manifest/可执行闭包。

## 4. 独立推进与交接

S主写面为ac-core源码/资源分析及唯一共享lowering，P为新例/定向测试；不新增dispatcher/recipe/wire或第二worker。现有`ac-av/owned-run-v1`和`ac/mir-native-scalar-v1`消费原源，现有TestCase发现新suite。native只改旧owned必要接缝，不改891 typed ABI；892保持scalar合同，未来AX-B消费本包正式资源事实。

独立不等于免身份同步：若lowering契约真变，依现行规则升版并同步AV/CTE/MN既有镜像/报告stale及受影响cache身份；若只前端准入扩面，冻结正确的source/verify producer身份，不能用不升版避H，也不机械改无关ABI。H仅限这类最小镜像、用例回绑与正式发现，不改共享dispatcher。888初读代码已到main `f51b14fdb`；本轮最终核实已archived 10/10，delivery `5a6c23afa`及cleanup `129ef6057`均在main，原树已清，公共写面/测量窗已释放。实施时按实际符号释放取窗，不依赖888/891/892整包；正式采样与同机构建负载错峰。

## 5. 首例验收

原源23/42、同根只读RHS/显式clone/异根move、借用与非法转移、分支/循环/出口清理、RHS失败事件、函数body A→B→A、fresh/hit/restart/损坏恢复与依赖漂移从首例纳入。原M1 expected/分母/legacy/a2r状态保持；旧T03“受限域拒绝”的当前测试随本次批准扩面迁为value=23，保历史依据，不同时保留互相矛盾断言。计时/内存沿现行交集，正式P90≤2000ms，完整支持/排除面和可运行反控由894冻结。

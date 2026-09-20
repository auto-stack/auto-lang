# 内存安全支持边界（PLAN-667 底线）

> 范围：本版（PLAN-667）VM 捕获、引用计数（RC）、a2r 逃逸分析三面的
> **已验证支持 / 明确拒绝或受检运行期错误 / 不属于本次保证** 三值边界。
> 本文件是当前实现状态的准确描述，不是下一版内存模型的设计承诺。
> 证据：`docs/plans/reports/667-memory-safety-evidence.md`（探针/命令/提交）。

## 1. VM 闭包捕获（by-reference）

| 入口 | 分类 |
|---|---|
| 创建者帧存活期内调用（含 sort/map/filter/find 等经 call_closure 的同步谓词） | 已验证支持 |
| 嵌套闭包捕获祖父帧变量（传递继承帧锚，outer 存活链内） | 已验证支持 |
| 生成器帧（独立任务挂起/恢复期间，帧 uid 保留） | 已验证支持 |
| env by-value 捕获（0xFFFF 槽位 / `self` e1 / future 合成闭包） | 已验证支持（复制语义，无帧约束） |
| 创建者帧已返回后调用（全局/容器逃逸、定时器等异步回调） | 明确拒绝——RuntimeError（槽位读写前），消息指名 capture/frame |
| 同 bp 帧复用后的旧闭包访问 | 明确拒绝（帧实例 uid 判定，非 bp/sp 范围判定） |
| 跨任务调用（closures 表 VM 全局，帧偏移按任务 ram 解释） | 明确拒绝（任务身份校验） |
| 挂起 async 体内创建的闭包在体完成后调用 | 明确拒绝（帧退栈即失效） |
| `.view`/`.mut` 借用捕获 | 编译期拒绝（compile_closure check_unsafe_capture，plan-071 既有） |

机制：`AutoTask.frame_ids: Vec<(bp, uid)>` 帧实例活跃表（6 建帧点发号/RET 摘除/
push 截断自愈）+ `Closure.creator_frame: Option<(task_id, uid)>`；uid 0=根帧
（任务终身）。守卫 `ensure_capture_frame_alive` 在 LOAD/STORE_CAPTURED 的
capture_slots 与 param_abs 臂任何读写之前执行。

## 2. 引用计数与持有（rc.rs / engine）

| 入口 | 分类 |
|---|---|
| 审计面引用流（LOAD/STORE/POP/RET/容器替换/捕获/native+FFI 交接的 tagged 值与 rc_push_id 裸 id） | 已验证支持——影子（stake）或 tagged 计份 |
| 真整数 ≥ 4,000,000 经 DUP/LOAD_LOCAL/LOAD_GLOBAL/LOAD_STATE_FIELD/STORE_GLOBAL/STORE_STATE_FIELD | 已验证支持——份额出处门（rc_push_slot）：零计数，与活 heap id 撞值不冒领/不扣除（探针锁定） |
| rc_retain_id 对不存在 id | 拒计（存在性门：幽灵 id 零计数） |
| 统计读取（rc_stats / auto.rc.live / UI 水位） | 已验证支持——**纯读**，不改变被观测程序生命周期；终态断言需显式 `reap_all()` |
| dying 宽限窗（8192 解释步） | 不属于本次保证的**对象回收策略**（rc==0 对象延迟回收）；不再承担有效引用保活——审计面内有效引用均带份额 |
| 真整数与活 id 值碰撞期间经**未审计路径**的瞬态副本 | 不属于本次保证（裸 i32 id 编码固有；出处置门后审计面已零计数；编码退役属下一版） |
| 类型化 var 声明（`var a Note = ...`）局部槽影子转移 | 已知泄漏（每声明 1 份，KD-051⑤ 族，P667-D1 在册；泄漏方向不破坏安全） |
| closures 表 env 份额 | 已知泄漏（VM 终身，安全方向；VM 生命周期结束即释放） |

## 3. a2r 逃逸分析（trans/escape → rust.rs）

| 入口 | 分类 |
|---|---|
| 非逃逸绑定 `.view`/`.mut` → `&`/`&mut` | 已验证支持（rustc 实编+运行正例门禁） |
| 闭包读/写捕获升级（含嵌套/源序遮蔽/Try/Reply；未覆盖 AST 形式 fail-closed 全可见升级） | 已验证支持——读捕获 Clone tier + W0007；写捕获 write_captures（Rc::clone + move） |
| 同名嵌套/兄弟作用域合并 | 已验证支持——fold_to_root 名称合并（逃逸获胜，保守方向） |
| `.mut` 于逃逸（Clone/RcRefCell）绑定 | 明确拒绝——Auto 侧诊断（VM 同场景 auto.rc.assert_unique 运行期拒绝，语义对齐；mutate clone 副本=静默偏离） |
| `.view`/`.mut` 于 Send 边界捕获（.go/tokio::spawn） | 明确拒绝——Arc 声明改写未实现（Plan 667 边界），不以 `.clone()` 伪装 |
| 闭包**读**捕获逃逸出函数（返回/存储闭包） | rustc 边界——生成 safe Rust 由 rustc 借用检查拒绝（E0373 族）；生成以实际 rustc 编译为门禁 |
| 完整 NLL / lifetime 参数 / borrow checker 接入 | 不属于本次保证——ownership/（lifetime/borrow/cfa）为未接线设计探针，不作安全证明 |

## 4. 不变量与已知限制

1. RC 偏差纪律不变：漏 incref=悬垂（致命）、漏 decref=泄漏（安全）；本版所有
   修复保持该方向（未计数的**有效**别名来源已关闭或收紧；新增缺口均泄漏方向）。
2. 内存安全 ≠ 完整静态借用保证：VM 侧捕获有效期是**运行期受检错误**，
   a2r 侧是**保守超集 + rustc 门禁**；两者都不宣称 Rust 级全静态证明。
3. 下一版候选（lifetime/from/weak/parent/extract/region 语法、新 borrow checker、
   裸 id 编码退役、类型化 var 槽位记账）均**未实现**，属另案设计。

> 来源: PLAN-667（f181051ba / 5c8f5ce8e / faaeeaf0e）；rc.rs / engine.rs /
> task.rs / trans/escape/ / trans/rust.rs；探针
> crates/auto-lang/src/plan667_memory_safety_tests.rs。

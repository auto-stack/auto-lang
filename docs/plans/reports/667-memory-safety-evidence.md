# PLAN-667 内存安全底线证据报告

执行 worktree：`D:/autostack/.wt/lang-667/auto-lang`（branch `plan-667-dev`）。
基线：master `21f0b7f72`（取号后计划确认提交）。
矩阵分类仅三值：**已验证支持 / 明确拒绝或受检运行期错误 / 不属于本次保证**。
测试模块：`crates/auto-lang/src/plan667_memory_safety_tests.rs`（21 探针，修复后 21/21 绿）。
命令：`cargo t plan667`（日常档滤串）。

## 1. 候选问题结论总表（F-01..F-07）

| ID | 判定 | 一句话结论 | 证据测试 | 修复提交 |
|---|---|---|---|---|
| F-01 | 确认+修复 | 帧相对捕获零校验；另修 STORE_CAPTURED 双 pop/内容释放/stake 不转移、CLOSURE pop_i32 标签截断、env 加载漏 stake、嵌套闭套祖父帧错锚五个同族缺陷 | plan667_capture_*（7） | f181051ba |
| F-02 | 确认+修复 | rc_stats() 读统计强制收割（UI vm_bridge 与 auto.rc.live() 生产入口同病）；统计纯化+soak 通道/测试 helper 显式 drain | plan667_rc_stats_pure_and_explicit_drain | 5c8f5ce8e |
| F-03 | 确认+修复 | find_escapes 闭包臂经 gather_var_refs 对 Closure 完全不下降——捕获检测为空集；改为源序遮蔽自由变量收集器（嵌套域恢复） | plan667_a2r_closure_read/write_* | faaeeaf0e |
| F-04 | 确认+修复 | 生成侧 current_scope_depth 恒 0+兄弟作用域撞键+未覆盖 AST 静默忽略；fold_to_root 名称合并（逃逸获胜）+Try/Reply 覆盖+未知形式 fail-closed | plan667_a2r_nested_scope/unknown/reply | faaeeaf0e |
| F-05 | 确认+修复 | ArcMutex→.clone() 与 .mut 于 Clone 层→.clone() 是已知坏 fallback；改 Auto 侧明确诊断（错误消息含 mut/escape/Send/Arc 边界） | plan667_a2r_mut_on_escaped/arcmutex(go) | faaeeaf0e |
| F-06 | 证实-不改 | ownership（lifetime/borrow/cfa）零生产调用面；仅 Spec 状态纠正（SD-01），不接入 | 静态 grep（§5） | — |
| F-07 | 确认+修复 | 防御性补持（STORE_GLOBAL/STATE_FIELD）按内容判 i32≥4M 冒领活对象份额+DUP/LOAD 内容性 retain 幻影份额（可复活 dying 对象）；三重修复：存在性门+防御补持收紧 tagged+rc_push_slot 份额出处门 | plan667_rc_integer_* | 5c8f5ce8e |

## 2. VM 捕获矩阵（T-02，提交 f181051ba）

基线红（修复前，静默损坏）→ 修复绿（RuntimeError 拒绝）：

| 探针 | 源码形态 | 基线（修复前） | 修复后 |
|---|---|---|---|
| plan667_capture_alive_creator_read | 创建者存活期内 by-ref 读 | 绿（正例保护） | 绿 |
| plan667_capture_alive_creator_write | 存活期内写捕获 `n=15` | 绿（正例保护） | 绿 |
| plan667_capture_nested_alive | 嵌套闭包祖父帧捕获 | **红：得 8 非 106**（槽位按编译期帧算错） | 绿：106 正确（传递继承帧锚） |
| plan667_capture_after_return_read | 全局中转，创建者已返回 | **红：静默读陈旧槽** | 绿：RuntimeError "creator frame has already returned" |
| plan667_capture_after_return_write | 写穿到同 bp 复用帧（victim.n 被污染路径） | **红：跨帧写污染** | 绿：写前拒绝 |
| plan667_capture_same_bp_reuse_read | 两次 mk 复用同 bp 后旧闭包读 | **红：读到第二次调用的值** | 绿：拒绝 |
| plan667_capture_cross_task_or_after_return | 全局闭包跨任务/事后调用 | **红：静默** | 绿：拒绝（任务身份校验） |

错误消息可定位：`captured variable '<v>' in closure <id>: creator frame has already returned — ... / created in task <A> but accessed from task <B> — cross-task frame captures are not supported`。
失败阶段：任何 capture_slots/param_abs 槽位**读写前**（读nv/写nv之前），符合"不允许读取失效槽后再诊断"。

实现要点：
- 帧身份 `(task_id, frame_uid)`：AutoTask.frame_ids 活跃表；6 个建帧点发号（CALL×2/CALL_SPEC/call_closure/CALL_CLOSURE/handler 激活）；RET 退栈摘除；push 截断 `bp >= 新帧 bp` 陈旧条目自愈（漏截断最多退化为守卫前行为，漏 push 是可闻拒绝——不对称风险可接受）。uid 0=根帧（任务终身）。
- 嵌套闭包传递继承：inner 对 outer 已捕获变量的槽位沿用 outer 的 (creator_bp, slot)（祖父帧锚）。
- STORE_CAPTURED 重写：单 pop（修双 pop bug）+弹出槽影子转移+旧槽按影子释放（release_slot_old_value，废除内容释放——真整数撞活 id 不再多扣）。
- CLOSURE：pop_nv 保标签（旧 pop_i32 把字符串/对象捕获截成垃圾整数）+env 份额语义钉定（LOAD_CAPTURED env 分支 retain+mark_top_stake，修循环加载逐次泄漏）。

适用入口：LOAD_CAPTURED/STORE_CAPTURED 的 capture_slots 与 param_abs 两臂（生产 VM 全部路径）；env by-value 捕获（0xFFFF/合成闭包）不受帧存亡约束（复制语义）。
持久回调/任务路径：TimerCallback::Closure 的 by-ref 捕获在创建帧死后调用同样被拒（此前为静默损坏；与 Plan 442 注释"closure form only supports by-value captures"的文档口径一致化）。

## 3. RC 矩阵（T-03，提交 5c8f5ce8e）

| 探针 | 断言 | 基线 | 修复后 |
|---|---|---|---|
| plan667_rc_stats_pure_and_explicit_drain | dying 窗口内连读两次 rc_stats 不回收；显式 reap_all 后归零 | **红：读即收割** | 绿 |
| plan667_rc_alias_survives_churn | 合法别名跨 >8192 步仍有效（keep.id==7） | 绿 | 绿 |
| plan667_rc_integer_no_net_claim_on_live_id | 真整数 4000000 存全局+丢弃真持有者：drain 后 live_heap==0、rc_count(4M)==0 | **红：幻影份额滞留** | 绿（P419_UAF_TRACE 事件链取证：修复后 DUP/STORE_GLOBAL 对整数零 retain） |
| plan667_rc_teardown_returns_to_baseline | 200 churn 收尾 drain 后 live_heap==0、underflow==0 | 绿 | 绿 |
| plan667_rc_capture_write_loop_no_growth | 50 拍 by-ref 写捕获循环 rc 无增长、值正确（50） | 绿 | 绿 |

统计/回收分离落地：
- `rc_stats()` 纯化（移除内嵌 reap_all）；生产读点 ui/vm_bridge.rs 与 native auto.rc.live() 不再改变被观测程序生命周期。
- 显式收尾通道：`vm.reap_all()`（测试 helper tests_rc_lifecycle::run_code_vm、plan510/608 已有调用、vm_bridge::heap_live_objects soak 断言通道——调用面全为测试，无生产 per-frame 调用者）。
- `rc_retain_id` 存在性门：id 无堆对象且无 rc 条目 → 拒计（幽灵 id 零计数；dying 宽限窗内对象仍在表，合法复活不受影响）。
- `rc_push_slot` 份额出处门（DUP/LOAD_LOCAL/LOAD_LOC_0/1/LOAD_GLOBAL/LOAD_STATE_FIELD）：源槽无影子且裸 i32≥4M = 真整数零计数；tagged 引用恒 copy-on-load。
- STORE_GLOBAL/STORE_STATE_FIELD 防御性补持收紧到 tagged（裸 i32 生产者已全部迁移 rc_push_id：stdlib.rs DefaultBodyLimit.max 一处）。
- 既有宽限窗保留为"不可访问对象的策略"（rc==0 对象延迟回收），不再承担有效引用保活——审计面（LOAD/STORE/POP/RET/容器替换/捕获/native+FFI 交接）内有效引用均带影子或 tagged 计份。

执行期新发现（未入 F 编号的同族缺陷，已修）：
1. STORE_CAPTURED param_abs 分支双 pop（engine.rs 旧 8490+8515）——吃掉无关栈值。
2. CLOSURE pop_i32 标签截断——by-value env 捕获非整数值损坏。
3. LOAD_CAPTURED env 分支 retain 无影子——循环内每次加载净漏一份（无界泄漏）。
4. 嵌套闭包祖父帧错锚（见 §2）。

## 4. a2r 矩阵（T-04，提交 faaeeaf0e）

| 探针 | 断言 | 基线 | 修复后 |
|---|---|---|---|
| plan667_a2r_closure_read_capture_escalates | 闭包读捕获 → 非 borrow tier | **红（捕获检测空集）** | 绿（Clone） |
| plan667_a2r_closure_write_capture_flagged | 写捕获 → write_captures 标记 | 绿 | 绿 |
| plan667_a2r_nested_scope_merges_to_root_identity | 内层同名逃逸反映到 depth0 查询 | **红（生成侧恒 depth0 漏报）** | 绿（fold 逃逸获胜合并） |
| plan667_a2r_unknown_stmt_fail_closed | try 内闭包捕获 → 升级 | **红（`_ => {}` 静默）** | 绿 |
| plan667_a2r_reply_escapes | reply s → 逃逸 | 绿 | 绿 |
| plan667_a2r_mut_on_escaped_binding_rejected | `.mut` 于逃逸绑定 → Auto 侧错误 | **红（发 `.clone()`）** | 绿（明确诊断） |
| plan667_a2r_go_capture_rejected | `.go` Send 捕获 → 拒绝或真 Arc | 绿（拒绝路径） | 绿（ArcMutex → 明确诊断） |
| plan667_a2r_plain_view_still_borrows | 非逃逸 `.view` 仍 `&s`（正例不变） | 绿 | 绿 |
| plan667_a2r_captured_binding_view_not_borrowed | 捕获绑定 `.view` 走 clone 通路 | **红（发 `&`）** | 绿（.clone()） |
| plan667_a2r_positive_borrow_compile_run（#[ignore]，rustc 实编） | 正例真实编译+运行输出 hello | — | **绿（rustc 0.9s 编译+运行通过）** |

rustc 实编门禁命令：`cargo nextest run -p auto-lang --lib --features test-trans plan667_a2r_positive_borrow_compile_run --run-ignored=only`。

已知边界（SD-02/SD-04 记载）：
- 闭包**读**捕获逃逸出函数（返回/存储）时，a2r 依赖 rustc 借用检查拒绝（E0373 族）——这是声明的 rustc 边界（safe Rust 生成仍以实际编译为门禁），非有害 fallback；写捕获有 Rc<RefCell> 真实通路。
- ArcMutex 生成（声明改写）未实现——本版明确诊断拒绝，不伪装。
- 折叠合并是保守方向（同名兄弟绑定损失借用机会，false positive 可接受）。

## 5. 静态审计记录

- ownership 模块生产调用面：`grep -rn "crate::ownership" src --include="*.rs" | grep -v src/ownership | grep -v src/tests`
  → 仅 ownership_tests.rs（自测）。lifetime.rs/borrow.rs/cfa.rs 未接入编译主路径，不作为安全证明（SD-01 纠正）。
- 裸 heap id 推法存量：`grep -A4 insert_heap_object ... | grep push_i32`（排除 rc_push*）
  → 唯一 stdlib.rs:8636（DefaultBodyLimit.max，已迁移）；native.rs:7226 为 0 值回退非 id。engine 建物全部 `rc_push(encode_object(...))` tagged。
- rc_stats 生产调用面：ui/vm_bridge.rs:1470（经 heap_live_objects，调用面全为 soak 测试）、native.rs:9577（auto.rc.live()，现纯读）。
- closures 表无 remove/clear——env 份额随 VM 终身（泄漏方向，安全纪律内；VM 生命周期结束即释放）。
- .at 语料无 auto.rc.live/count 使用（统计纯化零金样影响）。
- 生成侧 current_scope_depth 从不递增（rust.rs 仅 734/14311 两处置 0）——F-04 生成侧实锤。

## 6. 门禁与回归账

| 门禁 | 结果 | 说明 |
|---|---|---|
| cargo check -p auto-lang | 绿 | 177 既有 warning，零新增 |
| cargo t plan667 | 21/21 绿 | 三矩阵全量 |
| cargo t plan510 | 8/8 绿 | 池计数回归 |
| cargo tv（--no-fail-fast） | 仅 2 预存红（ui_gen mouse_area/autodown_panel_heading，stash 对照实证与本计划无关） | VM 语料 golden 无回归 |
| cargo tt（--no-fail-fast） | 与 stash 基线**零差**（预存红：007_shared_var/003/004_use_c/rustc_real_compile_gate/ui_gen×2，均干净树同红） | a2r 金样无回归 |
| cargo t（日常档 ui-iced --no-fail-fast） | 44 vs 43 基线：新增全部为 plan667 待修探针（现已绿）+external_config_poll 环境闪红（stash 亦红） | 全量回归清零 |
| cargo tf | 见提交后补记 | 触及 VM 核心协议，按计划跑全量 |
| rustc 实编（plan667 正例） | 绿 | 0.9s 编译+运行 witness "hello" |

## 7. 支持边界矩阵（SD-04 输入）

### VM 捕获（by-reference capture_slots/param_abs）
| 入口 | 分类 |
|---|---|
| 创建者帧存活期内调用（含经 call_closure 同步回调：sort/map/filter/find 谓词） | 已验证支持 |
| 嵌套闭包捕获祖父帧变量（outer 存活链内） | 已验证支持 |
| 创建者帧已返回（含全局/容器逃逸后调用） | 明确拒绝（RuntimeError，读写前） |
| 同 bp 帧复用后旧闭包访问 | 明确拒绝 |
| 跨任务调用（closure 表 VM 全局） | 明确拒绝（任务身份校验） |
| env by-value 捕获（0xFFFF/合成闭包/self e1） | 已验证支持（复制语义，无帧约束） |
| 挂起 async 体内创建的闭包在体完成后调用 | 明确拒绝（帧退栈即失效） |
| 生成器帧（独立任务挂起/恢复期间） | 已验证支持（帧 uid 跨挂起保留） |

### RC/持有
| 入口 | 分类 |
|---|---|
| 审计面内引用（LOAD/STORE/POP/RET/容器/捕获/native+FFI tagged/rc_push_id） | 已验证支持（影子或 tagged 计份） |
| 真整数 ≥4M 经 DUP/LOAD/STORE_GLOBAL/STATE_FIELD | 已验证支持（零计数，撞活 id 不冒领；探针锁） |
| dying 宽限窗 | 不属于本次保证的对象回收策略（不再承担有效引用保活） |
| 真整数 ≥4M 与活 id **值碰撞期间**的瞬态栈副本 | 不属于本次保证（编码固有；出处置门后全路径零计数，碰撞面已收敛到 tagged 载体） |
| 类型化 var（`var a Note = ...`）局部槽影子转移 | 已知泄漏（KD-051⑤ 既有，见 §8 债 P667-D1；泄漏方向不破坏安全） |
| closures 表 env 份额（VM 终身） | 已知泄漏（安全方向，VM 生命周期结束释放） |

### a2r 逃逸分析
| 入口 | 分类 |
|---|---|
| 非逃逸绑定 `.view`/`.mut` | 已验证支持（`&`/`&mut`；rustc 实编正例） |
| 闭包读/写捕获（含嵌套/遮蔽/Try/Reply/未知形式 fail-closed） | 已验证支持（升级 Clone/write_captures；写捕获 Rc 通路） |
| `.mut` 于逃逸（Clone/RcRefCell）绑定 | 明确拒绝（Auto 诊断；VM 同场景运行期拒绝，语义对齐） |
| `.view`/`.mut` 于 Send 边界捕获（ArcMutex） | 明确拒绝（Arc 声明改写未实现，Plan 667 边界） |
| 读捕获逃逸出函数（返回闭包） | rustc 边界（E0373 拒绝；声明边界非有害 fallback） |
| 完整 NLL/lifetime 检查 | 不属于本次保证（下一版内存模型另案） |

## 8. 债与既有红归因

- P667-D1（新入册）：类型化 var 局部槽影子转移缺失——`var a Note = Note{...}` 声明路径槽位无影子，覆盖赋值不释放旧值（每声明 1 份滞留；KD-051⑤ 同族延续）。本计划探针以无类型形态隔离；根治归 RC 槽位记账专项。
- 既有红归因（stash 双跑实证，非本计划引入）：ui_gen::rust::tests::mouse_area_emits_events_and_logical_extent、test_autodown_panel_heading_codegen（tv/daily 档）；a2r test_14_modules_007_shared_var、test_27_c_abi_003/004、a2r_rustc_real_compile_gate（test-trans 档本机基线）；external_config_poll_hot_apply_loopsafe（环境闪红，stash 亦红）。

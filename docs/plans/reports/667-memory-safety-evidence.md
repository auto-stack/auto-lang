# PLAN-667 内存安全底线证据报告

执行 worktree：`D:/autostack/.wt/lang-667/auto-lang`（branch `plan-667-dev`）。
基线：master `21f0b7f72`（取号后计划确认提交）。
矩阵分类仅三值：**已验证支持 / 明确拒绝或受检运行期错误 / 不属于本次保证**。
每项记录：最小源码、基线结果、期望、根因、修复/限制、实际错误阶段、测试名、提交与命令。

## 1. 候选问题结论总表（F-01..F-07）

| ID | 判定 | 一句话结论 | 证据测试 |
|---|---|---|---|
| F-01 | 确认+修复 | 帧相对捕获零校验（读陈旧槽/跨帧写/跨任务错帧/宿主 panic 风险）；另发现 STORE_CAPTURED 双 pop、内容释放、stake 不转移、CLOSURE pop_i32 标签截断、env 加载漏 stake 五个同族缺陷，全部修复 | plan667_capture_* |
| F-02 | 确认+修复 | rc_stats() 读统计强制收割（UI vm_bridge:1470 与 auto.rc.live() 生产入口同病）；统计纯化+显式 drain 分离 | plan667_rc_stats_pure |
| F-03 | 确认+修复 | find_escapes 闭包臂经 gather_var_refs 对 Closure 完全不下降——闭包捕获检测为空集 | plan667_a2r_closure_* |
| F-04 | 确认+修复 | 生成侧 current_scope_depth 恒 0（从不递增），嵌套作用域逃逸漏报；兄弟作用域 (depth,name) 撞键；未覆盖 AST 静默忽略 | plan667_a2r_scope_merge / unknown |
| F-05 | 确认+修复 | emit_borrow unknown→&、ArcMutex→.clone()（声明不改写）、.mut 于 Clone 层→.clone()（VM 同场景拒绝）；后两者改为明确诊断 | plan667_a2r_mut_on_shared / arcmutex |
| F-06 | 证实-不改 | ownership（lifetime/borrow/cfa）零生产调用面，独立模块不构成主路径执法；仅作 Spec 状态纠正（SD-01），不接入 | 静态 grep（见 §5） |
| F-07 | 确认+修复 | 防御性补持（STORE_GLOBAL/STORE_STATE_FIELD）按内容判 i32≥4M：真整数与活 id 相撞即冒领份额（泄漏方向）；rc_retain 增存在性门+防御补持收紧到 tagged+stdlib:8636 裸推迁移 | plan667_rc_integer_* |

## 2. VM 捕获矩阵（T-02）

（执行期填写：逐项基线红/修复绿、命令、提交）

## 3. RC 矩阵（T-03）

（执行期填写）

## 4. a2r 矩阵（T-04）

（执行期填写）

## 5. 静态审计记录

- ownership 模块生产调用面：`grep -rn "crate::ownership" src --include="*.rs" | grep -v src/ownership | grep -v src/tests`
  → 仅 ownership_tests.rs（自测）。lifetime.rs/borrow.rs/cfa.rs 未接入编译主路径。
- 裸 heap id 推法存量：`grep -A4 insert_heap_object ... | grep push_i32`（排除 rc_push*）
  → 唯一 stdlib.rs:8636（DefaultBodyLimit.max）；native.rs:7226 为 0 值回退非 id。
- rc_stats 生产调用面：ui/vm_bridge.rs:1470（UI 内存显示）、native.rs:9577（auto.rc.live()）。
- closures 表无 remove/clear/retain 调用——闭包 env 份额随 VM 终身（泄漏方向，安全纪律内，入档）。
- .at 语料无 auto.rc.live/count 使用（统计纯化零金样风险）。

## 6. 支持边界矩阵（SD-04 输入）

（执行期填写：后端/入口 × 三值分类）

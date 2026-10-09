# auto-vm — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 746 | pg-mem-fix | ✅（reviewed→archived） | archive/ | VM 协作式执行预算——AutoVM.deadline: Option<Instant> + set_deadline，run_task_loop 每轮清扫检查、超时终止全部任务置 ExecutionTimeout last_error，默认 None 零行为变化；lib.rs run_with_capture_*_with_deadline 公开面 + join_execution（执行线程 panic→Err）；SD-02 print 族 kwargs 编译期拒绝语义锚点 |

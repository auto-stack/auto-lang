**历史中间记录**：用户已授权恢复 34 个范围外文件并已恢复；后续修复、验证和冻结提交以 [R2 闭合表](repair-closure-r2.md) 为准。

# PLAN-824 R2 F03 修复进行中（未完成）

基面 `6a68240e89bdfd141ecfb893f83c7718bf503056`；工作树 `D:/autostack/.wt/lang-824/auto-lang`，`plan-824-dev`。未提交、未送审，Plan 仍 executing 3/11。F01/F02 历史关闭不变，F03 保持开放。

已做的尝试：执行器类型展开在构造期间检查临时类型字节、步骤、取消和墙钟，并更新峰值；portable 类型写入及已计步路径参与取消；portable 类型读写增加了分配前字节检查；恢复 binding adapter 增加了预估。诊断和原始反例见本目录 `repair-diagnosis.md` 和 `review.md`。这些更改尚未完成完整审计，不能作为关闭证据。

已观察：保存的 R2 重放中长名样例从 Success 改为 SlotBytes，peak 4058/4096；小对照 Success、peak 1867；portable 两条取消均为 Cancelled、polls=1。新增 `av_dynamic_824_r2_type_work_cancel_and_retry`、`av_dynamic_824_resources_r2_binding_long_names_are_bounded` 通过；资源族误用宽过滤器启动 13 项，12 项显示通过（含新增长名及原展开测试），无关 worker 内存采样被中止，整次调用为失败，不能作为全族通过证据。F01 一项及 F02 三项定向测试通过。

未闭合：deep_clone 中 List/Variant 类型复制尚未在复制前计量，恢复过程内派生类型比较未按类型工作轮询；执行器空 payload 宽 Variant 的逐 arm 工作及取消需核对；portable 临时分配、binding 预检计量及支持边界和三条执行器调用方仍需完整审计、真实边界/拒绝/取消/保源的持久断言。当前初稿可能过度保守，需核对合法输入，不可借节点预算隐式降低支持面。受影响测试、格式和最终代码提交仍未完成。merge-owned 全档与独立复审均 pending。

误操作：`cargo fmt --manifest-path experimental/ac-av-probe/Cargo.toml --all` 扫描了根仓库，遇到现存无关语法错误并格式化了原本干净的 34 个 `crates/` 文件。变更完整副本见 `accidental-format.patch`。两次撤回请求被自动审批拒绝，原因是会丢弃 34 文件的未提交改动且需要用户明确授权；因此这些文件保留原样，不可混入提交。后续仅应在用户授权后恢复明确的范围外格式化差异，并重核工作树状态。Plan 824 六个实验文件中的修复保持未提交。

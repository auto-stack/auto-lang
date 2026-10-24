# PLAN-840 r1 planning preflight

2026-10-24；stage:new；草案内容静态检查通过，实施前置 G839 pending。

- Plan：`docs/plans/840-autographics-public-behavior-api-and-user-scripts.md`。
- Plan SHA-256：`e1fbe77ca6c3c80fdfda6b7bc8da3ed060ea28abaad9ce0371061113fd25d44e`。
- 文档结构：0..10 节齐全；T 10 / AC 10 / V 10 / Spec delta 4，ID 唯一；任务 owner_stage = work 8 / review 1 / merge 1；所有 AC 均有任务负责。
- 状态：drafting 0/10；PLAN-840 在 active/archive 唯一；`.next-id` 已为 841；未建立 840 worktree/branch，未实施或准备依赖。
- 来源：观察清单 20 个 Git 源码对象，逐对象以其独立 ref 验证 blob、SHA-256 及长度通过；14 份文档输入 hash 验证通过。其中 3 份不在观察 main HEAD，清单已明确标为工作区输入，不能当已提交证据。
- 前置：839 最初交接 execution_done 8/10；本次调查期间 internal R1 needs_fix（F-839-R1-01）退回 executing 4/10，当前独立 final review、完整门、autographics-dev 落地及知识/清理收尾均未完成。候选 e58828ca64b7e26ce984faf850fb3b1d0887517f 只供规划观察，不是 840 实施基线。
- 语义核对：T-02 为最多两种现有公开 SDK 接法的 whole-path 可行性探针；必须真实 Auto callbacks/API → 导入模型与 2D 图形 → GPU → Stop。同 binary 用户源码改变、同回调 group 写后 child world/effective 读、首 API 错误黏着、句柄 0 合法、Starting 取消意图、原 profile6 等价回归和旧支持面都列入验收。
- 写面：仅 driver 内实现拟议；canonical 本轮不改。单文件来源显式 UI/CLI 选择；Save/Save As 不自动保存或发布脚本，多文件/包及生产集成另立范围；公开 API 不冒认安全沙箱或任意 heap 硬配额。
- 验证分工：work/review 定向测试及检查；最终审查通过后 merge 才在冻结版本跑完整门。本轮没有 Cargo/build/原生/GPU 测试；静态 preflight 不能替代实现或独立验收。
- 附件和相对链接存在性、忽略规则、空白检查在本轮尾检核对；只检查本轮 840 文件及分配计数，不触碰其他会话 WIP。

结论：草案可供评审；G839 完整交付且用户启动 840 work 后，T-01 重新核实并冻结正式 SHA/依赖/Spec，随后执行 T-02。

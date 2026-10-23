# PLAN-835 r1：提前规划预检

2026-10-23，stage=new。此为草案来源与执行合同检查，不是SDK/编辑器/文件保存运行验收。

## 选题与边界

- 当前方向依据v0.6创作目标：在两款真实Auto游戏之后，先形成查看/选择→修改→实时预览→保存→撤销→重开闭环；不继续增加游戏数量。
- 834为reviewed 9/10，独立final R1 PASS，merge门与实际合入仍在进行。本Plan不复用其工作树WIP或把reviewed当交付完成。
- 816尚在途，831/832有明确整Plan启动锁。835不是这些生产迁移任务的提前准备，范围仅在既有experimental工作面；AutoForge前置实验不抵扣生产I2/I3。
- 以[观察清单](source-observation.json.txt)中的834受审提交14份不可变源对象、14份文档来源为证据。SDK state_file helper只作能力/限制调查：atomic_write可评估，with_lock降级执行不能当独占保存保证。
- 规划期间834 Plan由其owner继续更新，内容hash漂移；保留初次观察。最后核验位置仍为活跃区、reviewed/9；最终G834/GSOURCE由后续冻结会话重核，不自动追新tip。

## 合同预检

- 主检出经既有allocation锁独占句柄、脚本 `scripts/new-plan.sh autographics-scene-authoring-save-and-undo` 取号；运行前最大号834/next-id835，之后唯一835/next-id836。Git Bash沙箱初始化限制经工具批准绕开，写入仅新计划骨架和计数器，未改既有锁文件。
- 文档0..10共11个编号章节，10个task全部未勾选，work×8/review×1/merge×1；9个AC、9个V-case、4个SD。每个AC对应task、每个work AC有未来执行case，写面/owner明确。
- T-02使用最小scratch fixture/临时文件，不依赖T-03/T-05尚未建立的正式模型/存储接口；最多两种公开接法，有实证的停止/回owner条件，不砍保存/撤销/真实Auto预览。
- 选择方式固定为对象列表与属性面板；预览由实际Auto配置/受检输出驱动，Rust只负责文档/封送/呈现。新有限authoring profile不改Breakout/Squash协议或golden。
- 固定资源面与预算：2D circle/quad、3D已有cylinder/player/mob；128对象、文件1MiB、64步/16MiB历史。失败是具名拒绝，不静默截断；历史按公布规则淘汰最老步，dirty依内容hash独立于history保留。
- 文档稳定ID、epoch/revision和runtime session/代际分开；迟到字段消息/保存完成/预览输出均有准入，删除undo不复活旧捕获。候选文档/历史/实际画面同时提交，坏输出/资源失败保last-good，下一合法操作须实际恢复。
- Save提交点前失败保旧字节；提交后读回失败为结果待核验，不假装旧文件未动/直接clean。恢复测试必须读回后继续编辑/保存/重开；foreign临时文件不能被模式匹配清理。
- 新进程Open验证场景内容/资源/画面，undo历史仅当前会话；dirty时Save/Discard/Cancel明确，Cancel/坏Open不丢文档。文件不保存VM/GPU ID、历史或截图。
- 现有SDK/生产crates/根Cargo不改；单文件单写者profile不冒称全局协作/断电/网络盘保证。I2/I3、正式Forge仓/发布及Shader/CAP/ACE边界保持。

## 静态检查与交接

已执行章节、task/AC/V/SD行数和映射、编号唯一性/next-id、行末空白及现有链接检查；创建本文件前13个现有本地链接可达，本文件补齐Plan中的preflight链接。源JSON解析通过。交付前再检查所有当前链接、材料可跟踪性和合同hash。

本轮未运行Cargo、SDK/Auto程序、GPU/窗口/音频、文件保存故障/重开或性能测试，未创建835树/分支/agent/聊天，未修改834或canonical Spec/ledger。

`outcome=blocked`只表示不放行实施：G834尚在merge，GSOURCE未冻结，本轮仅提前规划。草案编制完成；下一步为834完整交付后核源/冻结835，用户启动后由work从T-01进入。

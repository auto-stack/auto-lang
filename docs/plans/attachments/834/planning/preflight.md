# PLAN-834 r1：提前规划预检

2026-10-22，stage=new。此为来源/合同静态预检，不是游戏或SDK运行验收。

## 来源与选择

- 现行canonical图形规范保留I1双冻结样本分母、I2生产AutoUI/RQHost和I3 Forge/发行门。833只认领首个Auto Breakout与小场景，故下一份选择Auto Squash，沿用同一SDK桥，不加入新游戏/Shader/Forge等额外方向。
- 833处于review/修复。R1报告显示TextOverlay设备归属和上传计量findings，仍由其原owner关闭；本Plan既不修833，也不把这些问题当作允许延期的债务。
- 只读候选Git对象 `b136129fed9a018f07ea49445868066449eaf5e9` 定位有限协议/consumer缺口；18份不可变Git对象的blob/hash及6份主检出来源指纹在[source-observation.json.txt](source-observation.json.txt)。它们是规划证据，**不是实施基线**。
- 主检出的833 Plan在调查过程中由其owner继续更新，内容hash已漂移；原R1独立报告与候选对象不变。保留初次观察，不复制WIP、不提前认定修复通过。G833最终验收/冻结门/分支交付和GSOURCE仍待后续核验。
- 原Squash数据文件盘点得到3600tick、45spawn、91事件及最多7同时存活mob；仅是样本数据观察，不是本轮回放结果，也不把7设成产品容量上限。

## 合同预检

- 编号由主检出 `scripts/new-plan.sh autographics-auto-squash-and-dual-game-sdk` 分配，既有 `.allocation.lock` 独占句柄串行化；运行前核最大号833/next-id834，完成后唯一834、next-id835。Git Bash初始化受沙箱限制，按工具批准执行取号步骤，仅新增骨架/更新计数器。
- 11个编号章节（0..10），10个task全部未勾选，owner为work×8/review×1/merge×1；9个AC、9个V-case、4个SD。
- 每个AC有task，每个work AC有实际未来case；资源身份、发布事务、RNG序号、物理时钟、输入边沿和音频owner分别有正反控及关闭条件。新fixtures/CLI/API明确由任务建立，不冒称已存在或已跑。
- T-01先准备上游已交付来源，T-02在本树scratch/最小fixture做whole-path，不依赖T-03尚未实现的3D schema；最多两接法，失败具名回SDK/renderer owner，不自行改SDK/砍验收或放宽golden。
- 旧Profile1与Breakout行为保持，新3D协议独立准入；错profile/资源/设备、坏旋转/混合好坏包拒绝，状态/画面/HUD/音频保持last-good。新profile容量与预算由T-02实测决定，不承诺无限资源。
- 玩法由Auto执行；旧Rust SquashDriver仅作为参考/对照。通用数学/原始随机数服务与Rust代算玩法划清边界；变异原Auto源必须使正式oracle失败。
- 沿用原有限物理近似和全部既定精度：逐tick位置0.01、spawn位置/yaw1e-4、velocity1e-3等按原合同；整数事件身份/顺序/计分精确，不临时要求全部浮点bitwise或放宽数值。
- 音画分母包括原GLB与其多mesh/节点、场景级bob/pitch/朝向、HUD/Retry和唯一原背景音乐；无GLB骨骼动画不抵扣场景动画。retry/共享挂载不多开music，关闭宿主最后owner后释放；不添加不存在的音效。
- I1只有两个真实Auto游戏按原分母及公开来源均完成后才审定；I2/I3、CAP/完整ACE与生产迁移门不变。本文没有更新current Spec/ledger。

## 验证结果与交接

已执行task/AC/case/SD计数与映射、章节编号、行末空白、ID唯一性和next-id检查；创建本文件前的10个现有本地链接可达，本文件补齐Plan中第11个链接。source JSON解析通过。交付前再核Plan/本预检所有链接与材料可跟踪性。

本轮未运行Cargo、SDK程序、GPU、窗口、游戏、音频/IME或性能测试；没有建立834实施树/分支/agent/聊天，未修改833。

`outcome=blocked`仅指尚不向work放行：G833/GSOURCE未完成，且用户本轮只要求提前规划。草案编制完成；下一步由833 owner完成原review/修复与验收合入，之后冻结834来源，用户启动后从T-01执行。

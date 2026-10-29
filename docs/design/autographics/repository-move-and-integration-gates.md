# AutoGraphics源码归仓与AutoUI/ACE融合的分离门

> **2026-10-29最新拆分**：用户批准[auto-ui/002](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)仅引擎，G11完整移交[ag-examples/001](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)，硬等002实际交付。两份仅修合同、暂不实施；旧883混合分工为历史。

> **计划管理归属更正（2026-10-29）**：AG引擎计划在auto-ui/docs/plans，Gallery独立内容计划在ag-examples/docs/plans；跨仓计划由主责任仓持一份合同。882/883已迁至auto-ui（drafting r2），880最新收据已交付归档并迁至ag-examples。auto-lang原位置只留定位指引，历史Specs/设计本次未批量迁移；下文早期“中央Plan仍在语言仓”表述已由本条取代。

日期：2026-10-29；状态：[879纯迁仓](../../plans/archive/879-autographics-source-move-to-auto-ui.md)已交付，当前AG单一源码权威为auto-ui/experimental（4adf2d03）；语言autographics-dev=f07b5ec8仅历史/归属指引。实际映射/SDK保全/冷准备/隔离以[canonical Spec](../../specs/auto-graphics/source-repository-and-sdk.md)为准。下文10-28候选讨论保留历史，不再表示迁仓尚未执行。

后续新AG引擎能力在auto-ui实现，中央Plan/知识暂留语言仓；[882对象](../../../../auto-ui/docs/plans/001-autographics-runtime-object-lifecycle.md)→[883触发/新局](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)已规划，880独立内容树继续消费旧875。默认生产UI/声明树/HIR/ACE融合、SDK替换与切默认/退役仍各有独立门，纯迁仓不放行它们。

## 1. 已观察的现状

- [auto-ui独立仓Spec](../../specs/auto-ui/project.md)确认831迁仓落定，当前workspace为dialect/compiler/runtime/tools/shell-pack五包，语言SDK继续单源在auto-lang。831 archived 12/12，842 archived 17/17。
- 只读目标auto-ui master `7095f536747e5180338a470a7d73452c6b550ed1`，规划时工作树clean；其Cargo.toml为`members=["crates/*"]`，runtime依赖语言SDK/UI dialect与Iced0.14系。根README仍描述旧目录，实际manifest/来源Spec优先。本轮未重新运行其构建/测试，不据此保证所有平台/发行面已无问题。
- AG仍为autographics-dev `8b12632d7b5b1b0df72fbb4eef84297a054de31d`下两个experimental独立workspace，各有锁；driver消费固定旧SDK副本，AGP使用stock Iced/wgpu。依赖版本相近不等于已共享同一个生产UI设备/资源owner。

## 2. 应分开的四件事

| 动作 | 改变什么 | 主要前置 |
|---|---|---|
| 纯源码归仓 | AG实验源码/测试/工具/小夹具与开发权威从auto-lang移到auto-ui | 固定迁移清单/来源映射、默认构建隔离、固定SDK/资源、两端验证与单一权威切换/恢复 |
| UI/Graphics运行与声明融合 | 组件/场景/挂载/输入、共同声明树和真实UI host | UI领域/组件/资源合同、实际编译执行环境及兼容/主题/嵌套验收 |
| ACE替换与增量/HMR | 替换旧SDK执行、代码/状态代际、编译缓存/资源重绑 | ACE实际成熟与真实脚本支持；独立增量/状态/HMR门 |
| 切默认/退役 | 用户默认路径改变、旧入口移除 | 兼容矩阵、回退窗口、实际消费者迁移与明确退役合同 |

纯仓库归属**在技术上不必等待ACE完成**：它可以保持当前旧SDK、显式实验入口与所有运行语义。但它仍是有验证义务的跨仓切源任务，不是复制文件或一次merge即可宣称完成。后面三项继续依原成熟度门讨论，不因搬进auto-ui仓自动放行。

## 3. 建议窗口与和此前决定的关系

建议时机：**874/875实际交付并冻结后，下一批广泛修改公共运行协议之前**，另立纯搬仓Plan。此时角色/相机有完整来源/验证收据，短暂停写比在两计划review/fix期间同步搬迁风险小；也无需等整个M2或完整ACE才能讨论纯归仓。

这是一项对先前“图形归仓和生产融合都等ACE”的**排期调整建议**。用户本轮询问时机并要求规划877/878，未要求现在迁移。因此现行开发权威仍在auto-lang/autographics-dev，既有冻结Plan/门不自动改变；本轮只把建议分清，后继纯迁仓执行合同需再明确切源范围/窗口，不能用本页代替实施授权。

[878 XP-0](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md)先只读评估搬仓触面与目标边界，不是迁仓任务。[877 GY-A](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md)仍在独立ag-examples仓固定消费861；迁仓不应要求其改玩法代码或立即追新引擎。

## 4. 首次归仓的推荐形态

候选位置为auto-ui仓的`experimental/autographics-prototype/`与`experimental/autographics-auto-driver/`，继续独立`[workspace]`/锁/旧SDK内容身份，保持兄弟相对AGP依赖。先避免加入`crates/*`默认成员、合并锁、替换stock Iced、接生产auto-ui-runtime或迁入新语言SDK。未来正式crate分解/渲染集成另立Plan，不把纯搬仓做成重构大包。

候选布局不保证现有所有脚本/身份工具原样工作；迁仓Plan必须盘点并修正明确触面，尤其：固定源root、当前identity/导出清单、SDK来源、构建栈配置、测试夹具、asset根、受控runner来源和文档链接。保留旧身份/收据的历史含义，新增“旧repo＋commit＋path→新repo＋commit＋path”的内容映射/新来源身份，不能把旧commit当新仓祖先或重写原hash来装作没迁移。

完整示例/游戏仍在独立ag-examples及各游戏仓；旧M1资产/必要回归夹具不因体积推测随意删除。原始复审日志/截图/target不搬成源码，不把本地SDK证据副本误当生产包；固定SDK应在新仓脱离原工作目录后仍可按验证锁重建/消费。

## 5. 搬仓前与切源门（未来合同输入）

1. 874/875终态交付、源/SDK/资源/两锁/测试ID与实际完整门绑定，相关实现树不再有待修复提交。
2. XP-0提供依赖/导出/路径/平台特有触面与auto-ui目标隔离清单；审计未测项明确，迁仓不借机启动跨端适配。
3. auto-ui实际owner/default workspace/语言SDK边界确认，当前UI消费者不新增AG依赖/构建负担或循环。
4. 独立迁仓Plan明确两仓唯一树、停写窗口、迁入/切权威/旧源退出顺序及可恢复快照；不长期双写，不在877/878里顺便移动。
5. 新仓AG独立consumer/资源/两端完整与受影响native/owner/content门、auto-ui默认边界检查、Gallery固定消费及A→B搬迁验证通过；迁仓前后所有身份/资源/原分母可追溯。
6. 实际切权威后auto-lang仅保留设计/历史/新归属索引或显式兼容定位，不能留两套继续演进；待恢复/历史证明保全后再按guard清理旧实现树。

这里只给后继合同输入，不宣称当前门已全部满足。纯迁仓若获安排其成本单列，不隐入原M2功能数；当前不预占第三个Plan编号，也不修改auto-ui工作副本。

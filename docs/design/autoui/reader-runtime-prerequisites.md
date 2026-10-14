# Reader 真实阅读链路的框架前置设计

状态：draft；2026-10-14。来源：用户要求在 auto-lang 立项，承接 READER-001 r7 保留的三项验收目标。本文是拟实施合同，不表示能力已交付。

## 1. 目标与仓边界

READER-001 仍为 executing/r7、16/32；混排段落恢复、10MiB 成功导入和原生 VM 实际恢复不能用启动、错误保护或文档登记替代。框架前置拆成三个独立 Plan：

| Plan | 框架交付 | Reader 承接 |
|---|---|---|
| [781](../../plans/781-scroll-child-anchor.md) | ordinary scroll-pane 按真实子内容锚点定位；Vue/iced 同源语义 | T-25/F-12，接入 reading.at 并启用混排门 |
| [782](../../plans/782-reader-import-execution-budget.md) | 可信宿主有限执行预算及必要的可续跑能力，真实 10MiB Reader 导入可完成 | T-26 的成功导入、受管副本与内容一致性门 |
| [783](../../plans/783-native-reader-acceptance-channel.md) | 原生 VM 独立窗口的动作、布局观测、重启与可重放验收入口 | T-26 的原生恢复证据；支撑 781 原生框架验证 |

实施范围是 auto-lang 框架与验证资产；Reader 现有源码只读作为 consumer，可在其专用验收工作位运行隔离实例。业务接入/修改仍回 READER-001；不复制修改教学 demo、不修改用户真实书库、不替 Reader 授予 reviewed/archived。

## 2. 现状与已有设施

- [scroll-pane Spec](../../specs/widgets/scroll-pane.md) 将 ScrollAnchor 留作未来消费者；已有 controller + offset/extent、意图队列和双端 adapter。字符占比不足以证明不等高段落可见。
- vm/engine.rs 的 Legacy 分派仍是 10M 指令单段限制；CpuSliceBudget 与累计 10M 护栏是 UI 路径。HTTP 同步/闭包/段驱动要逐项核对，不能只改 CPU_CUMULATIVE_STEP_BUDGET。
- AutoUI HTTP MCP 已有 autoui_action/state/vtree/select_rect/screenshot；test_vm_mcp.py 与 acceptance_channel.py 已能驱动部分原生宿主。缺口是 Reader split VM+HTTP 的可靠编排、段落动作覆盖及新鲜可见几何证据，而非笼统的“没有原生自动化”。Select Anything 的中心包含语义不能用作视口交集判据。
- 在途 PLAN-776 正处理 HTTP 返回资源；Design 37/PLAN-778–780 正处理 UI/host 边界。新计划遵守一个共享热点一个写入者，在接线前重核实际 landed diff；计划调查不等于获得并发修改热点的授权。

取材基线、文件 hash 与工作区状态见 [立项证据](../../plans/reports/reader-prerequisites-planning-20261014.md)。旧 Spec 的总览快照不覆盖实际近期源码。

## 3. 子内容锚点

提出 scroll_to_child(handle, anchor_key) 与子节点 scroll-anchor 绑定的等价接口；确切拼写在 781 T-01 冻结，不能把提案当作已有 API。普通 pane 下允许 col/row 包装，锚点按 pane 作用域寻址，稳定非空串可映射 Reader 原始 paragraph_index，避免过滤空行后序号错位。

定位来自当前真实布局，目标选择 nearest：已可见不动，否则使目标与 pane 有正面积交集；高于视口的长段至少可见其起始区域。首帧未布局的请求等待布局，换章/销毁不能消费旧目标。重复、空、失效或不存在的锚点应有明确失败观测，不报“已经可见”；入队与完成必须区分。后端坐标变换、嵌套裁剪和 controller 回声不能改变语义。此阶段不承诺虚拟列表、terminal managed content、持久化 ScrollAnchor 或通用跨页锚点。

## 4. 10MiB 有限执行

支持档位为真实 UTF-8 TXT/Markdown 本地路径导入，文件 10×1024×1024 字节，含中文、ASCII 和多章节；在 Reader 既有 API/存储链路核对字节与章节正文。请求 JSON 仅包含路径，不应把文件大小误当 HTTP body 预算。

782 T-01 先做有界 whole-path 可行性探针，定位耗时/步数在读文件、解析、hash、章节拆分和写盘的实际分布，形成唯一方案。优先宿主可配置的有限请求累计步数；配置只由可信宿主传入，每 VM/请求快照隔离，不让请求 payload 任意抬预算。缺省保留 legacy 10M；0、负数、非整数、溢出或非法设置明确拒绝，不取消 runaway、请求期限及资源回收。

若同步跑法使既有取消/期限无法生效，复用 CpuSlice/continuation 做请求级分片；跨片和嵌套调用不重置累计预算，不把 Runnable 当成功。明确 HTTP handler、UI handler 与普通脚本的覆盖边界。测试 profile 的有限数值、请求期限、fixture hash 在探针决策中冻结，之后不能为了通过任意放宽；真正超出该档位需修订合同。需要新增业务级上传/流式导入协议则停止相关接线并另提范围变更，不能偷偷改 Reader。

## 5. 原生验收入口

783 复用既有原生 MCP；先启动独立 VM frontend + VM HTTP backend，确认实际 pid/项目/数据目录/MCP 和服务端身份，再从实际 VTree 选择动作节点。允许隔离书库种子建立前提，保存与恢复必须经过产品真实 handler/API，fixture 直接注入已恢复状态不能作成功证据。

后验通过当前渲染帧的目标矩形、pane viewport 和祖先裁剪交集证明可见，必须辨别陈旧/未测量/零尺寸/隐藏/错误节点。若现有 bounds 只在 debug 下采集或没有帧新鲜性，只补最小 gated 观测接口，不重建通用自动化平台。动作 ack、状态与几何来自同一实例，恢复后驱动不得代替产品滚动；截图作辅助证据。冷启动重新拉起受控实例，用户已有进程不属于驱动收尾对象。

## 6. 顺序与交付

三个计划可独立评审；推荐先 783 建立可用的原生 oracle，再 781 双端锚点，最后 782 whole-path 导入；调查可提前，共享生产源码接线串行。783 的通道和负例不依赖 781，已有 Reader 均匀内容恢复可作为真实正对照；混排框架最终验证在 781，其实际 Reader 接入留给 READER-001，避免环依赖。

各计划分别交付实际 commit/CLI 文件 hash、接口/配置说明、命令、原始结果与覆盖边界。三份前置 pass 并不自动使 001 完成；Reader 将框架接入、取消混排 fixme、重跑 10MiB 和原生行为，再独立终审。本次仅立项，执行尚未开始。

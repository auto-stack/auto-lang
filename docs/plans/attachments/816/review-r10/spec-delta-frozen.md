## 5. Spec 候选（SD-01 A 子集；T-14 受管沉淀，本节为内容候选）

目标文件 `docs/specs/tool-execution/project.md`（新增），A 子集规则：

1. **owned 执行设施**（T-02 已落，随本阶段一并沉淀；R1 修复后语义）：子进程执行的唯一设施 = `auto-tool-execution`（argv/cwd/env/stdio 策略/退出具名分类/deadline/cancel+收尸/reader 真终止/树退出）；deadline 覆盖整次执行（stdin 写入经 writer 线程、根等待与收尾共用同一绝对截止）；根退出后仍持管道的 owned 后代先终止、reader 有界收尾；`kill 发出 ≠ 已停`——观察到真停（收尸+reader 有界收尾）才返回；reader 读错/panic 与 writer/终止失败具名传播（`ReaderIo`/`ReaderPanicked`/`PipesHeld`），Windows 树归属在子进程任何用户代码运行前建立（挂起启动+入 Job+恢复）。
2. **I-TOOL 客户端**：客户端经真实进程消费工具宿主（单源帧/信封），不静态链接工具实现；宿主拒绝是数据（`Reply::Host`），客户端故障具名分类；控制流（stdout）仅帧且先 drain 已缓冲内容（完整帧与坏前缀不等待新字节）、日志仅 stderr，非帧字节 = 具名违规且流终态；请求 deadline 覆盖发送+等待，到期 = owned 树终止+收尸+writer/reader 有界收尾后才返回 `Timeout`（终止/收尸失败与线程 panic、管道被非拥有进程持有具名传播，不谎报）。
3. **发现边界（A 现状）**：客户端只接受显式工具路径，无隐式搜索；发现/bootstrap/安装 profile 规则等 M814-A 发布后的整链 probe（T-03/T-04）与 M815-B（T-04 完整正控）再定稿补入。
4. **测试分层**：crate 内联测试纯数据面+故障注入面（假 Read 驱动，无 sleep）；真实进程正反控在独立 fixture（driver 先构建二进制再以 env 钉定）；寿命反例用自带截止的具名测试子进程（te-child：sleep/grandchild/orphan/probe）。

**SD-05-A 处置**（R1 复审要求明确给出，替代此前"SD-02..05 整体排除"的一揽子表述）：
目标 `docs/specs/repository-boundaries/generated-ui-distribution.md` 增一段精确补文（T-14 受管沉淀），内容候选：

> **工具消费方式（PLAN-816 M816-A 起）**：`auto-ui-tool` 及后续 I-TOOL 工具宿主只能经**真实子进程 + auto-tool-protocol 帧**消费（客户端 `auto-tool-client`；闭包由 `test/fixtures/modularity/tool-client-consumer/verify-deps.sh` 钉住：无 auto-ui-tools 静态链接）。M816-A **不改变任何发行/安装闭包**：792 `DELIVERY_CRATES` 与生成 UI 分发清单维持 M815-A 现状——客户端是 workspace 库，供后续薄 CLI 组合（T-05+）消费，不作为独立发行件；独立安装 profile 的闭包增补属 B（T-07，profile 冻结门 = M814-A 后整链 probe）。

即：现有 distribution Spec **未覆盖**"显式进程消费 vs 静态链接"这条边界（需上述补文），但**已充分覆盖**"A 尚无安装变化"（分发清单未动，如实陈述即可）。SD-02/03/04 仍不在 A 沉淀范围（CLI 路由/服务分层/搬迁均未发生——这是各 delta 自身的现状理由，不是 SD-05 的替代理由）。


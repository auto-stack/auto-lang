## 5. Spec 候选（SD-01 A 子集；T-14 受管沉淀，本节为内容候选）

目标文件 `docs/specs/tool-execution/project.md`（新增），A 子集规则：

1. **owned 执行设施**（T-02 已落，随本阶段一并沉淀）：子进程执行的唯一设施 = `auto-tool-execution`（argv/cwd/env/stdio 策略/退出具名分类/deadline/cancel+收尸/reader 真终止/树退出）；`kill 发出 ≠ 已停`——取消/超时后观察到真停（收尸+reader join）才返回；只终止本次拥有的树。
2. **I-TOOL 客户端**：客户端经真实进程消费工具宿主（单源帧/信封），不静态链接工具实现；宿主拒绝是数据（`Reply::Host`），客户端故障具名分类；控制流（stdout）仅帧、日志仅 stderr，非帧字节 = 具名违规且流终态；请求 deadline 到期 = owned 树终止+收尸+reader join 后返回。
3. **发现边界（A 现状）**：客户端只接受显式工具路径，无隐式搜索；发现/bootstrap/安装 profile 规则等 M814-A 发布后的整链 probe（T-03/T-04）与 M815-B（T-04 完整正控）再定稿补入。
4. **测试分层**：crate 内联测试纯数据面；真实进程正反控在独立 fixture（driver 先构建二进制再以 env 钉定）；寿命反例用自带截止的具名测试子进程（te-child）。

SD-02..05（auto-cli/auto-man/http-service/generated-ui-distribution）不在 A 沉淀范围（薄 CLI 路由/服务分层/搬迁均未发生）。



# PLAN-842 R4 修复与最终重新验证

本轮用户授权的第三个自动修复循环：R4 复审发现 Cargo 普通依赖环、包身份和版本被漏判；诊断、Cargo 实测语义及循环计数见 [诊断](diagnosis.md)，独立原始反例见 [复审 R4](../review-r4/report.md)。最终来源 lang `4ba5b39570a034aa7a262bf3f3a187099dbcdd5d`、UI `171454071b8e19e0287f549ceb4f5b2ee87e0087`、Man `d11660ed74acfe15930e25f2bbcb746f77fcadb0`，down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；lang 并集基面 `a547f52c5fbd5a24a203e94051af1e040c808f42`。三仓分支干净；最终独立审查仍 pending。

非受管语言/UI 开发源按 workspace 继承与 target 各 dependency 表遍历本地包，比较声明依赖键或 package 重命名与真实包名、semver 版本，并对普通/构建边检测环；dev 边参与包有效性遍历但允许构成 dev 环。正式交付在写入前也调用相同语言来源预检；受管 SDK 继续采用安装收据与 staging 副本验证。注册表不在此步骤联网。原两个反例修复前先红后绿；源码定向测试 [4/4](r8-source-tests.log)，UI 7/7 与实际 delivery 1/1，`cargo check -p auto-lang`、三仓 diff-check、修改的 Rust 文件 rustfmt 和 Python 语法检查通过；基线 warnings 仍在。

以最终 Full 安装真实 CLI 做普通环、错误包名、错误版本、UI 包错身份，逐个在已存在的开发工作区和正式交付入口拒绝且旧 app 和清单保留；合法 dev 环与 package 重命名，两入口成功，实际 Cargo metadata offline 成功。工作区继承正确路径/版本生成和 metadata 成功，路径改成错误包后生成拒绝、旧文件保留，Cargo metadata 101；独立复现脚本 [cargo-graph-probe.py](cargo-graph-probe.py)、[摘要](r8-graph-cargo-oracle.log)、[错误生成](inherit-bad-gen.log)、[Cargo 对照](inherit-bad-cargo.log)。正式生成、反例与恢复逐项见 [实际门记录](r8-final-delivery.log)。

Full 安装 6806 文件，manifest SHA-256 `fb0da2b37a6c0d8aa98aa4577a691e167cfa9172e0aa30b9579cef1a82cf0e33`；实际 fresh lock、locked build、宿主入口和树、受管源码额外/缺失/篡改、根锁/patch、邻接及 detached 缺收据、[复制期漂移](staged-drift.log)、失败重试备份和恢复、verify 均通过。Core 安装 636 文件，manifest SHA-256 `2493769a477aa6c80c6cd930034bcced90ac393dd58111bb0f3b831a0eed920a`；安装 verify、实际语言 VM 和 UI Full 提示、SDK locked check、独立消费者 fresh lock/locked 3/3 通过。最终安装 Vue 真实生成和生产构建通过。非受影响的资源同长度篡改/未来 schema、blueprint 邻接与旧安装升级等证据沿用 [R3](../repair-r3/report.md)。

[最终选择摘要](final-selection-summary.json.txt) 绑定当前来源及 list-only：language 3212、UI 3117、tt 3563、tb 3102，21/21 被迁移 ID 的新目标均实际 selected；包含原始清单 hash。测试驱动 18/18；原 9 recipe、10 token、2 parser 定向执行代码及测试配置未改，沿用 [R2](../repair-r2/report.md)。[测试 ID 差异](../t08-audit/test-id-diff.json) 重算绑定本次 HEAD。测试选择不代表全档已执行。

环境事件：D 盘耗尽导致首次 Full 安装链接、测试清单及 Core 消费者构建失败；保留了失败日志并将本计划可重建增量缓存原样移至 C 盘，释放空间后以上受影响检查全通过。现存 warnings、MPV 真实 DLL/播放未实测和原 ignored 限制保留；其他主检出 WIP 未动。最终独立 R5 未运行，若 non-pass 必须停止自动修复并请求明确扩大循环上限；merge-owned 全档、到期批量回归和实际合入归档均未执行。

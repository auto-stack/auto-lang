# PLAN-815 T-05 R1 定向独立复审

**结论：needs_fix。七项 OPEN（P1 三项、P2 四项）；T-05 重开，Plan executing 10/17。** 本轮是 T-05 的 phase 提前审查，不是 M815-B/T-09 全计划终审。没有修实施、整档测试、合入、发布 Spec/ledger、归档或清树。

## 基线与权限

- Plan r3；主检出读取最新计划；唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`。
- 实际受审 HEAD `31a25f5cf0e40c42617140768cd2488df7b55257`，基面 `13b41e9d47cfe62768f68f73b658c3c92b9d3d48`。交接 `b0cbae18b986496c6db54ffcbbd0fb085da33c21` 与实际 HEAD 仅 `.config/nextest.toml` 空行 amend 不同；使用实际提交，未把旧 SHA 自动当成新证据。
- M814-B `93a25c924d1beafab130588bad4d1041dc86e48c` 已发布且是 HEAD 祖先，814 恢复闭环 `7a2323cc3` 已读；本轮不是外部 runtime 未交付阻塞。
- reviewer：Codex `/root`，延续独立审查上下文，有早期合同/账本参与限制，未参与 T-05 实施；从代码、真实入口与原始记录重建结论，未创建 agent/第二树。
- [baseline](baseline.json.txt) 绑定全部改动文件、根清单/锁/配置/777、计划/合同与相关 Spec；[终核](final-baseline.json.txt) 确认 HEAD、branch、clean 均不变，所有已绑定输入零漂移。真实 tool SHA256=`3410f74b127ffd2ee160d938b2ac1b654b3e911caedc47bc0980940753998b57`。
- 主计划 T-05 行引用“§9 八项 work 偏差记录”，但当前主检出及树内计划 §9 均未找到该 T-05 work 记录；没有替实施者补造记录或据此批准偏差。送审修复须补齐真实交接资料。
- [冻结规范增量](spec-delta-reviewed.md) 核准 SD-01..06 的当期边界；没有完整 B 草稿可视为已最终审定，T-08/T-09 保留。

## 逐 AC / 当期状态

| AC | T-05 当期结论 | 依据/范围 |
|---|---|---|
| AC-01 | partial | run_vue/run_vm 实体迁入 UI 域，Man 只转发；旧公开兼容/媒体正控通过；完整生成/支持矩阵仍归 T-06/07，不重开 T-04 已闭原例 |
| AC-02 | partial | 轻请求经 wire 输入，19 codec 与请求缝通过；本轮未改变请求来源合同；全字段/后端组合仍归后续任务 |
| AC-03 | **fail** | F01..05，实际进程 owned 子树、输入背压、VM 取消/EOF、错误退出、Rust 结果分类均有反例；work 已承认的 Launched/交互 quit 成功正控不能以 T-06 注记替代 T-05 的 V-03 完整范围 |
| AC-04 | **fail（已触及部分）** | F06 777 strict 红、F07 现有独立消费者 2/4 红；T-05 改了运行/发行投影，旧 T-04 792/闭包证据不能自动覆盖；全矩阵 T-06/07 pending |
| AC-05 | partial/pending | 净改动 Rust 格式与 diff-check 通过；22 条 ui-tools 编译诊断源行在基面已存在（warning-attribution.json.txt），未声称零全仓 warning；完整终审仍 T-09 |
| AC-06 | pending | 最终 pass 后由 merge 执行冻结裸 t + 必要触面档及实际发布/受管沉淀收口 |
| AC-07..12 | 既有阶段历史保持 | A/L 已发布未回滚；本轮无权把新 B 行为写成已发布，也不以历史窄 A pass 覆盖新运行路径 |

## Findings 与关闭 oracle

### F-815-T05-R1-01 / P1 / owned 子树取消后仍活着

**类型：原合同缺口；AC-03/V-03，T-05。** V-03 要求取消后 owned 子进程/reader 退出。`proc_host.rs:62-65` 只对直接 Child 调 `kill()`；Windows 的安装入口实际是 `cmd /C pnpm...`，真实工作进程可为其子孙。

- 入口/输入：`python protocol-counterexamples.py` 的 `cancel-tree`；合法 Vue Run 请求见 request-False.json.txt。生产真实 tool 完成真实 Vue 生成、进入安装；仅 pnpm 安装进程由 PATH 指向受控真实 OS 进程（cmd→Python→Python TCP listener），不是伪造 UI 成功。
- 已到达状态：收到真实 `install dependencies` progress，parent/leaf PID 文件存在；2 字节输入收到 InputAck。取消收到 Cancelled、最终 RunOk.outcome=cancelled，后续 Describe 正常。
- 期望/观察：owned 子树及 reader 收口后才最终完成；实际最终回执后 parent/leaf 均 alive，leaf TCP 仍可连接。见 protocol-counterexamples.json.txt、raw-process 的 helper/PID/安装 stderr。
- 已证实原因：登记/kill 只覆盖直接 Child，没有为这条 cmd→孙进程路径建立所有权终止/等待；读线程又是 detached。没有把“已 kill 根”当成“整树/reader 已退出”。reader 本身未单独采线程 ID，不能额外声称测得其存活时长。
- 关闭：真实工具入口覆盖安装和已启动服务、带后代与持有 stdout 的路径，cancel 和 EOF 均验证所有 owned PID/端口消失、reader 结束、最终回执与新请求不串流；保留普通结束正控。共享执行设施已有 owned-process 能力，可评估复用；不把某个未经验证的实现方案写成唯一修法。

### F-815-T05-R1-02 / P1 / 合法 stdin 输入阻塞整个控制循环

**类型：原合同缺口；AC-03/V-03，T-05。** `host.rs:570-581` 在协议循环和 registry 锁范围内直接调用 `proc_host.rs:76-79` 的阻塞 write_all/flush。

- 同上真实生产安装等待点，request-True.json.txt；子进程明确不读 stdin。发送 1 MiB 字节数组，JSON 约 3 MiB，低于 8 MiB frame 上限，没有发畸形/超限帧。
- 期望：被接受范围内的输入/拒绝不能令独立 cancel、describe 和 EOF 永久不可处理。观察：InputAck、后续 Cancelled、最终结果及 Describe 全部无响应，进程及 owned TCP 仍活着。等待窗口为 1+2+2+1 秒；不是声称合同承诺了这些秒数，而是同步 write_all 在无人读取的管道上无退路，控制循环无法再读下一帧。
- 正控：2 字节以及先前 64 KiB 的同状态输入均收到 InputAck（64k 原始记录保留），排除无 foreground/坏 request/codec 未到达。初版缺 output_root 的探针未到达 run，原记录保留在 initial.json.txt，只作无效探针，不计产品缺陷。
- 关闭：实际 tool + 不读取/缓慢读取/正常读取的 owned stdin，合法大输入下 cancel/EOF 仍可前进，输入队列/拒绝/完成语义可核验；不能只把这个样例改成小输入或单测 helper。与 F01 分开验证，修子树 kill 不自动证明控制通道可达。

### F-815-T05-R1-03 / P1 / 真实 VM UI 无法取消，EOF 也不退出

**类型：原合同缺口；AC-03/V-03，T-05。** Describe 对 run_ui 宣告 cancellable=true、shutdown_on_eof=true；`host.rs:216` 直接在主线程进入阻塞 Iced runtime，取消/EOF 只改 RunIo/清 hooks，没有结束这个运行中的事件循环。

- `python vm-cancel-live.py`，真实 VM widget 请求见 vm-request.json.txt。收到 vm interpreter progress；stderr 有 `AutoUI MCP: first state sync in view()`，确认真实 Iced 已进入 view，不是仅排队或编译失败。
- 实际收到 Cancelled 后 Describe 仍正常；没有 RunReport，真实 UI 继续运行；随后关闭协议 stdin，等待 3 秒仍存活。见 vm-cancel-live.json.txt。取消后先等 2 秒；源码阻塞关系证实要靠人工关闭窗口才返回，不能把有限观测伪称无限等待测量。
- run_vm.rs 开头只承诺“步骤边界取消”的源码注记不能缩减 V-03 的完整取消合同，也不能覆盖描述符承诺。上游 runtime 已交付，不能将本轮接线缺口归为未发布依赖。
- 关闭：真实 VM 已呈现状态下 cancel 与 EOF 都能收口 UI/后端/工作者、返回一致结果和退出码；开始前取消、自然关闭及失败路径保留；同类原生/分离后端分支需按实际支持矩阵检查。若确需改变对外合同，先修订批准，不能悄悄删能力绕过现有约定。

### F-815-T05-R1-04 / P2 / 工作者控制输出致命失败被当成成功 EOF

**类型：新增错误传播回归；AC-03/V-03，T-05。** `host.rs:519` 与 `549` 丢弃 respond 错误，progress/Launched hook 亦未把失败收口到宿主；EOF 路径 store(0) 无工作者失败优先级。

- 同样关闭真实 OS 控制 stdout 接收端：Describe 正控记录 fatal control write、进程 exit=1；合法 generate_ui 请求（未服务 jet，真实工作者生成具名错误响应）也记录同一 fatal，但关闭 stdin 后 exit=0。不是坏 JSON，也不需要 UI 成功才能验证协议写错误传播。
- 证据：protocol-counterexamples.json.txt 的两个 closed-control case；stderr 原文与控制 pipe 一致。期望 internal/control write failure=1，与描述符退出合同一致；观察 worker 分支把致命错误降成 clean EOF。
- 关闭：Generate/Run 最终响应和 progress/Launched 写失败均终止/取消 owned 工作且退出非零；正常 EOF=0、坏 frame=2 正控保持；并验证 EOF 与工作者失败竞争时不会覆盖非零。不得仅给日志换文字。

### F-815-T05-R1-05 / P2 / Rust 接受取消却最终报告 failed

**类型：原合同缺口；AC-03/V-03，T-05。** `rust_ui.rs:5812-5813` 将被取消杀死的 cargo 非零状态当作普通失败；`tool_ops.rs:132` 只识别精确取消 sentinel，因此未映射为 Cancelled。

- `python rust-cancel-result.py`；真实 tool Rust dispatch，私有生成缓存中的最小工程使流程进入 cargo 等待点。只将 cargo.exe 替换为文件复制的 Python 可执行程序和受控长等待脚本，marker 确认真正进入该等待点；这是退出/取消负控，不充当真实 Rust UI 编译/运行正控。
- 观察：cancel id201 得到 Cancelled(target200)；随后 id200 RunReport.outcome=failed，error=`Cargo run failed with status: exit code: 1`。见 rust-cancel-result.json.txt。同轮 Vue cancel 对照结果为 cancelled。
- 关闭：Rust 的取消/EOF 与 Vue、VM 的最终报告分类一致；取消在 cargo/build/运行不同等待点可观察；普通 cargo 真失败仍是 Failed，不可把所有非零都改为 Cancelled。

### F-815-T05-R1-06 / P2 / 新依赖后 777 基线失效

**类型：验收输入漂移；AC-04，T-05/T-07。** `crates/auto-ui-tools/Cargo.toml:22` 新增 runtime 清单边，未更新受管 777 锚定输入。

- 实际 `python scripts/modularity-audit.py --root . --mode strict --output <report>`：strict fail，1 violation/3 grandfathered；唯一违反为 stale_baseline，detail 列出锚定输入 auto-ui-tools/Cargo.toml 已变。详 777-report.json.txt/777.log.txt。
- 该 runtime 边是允许方向；没有证据证明新增了禁向环。不将 814 遗留 app_registry ownership warning 或 man_to_language 统计漂移误报为本轮新错误。
- 关闭：先完成代码/清单修复，再经受管通道显式刷新并核对基线/允许边/既有禁向 grandfathered，真实 strict 通过；不得放松规则或沿用旧 strict pass。

### F-815-T05-R1-07 / P2 / 已发布窄能力的独立消费者验收未同步

**类型：验收夹具过期；AC-03/04，T-05/T-07。** `test/fixtures/modularity/ui-tool-consumer/src/lib.rs:121,136-142` 仍断言无 generate_ui 且能力列表恰好两项。

- 用当前树真实 bin 和当前 lib，独立 fixture `cargo test --offline --manifest-path test/fixtures/modularity/ui-tool-consumer/Cargo.toml --lib`，AUTO_UI_TOOL_BIN 指向本轮真实 bin、CARGO_TARGET_DIR 指向树 target：实际 **2 passed / 2 failed**（consumer_reads_tool_descriptor_from_lib、consumer_describes_real_process），详 independent-consumer.log.txt。
- 新 capability 是合法协议增量，这不证明产品新增能力本身破坏 A 兼容；证明原独立验收仍冻结 A-only 假设，不能把旧 4/4 或单纯依赖闭包 PASS 当成本轮独立消费通过。
- 关闭：同步独立外部 consumer 的能力断言和实际 B 请求正/负控，保留 A describe/validate/version 拒绝/无副作用兼容断言；不得删掉能力、跳过两项或只断言进程 exit0 以消红。

## 已执行与未执行

- scoped-tests.log.txt：19 protocol + 20 常规 tool_process + 5 proc_host，共 **44/44**；nextest 定向筛选（package(auto-tool-protocol) 或 tool_process binary 或 proc_host::tests），不是 cargo t 整档。
- generation-real.log.txt：串行单条 ignored 真 Vue 生成 **1/1**，磁盘产物/报告/进度及 EOF=0 检查通过。
- man-compat.log.txt：旧公开路径、plan811 三项、请求 seam 两项 **6/6**。前述合计 **51 条通过**；另独立 consumer **2/4**，如实分开。
- protocol-counterexamples / vm-cancel-live / rust-cancel-result：以上生产入口反例，控制帧仍可解码，人工诊断/子进程输出未混入控制帧。
- 净改动 Rust rustfmt --check 与 git diff --check 通过。依赖/迁移主体已有 warnings 未冒充本轮新增；22 条 ui-tools 诊断源行均在父基面存在，归因见 warning-attribution.json.txt。没有因为总 warning 数而臆造 finding。
- 未运行整档 t/tv/tt/tb/tu/taa/tf/docs_gen。未把 T-04 792、历史 A 窄 4/4 或工作者自检替代本轮新增运行面。T-06/07 的真实多 profile build/run、Launched/交互 quit 成功场景及新发行投影完整实际证据仍须工作阶段收齐，T-09 再做完整终审。
- 脚本/结果均在本目录；raw-process 保留实际 helper、PID、stderr。首轮 request 缺 output_root 的无效反例与 64KiB 成功对照原件都保留，最终结论只用已到达生产状态的修正输入。
- 直接重放：从主检出运行 `python docs/plans/attachments/815/review-t05-r1/replay.py protocol` / `vm` / `rust`（或 `all`）。前提 Windows/Python 可用，树中 bin 已按目标修复 SHA 重建。包装器使用全新私有 target 目录和 replay-results 输出，不覆盖冻结结果、不把旧 ready/PID marker 当就绪。原执行脚本原件保持不变；包装器仅重定位 scratch/results，已做语法/字符串匹配核验，未冒称包装器又跑了一轮产品验证。受控安装/cargo 负控不能替代真实 UI 成功正控。
- 测试结束按已知 owned PID 清理自己的 tool/helper 子树；未关闭其他会话、未改源实现、未创建链接。

## 路由

原唯一树 work 一次修齐 F01..07，补足真实 Launched/交互/取消收口与 sibling 分支测试、更新验收输入/当期草稿，提交新 SHA 后再审。复审需回放原例并验证完整生命周期不变量；修一个 kill 或补一项能力断言不足以自动关闭其他项。T-05 勾选撤回、current_step 重算为 10/17；A/L、T-04 和既有独立关闭范围保留。最终 T-09 pass 后再由 merge 冻结执行裸 cargo t + 必要触面档，门绿前不得合入或宣称 B 已交付。

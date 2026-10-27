# PLAN-816 · T-13 对 T-05 的阶段受影响审查 R1

2026-10-23；**needs_fix：5 项 P2 功能问题、1 项 P3 健康问题**。仅审 T-05，不能勾选 T-05 或启动依赖它的 T-06。M816-A、T-03、T-04 的历史阶段通过与发布收据保留；当前复用的 T-13 重开，整体 executing **6/14**。

## 冻结与范围

- reviewer：本聊天 Codex `/root`，独立于实施会话；未创建 agent，模型/会话 ID 未暴露。依据实际源码和独立进程探针重建结论。
- Plan r2；主检出交接 `08a2d5993f6ce1af038dc6a462c612683dc50ef7`；唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，受审 **`b6148dd90d716038395ab32d5ce8d1d926267dfe`**，直接基线 **`785e4c20a068793f159a00a63b0b6edc15d9e83f`**，17 文件 +1849/-785。取证前后 HEAD 一致、实施树 clean。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。814 delivery `93a25c924`、815 delivery `33c9451fb` 均为受审提交祖先；同步 merge `3abde3f1f` 和开工前 777 重采样 `4e56d62e8` 在链上，启动门成立。
- [输入指纹](inputs.json)、[冻结 SD](spec-delta-frozen.md)；SD SHA256 `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。源码/二进制最终身份与证据摘要见 [final-state.json](final-state.json)。真实工具从当前源重建后再次跑 consumer 和反例；不依赖预存产物推断身份。
- 评估薄 CLI 基座、15 个语言文件入口迁移、所有新增 UiOps 映射、新客户端交互/事件/写确认/关闭，以及相邻旧 seam/argv/服务控制。T-06 service/serve/req/watch 全链、T-07 完整安装/bootstrap/重定位及矩阵、T-08 规范终稿仍是后续工作，不假称已验收；未跑整档或 merge 门。

## Findings

### F-816-T05-R1-01 · P2 · 输出目录与交付模式被普通生成吞掉

**既有规则**：T-05、AC-01/04、Plan §1/5.4 和冻结命令矩阵保留 `gen -o/-p/--delivery-root`；792 delivery 必须产生受管可搬动 bundle，不能成功返回默认开发工程。

**真实反例**：[replay.py](replay.py) 的 `rust-output` / `rust-delivery`，合法 `render: "rust"` + `src/front/app.at`，分别经真实 CLI `gen --output <absolute>`、`gen --delivery-root <absolute>`。两者均 exit **0**，却只写工程下 `rust-workspace/<member>`；指定 output 不存在，delivery 根和 `delivery-manifest.json` 不存在。`vue-control` 实际生成 App.vue、exit0，排除工具不可用或源无效。

**已确证原因**：`crates/auto/src/tool_ui_ops.rs:205-228` 丢弃 `output/project`，把 delivery 编码成普通请求的 output_root 后统一 `generate_ui`；已发布 `tool_ops.rs:129-131` 固定调用开发生成 `generate_rust_ui_with_request(root, None, true, request)`。815 的字段矩阵与 canonical Spec 明确 output_root **不重定向生成布局**；分支契约附录 §8.3 的“即 delivery 通道”判断不成立。

**修复/闭合**：与工具 owner 衔接能保留旧参数和独立交付模式的调用；不能仅继续设置被忽略字段。永久回归断言指定落点/实际 manifest、资源与 producer 身份，而不只检查返回码或 report。覆盖普通 Rust `-o`、development 无覆盖正控、delivery 正控及其既有 fresh/locked 适用负控；Jet/Ark/VSCode 的 output/project 同族映射也需盘点。本轮实际落点反例仅确认 Rust 两形态，不声称运行所有后端交付。

### F-816-T05-R1-02 · P2 · InputAck 被当作 run 的终回执

**既有规则**：T-05/AC-02/04、Plan §5.2 交互 stdin 保持、I-TOOL 的 InputAck 与工作终态分离。

**独立反例**：`client-input` 用当前公开 `run_ui(request, true)`。协议宿主实际收到 RunUi 后发 `HOST_RUN_READY`，外部才写 `q\n`；宿主收到真实 Input 后先回合法 `InputAck { target_id:1, bytes:2 }`，再回 RunOk。实际结果是 **MalformedReply: unexpected reply to run_ui: Other(InputAck...)**。输入不是提前注入，不使用坏帧，也不是宿主先拒绝 run。`client-control` 的非交互正常 RunOk + shutdown Success 是合法正控。

**已确证原因**：`lib.rs:635-643` pump 复用工作 id；`wait_reply:799-820` 只驻留 Event，InputAck 会结束等待，`run_ui:580-587` 随即拒绝它。writer ack 的 id 过滤并未处理控制流 InputAck。CLI 所有 run 臂实际使用 forward_stdin=true。

**修复/闭合**：正确区分输入请求回执、输入拒绝与工作的唯一最终回执，保留错误传播及事件顺序；增加真实工具在 Launched 后输入 q 的正控及此合法交错的永久回归，并确认输入失败不会冒充运行完成。不要只把所有非 RunOk 回执静默丢弃。

### F-816-T05-R1-03 · P2 · 输入泵残留并阻断优雅关闭

**既有规则**：AC-02、Plan §5.2 和已发布 tool-execution Spec：“任何返回路径都不带活的自有线程”。由客户端创建的线程不因注释改称进程生命周期就自动豁免；本客户端也不是只允许进程立即退出的私有入口。

**独立反例**：`client-held` 保持本进程 stdin 的管道打开，不投输入；宿主正常回 RunOk。run 成功，但 `shutdown_timeout=250ms` 的 shutdown 返回 **PipesHeld**，随后线程数 **4→5**；同进程仍存活可观察。相同宿主、相同 held stdin、forward_stdin=false 的 `client-control` 是 **Success、4→4**。`client-input` 也同样残留。

**已确证原因**：`spawn_stdin_pump:624-654` 丢弃 JoinHandle，阻塞在 stdin.lock()/lines()，stop 标志不能唤醒读；它还持有 jobs_tx。`shutdown:997-1003` 先等 writer 结束，直到后面的 `:1044-1048` 才设置 writer_stop，无法在正常等待阶段释放被 pump 克隆保活的 writer。当前报错不是外部非 owned pipe 持有者造成的旧 PipesHeld 边界。

**修复/闭合**：为输入读取提供明确可停/可回收的归属及关闭顺序；正常、拒绝、取消、超时后，stdin 不 EOF 时也须有界回收，不残留读取者或持锁者，并保持同客户端后续操作可用。用线程计数/实际结束和 shutdown Success 断言；仅看 stop=true 或 CLI 进程退出不足以闭合。正常 writer 完成竞态、坏流与旧超时/树清理回归须保持。

### F-816-T05-R1-04 · P2 · Vue 臂回读 pac 丢失已选择的后端

**既有规则**：T-05/AC-01/04 保留 `-r` 覆盖与已求值请求单源；815 Spec 的生产请求在场即权威，不能重新扫描主 pac 成第二真源。

**真实反例**：`override-vue` 的 pac 声明 rust，CLI `build -r vue --gen-only` exit **0**，工具日志却是 `Generating Rust UI code`，写入 rust-workspace，**没有 Vue App.vue**。同一有效 widget 在 `vue-control` 中正常生成 Vue。

**已确证原因**：`tool_ui_ops.rs:174-180` 的 vue_build/vue_generate 调 `base_request(root_dir)`，后者重新静态解析磁盘 pac，未接收 Automan 选定/求值后的请求，覆盖值丢失。workspace Vue 臂已有 request 则是另一条路径，不能用其通过掩盖此单项目反例。

**修复/闭合**：让所有生产 UI 臂消费本次有效请求，检查 render_override、动态求值、workspace/member 与相关 API/端口字段传递；真实反向覆盖用例须断言最终后端产物及不存在错误后端副作用，保留无覆盖合法正控。不能仅在 base_request 里硬编码 vue 来修本例。

### F-816-T05-R1-05 · P2 · strict 与 run argv 未穿过进程边界

**既有规则**：T-05/AC-04、命令矩阵和 main 既有注记：build 默认 strict，`--lenient` 降级；run 后的参数按原语义透传。交接把这两项列为 honest_notes，并不构成降约批准。

**真实反例**：合法 app + 不完整 `beta_store.at`。`strict-invalid` 经 `build --gen-only --strict` **exit0**，stderr 已报告真实 Parse error；`lenient-invalid` 同源也 exit0（这是应保留的合法降级正控）。旧 owner 的 `vue::tests::test_incremental_parse_failure_semantics` 本轮 **1/1 通过**，确证相同解析失败应在 strict 下返回错误、lenient 下保留健康输出。

**已确证原因**：`main.rs:1185-1192` 只设置 CLI 进程的全局 strict，远端工具没有接收此值；`tool_ui_ops.rs:193-198` 及各 run 方法明确丢弃 args，wire RunUi 只有 request，远端 `tool_ops.rs:249-280` 用 Vec::new() 启动所有后端。argv 丢失本轮取完整源码链证据，未伪称已经跑实际 Vite 参数正反控。

**修复/闭合**：与工具 owner 协商承接旧执行选项，不可用本进程全局状态冒充跨进程传递。补默认 strict/显式 strict/lenient 的区分性 CLI 回归，以及含空格、Unicode、选项样式 argv 的真实子进程观测；其余 run 后端同族盘点，保留现有脚本 argv=3 正控。

### F-816-T05-R1-06 · P3 · 已移除 feature 仍出现在生产 cfg

T-05 的 Cargo features 已把 ui-iced 改为 ui-extra，但 `crates/auto/src/main.rs:1028` 仍写 `#[cfg(all(windows, feature="ui-iced"))]`。default/Core 构建均发 **unexpected_cfgs**，该分支恒不编译，违反必需健康收尾。Core 新门控另有 osc_app/stub 未使用告警，见 build-core 日志，应按归属收齐新增告警，不要求清理无关 SDK 的历史 warning。**不称 clipboard 功能缺失**：前面的 runtime install hook 已覆盖媒体 provider 装配。闭合要求两 profile 当前改动净新增告警为零，门控表达真实可编译形态。

## 验证与负控质量

真实进程结果、输入及命令都在 [replay-results.json.txt](replay-results.json.txt)；执行 `python D:/autostack/auto-lang/docs/plans/attachments/816/review-t05-r1/replay.py`，cwd 为唯一实施树，先构建 default CLI/真实 tool。脚本当前 **exit1**：9 个精确断言中 3 个合法控制通过、6 个功能断言失败（01 含两个落点反例）。[client-probe.rs](client-probe.rs) 是独立消费者和具名协议宿主，使用单独 bin 名；真实 Input 被收到才回 InputAck，stdin 持有者与 reader/writer 的生命周期观测分离。fake 只证明客户端对合法交错及寿命的处理；不代替真实生成、安装或完整 UI 正控。

现有定向测试 **68/68**：CLI bin24、client lib15、独立真实 consumer23、strict owner1、旧 seam/non-UI service 集成4、旧 argv 复位1。日志分别为 cli-bin/client-lib/fixture-final/strict-owner-control/t03-old-controls/argv-old-controls；不是整档。消费者23包含原发现身份/版本/能力拒绝、EOF 坏帧/exit 非零拒绝、超时/孙进程/流隔离及新增真实生成。新增 run 生命周期问题没有被这些既有测试覆盖。

default/Core 构建通过；Core normal/build/dev tree **0** UI 实现/旧大包禁项，且缺 tool 时脚本输出42、透传 argv 长度3、dump-lowered 成功，读不存在文件非零；stdlib inventory 如实 partial/exit3，不能算整体功能全绿。main/compat 的15个文件入口转发已审，语言执行算法在 SDK，run_file_with_args 旧动态 UI 分岔保留；本轮未再跑完整 wrapper1402或 UI 全档。

健康：git diff --check 通过。仓级 cargo fmt --all 会进入 auto-down 历史生成物而红；四包 cargo fmt 又报告既有 examples/集测格式问题。改动文件单独 rustfmt 仍报告 wrapper lib 原有导入/空行等格式，不把外部生成物或旧文件所有差异算新功能 finding；没有声称 fmt 全绿。新增明确告警按06收尾，完整健康仍需最终 T-08/T-09 核对。

777 独立 strict：**fail，1 stale_baseline，3 grandfathered，0 新禁边**，过期锚为 `crates/auto/Cargo.toml`（见 [777.json](777.json)）。`4e56d62e8` 的 134 边绿色是开工前结果，不能绑定 b6148dd90。精确边重采样和失败对照按既定 T-07 完成；**未把 T-07 的该项追加成 T-05 功能 finding**。

JSON gen 真实观测也保留：stdout 不是单个 JSON，管理层诊断先行；旧该层打印并非本轮新增，完整 JSON 命令矩阵仍是 T-07 的原合同必做项，不能据此宣称 T-05/整B全矩阵通过。

## AC / SD / 交接

| 验收 | 当期结果 | 映射 |
|---|---|---|
| AC-01 / T-05 | partial/fail | 基座/单源入口/Core闭包通过；生成落点及有效请求失真01/04 |
| AC-02 / T-05新增client影响 | fail | 02/03输入回执/线程；原执行设施及旧窄面持久回归通过不覆盖新run臂 |
| AC-03 | partial | 真实生成/原发现拒绝通过；完整安装/bootstrap仍T-07，不能用fixture替代 |
| AC-04 / T-05 | fail | 01/04/05兼容；JSON/777/792完整矩阵归原T-07，尚未整体通过 |
| AC-05/08 / 当前复用T-13 | needs_fix | 06新增健康；旧阶段记录保留，非整Plan final |

SD-01/02/03 与实际新增执行客户端/CLI/管理适配面有关；SD-04 的 service 最终效果未到 T-06；SD-05 的完整 profile/792/身份须修01并经 T-07。generic SD 五行与此前 hash 一致，但尚不足以发布 B 终稿，按 T-08 补具体 before/after 和现行 Specs 调和。分支契约 §8 的 profile 概述可以用于识别双形态，**delivery 字段推断须纠正**；主检出同名契约尚无 §8，工作交接簿记应同步，不把仅分支附录当主合同已经落盘。

下一步：work 在同一树修齐以上6项，补永久定向正反控并提交新 SHA，再交 T-13 受影响复审；在通过前 T-05 不勾选、T-06 不越门。后续最终 review 通过后，merge-owned 冻结裸 cargo t 及最终 diff 触面的 VM/trans/UI 等整档、实际 consumer/发行门仍 pending，tf 仅主检出到期批量。未改产品实现、未提交、未合入、未修改 canonical Specs/live ledger、未归档或清树。

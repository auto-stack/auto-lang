# PLAN-815 r2 / M815-A 独立阶段复审 R2

2026-10-20。**outcome: needs_fix**；review_scope: phase；T-13 / AC-08。

受审提交 `faea8c9d727df1ab2b16d45fed95c0f345c48180`，修复基线 `9f27d54f013119a93ca832a2c43d83413592cb69`，原 diff base `b9f7ab7e1f400073006e096a08b436f933e4a836`。Plan revision=2。仍为同一独立审查会话 Codex `/root`，不继承实施会话自检结论，不创建代理/第二工作树，不声称未暴露的模型身份。

唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`；入场/测试后 clean、HEAD 未变化。809 已发布依赖、Spec 当前版本与哈希、main HEAD、所有实际实现/运行输入见 [baseline.json](baseline.json)。未消费 814 WIP，未修改实现或发布规范/ledger，未合入/归档/清树。

重建真实工具 SHA256：`038631ef0898ec5bdcdb10aa877853ea6fc5a5747cae5283b8ce8f05cba3a0f9`。

拟发布规范载体：[frozen-m815-a-candidate.md](frozen-m815-a-candidate.md)，SHA256 `b29522ef7ddc56ec66200c66e59e9181a3f07abea276791b758d68741625f332`；[frozen-contract-baseline.md](frozen-contract-baseline.md)，SHA256 `8005782326da55e9d85f80d6e78dcef906cbfd52ca9bbb35cdd05b5a226a0ed3`。它们均为候选，未作为 canonical 发布。

## 旧 finding 关闭表

| Finding | 原例闭合 | 完整不变量/修复回归结论 |
|---|---|---|
| F-815-R1-01 | CLOSED：members 诊断字段保留；新增 defines/interpolation 对照也无损；project/source/input/output 四类不可表示 Windows 路径均具名拒绝 | 原问题 CLOSED。新增诊断标签规则的准入不一致单列 R2-01，不抹去已修好的字段/路径证据 |
| F-815-R1-02 | C02 嵌套未知键、C03 缺槽已正确拒绝，普通嵌套固定对象的 strict helper 接线通过检查 | **OPEN/partial**：Value 标签对象漏 t 仍触 Map 索引 panic，三个实际字段位置均令真实工具 exit101；完整“各层缺键受控拒绝”未闭合 |
| F-815-R1-03 | CLOSED：byte256、1e300、NaN 均拒绝，4294967297 版本如实报告；byte255/u8/i8 控制正确 | CLOSED：真实 scanner 产生的分数窗口 800.1x600.1、两端边界及附近121个f32窗口文本往返无失败；没有复活已修 numeric finding |
| F-815-R1-04 | C09/C10 普通相对路径的两 cwd 结果一致；C11 普通文件根正确报错；新增无根/相对根/角色测试通过 | **OPEN/partial**：Windows 盘符相对路径经 root.join 后仍相对，input/output/front/dep 四条路径仍使用进程 cwd |
| F-815-R1-05 | 旧64层 Value 请求现在被 codec 以 DepthExceeded 拒绝；旧裸 JSON 手工帧仍 exit2，符合其超解析限的已声明分类 | **OPEN/partial**：120限制在请求子树/完整信封和两个读取入口间不一致；57层请求codec接受但标准协议writer拒绝，手工同帧被真实工具接受 |
| F-815-R1-06 | CLOSED：scoped all-targets check 无warning，三个新包和独立fixture fmt均通过 | CLOSED；不要求重复运行整档来证明健康修复 |

## 当前开放项（3 项旧不变量未闭合 + 1 项修复引入的新问题）

### F-815-R1-02 [P2，继续 OPEN] 缺 Value 标签会使合法帧内的坏 payload 杀死工具

AC-02/03/07，T-03/T-12，原合同缺口，非本轮新增验收要求。

`crates/auto-project-model/src/wire.rs:203..220`：默认分支仅检查对象长度为2且包含 `v`，未要求 `t`；随后 `obj["t"]` 在缺键时 panic。输入为有效帧/有效JSON，在请求中放入：

```json
{"v":1,"extra":2}
```

独立放在 `package_assets.npm_deps`、`raw[0].value`、`raw_nodes[0].props[0].value` 三个真实位置均复现：**exit101、零响应，stderr 在 wire.rs:220 报 no entry found for key**。追加的 describe 也得不到响应。对照 `{"t":"byte","v":256}` 能返回 InvalidField 后继续 describe、exit0，证明不是所有 payload 错误都被设计成杀进程。

期望：按既有 message-level 错误合同受控拒绝，保持控制流可服务；绝不能用“固定键对象已严格”把底层 Value 子对象漏掉。

修复范围：按具体tag校验必需键、类型和允许键，使用受控访问；覆盖 package-assets/raw/raw-node-props 中的缺tag/错tag类型/缺payload/额外键及合法变体控制。仅补 C02/C03 的模型对象键检查尚不够。

证据：[adjacent-process-results.jsonl](adjacent-process-results.jsonl) 的 missing-value-tag 三行与 CONTROL-invalid-value-then-describe；[回放脚本](adjacent_process_probes.py)。

### F-815-R1-04 [P2，继续 OPEN] Windows 盘符相对路径绕过根锚定

AC-02/07，T-12，原“路径不依赖工具 cwd”不变量的相邻路径形式。

`auto-ui-tools/src/validate.rs:48` 在 request root 绝对时直接 `root.join(raw)`，未验证合成路径仍绝对。Windows `C:front/main.at` 含盘符却没有根，是盘符相对路径；join 会保留它为 `C:front/main.at`，后续 FS 探测使用进程的该盘当前目录。

实际绝对 project_root 固定，工具分别在同盘 project / elsewhere 启动：

| 请求字段 | 同一值 | project cwd | elsewhere cwd |
|---|---|---|---|
| input_sources | `C:front/main.at` | ok=true | input-source-missing / ok=false |
| output_root | `C:result`（工程内该文件存在） | output-root-is-file / ok=false | 无finding / ok=true |
| front_decl | `C:result` | front-is-file | front-directory-missing |
| dep_roots.path | `C:result` | dep-root-is-file | dep-root-missing |

控制：普通 `front/main.at` 在相同两 cwd 均ok；R1普通相对 output/input/root-file 原例均闭合。缺口在新 anchor helper 对 Windows Prefix 路径的处理，不是根仍未传入。

修复：显式解析 Windows 路径种类，保证被探测路径已由请求数据确定；不能锚定的盘符相对路径应受控拒绝/明确不可校验，或按已明确的请求规则解析。所有共享 helper 消费点补两 cwd/同盘与异盘/绝对与普通相对控制，不能仅替换 join 后继续把相对结果标 Resolved。

证据：adjacent-process-results 的 windows-path-anchor 十行。实际盘符/临时目录由脚本从真实项目路径取值，不依赖本机固定 C盘假设。

### F-815-R1-05 [P2，继续 OPEN] 深度包络没有覆盖完整信封和两个读取入口

AC-03/07，T-01/T-03/T-12。原64层例的生产端拒绝已完成，但不能据此声称四处深度准入一致。

1. **漏算实际协议封装**：57层 Value 数组的 wire 子树深度119，request_to_wire/request_from_wire 均成功；同一请求放进真实 ClientEnvelope 后深度121，`write_frame` 因 MAX_FRAME_JSON_DEPTH=120 拒绝。56层（wire117/信封119）可正常写读。独立真实工具对手工帧化的57层请求又返回 ok=true。标准消费者与读端对同一请求不是同一可传输集合。
2. **读入口未接线120门**：`framing.rs:124,178` 的 read_frame / parse_frame 仍只使用默认JSON解析器。按当前 json_depth 计数，深度121/126的JSON，writer拒绝，两reader却成功；129才由解析器递归限拒绝。`MAX_FRAME_JSON_DEPTH` 只在writer使用。

冻结合同 §4 明确“四处共享同一上限”，候选SD-02也写生产/消费对称，当前实现不支持该声明。候选中“>128流致命”的描述也没有解释121..128的不同准入。

修复：明确限额的根、层数单位、完整信封开销及可表示/可传输域，在codec、标准消息封装、write/read/parse各实际入口一致执行并准确分类；用真实ClientEnvelope而非裸wire子树做上下边界控制。若选择不同输入/输出兼容范围，先明确修订合同及消费者预期，不能继续以“对称120”报告通过。本项不要求无限深度；所有反例都在已声明120附近，消息仅几KB。

证据：[adjacent-codec-results.log.txt](adjacent-codec-results.log.txt) 的 depth-envelope/framing-limit；actual process 的 raw-depth-envelope 56/57/58/64控制。

### F-815-R2-01 [P2，新修复回归] 诊断payload的新编码语法准入不一致

AC-02/07，T-03/T-12；由 R1-F01 的新实现引入，原成员字段丢失及路径问题不复活。

`wire.rs:953` 对 `type-mismatch:` 使用 `trim_start_matches`，会删除任意多个重复前缀。线上的未知标签 `type-mismatch:type-mismatch:members` 被解成合法 `TypeMismatch { field:"members" }`；实际tool返回 validate_ok/ok=true，而不是拒绝未知tag。这与候选中“编造payload拒绝、严格codec drift”的新规则不一致。

同一准入域在编码侧也不一致：`:919` 对公开DTO中的 `TypeMismatch {field:"future"}` 成功编码，随后自己的decoder以封闭字段集拒绝。若封闭字段集就是支持域，生产端也应明确拒绝域外值，不能先产出不可回读的wire。

修复：精确匹配tag语法（前缀仅消费一次）并使payload验证在编解码两侧一致；对 members/defines、未知字段、空字段、重复前缀及相邻普通unsupported变体做正反控。无需扩展scanner静态域或动态intern任意字符串。

证据：adjacent-codec-results 的 repeated-diagnostic-tag / unsupported-type-mismatch；真实进程的 repeated-diagnostic-tag 行。

## 本轮实际验证

| 检查 | 结果 |
|---|---|
| `cargo test -p auto-tool-protocol -p auto-ui-tools -p auto-project-model --features auto-project-model/wire` | exit0：49 model +15 protocol +4 resources +15 actual process =83 tests；[原始日志](targeted.log.txt) |
| `AUTO_UI_TOOL_BIN=<重建工具> cargo test --offline --manifest-path test/fixtures/modularity/ui-tool-consumer/Cargo.toml` | exit0，4/4；[fixture.log.txt](fixture.log.txt)；总87个不同现有定向测试通过 |
| `cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-project-model --features auto-project-model/wire --all-targets` | exit0、零warning；[check.log.txt](check.log.txt) |
| 三个包 `cargo fmt --check` + fixture `cargo fmt --check --manifest-path ...` | 均exit0；fmt.log / fixture-fmt.log |
| Git Bash verify-deps.sh | exit0；[依赖树](dependency-tree.txt)，normal/build无Man/lang/VM/gen/media大包 |
| `python scripts/modularity-audit.py --mode strict` | exit0；pass、0 violations、2 grandfathered、75 edges；[audit-summary.json](audit-summary.json) |
| `git diff --check b9f7ab7e1...HEAD` | exit0 |
| 原 R1 codec / process probe | [old-codec-replay.log.txt](old-codec-replay.log.txt)、[old-process-replay.jsonl](old-process-replay.jsonl)。原codec在C07因新增具名拒绝触发旧unwrap停止，**exit101不是全量回放通过**；已用新的四角色路径拒绝控制补齐验证 |
| 本轮独立相邻反例 | [codec probe](probe/src/main.rs)、[process probe](adjacent_process_probes.py)；全部结果文件持久化，Python每个真实子进程10秒截止 |

原进程回放仍确认：版本/能力拒绝不产生目录副作用，流可继续describe；bad-prefix/bad-json/输入超限为exit2；control/log分离；正常EOF为exit0；取消已服务和未知目标的现有测试通过。输出超限 C12 exit1仍属冻结合同允许结果：输入8,301,703字节，当前角色诊断更长，输出9,853,677字节，未写半帧。它不是本轮finding。

数字/窗口、资源登记与健康的旧证据未因其他finding重新标红。没有运行裸 cargo t/tv/tt/tb/tu/taa/tf。

## AC/任务与规范增量

| AC | 当前结论/映射 |
|---|---|
| AC-01 | A轻闭包/资产登记通过；完整生产迁移仍B pending |
| AC-02 | A当期fail：T-03/T-12，schema/路径/诊断准入当前开放；原无损字段/不可表示路径已修部分保留 |
| AC-03 | A当期partial/fail：T-01/T-03/T-12深度准入和坏payload非受控退出；正常版本/能力/日志/预算分类证据通过，完整运行寿命仍B pending |
| AC-04 | A资源/闭包/真consumer通过；792完整fresh/locked/迁移资源等原B支持面继续pending，本轮rust_ui.rs和发行表未变 |
| AC-05/06 | 整Plan final/merge未到对应阶段；不误给reviewed |
| AC-07 | fail：当前四个开放项需完成work |
| AC-08 | needs_fix：T-13不勾选 |
| AC-09 | pending：T-14未启动，没有发布收据可放行816 |

SD-02候选已补清载荷单位不含4字节前缀、未知版本真实u64、明确A能力与预算；SD-04已补wire feature/serde_json/支持拒绝集合，不再只留笼统草图。但“全部嵌套缺键受控拒绝”“诊断payload编解码一致”“深度120四入口对称”“路径不依cwd”仍受上述反例否证，当前不能认可为发布事实。SD-01/06仅可沉淀A实际窄工具/来源子集；SD-03/05和完整UI资产/兼容owner规则仍归B。所有精确路径、GOAL-003/007保留，当前canonical输入版本/hash已冻结，无冲突覆盖或ledger写入。

## 路由

保持 executing，重开 T-01（深度包络/分类部分）、T-03、T-12；保留 T-02以及已闭合 F01/F03/F06证据，current_step=1/14。不要再写“R1六项全闭”作当前就绪状态；历史work声称可保留，但以本轮关闭表为当前依据。

下一轮在原树一次修复：Value tag实际解析入口、Windows Prefix路径、完整消息深度包络、诊断tag精确准入。对上述共享回放集及相邻控制补永久回归；新代码提交后再绑定SHA送T-13。若实现改动未触已闭代码/依赖/测试配置，未受影响证据可按哈希复用，无需把所有旧finding复活，也不应仅补当前四个输入而漏掉共享helper消费者。

merge-owned pending：阶段pass后T-14串行冻结裸cargo t+必要新包/协议/外部consumer门，以及实际master合入、SD/ledger、consumer来源发布收据。当前diff仍未触VM/trans/UI生成，不触发tv/tt/tu；若合入输入变化需重新判定。tf只在主检出到期单实例批量。阶段审查未通过，不启动这些门，不合入/归档/清树。

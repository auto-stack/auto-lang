# PLAN-779 独立 final review R1

2026-10-15；结论 **needs_fix**。外部provider与UI bridge供值通过，发现门开程序轨验证未闭合，以及本轮代码格式未收口。不得合入。

## 冻结基线

review_scope=final；plan_revision=1；reviewer_identity=本拆分规划/其他探针评审会话，未实施779，独立从提交、合同及实际断言重建。

- worktree=`D:/autostack/.wt/lang-779/auto-lang`，branch=`plan-779-dev`，implementation clean。
- reviewed_commit=`451f612bdb67c1f6947db878bb37b3105c4eb8c5`；base=`2253134cbd8b6e4a6273bf92f2145c3f34ce8934`。
- auto-down只读兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；锁/全部受管输入及最终复核见[fingerprints.json](fingerprints.json)。
- vm/overview Spec SHA256=`0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144`；ui/overview=`cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5`。见[冻结SD](spec-delta-reviewed.md)。

## F779-R1-01 / P2 / blocking：门开程序轨验证未随宿主装配迁移闭合

位置：`crates/auto-lang/src/vm/engine.rs:812`默认provider快照；现有程序轨验收入口`crates/auto-lang/src/tests/plan716_supply_probes.rs:471–494`。

在全新测试进程仅运行该测试，明确设置`AUTO_FRAME_BENCH=1`：

```text
cargo test -p auto-lang --lib frame_program_track_gate_on --locked -- --nocapture
```

实际cargo exit101；frame_bench捕获真值 **2**，而`run_with_capture`的VM stdout为 **"0\n0\n"**。旧程序轨探针未创建VmBridge，也不经CLI/main或run_file_dynamic_ui_inner，故没有装配默认provider，AutoVM快照None。原core直读frame_bench改成provider后，此验证路径没有随迁移接线。证据：[gate-program.log](gate-program.log)、[命令与env](runs.jsonl)。

对照：相同env、独立进程只运行`frame_value_flow_gate_on`，经真实VmBridge路径读取真值，exit0（[gate-bridge.log](gate-bridge.log)）。说明bridge装配有效，不能据此推断程序轨也有效。门关的plan716_supply **14/14**包含门开用例的提前返回；不等于门开验收通过。

性质是既有门开程序轨验证/宿主接线遗漏，影响AC03、V04/V05、T04/T05。并不据此断言所有CLI或所有UI创建路径失效；也不要求无provider的纯Core改变0语义。

修复需明确程序轨的真实宿主启动责任并补有效正控：如果此探针应模拟显式UI宿主，则通过真实公开装配入口安装provider后测非零值，保留独立无provider=0负控；若对应的受支持库/服务路径仍承诺自动接入现有帧观测，则该实际路径也须接线并验证。不能仅删测试、忽略失败或把期望改成0；不能为过测试把core→UI直接依赖接回去。work收据中“全部work面已闭合”的结论需据此修正。若要改变既有公共支持合同，应先经计划修订确认。

## F779-R1-02 / P3 / closeout：本轮新增/改动代码未通过格式检查

位置：`ui/frame_provider.rs:33`、`vm/host_capabilities.rs:39/90`、`vm/native.rs:113/167/1682/1695`。

仅检查这三个provider触面文件：

```text
rustfmt --check --edition 2021 crates/auto-lang/src/ui/frame_provider.rs crates/auto-lang/src/vm/host_capabilities.rs crates/auto-lang/src/vm/native.rs
```

exit1，见[fmt-finding.log](fmt-finding.log)及[命令收据](fmt-finding.json)。新文件及新字段/方法/调用本身有diff；不是预存28文件drift。对全部改动文件的skip_children检查另见fmt.log，包含部分历史格式差异，未把那些差异归责于本Plan。

影响T02/T05及必需health check。按触面修正即可，不要求全仓格式重排。

## AC/任务/证据矩阵

| AC→Task | 判定 |
|---|---|
| AC01→T01/T02/T03 | 功能pass：core选定frame实现无UI直引；独立default-features=false consumer5/5，真实非零stdout |
| AC02→T02/T03 | 功能pass：双活实例及默认快照/实例覆盖、0降级、上界clamp、ID9918/9919、重注册与merge/proof相关控制通过 |
| AC03→T04/T05 | partial：真实bridge门开正控通过，注入者/未迁族清单存在；程序轨验证F01未闭合 |
| AC04→T06 | 独立审计完成，final未pass；F02格式也需收口 |
| AC05→T07 | merge-owned pending |

本轮实跑：consumer **5/5**、plan779 **3/3**、plan755 **4/4**、plan716_supply（默认门态）**14/14**；check auto-lang+auto exit0；diff-check通过。单独门开程序轨 **0/1失败**、门开bridge **1/1通过**。默认门态14/14与显式门开结果分别记录，不合并为全绿。

源码审查：frame实现直接UI引用已剪断；NI Clone保留实例provider，merge仍走原绑定失效规则；native catalog/ID与a2r直读臂未改。UI构造new_with_children/new_from_decls、run_file_dynamic_ui_inner及CLI注册点存在。其他native族/a2r仍反向依赖UI的事实按Plan明示，不冒充整个runtime拆完。provider Send+Sync不改变VM的线程亲和；本次未改媒体/heap/async合同。

运行证据见[runs.jsonl](runs.jsonl)、各命名log及consumer-deps.log（独立构建未启UI特征）。已有仓内警告不在本轮宣称清零。本次没重跑真实OS窗口；以门开bridge+frame_bench通道的运行证据验证当前有界标量provider，不外推呈现性能。

## 复放与下一步

```text
python docs/plans/attachments/779/review-r1/run.py
```

runner逐命令记录真实退出码，**整体exit0不表示所有子命令成功**；本冻结面gate-program=101，其余运行门0。所有构建/测试定位Plan树，main只存评审文档/证据；未改实现。

保留独立核实的T01/T03；T02因格式未完成、T04/T05因接线/验证未完成重开，T06未pass、T07待办，**executing 2/7**，revision1不变。同树work修复两项、提交后复审；没有自动needs_replan要求。

**merge-owned pending**：最终review通过后冻结提交裸cargo t+tv；未触UI生成器/aavm/trans，不新增tu/taa/tt要求。其后SD01/02沉淀、ledger、合流/归档、批回归到期判定及guard-clean清理。本轮未跑任何整档、未发布canonical/ledger、未合入或清理树。

# PLAN-815 T-07完整验收子批1定向复审

2026-10-22，**pass（仅子批1）**，无新增finding。协议四回执、帧深度边界对照、已披露real_tree Linux测试可移植缺口三块均有独立证据。完整T-07仍未完成，Plan r3保持executing10/17，T-05～T-08整体任务不由本轮勾选。

## 冻结与范围

- 唯一树D:/autostack/.wt/lang-815/auto-lang，codex/plan-815-dev；HEAD **f6fd0ce7c86fd3c4f70ddddec296f2d021642031**，code **661c0573c0e16630cd9c2c8e8da91c7736c40afd**，base c203d9642a58a3bceeb2075a7ee116e79a416e45。M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。
- 实现diff只涉及proc_host的cfg(test)模块及独立consumer fixture；没有生产行为、manifest、依赖、资源、wire或预算变化。工具SHA256 **f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5**未变。开始/结束同HEAD clean、钉定输入零漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。
- reviewer=Codex `/root`延续独立审查上下文，有早期合同/账本参与限制，未参与当期实现。按/auto-plan:review核对实际断言并独立运行，未创建agent/第二树；仅写主检出复审簿记/附件与私有target探针，不改实现/canonical/live ledger、不合入/归档/清树。
- review_scope=phase：交接[frozen-batch1.md](frozen-batch1.md)中的三块已交付测试，非T-09完整AC终审。新fixture是类型构造/serde往返证据，不当作新执行了真实tool的describe/validate/generate/build进程端点。

## 逐项验收

| 子批1内容 | 独立判定与覆盖边界 |
|---|---|
| 四种服务端回执 | PASS。DescribeOk含ToolDescriptor、LifecyclePolicy::contract和能力；ValidateOk含Error Finding及完整RequestSummary；GenerateOk多frontends；BuildOk含Failed和具名error。每个完整ServerEnvelope经serde_json to_value/from_value后assert_eq，实际载荷及id均参与比较，不只是检查类型名或测试数。与原RunOk/Event/InputAck/Cancelled/Error合并，先前缺四回执项关闭。 |
| 帧深度边界 | PASS。源码构造scalar深度1、每数组包装+1，limit-1个包装恰为122；写后parse保真，123写入口具名Malformed。为区分写/读门，另在私有consumer用原始长度前缀+JSON绕过writer，两个读入口分别122接受且内容保真、123具名Malformed，见下。不会把writer拒绝误称reader已执行负控。 |
| real_tree Linux可移植缺口 | PASS。Windows保留findstr/cmd分支；Unix改真实grep接收stdin并转发、sh后台sleep加exec sleep构造实际owned进程组用例。Windows和Linux冻结源码均实际执行通过，日志/输入/取消/reader检查没有被ignore或跳过。收口该测试的不可移植问题；不能由此声称所有退出/资源矩阵均已验。 |

## 独立运行与断言核验

1. 原独立fixture：`cargo test --locked --offline --manifest-path test/fixtures/modularity/tool-protocol-consumer/Cargo.toml`，CARGO_TARGET_DIR指向唯一树target，**7/7**（0.18s），重新编译无本目标告警；[日志](protocol-consumer.log.txt)。正常闭包门[通过](closure.log.txt)，未把serde往返扩大成真实运行/发行消费。
2. [reader-depth-controls.py](reader-depth-controls.py)在私有target仅复制consumer fixture、追加一个直接调用真实协议库的raw帧控制，未复制/修改生产实现或创建第二实现树。原七测加一控制**8/8**；其中raw depth122的read_frame与parse_frame都接受、123都以包含beyond the transmittable envelope的Malformed拒绝。[结果](reader-depth-controls.json.txt)、[日志](reader-depth-controls.log.txt)。这8例含原7例重复，不另累加为8个新增用例。
3. Windows `cargo test --locked --offline -p auto-ui-tools --lib proc_host::tests:: -- --test-threads=1`，**9/9通过，1慢例ignored**；real_tree确在通过列表，[日志](proc-host-windows.log.txt)。未把ignored计通过。
4. WSL Ubuntu-22.04，既有工具链/target=/home/visus/plan815-unix-target，同冻结源码`--locked --offline`的proc_host模块**10/10通过，1慢例ignored**，包括real_tree、unix_reader、captured_drain取消/自然EOF/截断。再以`--ignored --exact`单跑BoundOverrun，**1/1，10.06s**；[完整日志](proc-host-linux.log.txt)。不是仅读work的Linux4/4总数。
5. 两个净改Rust文件rustfmt逐字相等、diff --check通过，[格式结果](fmt.json.txt)。新fixture告警为0；依赖与UI lib test原有22条等诊断沿原归因，不冒称全库零告警。

未运行整档cargo t/tv/tt/tb/tu/taa/tf/docs_gen，未重采样777。两个生产无关测试入口实跑已足够核验本批增量，未机械重跑前轮真实lifecycle/全profile/GUI等未变范围。

## 规范、复用与下一步

[规范影响](spec-delta-reviewed.md)确认SD-01..07无新规则、协议深度上限及公开面不变；前轮已闭finding仅按未变且适用来源复用，详[evidence-reuse.json.txt](evidence-reuse.json.txt)。本批补的是既有AC-03/04测试/平台证据；AC-05当前改动卫生通过，不改变最终AC范围。

子批2仍由work完成：**792/fresh/locked发行与producer/source/embed/重定位/失败保留恢复/独立消费，资源owner/许可证/唯一源与实际发行内容审计，V-01..04完整字段/兼容/失败矩阵**。777 master并集按现行分工归T-10，必须对实际合入输入执行，不能用此前分支strict替代。

原“四回执/深度负控/real_tree Linux不可移植”三条T-07待办可以记为本子批已核验；不再把它们列作尚未开始，但本次不声称完整Linux资源/运行矩阵已完成。完整T-07之后T-08据实定稿、T-09整体独立终审；最终pass后merge owner统一冻结裸cargo t及实际触面档，再串行合入、Spec/ledger/发布/归档清理。所有这些后续门仍pending。

**下一责任角色work：推进子批2**。Plan executing10/17，唯一树保留，不以本批pass标整Plan reviewed或提前发布816-B。

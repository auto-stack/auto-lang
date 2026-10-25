# PLAN-848 F-848-R1-01 — 有界方案调查，尚未修复

2026-10-25，stage: work，outcome: needs_replan；Plan仍r2/executing5/10。
唯一实施树 `D:/autostack/.wt/lang-848/auto-lang`、`plan-848-dev`，
代码仍 `cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d`，base
`bf22db3465203f3babd649494f4b37aa2a012027`，SDK
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。三锁/AGP未变。

用户授权先找满足原预算的新方案，再修改当前计划并实施，完成后复审、不合入。
本轮尚未得到满足完整控制的方案，因此**没有采用产品实现或修改设计合同**，
没有关闭finding、提高预算/降低128节点/减少32credits，也没有发送一次必然失败的最终复审。
本记录是失败调查和下一设计入口，不是修复完成收据。

## 入口与不变量

先在原冻结代码重放R1 durable探针：实际Session和正式Host的equal成功；
late-z-miss/decreasing-t仍100000累计步失败。记录在candidate-a.log.txt的
两个 `review_*` 测试输出。正式Host各127叶子使用真实Cylinder资源和完整空间配置，
初始化Paused/tick0成立；失败是callback预算而非准入/配置/图形环境。

后续仅在临时 `tests/segment_r3_probe.rs` 的 scene_custom 中替换managed源码，
产品API/Session/SDK均未改。每个候选用128节点（1group+127叶子）、
两次group.x写、2cast+10get+6AABB=32credits；追加三轴方向的正/负斜向
逐个变近候选，避免只让轴向原例变绿。所有例继续实际Session默认预算。
这些只是可行性计量，还没有充分的数值/生命周期/Host验收，不计AC通过。

## 已执行探索

| 候选 | 变化 | 可行性观察（equal / 晚Z miss / 逐个变近 / 正斜向 / 负斜向） |
|---|---|---|
| A | 直接continue、去掉hit/choose/inside标志、最终胜者才生成point | 90 / 98 / budget失败；第三例断言失败后未跑斜向 |
| B | 扫描只保存t/target，最终胜者再恢复normal；其余同A | 87 / 95 / 失败 / 失败 / 失败 |
| C | B + shared derive局部数组别名 | 同B；未观察到预算改善 |
| D | B + derive现有world/effective数组原位写入 | 90 / 98 / 失败 / 失败 / 失败，比B更差 |
| E | B + 扫描scratch移到循环外 | 94 / 失败 / 失败 / 失败 / 失败，比B更差 |
| F | B + while替代ordinal for | 88 / 96 / 失败 / 失败 / 失败 |
| G | B + shared derive与查询融合遍历的预研 | 81 / 90 / 失败 / 失败 / 失败 |
| H | B + 初始化后仅更新X坐标的受限预研 | 71 / 78 / 91 / 失败 / 失败；**不是通用dirty实现** |
| I | H + 按方向预选bound索引、减少有限性检查中的冗余加法 | 68 / 76 / 89 / 失败 / 失败 |
| J | I + 各far延后统一比较，减少中途leave更新 | 65 / 74 / 94 / 失败 / 失败 |
| K | I + 显式标量类型 | 68 / 77 / 90 / 失败 / 失败 |

数字都是yield次数，不是精确VM步数或性能余量。失败均为100000累计步上限；
不通过墙钟耗时或exit0把它们改判成功。B之后诊断函数有意打印所有分支，
不在首次budget错误处停止；**其exit0只表示诊断执行完毕**，不是所有样例通过。
G前两次是生成器setup错误（大括号/残留i）；已单独保存，不计预算失败，
以candidate-g-compiled.log.txt的实际执行结果为准。

H/I/J/K只验证“修改X时只重算X”能减少工作量；未实现完整X/Y/Z/visibility
dirty跟踪。它们连当前追加斜向控制也未过，不能移入产品或冒领通用derive等价。
G是串接原trusted derive片段的实验，尚未形成可验收的单一共享图实现，亦未采用。
其余候选也未经过完整数值/token/旧版身份控制，不宣称语义等价已证明。

## 原因与下一设计入口

诊断分解（diagnose.log.txt）：128节点一次原shared derive约14次yield，
B的相同box一次cast约27，移动后cast约41。full-path-cost.log.txt用
逐个变近而非等深输入，单次移动后cast轴向约52、三轴斜向约61。
这说明需要同时削减图推导和无法被nearest剪枝的扫描工作；仅缩减最终结果
物化或改数组/循环写法不足，不能再用“首个hit后其他候选可剪枝”当支持论据。

还没有证明固定SDK/driver内所有方案都不可能，也没有据此要求豁免预算。
下一步需要在当前Plan的new设计阶段提出并先验证另一计算/数据组织方案，
同时覆盖原两个反例及代表性的2D/3D、宽/深、方向/父写路径；
验证成立后才修订r3设计并实施永久控制。若方案实际需要SDK/AGP写面或改变
数值/源身份/内存边界，应把具体变更和证据送回合同决策，不能自行扩面。

原F01仍P1 OPEN；T02/T05/T08保持重开，其余未变证据保留。
所有32credits成本、100000步/120resumes/250ms、初始化/整tick/pump截止时间
及128节点支持要求不变；不以丢弃斜向输入或仅支持X写作为修复。

## 复现与交接

- 本目录 `candidate-*.at.txt`、`generate*.py.txt`、`segment_r3_probe.rs.txt`
  保存探索源；`raw-sources/` 保存 scene_custom 实际交给SDK的按SHA命名Auto输入，
  包含失败setup，实际执行以对应日志为准。
- 原始失败、equivalent正控及所有临时Rust探针的命令均为driver目录下
  `cargo test --locked --test segment_r3_probe <函数名> -- --nocapture --test-threads=1`。
  candidate_a_distribution_probe是沿用的诊断函数名，不能由名字推断候选或pass。
- 每次候选通过 include_str 选择target/segment-r3中的对应源码；最终探针快照选择K，
  axis_candidate是H/I/J/K共同的X特化；完整源码变换及其局限如上。
- 测试结束删除唯一临时integration文件；没有修改任何tracked实现文件。
  final-source.log.txt再次核370源/681SDK/17local/29edges/479metadata通过；
  实施树HEAD仍cef25b5c4且clean。
- 没有跑完整package、root测试档或GPU大矩阵，没有发布canonical/live ledger，
  没有合入/归档/清树。未完成修复，所以没有新的review pass或execution_done。

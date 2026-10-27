# PLAN-815 T-05 R5 / T-06 / T-08 当期规范核验

Plan r3；phase定向，非T-09完整B终审。代码8a08e84fe25ec81e164ba50b9f9e970e705505e0，交接10e4e1cc1a840132513317424afa7bba4dec5262。frozen-handoff.md/frozen-fixes.md为实际提交原件；canonical输入hash见baseline.json.txt。没有改canonical或live ledger。

T-08已补冻结源SHA、实际二进制SHA256 ab88ccb440e007ad4ac243e28f693f05b431d809e4ef8f4e66ec3e9d0583d775（独立重建一致）、能力×profile、RunOk/BuildOk取消终态与SD-01..07正文。原“只有路径索引/源路径充二进制”的缺口关闭；以下真实保证尚不能批准发布。

| ID / 目标 | 当前核验 |
|---|---|
| SD-01 docs/specs/auto-ui-tools/project.md | 单目标四真实输出根、Multi [vue,jet]、VSCode请求字段已成立。主pac不再重读的全链保证仍被pkg::resolve_impl反例否定；旧Man语义冻结也被非托管Jet公开build返回变化否定。A子集已存在，B须在当前A文件增补，不能覆盖既有发布范围。 |
| SD-02 docs/specs/tool-protocol/project.md | 五能力/十九协议测试保持，build/run双通道取消文本正确；新ADB capture drain没有取消/停止/截止检查，取消后等待持管道者并最终completed，故全路径有界退出/终态保证不成立。Unix可停读代码已替换，但声明平台执行证据仍pending。 |
| SD-03 docs/specs/auto-man/project.md | 兼容/媒体/请求缝6例通过不覆盖构建返回语义。新严格失败逻辑需留在请求/协议边界或取得明确合同修订，不能用新tool修复自动授权改变旧公开入口。 |
| SD-04 docs/specs/project-model/project.md | 模型/codec无本轮修改；raw保留pkg声明。相同wire pkg=npm却由磁盘pkg决定执行npm/pnpm，说明“所有生产入口单源”仍不可沉淀。 |
| SD-05 docs/specs/auto-lang/ui/overview.md | 新Jet/Ark/VSCode命令接入owned设施是改善；拥有进程不等于捕获读取可停，drain_captured_stdout仍能独立卡住worker。须按生产调用链验收，不能凭命令清单宣称生命周期全闭。 |
| SD-06 docs/specs/repository-boundaries/generated-ui-distribution.md | 实际源/bin指纹已绑定一致；完整777/792并集、fresh/locked、资源矩阵仍T-07，新增Unix libc依赖与共享执行设施env_remove实际触面也须纳入后续发行审计。不能从既有清单或本次局部consumer推得完整发行通过。 |
| SD-07 docs/specs/auto-ui-runtime/project.md | 退出窗口本轮仅清两诊断，既有初始化q与真实VM交互/串行对照保持。新稿含公开面正文；不将VM成功证据外推到Jet捕获/Unix未执行分支。 |

impact元数据保持runtime/Man/UIoverview/model/distribution、tools/protocol、GOAL-003/007；无AC缩减，plan_revision=3。T-08依赖T-07且保证与当前实现仍冲突，保持未勾；此稿不能成为完整B已验收/已发布的依据。A/L与T-04历史关闭范围保留。

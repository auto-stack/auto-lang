### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/language-support.md | 无源码/内核/产品分层实证→登记本Plan实际source支持、尚缺能力及固定分母/证据等级 | 不以typed fixture冒领源码 | AC-01/02/06 |
| SD-02 | add | docs/specs/auto-av/source-program-conformance.md | 无通用source程序消费→限定scalar文件入口/快照/强oracle/资源/父侧判定及明确拒绝 | 真程序闭环 | AC-02/03/04/05/06 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | D798已交付→追加source-file身份/依赖/位置/恢复及反控 | 不缓存PASS/trusted | AC-04/05 |
| SD-04 | modify | docs/specs/auto-av/project.md | 实验typed执行→增加实际交付source入口，保留stdlib/UI/完整语言/默认切换未交付 | 如实支持面 | AC-02/06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记实际复用或必要新增的source recipe/worker（候选av/source-run-v1）、typed父侧oracle/输入来源与边界 | 不复制协议或旧解释器 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

Spec路径为proposed，当前尚未全部存在，D798后核modify/add；review最终确认，merge才沉淀。GOAL-002仅记有限源码周转贡献，不据此核销热重载/调试或整VM替换目标。


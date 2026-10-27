### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/dynamic-portable-values.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/dynamic-cast-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-824独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |
| SD-07 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 852底座分母→追加本计划实际支持起点/recipe/证据与范围外gap；不冒领source/native能力 | 统一验证且不扩原语义范围 | AC-11 / V-15 |

R2 修复后的 SD-01/02/03 拟明确：类型树展开、binding 与 portable 描述符的名字/内联容器计入现有字节口径并在分配前拒绝；类型读取、写入、准入和比较参与工作/取消轮询；恢复只持有根类型并逐节点核对子描述符。上限、支持面和资源/取消分态沿既有合同，merge 前须经独立复审。

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。



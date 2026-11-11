# ACE M1 扩大验收（PLAN-941 交付）

## 登记宇宙与组合器

- 扩大登记宇宙以 `crates/auto-lang/test/cases/ace-m1-expanded/registry.json` 为唯一事实源
  （`ace.m1.expanded-registry/1`）：1006 登记键、31 份 pinned 合同、awaiting_d/delivered_d
  具名行（939 七行已于 D=dfe5a1002 转正，宇宙 (1006,0,14)）。分类/关系/历史由应用自
  canonical 文档派生；agent 不写派生索引。
- 只读组合器 `scripts/test-governance/ace_m1_first_slice.py`（经
  `ace_m1_arch_acceptance.py` 的证据准入）消费正式 case-parent 事实边：不启动工具、不设
  第二 meter、不复制分析器；严格身份/路径/版本/预算（组合器活数据帽 48 MiB）。
- 组合报告 schema `ace.m1.acceptance/3`；总门谓词 `ace.m1.acceptance-reducer/1`（§5.2 八
  域门 AND；accounting/evidence/system 四类输入永不单独推导 can_claim）。旧
  `ace.m1.acceptance/2`（852 producer）原样并列，不重用其 schema 冒扩展通过。

## 真实签章与旧认证差异（AC-10/AC-11）

- `can_claim_expanded_m1` 仅当八域门全真且无 blocker；验收系统允许在真实 red + 具名
  blocker 下交付，绝不以验收系统实现 pass 冒 M1 通过。
- 旧 924 守卫语义随三域闭合（992..998：value 38 + trap 4 + scene 6）演进：旧门自身
  `can_claim_m1` 如实翻 True；expanded-M1 签章仍由本域八门独立裁决（legacy_current_gate
  只是其一）。cert-gap 计账 38→0 全程逐域接收、分母 hash 守恒、旧批准不改。
- D937 终态 bridge 走 store seam：组合侧 `bridge_store_query_observations` 消费 parent Job
  样本 + store（journal/bridge-scratch 产品面）；交换帧/nonce/request-hash 链在该 seam
  不可观测、不冒认。

## 交付面（2026-11-09，r11@9eb19674b 终审）

- 终面 verdicts 730P/0F/276U（registered 1006/31 合同）；acceptance_system_delivered=true；
  余七真实 blocker（legacy 执行 211 格、C-core/资源、Q4 oracle 认证、syntax 架构、精确
  增量 2 PARTIAL、budgets_time 14/16 窗、evidence_incomplete）全部为未执行产品义务，非
  验收系统缺陷。M1 里程碑未签。


## PLAN-1000 最终收口（2026-11-11，r1@84bf5fce8 + 两轮 LF 修复合并 04f3b410b）

- 45 个具名未闭键全部真实关闭（各关闭路径见归档计划 §9 round 43/44/47）：R05:avm
  （move 闭包按值捕获，`Closure.is_move` 消费于 `compile_closure`）；R02:avm（分配故障
  注入 seam：typed `VMError::AllocFault` + one-shot 旋钮 + avm `fail_alloc_at` 旋钮；
  GenName 静态调用修复消除泛型列表幻影首元素）；I12:a2r/avm（911 套件内注册红控，
  avm/a2r 适配器 expect-fail 准入）；I10:ac（mn store GC 观测面：软 root 化 + 有界
  历史窗 32 + 超窗摘引用物理回收 + `mn_incremental_store_512_revisions_gc_bounded`
  512 修订 churn+GC 判据测试）。
- 单终代统一重捕（freeze10，54 组单 invocation）+ 逐文件重指纹（avm 选项字段改变
  CheckSpec Debug 指纹：7 q4 合同、legacy-base、original-data×2、48 a2r 合同全部
  再生成）+ 20 个分域 phased 观察文件（8 认证族+7 执行族+frontdoor+ct_restore/
  incremental_restore/budget_time/architecture 四节 section 形态）。
- **终组合 1006 PASS / 0 FAIL / 0 UNOBSERVED、八域门全真、`--acceptance` exit 0、
  can_claim_expanded_m1=true、blockers=[]**。T-13 独立复审 PASS（签前凭据绑定冻结
  提交、双篡改负控、来源抽查；ARCH-08/fresh-union-spec-review 由独立凭据先行验证）。
- 941 的"系统交付而 M1 未绿亦可归档"完成例外**不再适用**：本包终验真实满足扩大 M1。
- 跨检出身份：pins 绑 LF 内容 sha（CRLF 工件修复两轮：5b2d8a987 两键、1d02cd41f
  53 文件规范化+级联；故意 CRLF fixture 不动）——master 与 plan worktree 读到逐字节
  相同的合同/源码，pin 链两边同时成立。

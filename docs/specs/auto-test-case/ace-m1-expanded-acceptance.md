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

## PLAN-1005 修正后完整重建与正式签章（2026-11-12，SD-02）

- 1000 的 1006P 终验结果保留为历史事实；本包在修复后的严格链上**以当前来源全量重采
  重建**（非复用历史摘要）：identity 合同 7/7 与 registry 冻结值逐值零漂移；四套件+
  45 组+终浪 9 组+六执行族+release/query 波+16 负面前门全部单终代真实捕获；48 份
  a2r/probe 族逐套件身份合同重建（48/48 pin 零漂移，F-1005-03 修复 1000 落地缺口）。
- **分域里程碑链**：68P（首次组合）→146P（legacy 三链）→226P（original_data）→
  273P→370P（probe 族）→388P/411P（base a2r+probe 双映射）→425P（rust 观察源）→
  433P（治理轴）→453P（text 认证）→635P（scalar/RT/coll）→673P（query/CTE/supp）→
  725P（orig 认证 52 行）→961P（执行六族）→999P（query 执行）→**1005P/0F/1U 冻结候选**
  （负面前门 6 行；V-12 目标）→凭据 R→**1006P/0F/0U、八域门全真、blockers=[]、
  `--acceptance` exit 0 可签**（t08/final-report.json）。
- 签前凭据 R（`b9304ef0…`）经消费链接纳解锁唯一 review 行；预签候选（无凭据组合）
  恒不可签 exit1——正式三态（exit0 可签/exit1 合法未签/exit2 坏身份）全部实测。
- TAG 复核（TAG-01..06 逐门结果 t08/tag-review.json）：候选树净、1006P/0F/0U、凭据
  有效不自指、反控全绿（287+1189）、来源闭包对账、本地无 `v0.6-ACE-M1` 标签
  （tag_ready=true 记录于计划；实际打 tag 为复核后的单独动作，本包不创建不推送）。

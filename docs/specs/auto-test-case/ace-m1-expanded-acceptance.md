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

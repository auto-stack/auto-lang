# PLAN-824 r8 受影响独立复审

- outcome: needs_fix；reviewer: /root/review824_sync（用户授权独立只读上下文）
- reviewed: 1e9c1686a2c0e8d6640346f7f5ab77910e64c6ab；parent: fd48dca605f8419d5787127d793d72b5812a79bf
- F-R8-01：824 baseline campaign 直接运行也无条件采用 P90<2000，违反仅门禁显式启用的合同。默认应保留原 P95<1000；须验证默认、启用、无效策略的真实接线。
- F-R8-02：818 body/rebuild/conflict 定向重放证据未齐，原始样本/P90/P95 不能全部从旧日志核验。须定向运行各实际入口、保存原始数组与 P90/P95，验证默认及无效策略不误用宽松门。
- 已核：796/798/818 的每样本正确性、结构、有限正值、严格边界和824≥30样本；828 campaign 的失败、结构、完整性和零门限负控；798/796/818-series/824 campaign 原始日志各30样本，策略边界2+1通过，guard自测通过。797测试不在日常或824 acceptance，未漏改。原有43条根测试编译警告需如实保留。
- merge-owned cargo t/tv及完整acceptance尚未运行；未经修复复审不得执行或合入。

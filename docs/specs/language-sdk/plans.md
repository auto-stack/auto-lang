
# Language SDK plans

| Plan | Change | Evidence |
|---|---|---|
| 840 | 依赖扫描共享词法状态，注释/字符串不误判；真实use/dep保留 | [archive](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md) |
| 840 Phase 2 | 可选Case测试注册、真实Session阶段计量与计时资源前提 | [archive](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md) · [review](../../plans/attachments/840/phase3/review.md) |
| 840 Phase 4 | Windows状态文件锁瞬态竞争有界重试，保留超时/硬错误降级 | [archive](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md) · [review](../../plans/attachments/840/phase4/review.md) |

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.
| [841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md) | 纯语言 SDK 保持无 UI 实现反向依赖；UI 上层经显式领域端口与受检 lowering 消费。 |

## PLAN-842

- [PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)：兼容入口按 0.2.0 边界退役，正式保留面、来源校验和迁移见项目规格。
| 843 | 七热点+根 lib 按语义职责拆主题模块（parser/codegen/engine/native/ffi/trans 家族+entry/flags）；纯搬移零算法变化；结构合同沉淀为 [architecture.md](architecture.md) | [plan](../../plans/843-language-core-structure-and-module-ownership.md) |

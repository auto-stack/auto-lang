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
| 873 | 构建成本基线测量与 member 拆分决策（THR-D1 闭合：暂不拆；触发条件 T-A..T-D）；crate 粒度与增量成本现状入 project.md | [archive](../../plans/archive/873-build-cost-and-member-split-decision.md) · [build-cost-baseline](../repository-boundaries/build-cost-baseline.md) |
| 843 | 七热点+根 lib 按语义职责拆主题模块（parser/codegen/engine/native/ffi/trans 家族+entry/flags）；纯搬移零算法变化；结构合同沉淀为 [architecture.md](architecture.md) | [plan](../../plans/843-language-core-structure-and-module-ownership.md) |

## PLAN-884

| 884 | language-structure-closure（运行桥接/stdlib 装配/Rust 转译职责） | ✅（reviewed→archived） | archive/884-language-structure-closure-without-crate-split.md | 运行桥接单源（G1/G2/G3）+ G4/G5 刻意分离；stdlib 域装配 13 域/19 个有序注册 helper/指纹闭包+V-09 守卫；call_inner 22 族 CallShape 三态；rust 主管线 9 主题模块+收集器去重（param_types 分叉留债）；见 [structure-closure.md](structure-closure.md) |

| 890 | AutoWeb独立仓与四仓交付 | 已合入；归档收据为准 | [PLAN-890](../../plans/archive/890-auto-web-independent-repository-delivery.md) | 中立local HTTP宿主/声明与launch快照，显式服务装配 |

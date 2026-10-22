# PLAN-830 r2：规划预检与AutoShader边界

2026-10-22。本轮用户已授权830与后续路线文档修订及小范围AutoShader设计，明确暂不实现。原r1调查见[preflight](preflight.md)，其旧f7候选/日期/状态仅历史有效。

## 当前观察与准入

829读取时frontmatter为reviewed、10/11，R18独立PASS所记ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74；merge-owned门及生产迁移交接仍pending。未核得满足G829全部条件的正式D829/获准E829消费收据，G829未放行；本轮不代发布、不建实施树。实际工作仍须T-01重新核来源，不把源码/哈希或reviewed当发布授权。

现行auto-hir/auto-mir Spec为有限类型/执行面，auto-graphics canonical目录本次仍不存在；本轮不写current Spec。候选AGP的手写WGSL、Device/Queue借用、target usage及BadShader只作为接缝观察，不是重新测试结果。

## r1→r2覆盖核对

- T-01：当前r2及实际开工授权/来源核验；T-02/04/08：资源布局事实、局部renderer分层、交接。
- 新合同§5.6收敛到V-10/AC-08与SD-04；不加任务/测试族。current_step仍0、total_steps仍10、全部未勾选。
- AC-01..07及AC-09/10、S-01..04、V-01..09、T-03/05/06/07/09/10逐行与r1核对保持。G829、生产迁移门、Windows单窗profile、真实Iced/中文IME、兼容/资源/反控分母保持。
- 830仍可用手写WGSL通过；不要求生成AutoShader、CAP求解、通用Shader插件/加载/替换/热重载或第二设备/后端；不存在新增自动fallback。
- AutoShader草案不抢占域章编号、不分配实现Plan。现在设计；未来具备ACE/图形接口后最小切片与有限CAP接通，不把AutoShader设为基础图形前置，不等待完整CAP全部profile才验证。
- 验证为规划结构/来源/链接及人工差异审查；静态检查不能证明方案正确。本轮未运行Cargo、GPU/窗口/IME或未来Shader样例，也不是独立final review。

## 读取时来源 SHA-256（改写前）

| Source | SHA256 |
|---|---|
| `docs/plans/829-autographics-independent-research-and-game-prototypes.md` | `fe5dc04e917e714c1e32d2e77da2fdc77ac25ed4f9b506f3ec05bb29017fedd0` |
| `docs/plans/830-autographics-surface-portal-composition-and-input.md` | `00b81c51941df20a758bc271cd9752d67c44ae5a2cdb3b277f383900a79322ca` |
| `docs/specs/overview.md` | `c23beeb31d1b44f3aa9ace38b9aae10482bf2595152d1c6f78a2e767f0de90a4` |
| `docs/specs/auto-hir/project.md` | `0bd3d9c90536681cd557bba38f9bffc7752f462c185e6918734f52e13eb12966` |
| `docs/specs/auto-mir/project.md` | `af021bcdc0d05a374dd09ac5887081c4281d53908002f6ee1d97969882d9d37d` |
| `docs/specs/auto-lang/ui/overview.md` | `80b0dce1066f5eaf608ef457829a33d1ad2dddaaf095f2dbe79ba6f572253b97` |
| `docs/specs/repository-boundaries/project.md` | `5d83f8e5767e9784b0aefaef3b5b3c2c40ec28a3e0c47459f8164ee5e652f764` |
| `docs/design/46-context-profile-capability-cross-ecosystem.md` | `2a7a044719c0d753992ea54b5f41decfc2738d0f7e5cccb402e769fa60cd1ca9` |
| `docs/design/strategy/cross-ecosystem-programming-strategy.md` | `2c6da699e755964eb7d1ed1eff0cc795fc4546fd4046f21cfa059bb63f6c370d` |
| `D:\autostack\.wt\lang-829\auto-lang\experimental\autographics-prototype\src\renderer\pipeline.rs` | `230bc668ab446583bf56e06563010bcc7a4873756fe4a471eb3ff4ee2b146b22` |
| `D:\autostack\.wt\lang-829\auto-lang\experimental\autographics-prototype\src\renderer\target.rs` | `56b0da7a166dff9004a9ffd08402b7e4357547f8eeab9827b64c5c20e7516f46` |
| `D:\autostack\.wt\lang-829\auto-lang\experimental\autographics-prototype\src\renderer\error.rs` | `6b7bfc0f19771a3f056b912c78c57ceb910e90bf2807749b8397db00576839ae` |

这些hash是读取内容身份，不证明树clean、正式冻结、已发布或消费许可；代码/依赖变化后须重核。新设计与本Plan输出身份记录在本次文档交付manifest中，读写前后都有内容校验。

## 结果与下一步

`stage: new`，PLAN-830 revision 2，drafting/0 of 10。规划合同预检通过；未来work入口的`outcome: blocked`只表示G829未核得，不表示本次文档任务未完成。当前交付结束；将来单独启动实施且有效来源成立后T-01核源进入，不预勾review/merge或创建实现树。

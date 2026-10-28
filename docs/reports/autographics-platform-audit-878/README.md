# AutoGraphics 跨端边界与依赖审计（PLAN-878 / XP-0）报告

只读静态审计交付。固定输入：AG `8b12632d7b5b1b0df72fbb4eef84297a054de31d`
（Plan 861 冻结）、旧 SDK 内容锁 pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`
（681 文件/17 本地包，plan-860 保留副本）、auto-ui `7095f536747e5180338a470a7d73452c6b550ed1`。
审计合同与执行证据：`docs/plans/attachments/878/`（合同 md 入库；原始
metadata/扫描输出本地 `D:/autostack/plan-artifacts/plan-878/`）。

## 文档索引

| 文档 | 内容 | 结论一句话 |
|---|---|---|
| [inputs.md](inputs.md) | 输入身份与核验（12 AG blob、681 SDK 文件、6 auto-ui blob、10/10 metadata） | 全部固定输入核验通过；android/ios 工具链未装（未编译） |
| [dependency-map.md](dependency-map.md) | 依赖图（两 workspace × 5 target）+ 222 第一方文件逐文件分类 + 三对照核验 | shared-candidate 22 / adapter-needed 16 / host-only 73 / evidence-tool 106；SDK 宿主栈（axum/tokio/reqwest/notify/libloading…）在全部 target 无门控 |
| [platform-seams.md](platform-seams.md) | 六触面对照 + DOM/Canvas/3D 边界 + 移动两路线缺口 + 共享候选总表 | core 语义族可共享（有禁令注释为证）；渲染 adapter-needed；宿主/UI/存储 host-only；7 项 unresolved |
| [next-probes.md](next-probes.md) | XP-1 具体入口、P-0 零成本预探、正负 oracle、阻塞清单 | audit_complete=**complete**；next_probe_readiness=**limited**（R-01..R-04） |
| [migration-readiness.md](migration-readiness.md) | AG 迁入 auto-ui 的纯搬仓评估（目标侧事实、触面清单、门分离） | 默认 UI 零新增依赖成立（静态判定）；SDK pin 对象不可达=迁仓前必须解决的保全风险；四门不捆绑 |

## 收口声明（防越界）

- XP-0 完成 = 有可核查审计；**不等于任何新平台可运行**。
- 不宣称：WASM/浏览器/移动可编译或可运行、跨端适配完成、Gallery 内容就绪。
- 未测项全部显式列名（U-01..U-08、H-01..H-03、R-01..R-04），无一伪装成支持。
- 迁仓/ACE 融合/切默认/退役四门独立；本报告只提交建议与事实，不启动任何后继。

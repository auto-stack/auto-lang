# PLAN-738 Phase 3 / T-13：AC ↔ 任务 ↔ 证据 ↔ SD 对账

- 绑定：Phase 3 提交链 `2c1b4a763` → `c1579ed71`（T-10）→ `84bec29ef`（T-11）→ `b6cfc6df1`（T-12）；SD 终稿 [738-sd-drafts.md](738-sd-drafts.md)（同轮 T-13 修订）。
- T-14 最终门禁已完成（[738-phase3-verification.md](738-phase3-verification.md)，最终提交 `b3a4d660e`）：裸 t 16 红全分诊/tv 162/tt 非基线=0/th 逐名分诊/服务链 1/1/双 witness 1/1——下表各 AC 的"待补"项均已在最终提交闭合。
- 完整分母重验：全库 inventory 分母/解析失败冻结由 `full_scan_no_silent_skips_and_frozen_parse_failures`（115 .at 分母恰清点）与 [738-stdlib-matrix.{json,md}](738-stdlib-matrix.md)（六模块×target×env 全格，非 Supported 必有原因）承载——分母是 inventory/矩阵面，非 call 单站点。

## AC-01 全库清点与六模块公开符号完整

| 证据 | 任务 |
|---|---|
| inventory 115 .at 分母（74 公共+23vm+13rs+5c）恰清点、失败诊断不吞；六核心 13 层 parser 分母冻结（11 OK+async/json.rs.at 破损在案） | T-01/02（历史）+ plan738 族持续绿 |
| 566 文件 R2 差异全归属（552 纯格式机械验证+4 注释级+2 金样+8 功能） | **T-09** [738-phase3-scope.md](738-phase3-scope.md) |
| 状态：**满足**（T-14 最终档：裸 t 分母族全基线）；SD-01 | |

## AC-02 生产 VM/Rust/C/persistent 共同装配计划

| 证据 | 任务 |
|---|---|
| AssemblyTarget 驱动层选择/persistent 双 bug 修复/VM 两入口同可见；同源 witness VM=41/Rust=42/C=43 | T-03（历史） |
| 适配契约注册表（17 条）四方一致——Rust 适配链有独立可检查契约，无 blanket 豁免/Resolved-only 放行 | **T-10** [738-phase3-strict.md](738-phase3-strict.md) |
| 引用闭包全路径（导入/裸名/通配/三段/反糖/Await 臂/生成 back 模块） | **T-11** [738-phase3-reference-audit.md](738-phase3-reference-audit.md) |
| 状态：**满足**（T-14：⑤腿 Rust 实编 1/1）；SD-01/02/03/04 | |

## AC-03 核心声明/实现/实际 native 绑定校验

| 证据 | 任务 |
|---|---|
| resolved+bound 双面成立才 Supported；ID 别名/冲突检测；独立 producer 契约（production() 回填） | T-04（历史）+ R2 |
| ADAPTER_RULES 契约驱动验证（cast/侧信道/元数分派/facade/参数 facade）；漂移=DRIFT、无证明=UNVERIFIED，正负例 10 测 | **T-10** |
| 状态：**满足**（T-14：plan738 69/69 契约正负例）；SD-01/02 | |

## AC-04 源段错误位置与 use 隔离

| 证据 | 任务 |
|---|---|
| 目标层语法错误归因真实文件；use_semantics 117 回归/双 session/root 守卫 | T-03/05（历史） |
| 导入闭包不破坏 use 语义：同名用户 fn 遮蔽优先/未引用 unsupported wildcard 不拒/545/635 语义回归绿 | **T-11** |
| 状态：**满足**（T-14：use_semantics/裸 t 回归全绿）；SD-03 | |

## AC-05 target/environment 与核心能力严格区分

| 证据 | 任务 |
|---|---|
| 六模块全矩阵（非 Supported 必有原因）；Browser 三族 Unsupported；Vue 不改 Native | T-04/07（历史） |
| RuntimeSpec 分家（post_bearer/3 参 post=Embedded 限定）；json.is_valid int/bool 漂移跨拼写一致拒绝；C ext 面 TARGET_UNSUPPORTED | **T-10/T-11** + R2 |
| 状态：**满足**（T-14：矩阵族 69/69 内复跑）；SD-02/07 | |

## AC-06 内容/目标/provider/依赖变化正确失效

| 证据 | 任务 |
|---|---|
| 段级指纹/同 mtime 失效/absent 台账/target-root 热换拒绝/依赖失效；内容级指纹敏感性+新鲜度真值表 | T-05/06（历史） |
| 双指纹失效面（target 变化改共同身份；业务输入只改收据身份；同内容异根稳定）；workspace lock 出现/变化/缺字段→陈旧 | **T-12** [738-phase3-manifest.md](738-phase3-manifest.md) |
| 状态：**满足**（T-14：服务链陈旧→再生臂 1/1 实证）；SD-01/03/05 | |

## AC-07 CLI 与生成 HTTP 服务真实消费 manifest

| 证据 | 任务 |
|---|---|
| CLI 三模式/退出码 0/1/2/3（实测）/JSON 稳定；真实生成→实编→serve→ready→业务→stdlib-only 失效→再生（R2 服务验收 1/1） | T-06/07（历史，R2 收据） |
| 三角对拍（CLI actual ↔ 会话快照 ↔ 生成收据共同身份全等）；back 模块证明并入 manifest；SD-06 退出码 2/3 纠正入终稿 | **T-12** + T-13 SD 稿 |
| 状态：**满足**（T-14：服务完整链 1/1 @b3a4d660e）**；SD-05/06 | |

## AC-08 验证等级、兼容与规范可沉淀

| 证据 | 任务 |
|---|---|
| R1..R9 修复全档；分级门收据（R2）；729/730/734/736 兼容回归持续绿 | T-02..08（历史） |
| 566 差异全分类零未知；strict 门/闭包/双指纹三轮零新增确定性红（tt 非基线红=0 两轮）；SD-01..07 终稿可应用（本文件） | **T-09..T-13** |
| 状态：**满足**（T-14：最终档零新增确定性红+SD 终稿在案）**；SD-01..07 | |

## 原 T-02..T-08 对账说明（不因新 phase 批量关闭）

- 历史 `[✅]` 证据保留为执行记录；R1 独立复审判 needs_fix 后其验收由 Phase 3 对账逐项收回：T-02（模型/inventory/对照）→AC-01 ✓；T-03（装配接线）→AC-02/04 ✓（T-11 补闭包）；T-04（native 校验）→AC-03/05 ✓（T-10 补契约）；T-05（缓存一致性）→AC-04/06 ✓；T-06（CLI/收据）→AC-07 ✓（T-12 补三角+lock）；T-07（矩阵/witness）→AC-01/05 ✓（T-14 复跑 witness）；T-08（门禁/报告）→AC-08（T-14 最终档闭合 ✅）。
- 未批准遗漏不转债务：既有登记 P738-D1（13 处 id 相撞 D3b 重编号）/P738-D2（json 面分裂/第四绑定面）维持原状，边界在 T-11 审计表显式引用。

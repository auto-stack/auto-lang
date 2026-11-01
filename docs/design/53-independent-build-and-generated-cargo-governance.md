# 53 · 独立构建边界与生成Cargo产物治理

状态：target-state r1；2026-11-01。当前只规划，未执行探针/测量/构建/清理或修改生产依赖。

承接[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[Design50](50-autoweb-service-framework-and-repo-boundaries.md)、[Design52](52-single-compile-profile-consolidation.md)。统一的是来源、依赖与构建生命周期的可验证边界，不预定全局共享可写target，也不把源码分仓当独立构建已完成。

## 1. 两个独立目标

| 目标 | 主责/计划 | 完成边界 |
|---|---|---|
| 生成Rust工程/Cargo产物治理 | 语言侧持[PLAN-905](../plans/905-generated-cargo-artifact-governance-research.md)，Man/UI/Web按实际producer/consumer参与调查 | 本期有界实测、方案选择、有限验收及可执行后继合同输入；研究完成不自动代表Cargo问题/M1已闭 |
| 语言仓不依赖Web兄弟检出 | 语言侧持[PLAN-906](../plans/906-standalone-language-web-boundary.md)，Web/UI/Man参与必要接收 | Core独立消费和完整语言workspace分别证明；不能只改Core的轻合同path就冒全仓独立 |

二者没有无关全局先后；共享source_owner/rust_workspace/SDK投影/锁/发行入口改动须短窗单写。906改变依赖身份时，905沿新SHA更新对应观测，不混算收益；UI014结构/AG020产品在各自合同执行，不追加全线前置。

## 2. Cargo治理：从已存在机制出发

实际代码已有auto-generation-core::rust_workspace（目录解析/成员命名/共享workspace/target/lock身份）、source_owner（受管来源校验），UI rust_ui和Web/Man服务调用共同消费。TOML/config/environment及具体cwd会影响真实落点，已有机制≠已证明跨项目隔离、稳定性及物理减量。

905先用至少一个真实UI生成工程、一个真实Web生成工程，测完整依赖/应用/调试/增量/历史缓存账；再比较稳定写入/稳定成员身份、分兼容环境workspace、独立target配编译缓存等候选。没有证据不重造auto-cache，不预定一套巨型workspace，不合并多worktree可写fingerprint/incremental。Kache迁移已由用户取消，本设计不复活它，也不自行改文件系统。

缓存/产物复用必须绑定源与导入、generator/SDK/runtime/resource、lock/feature/profile/triple/flags/toolchain及build-script/环境实际输入；未变输入不重写失效，真实变更不错误复用。支持多项目/多工作树/版本并发、取消/崩溃与恢复；清理只能针对受管自有可重建内容，活跃产物/源/用户数据不误删。收益按store+所有实验工程的完整占用及迁移峰值核，不只看单目录或逻辑求和。

## 3. Web依赖边界：必须先冻结完整闭包

当前至少三条活跃边：Core→auto-web-contract、facade auto-lang→auto-web、auto-api-gen→auto-web；Core的http_service_config.rs整包再导出轻合同，该合同同时含严格解析、hash、CORS/Host/可信代理等Web政策，不能未经分类全搬回语言。

906先冻结L1（Core/API独立消费）与L2（语言workspace完整普通构建）清单、直接/传递/optional/build/dev依赖和当前SDK/生成/测试调用。候选：

1. 只将语言宿主真正需要的中立端口/数据交给合适基础owner，Web纯策略/算法留Web；显式adapter保持。旧公开类型路径/序列化/配置语义的兼容须真实证明。
2. 已有受管固定发布/离线依赖渠道若可承接，按固定来源消费Web轻包/必要组合，明确冷源可获得性；不能依赖开发机绝对path、未发布版本或global cache冒fresh可用。
3. facade/generator剩余Web实现边改为实际域owner装配，必要compat adapter按版本明确退出。不能悄悄删workspace成员、optional包或测试以凑“单仓”。

以上仅候选，不承诺复制整Web实现、搬全部政策、改ABI或另建仓。方案涉及破坏公开API/跨owner归属/删除兼容承诺/需要远端发布时，906 T-02给具体迁移合同供用户确认，未确认不进入依赖实施。保留一个真实类型owner；不复制同名Rust类型/政策，也不删除links冲突守卫来容忍同名双source。

**T-02 冻结（2026-11-01，探针 P-A/P-B/P-C 全绿）**：906 选中候选1+3组合（C1）——`auto-web-contract` 包逐字迁语言仓（Web workspace 去成员、auto-web path 翻转回指语言单源；links/类型身份/hash/严格策略随包不变）；facade 与 auto-api-gen 的 auto-web 边清零（facade 公开 Web 再导出面四仓零外部消费者、Man/UI/生成代码均 auto_web 直连；`auto_lang::http_service_config` 改指 core 再导出，RateLimiter/ServiceRuntime 留 Web）；http e2e 家族迁组内独立探针工程显式装配 Web host；source_owner/交付投影/安装脚本按新源身份接线。C2（合同留 Web+固定渠道）有据否决：registry 未发布、无 git remote、vendor 复制=第二套实现且不解决实现边。负控实证：未打补丁清单树 metadata 红于 `auto-web\crates\auto-web-contract\Cargo.toml` 缺失；C1 正控全 workspace 无 Web 兄弟 `cargo metadata --offline` 绿。完整设计/兼容差额/回退见 [PLAN-906 §5.5](../plans/906-standalone-language-web-boundary.md)。

## 4. 验证和退出

两个计划都用专用worktree。外部验证在统一plan-artifacts scope隔离，无Web/UI/Man兄弟目录、无相关env/compiled-in fallback、保必要registry输入；不得移动/隐藏真实用户仓来制造环境。L1 Core API/VM正反消费与L2根Cargo metadata/build/普通固定组实际读取路径分列。服务装配仍显式消费Web，跨仓新组合fresh/locked/重定位/错误恢复和原子测试义务保持。

905只读/实验结果、906实施结果均需独立上下文review；源码/日志/生成工程/外置target只在计划scope，merge必要摘要折计划后全删，唯一具名未解决问题可持有。canonical在docs/specs，legacy specs.json只读，应用派生索引。

官方研究依据（2026-11-01查阅，实施仍核本机Cargo版本）：[build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)、[configuration](https://doc.rust-lang.org/cargo/reference/config.html)、[dependency overrides](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)。这些是机制依据，不是本机收益或候选可行性证明。

## 5. PLAN-905 有界实测结论（2026-11-01 交付，研究非实施）

基线与全部实验证据折[PLAN-905](../plans/905-generated-cargo-artifact-governance-research.md) §9；要点：

1. **现状判定**：Web 消费者路径（serve_rust/仓外 app）逐 app 本地 target 整编 auto-lang 闭包（生产 auto-chat 17.93G、auto-blog 8.48G，与 auto-lang/target 14.74G 三份同源 deps 互不复用）；rustflags 三源分裂（语言仓 config=lld+32MB、生成 ws config=256MB-STACK、裸默认）使共享 target 内部再裂双命名空间（实测 659 单元全量重编，census 落盘；单命名空间规模以 E2-S1 9.77GiB unique 佐证）。UI 路径 env 钉共享 target 但与语言仓自身构建混用同一目录（owner 混合）。
2. **复用成立的条件（实证）**：同 profile 同 flags 下，第二 UI 工程 22s/+1.11G（10x 时间）、Web release 跨消费者入共享 target +0G/6.8s 仅编译成员；取消/缺件/坏件均可恢复（fail-loud 或增量自愈，坏件需按单元 fingerprint 定位清除）；跨仓混合锚定 cargo fail-closed（好性质）。
3. **选定方向**：生成消费者统一共享 target + flags 单源化（§2 候选 B，按 profile 自然分 namespace）；不取内容寻址重建、不动 sccache、不合并多 worktree 可写增量。有限 M1 验收出口与后继实施合同输入见 PLAN-905 §9 T-05 记录（**后继实施不自动获权**）。
4. **移交缺口（不属本研究修复）**：生成成员键=项目 basename+mtime 判定 regen → 同名工程互劫持（双向实证）；needs_regeneration 无内容 hash；examples/ui/013-todo rust 轨生成编译失败（owner=auto-ui/UI014）。

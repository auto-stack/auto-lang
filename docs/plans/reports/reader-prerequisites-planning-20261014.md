# READER-001 框架前置立项输入（2026-10-14）

stage=new；只读调查 + 设计/计划文档；没有运行功能探针或赋予任何功能 PASS。

## 授权与基线

- 用户原文：“请帮我在auto-lang立项。”承接 2026-10-12 保留 Reader 原验收目标、单独推进框架前置的决策。本轮授权设计与立项；不等于执行、降标、跨仓业务修改或全工作区提交。
- auto-lang 主检出 D:/autostack/auto-lang，master；取材 HEAD=c171c1a09740fbd61e4bf6235286a3424e2e4a81；中央取号提交 db0f2eee12a09a5fe9e289587eac4440bdf5025d，781/782/783 唯一，next-id=784。既有 .next-id=781、磁盘最大前缀780；已保留777–780的预留号码。其他文档写入者仍活动，实施入场重核实际HEAD/接口，不以本轮取材SHA宣称未来实施基线。
- 分配通过 scripts/new-plan.sh，在 Local\AutoLang-PlanAllocation mutex 下串行，创建前/后核对 active+archive 唯一性。该脚本自身没有全局锁，后续写入者仍须统一串行分配。
- Reader 主检出 D:/autostack/auto-reader @29b5decf878ec2cdca5c6a39575e872a4597121d，计划 r7/executing；源码工作位按其仓规约为 D:/autostack/auto-os/apps/018-book-reader。001 原任务/AC/范围保持。
- auto-lang 入场已有其他 agent 未提交资料：design/00-intro、.next-id、742/744/770/776、KNOWN-DEBT、Design37、777–780及742/774/776证据等。仅中央计数顺延与本轮新文档/索引条目属于此次提交；不提交其他正文、实现或证据。Design37 在本地存在但尚未提交，本轮只记录其 hash/协调边界。

## 冻结取材 hash（SHA256，小写）

| 文件 | SHA256 |
|---|---|
| docs/specs/overview.md | 9784c6d8cc72c83e8c570da0c2ae469c66508ec5b427448e6e6d7d00a8e9aa16 |
| docs/specs/goals.md | 2bd9980c59477f4536c4047a97387bc68e6ab79c8662f342a1f8fd8ea6d1273d |
| docs/specs/auto-lang/ui/overview.md | e72027a96acf11dbe496fb19c6b12d76f1dc41788a1bd493290c0dff639adeaf |
| docs/specs/widgets/scroll-pane.md | 03e8d8b9b039152733a7f4b1e7ad864bf40a1fd5c6302e4b1eee8000360825aa |
| docs/specs/auto-lang/vm/overview.md | 0e22704f9a678f775633a706db8c6583a60218589b8b7dc3f87ccd1b30ccf3c3 |
| docs/specs/stdlib/design/http-server.md | 8e1a802426a1d42499e7f36e964d84747a1babc93812341752924f20fb07ae25 |
| docs/specs/stdlib/design/http-service-deployment.md | dc301b66d551b3411bff784ca2b571c4fb0b3fe4ea3f44fb7b3599e270b0ad10 |
| docs/specs/auto-lang/ui/design/select-anything.md | 7eabeb150de3a3bd391df42e0d74d5692929ccf607fd907b767632b2d6734795 |
| crates/auto-lang/src/ui/scroll/controller.rs | 1fadb7485a63ab065bf5900e359614177a7451bc199ec2487e8979804baaf795 |
| crates/auto-lang/src/vm/engine.rs | 94b93f71fe46f23a97a32d27c30e02e372ffb5165d8911fbe2f7eea41b5e420a |
| crates/auto-lang/src/vm/ffi/http_server.rs | 9690154e31dc38a634de84402f7cee3c4628bdb49f614c2059f993d52e018958 |
| crates/auto-lang/src/ui/mcp_server.rs | 7ac9102c2dc79e2a33114d8b9b8795d6847577d28221447eb3ee74d3dce1c4f9 |
| .agents/skills/autoui-verifier/scripts/test_vm_mcp.py | 2c2ae63ba8351ada93ab96928dae51abfc6dd6077683fcf0d57c3434c7658696 |
| docs/design/37-v06-modularization-and-repo-boundaries.md（入场未跟踪） | 6a67e0aa102fb41f064b86c623393007193ce547e8ab837d527492894df5d098 |
| auto-reader/docs/plans/001-real-library.md | 27ee1866eb1fa7c8e296eacc6fcce0ce56954de2c8570a7e57dc3866a6e53af7 |

## 源码核查结论与覆盖边界

1. native_catalog.rs:541–546 为9900–9905 controller/offset/state六函数，未找到 child-anchor 公共入口；controller.rs 未预热 intent 回队，已存在 priming/回声纪律，需保留2026-10-11滚动抖动修复。
2. engine.rs 的 CpuSliceBudget.cumulative_steps 可在 Rust 内部配置；Legacy 在 drive loop 仍硬编码10M。http_server.rs 同步命名、闭包与 segment入口不同。不能从 UI 常量推断真实 HTTP 导入路径已支持可配预算。
3. ui/mcp_server.rs 有 autoui_action、vtree、布局 bounds 与 screenshot；select-anything Spec 明示部分 bounds 仅debug覆盖。test_vm_mcp.py 默认仅 run -r vm，Reader 的 split VM+HTTP 需要 --server=vm；现有桌面 acceptance 通道也不能直接证明 Reader 独立窗口恢复。
4. reader reading.at 有 controller rsc + 字符占比估算；段落 idx=原始行号，过滤空行后显示序号不同；tests/real-book.spec.ts 的 T-R10 仍 fixme。t04_http_verify.py 的10MiB项目前只要求明确失败及库不变。
5. 这是静态立项证据，运行成功、资源阈值和新 API 均未测；由对应 T-01/T-02 产出冻结探针与新测试，不把预期病例冒充已有证据。

AGENTS.md hash=B5540A7FE007665981974B6B9EDF0F8DB8DBA4D78403E678ECF32B692648BD3A；适用新计划→worktree→独立review→merge规约。测试门以 AGENTS 的2026-09-30分级规则为准，Spec README 内旧 per-plan cargo tf 叙述不作门禁。本次文档工作禁止跑 cargo t/docs_gen。

## 立项预检与交接

三份草案均为drafting/r1、0/8，分别包含0–10十一节、8个唯一Task、6个AC及Spec delta；全项未勾选。已检查中央编号唯一、next-id=784、Task/AC/SD引用、文档本地链接和新增文档格式。功能病例均为拟验证项，没有运行或通过声明。

推荐顺序783通道→781锚点→782导入；调查可提前，生产共享热点接线按当前owner串行。立项后下一阶段为用户确认执行范围再进入work；首任务均先冻结具体接口/资源/观测决策。本轮交接只完成设计与立项，不创建实现worktree、不执行Rust/窗口/导入、不更新canonical Specs/ledger、不授予001终审pass。

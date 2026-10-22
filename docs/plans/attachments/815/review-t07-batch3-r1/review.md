# PLAN-815 T-07 子批3定向复审 R1

2026-10-22，**needs_fix：三项原 P2 两项 CLOSED、一项 PARTIAL；无新增 finding。**

本轮对象是 T-07 子批3修复，不是已完成的 T-03 重审，也不是 T-09 全计划终审。T-07 整任务继续未勾，Plan r3 保持 executing 10/17。

## 冻结及独立性

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`；HEAD **9aee8659d8571ad889d5ccc46086c5102d8f64d6**，base **503c9f67f5fca40661ca59a230dcd79c8418ef90**，生产修复源仍为 e22fc75924a7f058a5df8150956b2d66268c5ecd。M814-B 交付祖先 93a25c924d1beafab130588bad4d1041dc86e48c 保持。
- reviewer=Codex `/root` 延续独立审查上下文；有早期合同/账本参与限制，未参与本轮实现，未创建 agent/第二实现树。仅在原树检查和定向测试；全新独占 scratch 用于交付产物和负控，不改生产代码、canonical Specs 或 live ledger，不合入/归档/清树。
- 入场/终核同 HEAD clean；5943 项源码/依赖/配置输入与 282 项当前 Spec 输入无漂移，见 [baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。rust_ui 的生产前缀相对 base 逐字不变，其改动均在 cfg(test)；另一个 Rust 文件为集成测试。
- 入场 worktree tool SHA256 为 `3e1c500d9d151834216375ede77e1a66b388259eae645f75efc0b6d8e2dcf9bd`；定向 cargo test 重建后为 `753b1d71a43d3505e9f8266a2906f830ef68f684dfecc7277b25ad2f52fb0505`。本轮发行和独立字段/资源对照统一使用本轮新建包 B 工具，SHA256 **d2d0054ce8055abb701e5fe9fc3d4e58d9b533392c4e92c3cd0288d01d6d8f87**。不把不同路径/构建配置的 debug 二进制称为相同字节。handoff 首表 e319…是历史源构建指纹，T-08 定稿须据实际冻结产物刷新；本轮未授予 T-08 pass。

## 三项关闭表

| 原 finding | 结论 | 独立核验 |
|---|---|---|
| F-815-T07-B2-R1-01 / P2 新 tool 发行面缺证 | **CLOSED** | 扩展 driver 在新 scratch 全程 **33/33**，旧 25 项保持；A fresh app 构建 113s、B 重定位 app 构建 152s；A/B 随包 tool locked 构建 59s/56s。真实 framed 客户端 describe 五操作、validate、generate、组件物化与坏帧/未知版本/不服务 profile 均通过。另补未知版本 generate 的实际副作用负控及包内资源标记区分控制，均通过。 |
| F-815-T07-B2-R1-02 / P2 资源审计事实错误 | **CLOSED（原资产事实与发行路径范围）** | 三份文档纠正 Inter/OFL 和源随包事实，八组实际资源 **377 文件**独立逐字节对账全部一致；字体/许可、壁纸、壳声明及原三组资源都位于新包 B 的对应源位。组级 LICENSE/SNAPSHOT 在包内；保持 app allowlist / vendored 构建输入 / 编译后 embed / materialize 四条路径区分。 |
| F-815-T07-B2-R1-03 / P2 字段×profile 消费矩阵不足 | **PARTIAL，仍待修** | 新增三项测试都通过，mode/output_root/input_sources 准入、相对锚定、请求 authority 及输出根布局原例证据有效；但新表把数个实际生效字段误写为仅携带，不能据此宣称逐 profile 完整闭合。详下。 |

## 剩余 F-815-T07-B2-R1-03 / P2：矩阵把真实消费字段标为“无生成消费”

**现有规则**：AC-02/04，Plan §5.1/V-02/V-04 及上轮原关闭条件要求逐字段按实际支持 profile/操作、单源、消费点及 oracle 对照。不是要求所有字段都必须生效，也不是新增支持面。已发布 `docs/specs/project-model/design/generation-request.md` 的消费接线明确包含 PacSource 的 window/title/api/dep_roots 等读取。

**冻结输入与误述位置**：`docs/plans/attachments/815/t07-acceptance/batch3/field-profile-matrix.md`：

| 行 | 表中声称 | 实际生产链与独立观察 |
|---|---|---|
| 43 display.title/title_zh | 只在 Vue generate 消费，其余携带；zh 链仅 Vue 产物 | `PacSource::title_env` → `wrap_example`（rust_ui.rs:2728、2740–2746）。实际 generate_ui/rust：请求 REQUEST-A/REQUEST-B 分别写进 main.rs 的 AUTO_VM_TITLE 初始化；磁盘故意放 DISK-TITLE，未被采用。 |
| 45 runtime.window | 仅 validate summary，生成产物不按它改视口 | `PacSource::window_env` → `wrap_example`（rust_ui.rs:2727、2731–2737）。实际 generate_ui/rust：901×617 和 333×222 分别生成对应 AUTO_VM_WINDOW 初始化，绝非仅回显。此证据是生成物消费，不外推为本轮已实际打开两种尺寸的窗口。 |
| 46 runtime.api 与其他字段合并为携带 | 无 generate/build/run 主链消费 | `PacSource::api_is_rust`（rust_ui.rs:3306–3318）读取请求 runtime.api；run_vm.rs:297–322 用它选择默认 Rust 后端及 split 分支，继而调用真实 api 生成/启动路径。此项是完整调用链源码核验，不冒称本轮实际构建了 Rust 后端。 |
| 53 toggles.default_classes | 无生成消费，类归一只在809 | `VueReads::from_request`（vue.rs:2783）→ build/生成算法 `default_classes`（3745、3903 等）。同源实际 generate_ui/vue：true 产物含 `<span class="text-muted-foreground leading-7">HELLO</span>`，false 为 `<span>HELLO</span>`；请求只有该开关变化。 |

同因 sibling 还包括 dep_roots 表只列 Vue，但 rust_ui.rs:2934–2945 的 resolve_bps_dependencies 消费 `PacSource::dep_roots`；请按真实路径核对其适用列。以上均是**矩阵事实/验收归类缺口**，未证明这些产品字段已经失效，不能通过删除读取或削弱功能来对齐文档。

**可重复回放**：[field-and-resource-holdouts.py](field-and-resource-holdouts.py)，在同一 unique tree 执行 `python D:/autostack/auto-lang/docs/plans/attachments/815/review-t07-batch3-r1/field-and-resource-holdouts.py`；读取 driver-scratch.txt 指向的新包 B，私有目录每次唯一，无生产修改。若换基线，先用该基线 driver 生成新包并更新 scratch 绑定。完整请求/回执/产物路径和哈希见 [holdout-results](holdout-results.json.txt)，[Rust 初始化片段](rust-window-title-snippets.json.txt)、[Vue 开关差异](default-classes.diff.txt)、[消费源码摘录](field-consumer-evidence.json.txt)。回放成功恰好证明“这些字段实际生效”，从而反证表中的无消费声明。

**关闭条件**：纠正上述 profile/操作/效果分类，追完 adapter alias 和消费路径后校核其余“携带”行；为真实限制引用已批准语义/原 oracle，将 source-only、准入、实际生成/执行证据区分。现有三项新增测试、既有静态/动态单源与 profile 关闭证据可按未变输入复用，补上已有/本轮区分性证据的准确引用。无需新发明全部字段支持、更改产品布局或做无意义笛卡尔积。修复文档后冻结受影响范围再审；不能仅把错误分类标成“冻结语义”免验收。

## 实际执行及证据边界

- [driver](driver.log.txt)、[收据](driver-receipt.json.txt)：**33/33，exit0**。producer.git_head 精确为本冻结 SHA、git_dirty=false；源码包与根锁身份在 [delivery-manifest](delivery-manifest.json.txt)。剥离开发根环境、A fresh lock、B 无 target 重定位、负控保留/恢复、dev 同源及 tool 实际消费全部执行；未借用 work 的33总数充当本轮结果。
- [资源对账](resource-reconcile.json.txt)：4字体/许可+2壁纸+5壳+1gallery+9wm+5drawlist+346shadcn+5根输入=**377文件**全等；包根锁为消费者 fresh 解析锁，不要求等于 producer 锁。源码作为装配指纹的 include_str 输入随各 source crate，本轮资源结论不把“10组资产表”当成全部 Rust 源文件枚举。
- 包 B 私有 Button.vue 加标记后，新工具真实生成的组件带相同标记；恢复原字节后终核。证明来源是包 B 资源，不只是两个本来同字节文件的存在性比较。未知版本99的 generate 请求不建 gen，同流版本1随后真实生成成功。六个独立对照全部通过；重复执行仅作回放脚本可复用性确认，不双计总数。
- `external_backend_request_is_authoritative_over_disk_decl` **1/1**，[日志](external-backend.log.txt)；`validate_field_rules_output_root_and_input_sources_boundary` **1/1**，[日志](field-boundary.log.txt)；`generate_ui_field_matrix_and_pac_snapshot_independence --ignored --exact` **1/1，0.66s**，[日志](field-matrix.log.txt)。这些有效证据保留，未因表错而否定。
- 两个改动 Rust 文件 rustfmt --check、diff --check、driver AST 解析均通过，[卫生记录](hygiene.json.txt)。未运行整档或机械复跑全部 lib。编译显示既有 lib/lib-test 22/18 warnings，不冒称全库零诊断，未发现本次测试新增未处理诊断。
- [复用核对](evidence-reuse.json.txt)：相对上轮依赖/锁/测试配置/protocol/host/proc_host/运行链及已发布 Spec 未变；只有本批两处 Rust 测试、driver 和所列文档变。A/L/T04、此前联合修复已闭项、子批1 receipt/depth/Linux 证据与子批2两个实修复保持原范围通过，不报为本轮重新执行。

## 规范、状态和后续

[SD-01..07 核验](spec-delta-reviewed.md)保存本次规范影响判断。发行/资源事实的修正可接受，但错误字段表不能成为 SD-04 的最终全量依据。T-08 仍需据实定稿和选定实际发布二进制指纹，T-09 再审全计划。

**next=work**：仅修原 F-815-T07-B2-R1-03 的矩阵事实/证据映射，保留两项关闭和33/33，不改产品以符合错误文档、不缩减 AC。T-05/T-06/T-07/T-08 本次不作整任务联合勾选；Plan executing10/17。777 master并集及最终pass后的裸 cargo t/实际触面档、串行合入、Spec/ledger/发布收据与归档清理仍归后续 owner，本轮未执行。

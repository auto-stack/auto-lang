# PLAN-841 r1 — R3 affected final review

2026-10-26，outcome: **needs_fix**。当前 Codex `/root` 延续 R1/R2 独立审查上下文，未参与实施，未启动子代理/新审查树，未修产品实现或合入。

## Frozen baseline and review scope

- 既有树 `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，均分支 `codex/plan-841-dev`；进入和退出均 clean、HEAD 不变。
- UI `60011b3029a7e8a7b9f95aeecff821fbd448e962`，repair base `a056dd60171b9f4a1ee61fc1ade32b45d4a4ba1c`。
- lang `6892bad1ae9556f93da743bb1af265625330d809`，repair base `a0473d515129ad1016a164fc0f3201c9334805cc`；本轮语言树仅文档/候选 Spec，无 Rust 变化。
- Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` 未变；down clean clone `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未变。
- 主计划读自主检出，revision 1。依据 [work 诊断/关闭矩阵冻结副本](repair-r2-closeout.md)，检查 R2 两开放项及新增共享预检的合法输入、状态和失败传播；不重跑无关已闭验收或全档。
- 提交、Plan 输入 hash、依赖/配置和主检出/候选 Spec hash 见 [baseline.json.txt](baseline.json.txt)。完整候选规范差量 [spec-delta-reviewed.diff](spec-delta-reviewed.diff)，SHA256 `a25d999e4d9b3b325eecadf547463870c9d87bb0c73d96b96e9025e0e0faf11b`。未发布 canonical 或 ledger。

## Prior finding closure

| Finding | R3 result | Evidence |
|---|---|---|
| F841-R1-01：工程入口漏准入 | **CLOSED，对原无效输入/生产接线** | 原 request Jet/Ark/Rust 三坏输入均拒绝且零输出，三合法输入生成；永久 tools 4 测涵盖四 backend、四领域家族、导入和 incremental；真实编译入口与缓存/写盘时序已检查。新增合法导入回归单列于下，不倒写成旧反例未修。 |
| F841-R2-01：codec 误收尾随数据 | **CLOSED** | 原 garbage/第二份 JSON/旧 schema 第二信封全部拒绝，合法空白通过；源码 `Deserializer::end()` 已接入；旧深视图/字段/span/source_lines/语义/payload/Weak 保持。 |
| F841-R1-02/03：深度与格式 | **保持 CLOSED** | 原4/16/32/48层仍过；R2已审181文件格式证据保留，本次8个受影响 Rust 文件 rustfmt 与两个 repair diff 的 whitespace check 全过。 |

## F841-R3-01 / P2 / OPEN — 共享预检在导入配方装载之前解析，误拒合法工程

这是 **本轮修复回归**，违反既定 AC-02/04/05、T-05/T-08/T-09：UI 生产准入必须保留现行样式/导入语义，不能为拒绝无效领域数据阻断合法程序。命名/通配符 pub style 导入是既有支持（`auto-ui-compiler/tests/design_tokens_recipe.rs` 的 PLAN-635 named/wildcard 控制），不是新增需求。

### Actual path and established cause

新 `auto-ui-compiler/src/ui_gen/api.rs:612..621::validate_ui_file_with_modules` 读文件后立即 `parse_validated_ui`，没有先建立该文件的 imported recipe 可见性。`auto-ui-tools/src/vue.rs:3736`（full）和 `:6817`（incremental）新增 `validate_ui_tree` 在原生成流程之前调用它，因此在冷 registry 状态下提前返回 `UI file validation failed ... undefined variable`。

对照：既有 `generate_component_from_file` 在同文件 `api.rs:682` **先** `prepare_style_recipe_imports`，再解析/校验（`:685` 之后），对完全相同的源码能够成功并生成实际 recipe class。预检误拒发生在这条既有正确生成路径之前。不是缺 Gradle/浏览器环境、无效语法或待生成的新接口。

### Bound input and commands

持久 probe：[review841_r3_compatibility.rs](review841_r3_compatibility.rs)。临时项目目录下：

```text
pac.at: name: "review841" / scene: "ui" / render: "vue"
src/front/styles.at: pub style chip = "rounded-full px-4"
```

`src/front/app.at`：

```auto
use styles: *
widget App { view { button (style: chip) { text "ok" {} } } }
```

以公开 `clear_style_recipes()` 让 thread-local registry 处于真实首次使用的空状态；真实 `parse_file -> GenerationRequest::from_descriptor` 后调用 `VueProject::from_workspace_with_request`，以及 `incremental_compile_changed(root, Some(&req))`。分别对比命名导入 `use styles: chip`、同文件 `style chip = "rounded-full px-4"`，以及先调用真实组件生成入口后的已装载状态。

重放：把 probe 临时复制到原 UI 树 `crates/auto-ui-tools/tests/review841_r3_compatibility.rs`，置空 RUSTC_WRAPPER，执行 `cargo test -p auto-ui-tools --test review841_r3_compatibility -- --nocapture --test-threads=1`。不创建新 checkout、不改产品。

### Expected vs observed

预期：named/wildcard/local 合法源码在 full/incremental 冷状态都正常准入与生成，不依赖此前同线程处理过哪些文件；实际 class 包含 `rounded-full`。

[recipe-final-controls.log.txt](recipe-final-controls.log.txt) 是最终 probe 的完整原始记录：

| Case | Cold | After legitimate component generation |
|---|---|---|
| local recipe，full | 成功 | 成功 |
| named import，full | **undefined variable，失败** | 成功 |
| wildcard import，full | **undefined variable，失败** | 成功 |
| wildcard import，incremental | **undefined variable，失败** | 成功，并实际写出 App.vue |

另外直接调用新增 file validator 同样失败；同一源码走原 `generate_component_from_file` 成功。probe 明确断言该组件生成的 `vue_code` 包含 `rounded-full`，不以仅返回 Ok 或空壳作为合法对照。两项兼容测试均 exit 101；私有 registry 不是探针伪造，空状态与正常公开装载路径均可重复重现。

已证影响为 **Vue full/incremental 与共享 file validator 的合法 named/wildcard 配方**。其他工程工具也调用该 helper，但本轮不据此冒称其原本支持的所有配方路径均已实测退化。未引入新规模/深度承诺。

### Closing oracle and repair scope

在正确的模块/源文件上下文完成预检所需的导入可见性，确保上述冷态输入通过且实际样式保持；合法本地/已装载控制保持，不能靠预热全局 registry、文件遍历顺序、skip_check 或取消准入来修。根据实际导入初始化改动复证私有/缺失配方的既有拒绝控制，检查邻接模块/顺序不会借用前一文件残留注册。原重复 widget/store/model/message、payload 的拒绝/零输出与 codec 已闭控制仍需按影响面保留。

work 先修该共享状态/时序问题与永久 cold 控制，再提交受影响复审。无需整档 cargo 或完整发行矩阵来定位本项；不要求为每个 finding 重跑全部已闭家族。

## Check → invariant evidence

| Check | Result | Bound evidence |
|---|---|---|
| 原 R2 工具反例 | 1 pass，含6次真实生成调用 | [original-tools.log.txt](original-tools.log.txt)；三合法成功、三无效拒绝且零输出。 |
| 原 compiler/codec/Core probe | 5 pass | [original-codec-and-compiler.log.txt](original-codec-and-compiler.log.txt)；尾随、深度、Core expr/fn/scene/task 及旧高层拒绝保持。 |
| tools 永久 `ui_domain_validation` | 4 pass | [permanent-tools.log.txt](permanent-tools.log.txt)；实际断言/循环已读，无“banner当覆盖”的推断。 |
| codec 永久 `codec::tests` | 9 pass | [permanent-codec.log.txt](permanent-codec.log.txt)；合法/坏形状/payload/尾随/寿命。 |
| 新合法配方兼容 probe | **2 fail** | [recipe-final-controls.log.txt](recipe-final-controls.log.txt)；冷态、命名/通配符/local/full/incremental/实际 class 对照。较早同组日志保留为探索记录，以此最终版本为准。 |
| repair touched rustfmt / diff-check | 8 files / 0 fail | [format.log.txt](format.log.txt)；保留 R2 未变文件的格式证据，不重放181文件。 |

编译使用既有基线警告；未发现本次新增未处理 warning/debug 输出。未运行裸 cargo t/tv/tt/tb/tu/taa/tf 或整体 792。

## AC/SD and routing

- AC-01：保留 R1/R2 的中立分类/依赖边证据，Core 未改；AC-03：本基线通过受影响 codec 复证。
- AC-02/04：原无效数据准入子项已闭，但合法样式支持回归使整体 fail；AC-05/06 fail，AC-07 pending。
- 两轮失败诊断确有完成并覆盖真实工程入口与完整 JSON 消费，不能因本项新回归否定那些实际修复。新缺口是共享预检没有携带既有导入/registry 上下文，纯简单 widget 的合法控制覆盖不到它。
- SD 的 codec、AST所有权与旧实现退出证据可保留；生成/导入/支持面部分在 F841-R3-01 关闭前不能判最终通过。冻结 SD 不发布，不降低合法程序承诺来迁就当前预检。
- 未变 runtime/SDK 的 R2 具名证据按调用路径/配置/输入不变复用；新增 helper 未改变其已有实现。旧 777/792 是历史来源和未变项参考，统一发行仍需 merge 在最终冻结提交执行。

主计划回到 **executing 6/12**：重开 T-05/T-08/T-09，保留 T-01/T-02/T-03/T-04/T-06/T-07 已完成产出与全部历史 review。F841-R1-01、F841-R2-01 本轮关闭，F841-R1-02/03 保持关闭，新增 F841-R3-01 OPEN。统一裸 cargo t 与适用 tv/tt/tu、三仓交付/Spec-ledger/归档/guard 等仍 merge-owned pending。未合入，不放行 U841。

临时 probe 已从实施树撤去，源码/日志保存在本目录；最终状态与 hash 见 final-state.json.txt。


存证说明（merge）：Git 将可读差量规范化为 LF；若核对原复审 CRLF 字节的 SHA256，解码 [原字节 base64](spec-delta-reviewed.crlf.base64.txt)。行尾映射与 hash 见 [存证映射](../merge-r1/evidence-line-endings.json.txt)，原 verdict 与规范内容未改。

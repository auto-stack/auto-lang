# PLAN-831 T-04 导入台账（冻结源与候选清单）

执行组：`D:/autostack/.wt/lang-831/{auto-lang,auto-ui,auto-down}`。
本台账冻结 T-04 导入的来源身份、逐类映射与逃逸面清单；T-05/T-06/T-09 以此为对账基线。

## 1. 冻结源

| 项 | 值 |
|---|---|
| 语言仓源 commit | `e199caf8a`（lang-831 树，codex/plan-831-dev） |
| 五包内容恒等证明 | `git diff a61209cc0..master HEAD` 与 worktree 对五路径均空（纯簿记提交差异）；导入后逐字节对比 841 文件，除 Cargo.toml 外零差异 |
| auto-ui 落点 commit | `c6d3ac6`（迁移分支 codex/plan-831-dev，基于 f7d92ee） |
| down 兄弟 | detached worktree `@c39ac7f9`（主检出 HEAD；§4 预案"依既有解析补 detached"，零实现改动） |

## 2. 逐类映射（§5.3 rename/copy/generated/compat）

| 类 | 项 |
|---|---|
| **copy**（逐字节） | 五包全部 841 文件中除各包 Cargo.toml 外的全部内容（src/tests/ui_tests/assets/shadcn-ui/字体/build.rs 等） |
| **manifest adjust**（仅机械改写，语义等价校验 PASS） | ① 跨仓语言基础 path：`../<pkg>` → `../../../auto-lang/crates/<pkg>`（15 边/13 包：auto-parser-api、auto-lang-core×5、auto-atom、auto-val×3、auto-media-core、auto-text-core、auto-terminal-core、a2r-std、auto-tool-protocol、auto-project-model、auto-tool-execution、**auto-man-core（过渡台账保留）**、auto-api-gen）；组内兄弟与主检出同级双布局成立，最终固定机制 T-06 冻结。② workspace 继承内联 25 行/16 键（spec=语言仓根冻结快照，features 按 cargo 继承语义并集）。autodown-core（`../../../auto-down/...`）同深度成立**未改写**；五包内部互引（compiler→dialect、runtime→compiler、tools→compiler/runtime、shell-pack→runtime）**未改写** |
| **generated** | shell-pack 整包为 a2r 生成物（入库形态，PLAN-036），随源 copy；其再生器 canonical import 规则随源根更新归 T-06（815 F-T09-01 合同） |
| **compat** | 本任务零新增：旧 auto-lang 兼容 crate（lang-monolith）不迁，留语言仓（contract-baseline §1.2）；UI 仓侧 re-export/薄转发机制归 T-06（§8 未闭决策 #3） |
| **不随迁（留语言仓）** | auto-api-gen（仓属归 T-05/§8-1）、auto-man-core、全部语言基础包、examples/ui（381 文件）与 auto-examples/gallery（消费切源归 T-06）、随迁 fixture 六件（ui-dialect/ui-grammar/ui-package/ui-runtime/ui-tool-consumer/generated-ui-consumer，T-06 源根更新时迁移）、language-sdk-consumer（恒留，SDK 反向控制） |

## 3. 逃逸面清单（T-06 布线输入）

| 类 | 事实 | 处置 |
|---|---|---|
| A 随迁即满足 | compiler/build.rs（walk `src/ui_gen`，crate 相对）；runtime 字体/壁纸 `assets/`（crate 相对 include_bytes!）；vue.rs `include_str!(MANIFEST_DIR/../auto-ui-runtime/assets/desktop.at)`（intra-五包，相对布局不变） | 无需动作 |
| B 仓库根级 producer 输入 | tools `DELIVERY_ROOT_INPUTS`×5：`stdlib/assembly-providers.json`、`stdlib/auto/image.{at,rs.at,vm.at}`、`docs/schema/aura.at`（运行期自源根读取） | T-06 随固定源机制处置（随迁或定点引用），V-03 发行面负控覆盖 |
| C 测试期根相对引用 | runtime desktop_protocol 六处 `MANIFEST_DIR/../../examples/ui/` 与 `/../../` 根行走；runtime ui_tests 若干 `../../../` 指语言仓 stdlib/lang-core 源；compiler 测试指 `examples/capability-tests/tabs-variants`、`../docs/schema/aura.at`；tools 测试指 `back/rust` fixture | T-06 测试迁移时逐处改写或随迁输入；测试 ID 对拍（TID-A1 正典法）防丢 |
| D 跨仓 optional | runtime `autodown-core`（optional，同深度路径未改写） | T-06 固定机制冻结（§8-2） |

## 4. 许可面

- compiler/runtime/tools：`license = "MIT"` 字段在位（随源）。
- dialect/shell-pack：**缺 license 字段**（源即缺）——具名交 T-06/T-09 归一化（非 T-04 缩面：不改候选内容补字段）。
- 语言仓根 LICENSE 文件不随 T-04；UI 仓沿用其自有旧 LICENSE（legacy 面）；终态许可归属归 T-09 清单。

## 5. 验证记录（2026-10-24）

- `cargo metadata --no-deps`（auto-ui 树）：38 包解析，五包全在；语言仓树同步恢复解析（46 包）——**既有条件修正**：两树 optional autodown path 自 worktree 悬挂导致 metadata/成员加载红为语言仓 worktree 既有态（主检出恒绿），按 §4 预案补 down detached 兄弟后双绿，非导入引入。
- driver `import-window` 门全绿：SRC-P1（5/5 导入+权威保留）/P1b（双侧布线全解析）/P2（语言仓权威闭包 29 包零禁断）/P2u（UI 候选闭包 19 包零大库/monolith，Man-core 台账内 1 条=窗口期望恰等）/P4a/P4b（零 vendoring、零窗口违规、legacy 1 边记档）；P3/P3u/P5/P6 记档（收据 `scratch-ui-repo-fixture/t04-window.json`）。
- manifest 语义等价：五包全部依赖项（含 target/dev 表）与源按 cargo 继承语义解析结果等价，仅跨仓 path 前缀按预期变化（脚本对拍 PASS）。
- T-03 接口具名修订：`import-window` 阶段新增（lang 侧 `fb729b04a`；selftest 扩至 16 项含窗口走私反例）——原因=T-04 导入产生 lang+ui 双侧五包窗口态，两阶段判别无法命名；属加细非放宽，预迁移基线仍绿。

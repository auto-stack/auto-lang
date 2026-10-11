# PLAN-759 落地后独立核查（2026-10-11）

**结论：needs_fix。原日常红族的主要修复有效，但新增生产诊断存在误报，photo 测试的裸 libtest 环境隔离没有落实。不能把日常档清零等同于全部实现正确。** 原 Plan 已 archived，本次不重开、不改其状态；问题通过本报告和新增债务候选记录，后续修复须使用独立 worktree。

## 冻结基线

- stage: review（post-landing independent audit）；plan_id: PLAN-759；plan_revision: 2。
- reviewed_commit: `c128c27f699ede8af62585af0bdfd50859b5f9fe`；实现提交范围 `c279a6490..e905607ff`，diff base `afee09bcdefb95ac525a4a712124209ce0c085e9`。
- `git merge-base --is-ancestor e905607ff HEAD` 成功，证实实现已落地；原 lang-759 worktree 已不存在。另建 `.wt/review-759` 只读源码审核检出，所有构建/测试在其中完成。
- dependencies: auto-os `b5a233323f3dd5c0e7d023a74f5c8bc34d6546e5`（主分支已包含金样提交，本审核固定此提交）；auto-down `895f8d0f`；auto-term `7a0d0e8b718fb260641f0d09ada95a7346771cd3`。三依赖均建立独立 detached 兄弟 worktree，不读取其他会话的未提交实现。
- Plan SHA256: `D48EA00FBC82B35CA59B6575002019815C2B72DFF79F385B05B72DE1FDF4B424`。
- spec_inputs: vm/design/vm-fn-call-semantics.md SHA256 `FA2A470E05182F8835C60FC138986FA6CEDE8D121012545A97FEC445E72F2732`；ui/overview.md SHA256 `BC138E9447A12D6C8EB0826728D112ED7505EE3AACEF73FBCCEA3580684D8B00`。
- 数据输入：schema/aura.at SHA256 `FE29D4516AA652BF0A26A92BB20FC5FDA31AC382F37474A4F6736D4EB313384C`；schema_drift_baseline.txt SHA256 `C6289167ACDD0513820805C8E4762143DA00DD0D1714776E9B85B9B44153761C`。
- 从已提交工件重新建立判断，未复用实施者的 pass 结论。主检出既有 Plan 760 修改、临时文件及依赖仓其他会话 WIP 均未触碰。

## 本次实际验证

| 验证 | 结果 | 说明 |
|---|---|---|
| 标准环境 `cargo t --no-fail-fast` | **5232 passed / 0 failed**，100.500s | 1540 skipped（含显式 ignored 的 DLL 探针及其他档位）；原目标族在并行全档内均绿 |
| `cargo tv` | **162/162 passed**，2.192s | 当前审核 HEAD 实跑 |
| `cargo tu` | **857 passed / 2 failed / 2 ignored**，测试段 0.90s | 两红与已有复审记录相同：desktop_surface 金样、blueprints root_resolution 环境竞争 |
| root_resolution 失败单例串行复跑 | **1/1 passed** | `walks_up_from_cwd_to_nearest_blueprints_dir`，与原记录 flake 分类一致 |
| auto-term 专用检出 `cargo build -p autoterm-core` | 成功 | 独立 target，未构建/改动主检出 |
| `AUTOTERM_ENGINE_DLL=<审核组 DLL> cargo nextest run -p auto-lang --lib ash_stream_leak_probe --run-ignored only` | **1/1 passed**，20.944s（测试20.925s） | 20000 键入循环真实执行；不是忽略/return 冒充通过 |
| photo + http_base_url_resolution 单进程双线程 | **8轮各2/2 passed** | 使用 `RUST_MIN_STACK=33554432`；本次未复现 env 竞争，不能据此声称有互斥或 panic 恢复 |
| 生产库渲染能力/实际 View 探针 | **复现 F-759-R1** | 源码与实际输出见下节 |
| `git diff <base> e905607ff --check` | 通过 | 无 diff 空白错误；构建有仓内既有警告，未宣称零警告 |

首次日常运行曾显式设置 `AUTO_OS_ROOT`，得到 5231 passed / 1 failed，唯一失败是原有 `manifest_daemon_lookup_via_sibling_probe`：该测试期望临时兄弟 fixture，但 env 权威解析将它导向审核 auto-os。核对源码后清除审核 env，通过组内兄弟解析进行上述标准全档，得到5232/5232。该首次失败是审核配置造成，未计入 Plan 回归。首次裸 libtest 双线程尝试也触发默认线程栈不足；设置32MiB栈后的8轮才是有效环境竞争采样。

## F-759-R1 [P2] 新增 render-support 诊断与实际能力不符（T-01 / AC-05）

`crates/auto-lang/src/ui/render_support.rs:314-319` 把 command 全族统一归入 fallback，声称 `style/placeholder/value/text/label` 被忽略，且“command palette not implemented — renders as Column”。但 `ui/aura_view_builder.rs:2922-2942` 已将 command_input 转换为原生 Input、command_item 转换为 Button；`:12562` 起的 convert_input 真实读取 placeholder、value、style。carousel_content 也是 Row，carousel_previous/next 是 Button，并非统一 Column。

这不是无影响的注释：`ui/mcp_server.rs:2435` 将 Fallback 统计为 ERROR，`:2356` 起据此产生 ignored-props 诊断，影响 autoui_check / MCP 快照给出的能力判断。用户传入有效 placeholder/value/style 会被错误地报告为忽略。`schema/aura.at` 的13条 partial→fallback 修改也把既有诊断从 WARN 升成 ERROR，超出“零用户可观察语义变更”的笼统说法。

还有拼写不一致：新增详情 match 只有下划线。`get_support` 只从 schema 覆盖 level，未按 canonical 标签查详情，因此 `command-input` 返回空 ignored_props/note，`command_input` 返回上述错误列表与文案。

实跑已编译生产库的小型探针，结果：

```text
SUPPORT command_input: Fallback; ignored=["style", "placeholder", "value", "text", "label"]; note=command palette not implemented — renders as Column
SUPPORT command-input: Fallback; ignored=[]; note=
SUPPORT command_item: Fallback; ignored=["style", "placeholder", "value", "text", "label"]; note=command palette not implemented — renders as Column
SUPPORT command-item: Fallback; ignored=[]; note=
ACTUAL_VIEW Column { children: [Input { placeholder: "Search", value: "hello", ... style: Some(...) }], ... }
```

**修正方向**：按每个子件实际转换实现确定级别、真实忽略属性和准确文案；缺少完整 carousel/command 高阶交互不代表子件的输入/按钮/样式属性全不支持。规范化标签后查询同一详情，保证 canonical/alias 一致；同步 schema，补能力诊断与实际 View 对照验证。保留真正有缺失的交互说明，不为了围栏一致而统一降级。

## F-759-R2 [P2] photo 测试未落实裸 libtest env 隔离和异常恢复（T-06 / AC-02）

`crates/auto-lang/src/tests/plan606_gallery_tests.rs:163-165` 读取旧值并直接 `set_var`，`:308-312` 只在全部断言通过后恢复；没有 RAII Drop，也没有跨测试互斥/子进程隔离。中途 build/驱动/断言 panic 会跳过恢复，在同进程中污染后续测试。注释所谓“独占该 env 的窗口”没有代码保障。

`vm/ffi/stdlib.rs:11864-11882` 的 `http_base_url_resolution` 同时写/删除同一环境变量，且无共享锁；裸 libtest 并发时两个测试可以覆盖彼此。日常 nextest 一测一进程，所以本次5232全绿和8轮双线程绿只说明这些采样未撞上，并不完成计划承诺的双形态隔离。

**修正方向**：采用真正的进程隔离，或让所有该 env 的修改者遵守同一串行规则并加入 panic-safe 恢复；单给 photo 测试添加私有锁仍不能挡住另一测试。保留精确URL、24张恰一次、所选ID全图、Cover/Contain和stub请求收据断言。

## 其余核查结论

- R1双臂现在既锁exports/3142发射，也实际执行直调返回值与错误路径；原warn命名已撤，修复合理。
- p748服务器accept/read/write已有有限等待；noawait不再启动孤儿服务器；TempDir作用域持有；join后补泵并精确断言W:5/W:6，修复有效。各段有界不等同于绝对严格的总15s上限，但本次没有发现新的无限等待。
- projector显式default与裸按钮分测、壁纸仅两测试临时目录隔离、plan502保留原几何断言并等完成态，均符合主要修复目的。
- schema别名挂在既有canonical元素，没有生成重复组件；carousel覆盖登记理由与D4退役表一致。大幅baseline刷新是扩展的白名单校准，不能视作几百项漂移被逐项修复；P759-D2扫描器债务仍真实存在。
- auto-os金样提交已在主分支，双仓落地闭环实证成立；KNOWN-DEBT的REG-4行仍写“待合流”，清偿节链接也未迁到archive，属于应更新的簿记遗留。另一个tu环境竞争虽在759原复审中记录，但尚未独立登记到KNOWN-DEBT。
- canonical VM Spec已撤“唯一余留p053_4”；壁纸测试态隔离规约已沉淀。F-759-R1说明schema/render-support这部分的“零逻辑/语义改变”范围声明仍需准确化；本次不改canonical Spec。

## 验收映射与后续

| AC | 核查结果 | 依据 |
|---|---|---|
| AC-01 | pass（目标日常清零） | 标准全档5232/5232，DLL显式集成实跑 |
| AC-02 | partial | 原目标族执行绿；裸libtest隔离承诺存在F-759-R2 |
| AC-03 | partial | 已验证成员销账基本合理；REG-4落地后措辞/归档链接未更新 |
| AC-04 | pass | 两依赖固定版本、独立构建和金样落地均核验 |
| AC-05 | needs_fix | F-759-R1是新增生产诊断误报，不能以围栏绿当正确性证据 |

next：在新的独立修复worktree处理F-759-R1/R2，保留759 archived终态；补齐针对诊断与环境隔离的验证后再复审。未运行批量tf（此次为per-plan核查），未改实现、未合并、未重开归档计划。

## 审核环境清理

Git Bash 执行 `D:/autostack/wt-guard.sh` 对四个审核worktree均输出clean，随后逐仓 `git worktree remove` 成功；没有删除其他会话的检出。剩余 `.wt/review-759` 只有本次日志与渲染探针产物。自动审批拒绝其目录/文件清理（返回 `blocked by policy`，无具体原因），因此保留该目录，不改用其他途径绕过拦截。关键证据已在本报告摘录，探针源码也已存为 `reports/assets/759-post-landing/render_probe.rs`。

## F-759-R1/R2 修正后的独立复审（2026-10-11）

**outcome: pass（本轮两项 P2 修正范围）。** F-759-R1/R2 均已修正，未发现新的阻断问题；原报告和原 Plan 的 archived 状态保留。此结论不表示仓库所有档位零红，亦不覆盖尚未合入的 PLAN-760 实现。

### 冻结输入

- stage: review；plan_id: PLAN-759；plan_revision: 2（归档不重开）。
- reviewed_commit: `35be7f1e0bbbe1831f068201d7269e6626e0ab3d`；修正提交 `fd55d2d4288b64b49fc6eee8705979aee4e14563`；diff base `fd55d2d42^`。祖先检查成功，修正已落地；从上一审核 HEAD 到本轮 HEAD 的实现变化仅为本次四文件修正。
- 独立 detached 审核检出：`D:/autostack/.wt/review-759-r2/auto-lang`；源码 clean。全部构建与测试在此进行，未触碰主检出临时文件或其他会话的实现。
- dependencies：auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846`；auto-os `b5a233323f3dd5c0e7d023a74f5c8bc34d6546e5`。auto-os 当前主分支 `0532b8fbc0170deb6efa970196d337c26ebe51c6` 已先行带入 PLAN-760 的 remote-frame 金样，而本轮 auto-lang HEAD 尚不含该实现，故审核固定为759配套版本，并通过组内兄弟解析；未设置 AUTO_OS_ROOT。
- Plan SHA256：`D48EA00FBC82B35CA59B6575002019815C2B72DFF79F385B05B72DE1FDF4B424`；UI Spec SHA256：`BC138E9447A12D6C8EB0826728D112ED7505EE3AACEF73FBCCEA3580684D8B00`；VM Spec SHA256：`FA2A470E05182F8835C60FC138986FA6CEDE8D121012545A97FEC445E72F2732`。规范文本与上一核查相同，未发布新规范/ledger。
- 本轮 schema/aura.at SHA256：`90653CFC463513FB87ACFA603731CF0FE34D38D1B9C1D951D06288B20DCF12A5`；规范相关数据变化为12项 iced 能力级别校准，审核实际转换实现与围栏一致性。

### 复现结果

| 验证 | 本轮结果 |
|---|---|
| `cargo t --no-fail-fast` | **5236/5236 passed**，99.418s，1540 skipped；较上一轮新增4个回归测试全部执行 |
| `cargo tv` | **162/162 passed**，2.284s |
| `cargo tu` | **858 passed / 1 failed / 2 ignored**，1.06s；唯一失败为此前已有的 `ui_gen::vue::tests::test_a2vue_desktop_surface_asset` desktop金样；root_resolution本轮通过 |
| 原生产库 render_probe 再跑 | canonical/kebab 的 level/ignored/note 一致；实际 Input 的 Search/hello/style 仍落位，原误报不再出现 |
| 裸 libtest：`plan606_gallery http_base --test-threads=3` | **4轮，各3/3 passed**；含photo实际stub、URL解析、panic恢复；`RUST_MIN_STACK=33554432` |
| 修正 diff 空白检查 | `git diff fd55d2d42^ fd55d2d42 --check`通过；构建仍有仓内既有警告，未宣称整个仓库零警告 |

生产探针的关键输出：

```text
SUPPORT command_input: Partial; ignored=["type", "maxlength", "min", "max", "step", "pattern"]
SUPPORT command-input: Partial; ignored=["type", "maxlength", "min", "max", "step", "pattern"]
SUPPORT command_item: Partial; ignored=["value"]
SUPPORT command-item: Partial; ignored=["value"]
ACTUAL_VIEW Column { children: [Input { placeholder: "Search", value: "hello", ... style: Some(...) }], ... }
```

### 问题收口与覆盖边界

- **F-759-R1 / P759-D3：pass。** get_support先按schema归一，再取详情；command/carousel各子件按真实Input/Button/Column/Row/Empty转换校准，schema同步。保留缺失的高阶交互说明，未修改渲染器产品逻辑。三个新增回归测试执行通过，原生产库探针也独立确认修正。
- **F-759-R2 / P759-D4：pass。** photo与stdlib URL测试共用HTTP_BASE_TEST_SERIALIZER，RAII守卫在锁释放前恢复环境；查明auto-lang库内该env的测试修改者均遵守此锁。panic恢复测试和三线程同进程复跑通过。photo原精确URL/24张恰一次/所选ID全图/Cover/Contain/stub收据断言均保留。
- **非阻断补强项：** `http_base_env_guard_restores_after_panic`在父线程持锁、子线程仅持env守卫，因此只验证env守卫的panic恢复，未实际让锁poison；其注释中的“poisoned后可接管”不能当作已实测的覆盖。实现使用 `poisoned.into_inner()`，源码处理合理。后续可补真实持锁panic与原env未设两种专门用例。
- 原AC-02的环境隔离缺口、AC-05的新增诊断误报现已关闭。未重跑此前已验证且本轮零改动的auto-term DLL集成门禁（实现及依赖未变，复用上一轮20k循环实跑证据）；未运行批量tf。

next：本轮P2修正无需进一步实现修复；保留既有desktop金样及扫描器等范围外债务。本报告提供独立复审证据，不执行合并、不修改归档计划状态。

本轮三个审核worktree均经Git Bash的wt-guard输出clean后成功移除；组根的门禁日志与生产探针产物保留为辅助证据，关键可复核结果已摘录至本报告，不依赖临时目录存续。

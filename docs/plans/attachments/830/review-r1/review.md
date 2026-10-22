# PLAN-830 final review R1 — needs_fix

2026-10-22，独立于实施的本次 Codex 审核上下文；未创建子 agent，没有实施阶段上下文/作者测试结论的授信。只审不修，未合并、未归档、未触根测试档。

## 冻结基线与证据

- 主计划 r3，输入主检出 `547e9dbda6469a8e62980e663a2116a2dbda6ce8`；输入 SHA256 `77d5bd186a7bba974e19a774b3d1eec434dc719f743919e1d8c4b8041f855cf4`，副本 [plan-input.md](plan-input.md)。本轮仅修改计划簿记和审核附件、追加风险记录，保持语义 revision=3。
- 同一实施树 `D:/autostack/.wt/lang-830/auto-lang`，分支 `plan-830-dev`；代码 `dd2788bcc16ce7cff247895346afbbf343352496`，base `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`。进场/出场 Git clean、HEAD 未漂移。主检出有其他任务预存 dirty，保持原样。
- G829：`git merge-base --is-ancestor ddbde3e62 ab56c32ec` exit0；diff 30 文件，仅 AGP 与两份指定 SD 草稿；根 crates/test/manifest/lock/.cargo 零改动。
- Cargo.lock SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。GPU RTX4060Ti/Vulkan/617.14；本轮报告 fingerprint 均 `dd2788bcc16c`。
- 当前 Spec 输入：project `a706ef0e5f2f6db3a82ec7c8c2cb6abedd2f408644656129221dc0a2012eaeb3`；prototype-profile `59b42126811545156b6d5339aacde156974f43259fab51bf6191dc174d527832`；integration-contract `1553c8f94b6a407946e7380d9e14edc0e137cfd6be65f851eeedb01c81f05669`（均 SHA256，主检出）。
- 受审 SD-01 [冻结副本](SD-01-frozen.md) SHA256 `fc611b232d643d6d7f897135f2938781f5f0c90c9abbb3616af309e8c3f9c391`；SD-02 [冻结副本](SD-02-frozen.md) SHA256 `cc7e7b8fa4fc9ff993f029c11b8a0cc5df1403779ae0a2cf44459a43b5aa30bc`。SD-03/04 为冻结 plan-input §5.5 的 modify 提案，未发布 current Spec/ledger。
- 技能版本：auto-plan-review SHA256 `82ab8716bbd7abd97bc08a2978972115f87941d5a5dd76aa5b01f2bb302d6c68`；verification-contract `73ea4ef36cd8b9010f17f648201a685338272d85d1eef5c8862b0f1ddc14e0bf`。仓库 AGENTS `b804e03ffa284f6b3fcc93e55889384be12ba9d9b42cf270d294bb77f5a6acc5`。

AGP 下列文件位置均以 `experimental/autographics-prototype/` 为前缀，行号绑定上述代码提交，可用 `git show dd2788bcc:<path>` 恢复；反例和原始 JSON 已复制到本目录，清树后可追溯。

## 本轮实际执行

所有构建/运行均在原实施树。`cargo check --manifest-path experimental/autographics-prototype/Cargo.toml --locked` exit0，[check.txt](check.txt) 有17个 lib warning；`cargo test --manifest-path ... --locked` **135 passed / 0 failed**，surface_contract 18，其中另有1个集测 dead_code warning。`cargo build --manifest-path ... --locked --release --bin graphics-probe` exit0。

`graphics-probe --case CASE --replay canonical --report scratch/830/review-r1/CASE`，window 三项追加 `--mode window`：

| CASE | 实际结果/持久原始报告 |
|---|---|
| surface-offscreen | exit0，6/6，[JSON](surface-offscreen.json) |
| surface-compose | exit0，10/10，[JSON](surface-compose.json) |
| surface-input | exit0，14/14，[JSON](surface-input.json) |
| surface-ime-os | exit0，7/7，真实 preedit/CJK commit=`爱彼此你好你好`，[JSON](surface-ime-os.json) |
| surface-scenarios | exit0，8/8，[JSON](surface-scenarios.json) |
| surface-lifecycle | exit0，10/10，[JSON](surface-lifecycle.json) |
| ag2d-breakout / ag3d-squash | 两者exit0，[Breakout](ag2d-breakout.json)、[Squash](ag3d-squash.json) |
| iced-shared-device | exit0，[JSON](iced-shared-device.json) |

`cargo fmt --manifest-path ... --all -- --check` exit1，[fmt.txt](fmt.txt)。`git diff --check ab56c32ec` exit0（后者不能替代 rustfmt）。

独立链接当前 AGP debug rlib，未改实现：[counterexamples.rs](counterexamples.rs)、[复跑脚本](replay.ps1)、[原始输出](counterexamples-output.txt)。**11 tests，1 pass / 10 fail**，含合法 revalidate 对照。首次直接 rustc 缺 native windows.0.52.0.lib 的链接错误是审核脚本环境问题，补入已缓存 MSVC 库目录后复跑；不算产品 finding。

`release_after_unmount_does_not_require_intermediate_move` 是调用顺序边界探测：不 revalidate 时返回旧 target；显式 revalidate 对照通过，因此不将其单列为“即使正确调用仍失败”的缺陷。F06 要求把必要 revalidate/cancel 接进真实宿主生命周期，而非禁止显式协议。

## Findings（全部 OPEN，合同内必修；不以记债批准延期）

### F-830-R1-01 / P1 / 代码：结构变更没有失效 dirty，卸载残影且 resize 空白

AC-02/06，T-03/04/08，§5.1/5.4，V-07。`src/core/composition/mod.rs:526-532` 在提交后仅为新 Surface 插入 dirty；SetMount/RemovePortal/RemoveView/SetSurfaceSize 没有失效消费者。`src/host/surface_runtime.rs:304,353` resize 重建 texture 后 clean Canvas 不再 render。

独立 `structural_changes_dirty_the_consumer`：SetMount=false，resize(parent,child)=(false,false)，RemovePortal=false。`detach_erases_old_pixels_on_clean_canvas_parent`：卸载前/后均 `[255,0,0,255]`，显式 mark_content_dirty 正控才回 `[0,0,0,255]`。`resized_canvas_redraws_its_content`：红色变 `[0,0,0,0]`，last_frame_renders 为空。

闭合：结构/尺寸变化自动刷新实际受影响内容与消费者，旧 footprint 消失、resize 新目标有内容，无关兄弟 texture/身份保留；覆盖 add/remove/set/关闭 View 等同源出口，不能以每次调用者手工补 dirty 代替既有结构更新合同。

### F-830-R1-02 / P1 / 代码：clean 父纹理反复叠半透明 Portal，静态画面每帧变色

AC-03/06，T-04，§5.2 预乘 over，V-02/08。`src/host/surface_runtime.rs:351-389` 先跳过 clean Canvas 自身内容，随后无条件在其上以 Load 继续合成全部 Portal。前一帧已合成的像素被再次作为背景使用。

`unchanged_translucent_portal_does_not_accumulate`：无任何状态改变，连续帧 `[188,138,138,255] → [188,100,100,255]`。第一帧恰等于正式 alpha oracle，因此仅测首帧不能闭合。

闭合：静态半透明组合跨帧稳定；发生子变更/移除时从正确的底层内容重新合成，保持 DAG 子先于父与一次内容更新合同；补连续多帧及子变更后的像素断言。

### F-830-R1-03 / P1 / 代码：GPU 目标创建失败前已提交图，panic 后留下半个 Surface

AC-06，T-04/08，V-07 坏资源/失败保 last-good；不要求无限尺寸支持。`src/host/surface_runtime.rs:229-250` 先 graph.submit AddSurface，再创建 target；`src/renderer/target.rs:81` 未先校验设备限额/转成受控错误。

`failed_gpu_target_creation_preserves_last_good_graph`：请求宽度=当前设备 limit+1（32769×1，明确无效，不是压力测试），wgpu `Dimension X ... exceeds ...32768` panic；catch 后 surfaces **1→2**、snapshot 改变，而 slot 未创建。resize 同样先提交 model 再创建 GPU texture，需一起核。

闭合：无效资源具名 Err、图/slot/目标/身份与旧可用帧不变，随后合法创建/resize 仍能工作；将 GPU 准备与最终提交边界真实闭合，补尺寸限额与资源失败出口，不用仅模型 cycle 拒绝代替。

### F-830-R1-04 / P1 / 代码：3D 命中遍历所有相机，丢失实际父 Surface

AC-04，T-05，§5.3 每层实际路径，V-03/06。`src/host/composition_input.rs:150-171` 的 hit_all 对 HashMap 所有父相机取首个命中；`plane(...,_graph)` 不识别当前 Portal/host。core PlaneHit 签名只有 mount/point，实际父身份未传递。

`plane_mapping_uses_actual_parent_camera`：两个合法 3D 父面、不同 camera、共享子内容，投影目标 local=[20,30]；24个 router 中15个返回约 `[35,30]`（首轮16/24，HashMap顺序造成结果不稳定），其余正确为正控。无需第二设备/非法输入。

闭合：按每一步实际 host 选同一呈现相机；多相机、嵌套相机与共享子面换序均确定映射 [20,30]，未注册当前父相机时不得借用其他父相机成功。

### F-830-R1-05 / P2 / 代码：嵌套非交互遮挡被父递归吞掉，点击落回底层父内容

AC-04，T-03/05，§5.3/V-03 非交互仍遮挡。`src/core/composition/input.rs:111-123` 把 descend 返回的 None（被遮挡）当成“没有更深命中”，回落到当前 child Hit。

`nested_occluder_does_not_deliver_to_parent_content`：top→交互 mid→非交互 leaf，点在 leaf 内，应无投递，实际 Some(mid,local=[20,20])；leaf 外 mid 正控通过。现有 non_interactive_overlay 只测顶层，没覆盖递归传播。

闭合：遮挡结果沿整条调用链传播，不把事件投递给其后的父内容/兄弟；补两层/三层遮挡与无遮挡合法对照。

### F-830-R1-06 / P1 / 代码：取消没有闭合焦点/控件/OS 输入生命周期

AC-04/05，T-05/06，§5.3/V-05。`src/core/composition/input.rs:282-286` cancel 清 capture 但保留 focus；`host/composition_input.rs:272` 的 pointer_left_window 也如此。窗口 handlers `cases/surface_input.rs:821-892` 与 `surface_ime_os.rs:350-483` 都不处理 Focused(false)/CursorLeft，未向 iced 发取消/失焦/IME关闭；不存在交接声明的 cancel_all。

`window_cancel_clears_keyboard_focus`：显式 WindowFocusLost 后 keyboard_target 仍 Some(top)。OS 桥 `surface_ime_os.rs:363-377` 把 IME 固定发给 ui_surface，键盘 `:399-402` 在无焦点时 fallback 到 ui_surface；因此当前真实 CJK 正控证明单框可输入，不能证明“只发焦点/切焦点不串写”。合成 case 的 A/B 字符又直接按预定 surface id 构造，未经过统一 key 路由。

闭合：真实失焦/离窗/卸载/销毁清理 capture、编辑/组合状态和有效键盘目的地，必要 revalidate 接到实际结构变更；无焦点不 fallback 到旧编辑框；真实两框在 preedit 中切焦点/取消/迟到 release 有最终控件状态断言。明确 root focus 是否保留的协议也必须保证不向失效目标投递。

### F-830-R1-07 / P1 / 代码：IME 锚点按 Surface BFS 第一挂载定位，忽略活跃挂载

AC-05，T-05/06，§5.3 共享内容挂载与候选锚点。`src/host/composition_input.rs:302,333-372` 不用 state.focus.hit.path，而通过 find_path 找第一个引用；root 空路径也返回 None。

`ime_anchor_uses_focused_shared_mount`：点击第二挂载已确认 focus.path=第二 Portal，local[10,10] 应窗口[160,20]，实际[20,20]（第一 Portal）；root anchor 也返回 None。OS 用例用固定估算 caret `surface_ime_os.rs:635`，仅检查窗口内，未检验实际光标/选择/缩放后锚点。

闭合：由活跃焦点/捕获挂载路径及真实控件 caret/输入区域生成锚点；同内容两挂载选第二、根框、路径变更/布局与光标移动都准确，OS 候选定位须有可观察证据。没有要求候选窗逐像素模板比对，但“落在整个窗口内”不足以证明约定的候选定位。

### F-830-R1-08 / P1 / 验收/实施遗漏：约定的三层方向与游戏场景没有全部实现

AC-03/04/07，T-04/07/08，S-01..03 / V-03/04/06。`cases/surface_compose.rs:232,379` 实际是3D→UI→2D；`surface_input.rs:209-251` 是3D→UI→UI。没有 S-01 **外层真实UI→3D→内层真实UI** 的完整路径/输入验收。S-02 PauseUi `surface_scenarios.rs:122-145` 仅 Pause/Resume，没有 reset、文本框/游戏键盘焦点切换；操作直接向预定 ui 注入，未测游戏与文本互斥。S-03 `:508,635` 使用另一个 Breakout，合同指定共享 Squash 没有采用；两挂载“一致”仅 mounted_by.len()==2（`:600`），没有对应最终输出对拍。V-03 动态 resize/DPI1/2、倾斜平面也未形成该入口证据。

闭合：按现有合同补真实 S-01、S-02 reset/焦点操作游戏、S-03 共享 Squash 多视图与最终呈现/一次模拟断言；沿完整挂载链注入而非直接指定 child surface。不能用旧独立 Squash golden 绿替代共享 Squash 场景，或静默把 S-01 改名为另一方向。若需范围调整，必须走显式合同修订，当前 r3 不支持这些遗漏。

### F-830-R1-09 / P1 / 门禁：负控和生命周期计量没有验证其宣称的故障

AC-06/09，T-08，V-07/08/09：

- `surface_lifecycle.rs:178-186,294-297` target_rebuilds 仅每隔一轮自增1并比20/2；每轮调用两次 resize，底层同尺寸也实际重建，因此既非目标创建/释放计数，也不能抓每帧额外重建。20轮原始 round_facts 确有循环，不是把20写死伪造；但实际未关闭/重开 View、隐藏/恢复、销毁槽位或检验资源回收/warm owner 关系。
- `:318-350` clip 只把 child 变洋红，没有移除/错设 clip；采样的外点(190,30)还在挂载几何外，即使漏 scissor 也看不到该 child。`legit_ok/caught` 本质同一个条件。
- `:428-439` alpha 的 sabotage_pixel 是常量 `[137,0,0,255]`，从未把实际 blend 改 opaque 后运行正式 gate；`:447-454` 第二设备只对 fake 字符串做 matches，没有在真实入口制造设备违规。错UV、重复事件/加倍模拟、每帧readback、错误依赖顺序没有相应正式 gate 反控。force_double_render 确调用内容路径，但人为 counter +=2，不是模拟 tick 的旁观计量。
- root/场景坏资源出口未测，导致F03的半提交漏检；`surface-ime-os` 所有 gate 不过统一 exit4（`:245-259`），包含产品断言失败，不能仅按 exit4 一概排除为环境问题。
- V-09 上传/内存/实际目标生命周期计量缺口未具名关闭；当前树/主附件没找到本冻结提交的 independent-consumer 重定位原始报告。本轮未执行其额外两份冷构建，AC-07重定位保持未验证，不据文案授pass。

闭合：在同一正式行为路径真实改变 fault，证明 mutation发生、正控过、错误分类和整体非零传播成立；记录实际 allocation/drop/owner/上传/readback计数及原始20轮事实，补冻结来源绑定的重定位证据。保持已正确说明 GPU timing=false。不得只改报告布尔值或再次统计自增次数。

### F-830-R1-10 / P2 / 文档：SD/交接与当前代码互相矛盾

AC-08，T-08，SD01..04/V-10。SD-02 §5仍说“本Plan未取得OS证据”，与本轮真实CJK报告及surface-result冲突；SD-01 §5的“零尺寸暂停”在 graph size 校验只会 InvalidSurface，不能作为已支持行为写入。handoff :13 声称 SetMount 置脏（F01实测否）、:15 列不存在的 cancel_all，:46 写231测试（实际135）。Null剪贴板与 Iced/3D 无条件重绘的交接限制未列全。Shader 接缝的 stage/binding/producer-consumer 已有有用事实，但还需按§5.6列全各管线参数 offset/size/alignment（PortalQuad buffer分配160B，WGSL实际struct32B，应区分）及真实错误出口；不能把未捕获的GPU错误写成均已分类。

闭合：先与修复后的代码/断言逐项核对再冻结 SD01/02 与 SD03/04的 before/after提案；将仍未实现的行为如实待定，真实OS证据引用准确。AutoShader/CAP、I1–I3明确 pending 这一点保持，通过实验不抹掉生产门。

### F-830-R1-11 / P2 / 卫生：新增警告与 rustfmt 不通过

AC-09，T-08/09，仓库 mandatory health。独立 check 有17个lib warning，另surface_contract 1个 dead_code；例如 `surface_compose.rs:23` unused IcedSurface、`surface_input.rs:409` 未用 text_a、`surface_lifecycle.rs:320` 无效初始赋值。rustfmt check exit1，差异涉及新增 composition/input/runtime/cases/tests，见持久日志。

闭合：清理新增 warnings、在相同冻结源 fmt check 过，无默认调试输出；AGP_INPUT_DEBUG门控诊断允许保留，不用笼统 allow 抹掉实际问题。

## AC / task / 规范结论

| AC | 结果 | 依据/任务 |
|---|---|---|
| AC-01 | partial | T01来源/独立锁/零根写面通过；V09重定位当前证据未闭 |
| AC-02 | partial | 原子model批次/代际/cycle/depth/拓扑与18测试通过；结构dirty F01未闭，T03 |
| AC-03 | fail | 单帧采样/颜色与双纹理修复有效，但跨帧alpha F02、S01缺失F08，T04/07 |
| AC-04 | fail | 相机F04/嵌套遮挡F05/取消F06与场景F08，T03/05/07 |
| AC-05 | fail | 真实CJK单框输入通过；焦点/取消F06与活跃挂载锚点F07，T05/06 |
| AC-06 | fail | 结构更新F01、跨帧F02、坏资源F03、生命周期/反控F09，T03/04/08 |
| AC-07 | partial | 冻结游戏实现/golden未改，旧两游戏与spike过；场景F08/重定位F09未闭，T07/08 |
| AC-08 | partial | 实验/生产分层与AutoShader pending正确；SD/交接F10需修，T08 |
| AC-09 | fail | 本轮已独立审查，F01..11 OPEN；不勾T09，health F11未闭 |
| AC-10 | pending merge | 本轮不执行、不提前授完成 |

SD-01/02因上述实现与事实冲突未通过；SD03/04沉淀方向保持，但要等修复事实再冻结可发布文字。保留frontmatter现有两个modify、两个add和GOAL007/008；不改变知识归属，不发布canonical/ledger。

## 路由与冻结后的合入门

outcome=**needs_fix**，计划回 `executing`，T03..08重开，T01/T02保留完成，current_step=2/10；历史work绿色记录保留但由本轮结论覆盖当前状态。原树work应一次处理本轮不变量/覆盖缺口，记录原反例、同源出口、合法对照、真实故障负控与剩余边界，提交后再审；本审核不代修。

merge-owned pending（仅供后续最终pass使用）：新冻结提交的AGP locked check/test/release、六surface验收与真实OS输入/IME、冻结829兼容与独立consumer/重定位、修复后正式负控和资源生命周期门；再按合同并入autographics-dev、沉淀SD/模块docs+ledger/index、归档与wt-guard clean清树。仅experimental Rust不触根cargo t/tv/tu/docs_gen；tf仍主检出到期单实例批量。未达到最终review pass前不执行合入、不称已交付。

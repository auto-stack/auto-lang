# PLAN-737 交付回执（jade-edit PLAN-037 T-02 解锁缺陷跨仓修复单）

日期：2026-10-03。产出：master 59ff4e66f（分支 plan-737-dev
5d967320e..b88a2e3a2，基面 ccc3a2f05）。本件是缺陷单五项交付的对岸供料
回执；jade T-02 复验按其契约 §1 重绑规则以本件 §4 binary 执行。

## 1. 根因定谳（推翻缺陷单主假设）

缺陷单假设「DocLayout 写回与可见文档脱节」被插桩实测**否定**：布局写回
每帧健康（render_frame WRITE w=876 blocks=8 links=5，press 逐块命中正确，
block rect 与链接 rect 与可见文本对齐）。缺陷单引用的 focus 带退化
（{3: y2..44, 4: y47..359} / block0 首中 y=51）实为**读数竞速伪影**
（见 R-2）。真实根因两个：

- **R-1（主根因）合成通道丢激活**：MCP `editor_drag`（`__mcp_drag_ade`）
  直调 `core.handle_input`（PLAN-057 keyed 寻址、不经 widget），只消费
  `focus_changed`/`text_changed`——PLAN-732 的 `DocOutput.link_activated`
  在该通道被整条丢弃。插桩铁证：点击门 `same=true` 18 次恒成立、
  widget.publish（ACTIVATED）0 次。真实事件泵路径无此缺陷（六环语料
  13/13 一直绿的原因）——缺口精确特属 MCP 合成通道，与缺陷单方向 a/b/c
  均不符。
- **R-2（伴生）合成动作「入队」语义**：MCP 动作经 channel→60fps 泵→
  update 异步应用，工具立即返回——探针 press→立即读状态与泵竞速，
  focus 带映射系统性偏移。

### 首帧节拍复证（回执⑤口径）

`autodown_editor_sync` 首帧 no-op（sync 先于 DocEditor::new 注册
=UNREGISTERED）→次帧重降层 rebuild——SD-01 ⑤注意点按设计工作，非缺陷。

## 2. 修复内容（master 59ff4e66f）

- **D-1 派发表（R-1 根修）**：renderer.rs `ADE_LINK_DISPATCH`（storage
  key → LinkCallback<IcedMessage>）；`convert_view_messages` AutodownEditor
  臂以**与 widget on_link 同一闭包**装配注册（消息构造单源，无第二套
  拼装）；`__mcp_drag_ade` 抬起腿经 `ade_link_dispatch_message` 消费
  （查表前 `normalize_payload_key` 幂等归一——MCP 载荷 sk 为裸键）。
  真实事件泵路径零变化（C-04 结构性实例绑定不动摇；O 臂双 key 实测各发
  各来源零冒领）。`__mcp_click` press-only ghost 负例语义不变。
- **D-2 应用栅栏（R-2 根修）**：`SharedState::send_action_applied`——
  `__mcp_*` 合成事件 event 尾附 `|ack=<id>`，应用侧 noop 回执点/臂尾
  `mcp_ack_apply` 回执（复用 fixture ack 表），工具侧限时轮询（1s 上限、
  轮询期不持锁）。非合成事件（press/type_text/键盘 handler 直派）保持
  既有异步语义零改动。
- **D-3 观测面**（AUTO_ADE_TRACE=1 常驻，零门控开销）：`RELEASE gate
  (x,y) pend=(block,lo,hi,target) same=` / `mcp_drag link_activated …
  dispatched=` / `link_dispatch MISS sk (table keys=…)`。
- **D-4 MCP HTTP 加固**：工具执行下沉 `spawn_blocking`——服务器为
  current-thread tokio runtime，handler 内同步阻塞（快照大树/截屏/栅栏
  轮询）冻结 accept/IO 线程，是探针连发 ECONNRESET 的放大器。

## 3. 验证（实机 + 门禁）

### 实机（真实 jade 应用 merged 窗，`auto run -r vm` + JADE_WORKSPACE
fixture + MCP，jade 探针 tests/probe_native_wiki_link.mjs 原样只读消费）

| 项 | 修复前（v0.4.2-2650） | 修复后（工作树/交付 binary） |
| --- | --- | --- |
| mapBlocks 块带 | {0:31}/{1:166} 等漂移 | **8 块全对齐 {0:6,1:36,2:66,3:101,4:131,5:166,6:196,7:231}** |
| N2 Goals 真实点击 | 零激活（nav_seq 恒 0） | **PASS（6,6；nav_seq +1 恰一；Goals.ad 打开）** |
| N3 目标页真实点击 | — | **PASS（6,40；目标页.ad 打开）** |
| O1..O5 双实例 origin | — | **5/5 PASS** |
| N4 别名页 | — | FAIL（消费方域，见 §5） |
| P1 真实键入 | — | PASS |
| P2 脏源点击 | — | FAIL（冻结契约域，见 §5） |
| ECONNRESET | 两次（旧 binary 即有） | 放大已除（spawn_blocking）；残余偶发=既有基础设施债（§5） |

trace 级证据：`RELEASE gate … same=true` → `mcp_drag link_activated
target=… dispatched=true`；AUTO_SCHED_DIAG update 入口收到
`OpenWikiLink s别名页 s`（消息到达 update 面实证）。

### 门禁（fix-test-tiering 分级；Category B）

```
cargo check -p auto-lang                      # 默认/autodown/ui-iced 三 feature 集 ✅ 零错
cargo nextest run --lib --features autodown plan732   # 13/13 ✅ 零回退
cargo nextest run --lib --features autodown plan737   # 2/2 ✅ 新增
cargo t autodown                              # 79/79 ✅（基线 78+新增覆盖面，零红）
裸 cargo t（工作树，复审门禁）                  # 14 预存红与 master 基线逐名 diff 全等 ✅
                                              # + 2 flake（state_file 锁/plan730 矩阵）隔离复跑绿
cargo fmt --check                             # 我方三文件零 diff（仓内 28 处 fmt 预存）
```

## 4. 交付 binary 指纹（jade T-02 重绑目标）

- 路径：`D:/autostack/auto-lang/target/debug/auto.exe`（AUTO_EXE 默认）
- 版本：`auto 0.1.0+v0.4.2-2702-g59ff4e66f`
- SHA256：`43355F6536FC36CCBA14D8155544CCA40F537C76280FC6D820F95A7CEE628F43`
- 说明：重建时旧 binary 被共机 musk-097 会话进程持有，已 rename 为
  `auto.exe.pre-737`（其运行中镜像不受影响）；新 binary 由 master
  59ff4e66f 全量链接。**交付 binary 自检：本机复跑 jade 探针 --phase
  native，N0..N3 PASS**（4 PASS/1 FAIL=N4 消费方域）。

## 5. 非供给面残余（明文分界，供 jade 复验对齐）

- **N4（别名页）**：供给链全通（门 same=true→dispatched=true→update 收到
  消息；单次点击实测走到 jade handler 的 missing-confirm 分支——
  `create_confirm_open: true`）。断点=jade `resolve_wiki_link`（其
  back/wsys.at 的四级解析：stems∪aliases∪rels 未把 别名页 →
  AliasTarget.ad）。SD-01 §1 明文「身份判断四级解析归消费方后端，本层
  不做」。jade 复验时核对其 Vue 轨同 fixture 的解析行为与
  `collect_ad_pages`/`page_aliases` 在 merged 模式下的读面。
- **P2（脏源点击）**：探针在块 0 键入 XYZ 后点击块 0 自己的 [[Goals]]
  ——冻结契约 U-02/`plan732_local_edit_deactivates_until_rebuild`（被
  编辑块链接暂态失活直至外部真变化 rebuild；自回显走 PLAN-057 回声守卫
  不重建）。探针前提与冻结契约相抵，非回归；块级失活粒度下非编辑块的
  链接点击不受影响。
- **ECONNRESET**：非 panic（应用进程不退出、trace 无 panic 痕迹）、非
  本缺陷根因（jade 侧旧 binary 亦两见）。已定谳的结构弱点=current-thread
  runtime 上同步 handler（本次 spawn_blocking 加固）；残余偶发复位登记
  KNOWN-DEBT（建议后续以常驻 AUTO_ADE_TRACE/AUTO_SCHED_DIAG 双观测面
  + 服务器侧 stderr 捕获定向排查；疑点=多 app 顺序 runs 的 keep-alive/
  端口生命周期）。

## 6. 证据索引

- 计划：docs/plans/archive/737-ade-layout-writeback.md（复审记录/AC 绑定）
- 交付 commits：5d967320e（R-1+R-2 主修）、b88a2e3a2（spawn_blocking
  加固）、merge 59ff4e66f
- 新增测试：plan737_mcp_drag_single_point_full_click_dispatches_wiki_
  activation / plan737_press_only_still_never_dispatches（renderer.rs）

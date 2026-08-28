# Plan 446 下游对账运行说明（交 auto-os-config agent 执行）

日期：2026-08-28。上游基线：auto-lang master（含 446 批一~批五开篇：
A1/B1/B3/C1/D1-D6/E1-E4/F1/F3/F4/G1/J1/J2/K0/K1/U7 + U1 验证解除）。
本说明给出：可撤绕行清单（逐条判据）、e2e 双门禁复跑步骤、验收口径与
失败时的取证要求。执行环境：auto-os-config 仓（`D:\autostack\auto-os-config`），
须先 `git pull` 其 main 并确认 auto.exe 已更新到含上述修复的构建。

## 一、前置：重建上游二进制

```bash
cd D:\autostack\auto-lang
git log --oneline -1                # 确认含 "批五开篇" 合并
cargo build -p auto                 # 或取你们的 auto.exe 分发通道
auto --version
```

## 二、e2e 双门禁复跑（先跑这个，再撤绕行）

```bash
cd D:\autostack\auto-os-config\auto
node scripts/e2e-vm.mjs             # VM 轨 9 断言
./scripts/e2e.sh                    # vue 轨
```

预期：绿。已知非阻塞偏差：e2e-vm 按压对自注册模块 "Harness Roles"
错位（门禁脚本自身问题，446 §J 批增补在案，勿计入上游）。

## 三、可撤绕行清单（VG 条目 → 上游判据）

撤除原则：一次撤一条 → 跑对应验证 → 绿则提交，红则取证回传（见五）。

| VG 条目 / 绕行 | 上游修复 | 撤法 | 验证 |
|---|---|---|---|
| VG16 store 方法名全工程唯一（Open/Pick/NewEntity/… 改名） | A1 消歧：漏声明/撞名→编译错误；限定调用 `Store.Method()` 可消歧 | 把 Collection 方法名改回 Init/Select 等；调用点可用 `collection_store.Init()` 限定形态 | `auto check`（vue+vm 双轨编译）+ e2e-vm 导航/选中断言 |
| 影子数组（names[]/entry_keys[] + 索引参数模式，041 模式） | B1 验证解除：`for x in .store.list { onclick: .F(x.field) }` 正常 | 删平行数组，循环直取 `e.name` 等字段作实参 | 上游回归 `plan446_b1_tests`（已锁）；跑你们详情列表选中 |
| VG3 http 三不用（res.status()/builder 链/res.body()） | E1/E2/E3 全修：status()=线上真值；builder 链后同 handler 连续调用存活；body()=UTF-8 文本 | 恢复直接使用三 API | 上游 `plan446_batch2_tests` e1/e2（真实 TCP 断言）；你们的 put-后-GET 可简化为 status 判读 |
| VG8 json.parse 禁用（全走 keys/get 文本工具链） | D1：parse 物化 + 点访问/数组迭代可用；**且文本工具链双态兼容**（get/get_at/has_key/len/keys/is_valid 接受 parse 产物与文本两种入参） | 两种写法都合法——可渐进迁移到 parse+点访问；现有文本管线不必急改 | 上游 `plan446_batch3_tests` d1 双探针 |
| VG11 json.get_at 仅接受文本 | D5：get_at 接受 json.keys() 返回的键列（负索引编码） | `json.get_at(json.keys(x), i)` 直接可用 | 上游 d5 探针 |
| VG12/13 数组不跨 fn 边界 / fn 返回一律扁平 | D3 验证解除：数组跨 fn 实参存活 | `fn arr_len(a []any) int` 类签名恢复 | 上游 d3 探针 |
| VG14 .find(闭包) 不可用 | D4 根修：i32 强转剔除 + miss=None 契约（fn/handler/局部/[]any/obj 全语境） | `.modules.find(x => x.id == id)` 直接可用；miss 判 `== None` | 上游 `plan446_batch4_tests` d4 双探针 |
| VG4 handler 内单跳链读限制（D2 族） | D2 验证解除：循环变量字段读在 handler 正常 | 两跳读/局部中转可恢复 | 上游 d2 corpus 探针 |
| 确认层用 if 块替代 popover（C1） | C1 渲染半已验：现场属性组合形态可渲染（上游常驻锁） | 可迁回 popover；建议先小面积试点 | 上游 `plan446_c1_popover_tests`；你们的 e2e 详情确认流 |
| 输入框"单参 + Apply"（B3） | B3：多参输入分发现在**显式报错并丢弃**（stderr 有 [VM-INPUT] 诊断） | 不再静默错位——但多参输入 handler 仍不支持；保持单参或等待后续 args 透传 | 触发一次多参输入看 stderr 诊断即验证机制在位 |
| regen VM_ONLY 过滤 + 006 ext facade（G1） | vue codegen store 导入前缀可配：`ComponentGenOptions.store_import_prefix`（默认 @/stores 不变） | 部署若把生成 store 放 `src/stores/auto/`，生成时配 `@/stores/auto` 后不过滤 | vue-tsc 零 TS2307 |
| 字母序已知偏差（D6） | preserve_order：vm 轨字段顺序=插入序 | 删偏差登记；对拍字段顺序应与 vue 一致 | 你们的 track-parity 分区 diff |
| 侧栏 nav 条件展开双态静态串 + label 全量 text-prop 化（commit 1610c21） | U7：label { text (text: e.label) {} } 子元素折叠已修；button class: 求值链已通（自定义非 Tailwind 类名会显式 warn 而非静默丢） | 撤静态串双态，恢复单态条件展开；label 恢复 children 形态 | 上游 `plan446_batch5_tests` u7 四链矩阵；对拍侧栏 diff% |

## 四、U1（P0 事件冻结）专项复验

上游已在**真实 iced 窗口 + MCP 通道**用镜像 corpus 验证连续 press 正常
（上游 `test/ui/plan446_u1_event_freeze/mcp_probe.py` 可参考）。你们侧请在
**真实集合页**复验：进入集合页 → 实体列表加载（循环构建）→ 侧栏连续切换
3 个视图 → autoui_state 断言 active_id 逐次翻转。若你们的完整形态仍冻结，
按第五节取证——上游 reduced corpus 未覆盖的形态差异就是定位输入。

## 五、失败取证要求（回传给 auto-lang 侧）

任一撤除后红：保留 `auto/` 最小复现 diff + e2e 输出 + （若有）stderr 中
`[VM-HANDLER]`/`[446-U7]`/`[VM-INPUT]` 诊断行 + autoui_state 前后快照。
上游各修复均带回归测试名（见上表"验证"列），报障时附对应测试在你们
形态下的等价最小化 corpus 最佳。

## 五A、批五续二增补（U3 截图通道 + U6 快照窗口，2026-08-28 晚）

上游新增（须含在重建的二进制里）：

- **U3 详情态截图超时已修**——真因不是"富子树>10s",而是 roles 实体
  soul sidecar 数据损坏(见下)触发 cosmic-text 字形整形天价(≈60µs/字符,
  连续 `-`/`\` 字符),把整个 iced 事件循环冻结。上游 renderer 现对
  **>64KB 的 textarea 值降级只读预览**(提示行+前 32K 字符截断,store 原
  值不动)。验收口径:详情态 `autoui_screenshot` 应 <2s 返回(原 10s 超时)。
- **U6 快照首帧前窗口**:未完成首渲染时 `autoui_snapshot` 回退源模板,
  现带 `PRE-RENDER FALLBACK` 前缀自标识——见到它=应用还没画完第一帧,
  重试即可(实测 <1.5s 自愈),勿再当"应用空壳"取证。

**数据损坏对账项(重要,双端)**:`~/.config/autoos/roles/assistant.soul.md`
磁盘上已是 655K 连续反斜杠(08-25 产生,2^k 增长=保存路径每轮翻倍);经
HTTP 文本管线进 UI 后再翻倍至 1.31M(线上转义形式未还原)。请:①恢复该
文件(从 .bak 或重写);②定位保存路径的翻倍点(VG13 文本重建手术嫌疑最大);
③如定界到上游(escaped 形式当明文交付),带最小复现回传。

**已知未决(非本轮范围)**:collection 模块体(Roles/Skills)在当前上游
二进制下主区像素空白(页标题渲染、快照树完整、Daemon 页正常)——css-era
基线有内容,区间回归,上游已登记台账(446 渲染回归条)待立项。track-parity
若复跑,03/04 两视图预期红,属该已知项而非撤绕行引燃。

## 六、预期收益

§H 快照所列常设成本中，store 改名、影子数组、API 文本化、单参输入、
popover 替代五项绕行均可撤；VG 清单预计可删 VG3/4/8/11/12/13/14/16 共
8 条。撤完后跑一轮 track-parity 全视图对拍，diff% 应不升（U7 修复后
侧栏两链对拍面反而应降）。

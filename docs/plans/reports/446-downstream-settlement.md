# Plan 446 下游对账结算报告（auto-os-config → auto-lang 回执）

日期：2026-08-29。执行依据：`docs/plans/reports/446-downstream-runbook.md`
（2026-08-28，含五A 增补）。执行环境：auto-os-config main
（起点 282f7f1）；上游二进制 auto-lang master c83435764
（v0.4.1-2486，446 收口 5877603be 在祖先链，src 树 clean，含批一~批五+五A 全部修复）。

## 结论

**Runbook 全项执行完毕，零红项。** 双门禁终验绿（e2e-vm 17/17 + vue 三套件
ALL PASS）；§六预期收益全额兑现：五项常设绕行撤除、VG 清单删 VG3/4/8/11/
12/13/14/16 共 8 条。

## 一、e2e 双门禁（runbook §二）

- 前置基线：VM 17/17 PASS（门禁脚本自 9 断言已演进至 17）+ vue ALL PASS。
- 终验（全部撤离后）：同上双绿。已知非阻塞偏差（Harness Roles 按压错位）
  本次全程未再现。
- 中途一次 e2e.sh 失败为 daemon 端口竞争瞬时态（与本仓改动无关，复跑即绿）。

## 二、五A 专项：soul.md 数据损坏（双端对账项）

- **翻倍点定界确认**：与本仓 `back/api.at` fetchEntitySafe(727)/
  fetchEntityFlat(1059) 的 `unquote(json.get(t, "sidecar"))` 成语完全吻合。
  实证：修复前一次 e2e save 循环使磁盘文件 327,747→655,494（×2）。
- **修复（下游）**：字符串内容取点 parse-first（sidecar + fm_name/
  fm_description/fm_body 四点；`auto/src/back/api.at`，提交 8a7b85f）。
  `value`（atom 体）**有意不动**——它以字面量形态裸插 PUT payload，
  json.get 的重转义字面量恰好是合法 JSON 传输形态，parse-first 反而破坏
  契约。修复后 save 循环文件尺寸稳定（639B 恒定）。
- **文件恢复**：assistant.soul.md 磁盘 655,494B 连续反斜杠、无 .bak、
  css-era/git 均无种子——原文不可恢复，按干净 markdown 重写（639B）。
- **U3 验收**：详情态 autoui_screenshot 1587ms（<2s 门，原 10s 超时）。
- **U1 验收**：真实集合页（循环构建实体列表）→ 侧栏连续切换
  ai-daemon/skills/roles 三视图，active_id 逐次翻转，全程无冻结——
  上游 U1 验证解除在本仓完整形态成立（plan010 门禁段序绕行可废止）。

## 三、撤绕行清单结算（runbook §三，逐条提交）

| 条目 | 结算 | 提交 |
|---|---|---|
| VG16 store 方法名 | Collection 五方法回归 plan006 自然名（Init/Select/Create/Save/Remove）；6 处同名点按 A1 限定形态 `Collection.X()`/`Modules.X()`；A1 编译期歧义报错实测有效（改出过 5 处歧义全数精确报出） | 4b69379 |
| 影子数组（B1） | names/entry_keys/view_names 三平行数组全删；8 个索引参 handler 的 index→key 扫描样板全删（净 -235 行），事件实参直取 `e.key`/`e.name` | a781d0e |
| VG3 http 三不用 | get_text 改 `http.get` + `res.status()`/`res.body()`；四个写后 GET 验证简化为 status 判读（axum Ok 臂 200） | b1dc8f7 |
| VG8 json.parse | 台账删行；现有文本管线保留（双态兼容，渐进迁移不强制）。parse-first 已在 soul.md 修复中实战使用 | 66d54f0 |
| VG11 get_at | entryAt 手写键序扫描 → `json.get_at(json.keys(body), i)` 一行 | 9042220 |
| VG12/13 扁平 | 台账删行；flat api surface 保留——它同时是双轨共享接口（api.ts 镜像），已从"绕行"转为"接口设计" | 66d54f0 |
| VG14 find | modules_store 选择回归 `.modules.find(x => x.id == id)`（miss 判 `!= None`）；selectInfo 沉降 fn 删除（Init deep-link 同步） | 9042220 |
| VG4 两跳读 | 同上批：find 返回对象字段直读（D2 修后形态） | 9042220 |
| C1 popover | 删除确认层迁回 popover（点锚+ondismiss+class 面板）；VM 像素级验证开→关 | ee0ba32 |
| B3 单参输入 | 保持单参；多参分发显式丢弃实测在位（临时双参 handler → stderr `[VM-INPUT] ... (plan-446 B3). Dropping this input dispatch.`，探针后已还原） | — |
| G1 regen 过滤 | VM_ONLY 清单 batch6 已退役（先期）；`store_import_prefix` 未暴露 CLI，regen sed 形态保持 | — |
| D6 字母序 | 偏差登记删行；实测 roles 详情字段序 = 插入序（Name→Description→Inherit→Model Tier→Soul，与 vue 一致） | 66d54f0 |
| U7 侧栏双态 | 三处 nav item 双态分支合并为单态 `class: m.nav_class`（完整类串预计算进 store）；12 处 label text-prop 恢复 children 形态 | 0875ff4 |

另：五A soul.md 修复 8a7b85f；台账结算 66d54f0。

## 四、track-parity 对拍（runbook §六）

- vue-vs-vm 全视图：00=2.15 / 01=4.99 / 02=3.48 / 03=2.24 / 04=4.16 /
  06=2.73 / 07=2.11（05 截图通道本次抖动 SKIP）——L2 已知缺口
  （select 缺位/thead 暗色）+ L3 光栅量级，与终值台账同量级。
- **03/04 未红**：五A 所登记"collection 模块体像素空白"回归在
  c83435764 二进制下**未复现**（快照树完整 + 像素有内容），建议上游
  台账复核该条的现役性。
- css-era 基线 PNG 不入库无法复现原口径；本轮以 vue-vs-vm 口径替代。

## 五、上游回传（本轮新发现，3 项）

1. **codegen applyAccent 撞名（已下游改名规避）**：Plan 409 §8/458 起，
   拥有 `accent_color`/`dark_mode` 的 store 会被注入内嵌 applyAccent
   助手（ACCENT_PALETTE_JS + Plan 458 watch 同步行）；若该 store 同时
   `use back.api:` 导入同名 fn，生成 TS 即 TS2440（import vs 局部声明
   冲突）。下游已将 back.api 的 applyAccent 改名 saveAccent（与
   loadAccent 对称）。建议：注入前检测 use 清单撞名并避开/告警。
2. **merged 模式链接面文档**：back.api 符号链接以**外部 back 工程**
   （auto-os-config-back/api.at）导出为准，in-project 的 auto/src/back/
   api.at 只供实现体——改名/增删 fn 须两份 api.at 同步。本次只改实现侧
   曾致 `Undefined symbol: api.saveAccent in module App` boot 崩，报错
   未指向第二份文件，建议报错提示补"检查外部 back 的 api.at 导出清单"。
3. **vue codegen popover 半缺口**：popover 在 vue 侧为惰性 div 透传
   （`:open` 不门控 → 面板常显；`@dismiss` 不接线）。VM 渲染半（C1）
   已修，vue 半下游以 regen 部署侧 sed 补偿（`:open`→`v-if`）。建议
   vue 半补 shadcn 式 v-model:open 门控或 v-if 输出。

## 六、遗留与备注

- e2e-vm 对对象数组的 state 投影为 `[<vmref>]`（内容不可见）——门禁两处
  断言已改快照口径（更强：直接验证实体名进 UI）。上游如做 state 投影
  增强（对象列表摘要）可再回收。
- U1 修后 plan010 门禁的"集合段排最后 + skills→roles 强制 reboot"段序
  约束理论上可废止（本仓已实测连续切换无冻结），属门禁脚本整理，另案。
- 通知：本报告落盘未提交（auto-lang 工作树有并行会话在途改动，提交权
  留该侧）。

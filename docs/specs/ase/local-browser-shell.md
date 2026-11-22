# AutoScape 本地浏览器七区外壳（已验现状）

> 来源：PLAN-1118（2026-11-22 交付）。实施合同见归档计划；外壳设计见
> [Design autoscape/browser-shell](../../design/autoscape/browser-shell.md)。
> 本文件只记录**已验证**的能力与边界；未交付面一律不视为已支持。
> UI 侧消费合同见 auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md`。

## 冻结身份与地址（T-01）

- 演示注册表（冻结）：`app://auto.demo.counter/`（Counter，源 auto-lang
  `experimental/ase-m0/fixtures/counter-a/main.at`；按钮声明序 0=`-`/1=Reset/
  2=`+`）、`app://auto.demo.clock/`（Clock，`fixtures/clock/main.at`，
  Tick(int) 校时）、`app://auto.gallery/`（内建目录，元数据页零装载）。
- 地址输入归一化（冻结）：trim → 显示名大小写不敏感 / 裸 identity →
  `app://<identity>/`（根路由缺省补 `/`）→ ase-m0 `parse_app_uri` 严格解析
  （单一语法源：仅 `app` scheme、小写 authority、仅根路由、无
  userinfo/port/query/fragment）→ 冻结表 lookup；未登记具名拒。
- 归一化/解析/登记拒绝均不改变已提交状态（失败独立展示 + 重试入口）。

## 导航与历史语义（已验）

- 每页签独立 `nav_generation`（进行中请求代次）、已提交地址、历史游标与
  视口。受理导航 = 代次推进 + 分段装载（`begin_open` 真实编译成票据）；
  挂载在下一 tick `mount_open` 前核代次与容量（权威复检）。停止、同页新
  请求、关页使旧票据失效——票据 drop 即引擎 owner 释放，零 session/零
  UI 效果，已提交页面与原历史记录不动（负控在案：stop/抢占/关页）。
- 成功的新目标一次提交（标题/规范地址/历史/视口）；地址与当前已提交相同
  → 刷新语义。新目标截断前进记录；后退/前进只移游标，未驻留历史页按原
  地址显式重开（不加历史、失败保留游标）；刷新不加历史、guest 会话换新
  代次种子 0。地址栏编辑、悬停提示与错误信息不提前替换已提交来源。
- 资源上限：复用会话层两活上限（`session.limit` 具名拒）；准备中的装载
  同样计入（begin/mount 双检）；容量拒保留已提交页面，释放后重试可达。

## 会话与内容区（已验）

- 会话层原样复用 1106/1109 的 `AseSessionHost`（选中车道 + 引擎常驻、
  close 代次、Clock 一秒唤醒）。外壳每页签至多一个 guest 会话；同页签
  替换导航在新目标可呈现后释放旧会话；切到无 guest 页签撤出向车道
  （`deselect`）——隐藏页签零业务/零呈现，Clock 恢复恰一条新校时。
- 双页签同源 Counter 隔离（3/7 经真实 typed 队列；切换零装载）在案。
- 内容视口二态：guest AutoFrame（全局 HOST 当前 session 的 AV 真值 + 真
  实按钮）或内建 Gallery 目录（非 guest 原生内容进入同一视口；目录/新
  建首页零编译、零 session）。

## 收藏与配置（已验）

- 收藏落宿主用户配置目录 `~/.config/autoos/apps/autoscape/bookmarks.json`
  （env `AUTOSCAPE_HOME` 覆盖根，隔离测试）；条目 = 标题 + 规范地址 +
  打开方式（M0 冻结 `tab`）；同地址去重；原子写经
  `auto_lang_core::state_file::atomic_write`（与 desktop_config 同入口）。
- 添加/删除经重启一致；坏配置具名记录 + 空表可用（盘上旧有效写入不被覆
  盖）；写失败具名可观测且保留最后有效配置。

## 观察口与入口

- CPU 档：auto-ui `cargo test -p auto-ui-runtime --features ace-bridge
  --test autoscape_shell_navigation -- --test-threads=1`（归一化语料、
  历史/提交/负控、容量、隔离、Clock、收藏重启/坏配置/写失败）。
- 原生窗口：`examples/autoscape_window`（`--evidence <json> --config-home
  <dir> [--script tick:动作] [--prep-only]`；窗口标题 AutoScape；窄窗档
  env `AUTO_VM_WINDOW=520x700`，常用档默认 1280x800）。排窗档
  `AUTO_UI_ASE_M0_E2E=1` 单实例覆盖常用+窄窗两档（V-01/V-06 自动面）。
- 证据 JSON：页签/历史/游标/进行中/失败、nav_log、收藏、会话层全套计量
  （loads/ui_lane_opens/dispatches/clock 送达）、错误表。

## 边界

真鼠标/键盘注入与真人走查档 pending（与自动注入分账，1022/1023/1109
同界）；Ctrl 快捷键经共享 keyboard_subscription 的声明式扩展投递（组件
`key_bindings` 声明 `Ctrl+<Key>` 才投递），真窗 OS 级键盘注入未验；
`https://`/`auto://`/`file://` 目标、权限、多窗口、发行与 Todo 内容不在
本域（1119/1111/1112 各自合同）。跨页签并发导航的取消面 = 活动页签单
导航（后台页签进行中请求在其重新成为活动页签后的首个 tick 提交）。

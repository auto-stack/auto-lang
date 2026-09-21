---
plan_id: PLAN-675
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: routes-in-embed
author: [agent]
created_at: 2026-09-21
updated_at: 2026-09-21
plan_revision: 2              # rev2=增补 T-08 lazy 会话装载(用户 2026-09-21 批准「按你推荐」开工)

# /auto-plan:review 结束时填写：
supersedes_spec_components: []  # 无退役组件(SD-01/SD-02 为既有 project.md 增补)
new_spec_components: []  # 无新组件;SD-01→auto-man/project.md 画廊节、SD-02→auto-lang/project.md back_proxy 节(merge 落档)
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man, auto-lang] # gallery host 生成器(auto-man)+ back_proxy 参数绑定(auto-lang)
current_step: 6
total_steps: 8
---

# [PLAN-675] routes-in-embed

## 变更摘要

Vue 臂画廊新增第四内嵌档 **routable**:带 `routes {}` 块的 demo(018/019/021/022/023,勘定后完整名单五家)不再落「独立运行」静态面板,而是以 **per-demo memory-history router**(vue-router `createMemoryHistory`)整体嵌入 `demo-mount-root` 单视口——壳视图、`router-link`/`router.push` 导航、动态路由参数(`:id`/`:slug`/双动态段)全部可用。VM 臂**零变化**(route_stub 档原样保留,「路由语义进 VM」仍是 PLAN-642 既定非目标)。随附清偿 **P672-N5**(back_proxy 会话路径参数按 `#[api]` 签名类型绑定,PLAN-669 先例)——018/019 的 `GET /api/*/:id` 数据面硬前置。与 **P670-D1 的关系裁定:无依赖,独立立项**(详见 §需求分析——P670-D1 是 a2r codegen 的 API 端点 fn 体降体债,「route A」指端点体策略,与本计划的前端路由无关;P672-D2 债条目中「属 P670-D1 家族」为名字撞车误注,本计划落档时更正)。

基线:master `51d39957c`(本计划骨架提交);硬依赖:PLAN-672 先落 master(fullstack 发射管线 + back-proxy 接线是其基座,plan-672-dev @ `bdcbddae2` 在档)。

## 目标

1. **五家路由 demo 全量内嵌可交互**:022(最小单路由样本)→ 021 → 019 → 018(双动态段 + N5 依赖)→ 023(7 路由 + 26 端点最大面),Vue 臂画廊点击即用,不再出现「请独立运行」面板。
2. **路由态与宿主严格隔离**:内嵌导航不改写宿主 URL(hash/path 双零污染);每次挂载 fresh 路由态(卸载即重置,无跨 demo 泄漏)。
3. **standalone 与既有档位零回归**:standalone 生成仍 hash-history 单例 router;loadable/fullstack/route_stub 三档判定、registry.at、VM 臂发射面逐字节语义不变。
4. **P672-N5 清偿**:back_proxy 会话路径参数按签名类型绑定,`GET/PUT/DELETE /api/*/:id` 数据面可用(018/019 硬前置)。

### 非目标

- **VM 臂路由语义**(路由导航进 VM 解释轨)——PLAN-642 :57 既定非目标,route_stub 档保持;将来另立项。
- **P672-N4**(013 Enter keyup 疑 codegen 绑定)——与路由内嵌无关,待 standalone 差分定归属。
- **P672-N2**(托管后台包装进程不定时退出、in-process proxy 随亡)——结构性弱点另账;本计划验证通道因此**钉死前台 `auto build`** + 用户终端活体 `auto run`。
- **P672-D1 双物化路径根治**——沿用手动同步纪律(os 源副本 + gen ext 副本字节对齐);根治另账。
- **browser 前进/后退按钮驱动内嵌路由**——memory history 天然无 URL 历史,既定取舍(见待澄清 Q-4)。
- **standalone 生成形态变化**——`src/router/index.ts`(hash 单例)不动。

### 约束与依赖

- **硬依赖:PLAN-672 先 merge**(fullstack per-demo 发射管线、`AUTO_GALLERY_BACK_PROXY` vite 接线、api client 现生成兜底全部在 plan-672-dev,未落 master)。本计划 worktree 必须从 672 落地后的 master 开出。
- 内嵌形态为**直挂组件非 iframe**(现画廊既定,gen 树 iframe 零命中)——路由隔离只能靠 history 模式,不靠文档隔离。
- vue-router 画廊侧按需注入(`^4.2.0`,与 standalone 同版;memory history 自 4.1 起一等公民)。

## 架构方案

### 候选方案对比(勘定结论)

| 候选 | 思路 | 尺寸 | 利 | 弊 | 裁定 |
|---|---|---|---|---|---|
| A. 复用 hash history 直挂 | 现模板零改动,直接挂载带 router 的 App | ~30 行 | 最小改动 | 改写宿主 URL hash;切换 demo 残留 hash 污染下一个挂载(不匹配路由→空 router-view);需跨 demo hash 重置,耦合脆弱 | 否 |
| **B. memory history per-demo** | per-demo router 用 `createMemoryHistory()`,路由态纯内存 | **~300-400 行(本计划)** | 与宿主 URL 净隔离;每挂载 fresh 态;router-link/push/动态段全语义保留;standalone 模板零回归 | 浏览器前进/后退不驱动内嵌路由(可接受);URL 不可分享(画廊场景无所谓) | **采纳** |
| C. `createWebHistory('/apps/<id>/')` | base 前缀真 URL | ~80 行 | 真 URL 可深链 | 需宿主服务器按 base SPA fallback;内嵌导航会**改写宿主浏览器地址栏**,刷新即离开画廊 | 否 |
| D. 页面拆分为多个可嵌入条目 | 每路由页独立 App 条目,画廊侧切页 | ~200 行 | 无 router 依赖 | 路由语义全丢,导航 UI 重复造;codegen 分发复杂度高收益低 | 否 |
| E. iframe 隔离 | 每 demo 独立 HTML 入口 | ~500+ 行 | 隔离最彻底(自带 history) | 多入口 vite、消息桥、样式穿透、双物化面扩大;现画廊零 iframe 先例 | 否(记录为远期替代) |

### 落地形态(候选 B 展开)

1. **第四档判定(auto-man `gallery_demo_row`,vue.rs ~:8360-8446)**:`GalleryDemoRow` 增 `routable: bool`。判定 = `!is_vm_only && vp.has_routes && !corpus.contains("@/ext/") && !corpus.contains("@/locales/") && !vp.i18n.enabled`(即 base_ok 去掉 `!vp.has_routes` 否决项)。**loadable/fullstack/route_stub 判定与语义零变化**(routes demo 三档维持 false/按旧规则)。五家实证无 i18n/ext 面(642 T-14a 已勘),预计五家全中 routable。
2. **per-demo 发射(auto-man,复用 P672 条目4/5 fullstack 管线骨架)**:`src/apps/<id>/` 下新增——
   - `pages/*.vue`:vp.components 的 `pages/` 面逐文件发射(既有 App/components/store 发射路径的镜像扩展);
   - `router.ts`:导出**工厂** `createAppRouter()`(非单例——保证每挂载 fresh 路由态),`createMemoryHistory()`,路由表复用 `generate_router_file`(vue.rs:2777)的路由表构建逻辑,import 路径改 `@/apps/<id>/pages/<X>.vue`,含参路由保留 `props: true`;
   - `main.ts`(入口):`export default App`(兼容)+ `export function mount(el)`(createApp + use(createAppRouter()) + mount)+ `export function unmount()`(卸载当前实例)。App.vue 本体复用既有生成(outlet→`<router-view/>` 映射已存在于 ui_gen vue.rs:8289)。
3. **demos-registry.ts 发射(auto-man `generate_demos_registry`,vue.rs ~:8449-8460)**:routable 行发射 `load: () => import('./apps/<id>/main.ts')` + `routed: true` + `loadable: true`(视口 v-show 门)。TS 类型面同步(load 返回类型放宽 entry 契约)。
4. **画廊 package.json 依赖注入**:存在任一 routable 行时注入 `vue-router@^4.2.0`(当前画廊依赖无 vue-router,实测 package.json 零命中;注入点沿 standalone 先例 vue.rs:526-530 的条件式)。
5. **AppViewport.vue 契约扩展(os ui-gallery scaffold)**:`mountApp()` 优先走 entry 工厂——`mod.mount?.(el)` / 切换卸载时 `mod.unmount?.()`;无此二者走既有 `createApp(mod.default)` 兼容路径(28 条既有 load 条目零改动)。`load` 存在即自然不进「独立运行」静态面板分支。**P672-D1 同步纪律**:os 源副本(`ui-gallery/src/gallery/AppViewport.vue`)与 gen ext 副本(`gen/front/vue/src/ext/src/gallery/AppViewport.vue`)字节对齐提交(69ee597 先例)。
6. **P672-N5 清偿(auto-lang `back_proxy.rs`)**:会话分派的路径参数绑定对齐 PLAN-669 按名绑定先例——按 `#[api]` 签名类型(str→int 等显式转型)绑定 path param;先定点(与 `http_server.rs:3489 bind_api_args_by_name` 的 typed 语义比对,同 crate 可考虑抽公共 fn),再修,补 GET/PUT/DELETE `:id` 语料测试。back_proxy 会话注册**不看档位**(有 `src/back/api.at` 即注册,vue.rs:6932-6940 实证)——018/019/021/023 的会话今天已在册,修好绑定即通。
7. **VM 臂零变化**:registry.at 三档并集(`loadable||fullstack||route_stub`,vue.rs:6654)、`emit_gallery_vm_demos` route_stub 发射、AppViewport.vm.at 全部不动。

### 技术栈

Rust(auto-man `vue.rs` 生成器;auto-lang `back_proxy.rs`);Vue 3 + vue-router 4(memory history);os ui-gallery scaffold(手写 Vue SFC 资产,双物化同步)。

## 需求分析与背景调查

### 授权记录

- 授权源:用户立项 prompt(2026-09-21,PLAN-672 收尾会话移交)——L1 立项,交付物=勘定+Plan 文档,**方案经用户确认后才进 worktree 执行**;若取号撞号可用新文件序号(实取 675,无冲突)。
- 允许仓:auto-lang(本仓)+ auto-os(ui-gallery scaffold 副本);验证通道:前台 `auto build` + 用户终端 `auto run` 活体走查。

### 受影响 demo 全名单(勘定实测,五家非三家)

| Demo | 路由 | 动态段 | 页面(pages/) | 后端消费 | N5 撞面 |
|---|---|---|---|---|---|
| 018-book-reader | 4(`/`,`/book/:id`,`/book/:id/chapter/:ch`,`/settings`) | **双动态段**(最深) | 4 个 602 行 | book_store 直调 list/create/delete/update;`/api/books/:id`、`/api/books/:id/chapters`、`/api/books/:id/progress` | **是**(GET :id) |
| 019-video-app | 2(`/`,`/watch/:id`) | `:id` | 2 个 481 行 | watch 页 `get_video/add_view/toggle_like`;`/api/videos/:id` 族 | **是**(GET/POST :id) |
| 021-blog-viewer | 3(`/`,`/post/:id`,`/new`) | `:id` | 3 个 510 行 | detail 页 `.to_int()` 显式转型先例 | 是(GET :id,前端转型规范) |
| 022-kanban | 1(`/`,单页壳) | 无 | 1 个 158 行 | store 有 back(api.at 在档) | 否(无路径参数端点) |
| 023-realworld | 7(`/`,`/login`,`/register`,`/article/:slug`,`/settings`,`/editor/:slug`,`/profile/:username`) | str 型 slug/username | 7 个 623 行 | **26 个 `#[api]` 端点**,双 store;登录后 `router.push("/")` | 否(str-str 天然同型;唯一 int 路径参数来自点击事件非路由) |

全部扁平路由,无守卫/重定向/嵌套 routes 语法(全库 grep 零命中)。对照组:013/015/017(已内嵌)确认无 `routes{}` 块,单 App 壳+子组件组合,故此前不触发本缺口。前端 `router.param` 返回 str——018 `book_detail.at:109` 与 019 `watch.at:180` 存在 str 未转型直入 int 字段(VM 臂既有语病);Vue 臂经 HTTP 字符串化后无害,不在本计划修(VM 臂语病属语料侧,另账)。

### 与 P670-D1 的关系裁定(立项必答,结论:独立,无依赖)

P670-D1 原文(KNOWN-DEBT-AND-RISKS.md:2564)是 **a2r codegen 后端债**:无类型契约端点生成 `// TODO` 空体桩,根因=「route A(端点 .at fn 体降体进 handler)从未接线——`ApiEndpoint.body` 字段生产不读(Plan 399 走了 route B db 委托)」,四缺口=①伴生模块通用转译 ②route A 接线(~100-150 行) ③trans/rust.rs 内建面(Env.get/fs.tree) ④a2r-std fs.tree 新宿主,合计 ~210-350 行跨三 crate 动 trans/ 本体(docs/reports/p670-fr1b-survey.md:24-33)。**此「route」是 API 端点体实现策略(A=fn 体直降/B=db 委托),非前端路由**。本计划的前端 routes{} 内嵌不消费四缺口中任何一条:路由 demo 的后端真体经 back_proxy 会话**解释执行**(PLAN-658 形态,不走 api_gen Rust 降体),018/019/021/023 的 `#[api]` 端点今天已注册会话。→ **裁定:与 P670-D1 零依赖,两计划互不阻塞**;P672-D2 债条目「属 P670-D1 家族」为名字撞车误注,本计划复审落档时在债册更正。唯一真实交联=**P672-N5**(back_proxy 路径参数类型绑定),以 T-02 纳入本计划清偿(见待澄清 Q-2)。

### 计划史引用

- **PLAN-642 T-14a(archive/642:411-433)**:route_stub 档裁定——routes demo 在 VM 臂以「壳视图+outlet 替换首页组件」stub 入可交互并集(registry.at `loadable||fullstack||route_stub`,vue.rs:6654);「routes/i18n 族内嵌化……另立项」(:57)即本计划的前身裁定。web 臂当时明确不动(route_stub 字段文档 vue.rs:8438-8446「Vue 臂语义不变(web 端维持静态面板)」)。
- **PLAN-658**:back_proxy 多后端宿主(`/apps/<id>/api/*` 会话面);会话注册按 `back_needs_session(back/api.at)` 不看档位(vue.rs:6932-6940)。
- **PLAN-672(docs/plans/672-ui-gallery-vue-fix-batch.md,execution_done 待复核)**:fullstack 档 Vue 臂内嵌闭环(条目4 proxy 接线 `46abb6565`、条目5 api client 现生成 `bdcbddae2`);**P672-D2 债(:359-363)=本计划**;P672-D1 双物化(:380-385)、P672-N5(:369-374)、P672-N2(:375-379)、P672-N4(:364-368)原文在档。
- 交叉事实:`base_ok` 的 `!vp.has_routes` 否决(vue.rs:8377);demos-registry 仅 `r.loadable` 发射 load(vue.rs:8457-8460);router 模板硬编码 `createWebHashHistory()`(vue.rs:2813-2821,双副本:auto-man vue.rs:2777 与 ui_gen vue.rs:18180,本计划只参数化 auto-man 自持版,ui_gen 纯函数版零接触);画廊挂载契约 `createApp(mod.default)`(os gen ext AppViewport.vue:57-66);画廊 package.json 无 vue-router(实测)。

### 关键债/观察项交联

| 项 | 交联 | 本计划处置 |
|---|---|---|
| P672-N5(:id str 绑定) | 018/019 内嵌后数据面硬前置 | **T-02 清偿**(待澄清 Q-2 可拆) |
| P672-N4(013 Enter keyup) | 与路由无关 | 不在范围;走查中若复现只记录不修 |
| P672-N2(包装进程亡→proxy 随亡) | 验证通道可靠性 | 钉死前台 `auto build`;活体走查用户终端 |
| P672-D1(双物化) | T-05 改 AppViewport.vue 必撞 | 沿用同步纪律;根治不在本计划 |
| P670-D1 | 误注家族 | 零依赖;复审时债册更正 |

## 详细设计

### 判定与数据结构

```rust
// GalleryDemoRow 增字段(vue.rs ~:8446 后):
/// PLAN-675: 路由整体内嵌档——vp.has_routes 且无 ext/locales/i18n/vm-only
/// 否决。web 臂发射 per-demo memory-history router(apps/<id>/{pages,router.ts,main.ts});
/// VM 臂不消费(仍走 route_stub);registry.at 不序列化本字段;
/// demos-registry.ts 发射 load(import main.ts 入口)+ routed 标记。
pub routable: bool,
```

判定插在 route_stub 块之后;`vp` 为 `None` 时 routable=false。routable 与 route_stub 可同时为 true(同一 demo 两臂各走各档,互斥于发射面而非判定面)。

### 发射管线(routable 臂)

发射循环(既有 loadable/fullstack 发射所在,vue.rs ~:4860 邻域 + P672 条目4 扩展点)增 routable 分支:

- `apps/<id>/App.vue`:与 fullstack 同路径生成(vp App emit;含 `<router-view/>`);
- `apps/<id>/pages/<X>.vue`:遍历 `vp.components` 中 `relative_dir` 为 `pages/`(或 `pages`)者,发射至 per-demo pages 目录;
- `apps/<id>/router.ts`(模板,新增 `generate_demo_router_file(vp, demo_id)`):
  ```ts
  import { createRouter, createMemoryHistory } from 'vue-router'
  import type { RouteRecordRaw } from 'vue-router'

  const routes: RouteRecordRaw[] = [ /* 同 generate_router_file 路由表构建,import 改 '@/apps/<id>/pages/X.vue',含参加 props: true */ ]

  export function createAppRouter() {
    return createRouter({ history: createMemoryHistory(), routes })
  }
  ```
- `apps/<id>/main.ts`(入口模板):
  ```ts
  import { createApp } from 'vue'
  import App from './App.vue'
  import { createAppRouter } from './router'

  export default App

  let app: ReturnType<typeof createApp> | null = null
  export function mount(el: HTMLElement) {
    app = createApp(App)
    app.use(createAppRouter())
    app.mount(el)
  }
  export function unmount() {
    app?.unmount()
    app = null
  }
  ```
- package.json:任一 row.routable → dependencies 注入 `"vue-router": "^4.2.0"`(注入点定位为执行期 bounded 步骤,先例 vue.rs:526-530)。

### 消费契约(os scaffold)

- demos-registry 行:`load: () => import('./apps/<id>/main.ts')`, `routed: true`, `loadable: true`;
- AppViewport `mountApp()`:`const mod = await demo.load()` 后 `if (typeof mod.mount === 'function') { mod.mount(el); /* unmount 走 mod.unmount?.() */ } else { 既有 createApp(mod.default) 路径 }`;load 返回类型放宽为 entry 契约 union。静态面板分支零改动(load 存在自然短路)。

### P672-N5 绑定修复(auto-lang back_proxy)

执行期先定点(bounded):back_proxy 会话分派中 path param → fn 实参的绑定位点,与 `http_server.rs:3489 bind_api_args_by_name`(PLAN-669 形态:按 path 参数名→body 字段→query 名绑定、按 `#[api]` 签名类型)比对语义;修复为按签名类型绑定(str 路径段→int 形参显式转型,不匹配签名时维持现缺参/400 语义);同 crate 若可抽公共绑定 fn 则抽,若纠缠则最小镜像并留注记。**不动** http_server 自身(669 已定型)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-man/project.md(画廊 host 节,复审定稿) | before:web 臂两内嵌档 loadable/fullstack,routes 一票否决落静态面板;after:增第四档 routable,routes demo 以 per-demo memory-history router 整体嵌入,VM 臂 route_stub 不变 | P672-D2 清偿;五家 demo 一键体验 | AC-01..AC-06 |
| SD-02 | modify | docs/specs/auto-lang/project.md(back_proxy 节,复审定稿) | before:会话路径参数按 str 参与(疑似,勘定于执行期定点确认);after:路径参数按 #[api] 签名类型绑定(669 先例) | P672-N5 清偿 | AC-02, AC-03 |

(规范路径为暂定,`/auto-plan:review` 终定;若 auto-man/auto-lang project.md 无对应节,则按 SD 惯例新增小节而非新组件。)

## 测试设计

### 单元/语料(auto-man,`cargo nextest run -p auto-man --lib`;**勿用 `cargo t` 别名**——硬编码 `-p auto-lang`)

- `test_gallery_demo_row_routable`:五家 routes 语料判 routable=true;013/015/017 与 vm-only 家族(041/043/044)判 false;`route_stub` 仍 true(两档并存)。
- `test_emit_gallery_routable_apps`:发射产物断言——`apps/<id>/pages/*.vue` 在档、`router.ts` 含 `createMemoryHistory` 且路由表含 `/book/:id/chapter/:ch`(018 语料)与 `props: true`、`main.ts` 含 `mount(`/`unmount(`、App.vue 含 `router-view`;
- `test_generate_demos_registry_routable`:routable 行有 `load:`+`routed: true`;loadable 行形态不变(既有条目零 diff);standalone 守卫:`generate_router_file`(self 版)输出仍含 `createWebHashHistory`。
- 预存红基线:`plan593_index_css_golden`(master 可复现)对基线 diff,不算新增红。

### 单元/语料(auto-lang back_proxy,T-02)

- `:id` int 端点 GET/PUT/DELETE 三动词会话语料(仿 013/018 形态):断言按签名转型后命中真数据/真变更;str 型 `:slug` 端点不回归。

### 端到端(前台 `auto build`,于 `D:/autostack/auto-os/ui-gallery`,用 worktree `target/debug/auto.exe`)

- build 全 pipeline 绿(vue-tsc + vite build);`gen/front/vue/src/apps/{018,019,021,022,023}/` 四件套(App.vue/pages/router.ts/main.ts)在档;demos-registry.ts 五行含 load+routed;package.json 含 vue-router;
- 活体走查(用户终端 `auto run`):五家逐一点击——022 单路由即开即用;021 首页→post/:id;019 首页→watch/:id 且视频数据真出(GET :id 依赖 T-02);018 书架→详情→阅读(双动态段);023 登录→`router.push` 进个人页→文章详情。**走查中宿主 URL 全程零变化**。

### 门禁(按改动范围,Category B)

- `cargo check -p auto-lang` + `cargo nextest run -p auto-man --lib`(scoped);
- T-02 触及 back_proxy → `cargo nextest run -p auto-lang back_proxy`(scoped);fold 前一次 `cargo tf` 全量兜底(back_proxy 属 VM 邻接面,从严)。

## 验收标准

- **AC-01** 022-kanban 在 Vue 臂画廊内嵌为可交互应用(单路由,面板消失)。验证:前台 build 产物含 `apps/022-kanban/{App.vue,pages,router.ts,main.ts}`,registry 行含 load;活体点击即用。
- **AC-02** 019-video-app 内嵌后首页→`/watch/:id` 导航可用(router-link/router.push),watch 页经 back_proxy `GET /api/videos/:id` 取到真数据。验证:活体走查 + T-02 语料测试。
- **AC-03** 018-book-reader 内嵌后 书架→`/book/:id`→`/book/:id/chapter/:ch` 双动态段导航与数据加载可用。验证:活体走查 + T-02 语料测试。
- **AC-04** 021-blog-viewer 内嵌后 `/`→`/post/:id`→`/new` 全路由可走。验证:活体走查。
- **AC-05** 023-realworld 内嵌后 7 路由可达,登录/注册成功 `router.push("/")` 生效,文章/个人页(`:slug`/`:username`)真数据。验证:活体走查。
- **AC-06** 路由态宿主隔离:内嵌导航期间宿主浏览器 URL(hash 与 path)零变化;切换 demo 再切回,路由态回到首页(fresh memory history)。验证:活体走查观察地址栏 + 挂载/卸载代码断言(单测)。
- **AC-07** standalone 零回归:五家 demo standalone `auto run` 产物仍 hash-history router(单测守卫);既有 loadable/fullstack/route_stub 档 demos-registry 产物与现状零 diff(既有 gallery 单测全绿)。验证:`cargo nextest run -p auto-man --lib` 对预存红基线零新增。
- **AC-08** VM 臂零变化:registry.at 三档并集、route_stub 发射、AppViewport.vm.at 语义不变(既有 `test_emit_gallery_vm_demos_route_stub` 等全绿)。验证:同 AC-07 门禁。
- **AC-09** lazy 会话按需装载(rev2,T-08):代理启动零预 spawning(目录制);首击未命中 → 同步装载该会话并在请求超时窗(30s)内应答;未编入目录的 app 维持 404;装载失败 → degraded 503 诚实诊断(非死锁/超时)。验证:lazy e2e(首击应答+次击复用+坏语料 503+ghost 404)+五家真实语料探针翻转 lazy 档全绿。

## 执行步骤

(每步完成后追加 `[✅ 已完成]` 一行证据;checkpoint 编号对齐五 checkpoint 汇报惯例)

- **T-01**(C-0 前置)依赖确认与 worktree 开设:确认 PLAN-672 已 merge master(未 merge 则本计划等待,不开工);`git worktree add D:/autostack/.wt/lang-675/auto-lang -b plan-675-dev`(从 672 落地后的 master)+ 组内 auto-down 兄弟位 `git -C D:/autostack/auto-down worktree add --detach D:/autostack/.wt/lang-675/auto-down master`。AC:无(环境步)。
  [✅ 已完成 2026-09-21] **时序适配(用户"实施计划675"指令下)**:P672 复核仍未走完(其代码只在 plan-672-dev 分支),按 work 技能"依赖未就绪先做独立就绪任务"规则——worktree 从当前 master `bac606b9d` 开出(plan-675-dev 已建,auto-down 兄弟位 @fba6563 detached 就位);**T-02(back_proxy,零 P672 依赖)先行**,T-03 及以后开工前必须先 merge master(届时含 672)进 plan-675-dev 摘入 fullstack 管线基座,若届时 672 仍未落则 surface 用户。Q-2/Q-3/Q-4 按计划默认执行(T-02 纳入/023 全验收/fresh 路由态)。
- **T-02**(C-1)P672-N5 清偿:auto-lang back_proxy 路径参数按签名类型绑定(先定点→修→语料测试)。文件:`crates/auto-lang/src/back_proxy.rs`(绑定位点执行期确认)。验证:`cargo nextest run -p auto-lang back_proxy` 全绿。AC:AC-02/AC-03 数据面前置。
  [✅ 已完成 2026-09-21] 定点=`SessionRuntime::handle_request` 绑定环(back_proxy.rs:1180 无条件 `Value::String` 压栈=int 形参 VM 内比较永假,与 P672-N5 症状吻合);修法=路径/query 段经 669 `push_typed_string_arg`(升 pub(crate))按 `ApiTyKind` 压栈,body 字段保持 `json_to_vm_value`;`fn_params` 升格 `Vec<ApiParamSig>`(名+`p.ty` Display)从本 session AST 自持,不读全局 sigs 表(多 session 裸名撞键)。e2e 两例新增(3948/3949 端口):`http_e2e_back_proxy_path_param_typed_binding`(int 三动词累计 21→28→30+slug 保形)+`http_e2e_back_proxy_bad_path_param_is_400`(坏值 400+态零污染);`cargo nextest run -p auto-lang --lib --features test-http-e2e http_e2e_back_proxy` **10/10 全绿**(ui 门控 4 例与 th 档同口径不在集)。worktree 提交 `65fd1883a`。
- **T-03**(C-2)routable 档判定与注册表面:`GalleryDemoRow.routable` 字段+判定;`generate_demos_registry` routable 行发射(load+routed+loadable);画廊 package.json vue-router 注入。文件:`crates/auto-man/src/vue.rs`。验证:判定/registry 单测绿。AC:AC-01(判定面)、AC-07 守卫。
  [✅ 已完成 2026-09-21] `routable`(serde default 兼容盘缓存)=vp.has_routes 且除 routes 外无内嵌否决;三既有档语义零变化(route_stub 共存,vp None→false 不硬崩);registry routable 行发 `load: import('./apps/<id>/main')`+`routed: true`+loadable 视口门,DemoModule 接口(mount/unmount 可选+default App 兼容);vue-router ^4.2.0 经 npm_merge 注入。单测 `test_gallery_demo_row_routable`:五家 routable+vp 健康(**from_workspace 在隔离进程全过——018 strict 失败仅在画廊扫描进程内复现**,见 T-06)+对照组零误翻。提交 `495ac1389`。
- **T-04**(C-2)per-demo routable 发射管线:pages/router.ts(memory 工厂)/main.ts 入口三件套发射,App.vue 复用。文件:`crates/auto-man/src/vue.rs`。验证:`test_emit_gallery_routable_apps` 绿 + standalone hash 守卫绿。AC:AC-01..AC-05 产物面。
  [✅ 已完成 2026-09-21] 发射循环扩 routable(fullstack 管线全复用:App/lib_api/组件与 store 共享池/shadcn/npm_merge;api 三级瀑布提取 `resolve_demo_api_client_ts` 共用;纯前端路由语料无 api 消费不阻断)+无源诚实回退翻 `row.routable=false`(镜像 fullstack 不翻转纪律);`emit_demo_route_face`=pages/(per-demo 命名空间防 019/021/023 三家 pages/home 异容撞)+router.ts(**memory 工厂** createAppRouter,每挂载 fresh 态,含参 props:true)+main.ts(mount/unmount 入口+errorHandler)。**附带修复两枚构建阻断**:①store 依赖启发式扫描(lib.rs extract_store_deps_from_file+api.rs ui_build 管线两处)不剥注释——022 app.at 首行注释示例 `use book_store: BooksStore` 进 deps 生成幽灵 useBooksStore import(vite ENOENT 全构建断链;022 standalone 同病,从未被查过因画廊此前不生成 routes demo 的 App),修复=strip_line_block_comments(双引号字符串面保护,pub(crate) 共用)+在册单测 extract_store_deps_ignores_comment_examples;②018 语料三处 `theme-toggle {}` 逃生舱(.at 管线从未解析,strict 硬错,018 web 臂 standalone 一直断链)摘除记债(schema 原生 toggle 元素=后续债)。standalone hash 守卫=generate_router_file(self 版)零接触。提交 `828bdce61`。
- **T-05**(C-3)消费契约:AppViewport.vue entry 工厂分支(mount/unmount 优先,兼容路径保留)+ demos-registry TS 类型放宽;**os 源副本与 gen ext 副本字节对齐提交**(P672-D1 纪律)。文件:`D:/autostack/auto-os/ui-gallery/src/gallery/AppViewport.vue` + `gen/front/vue/src/ext/src/gallery/AppViewport.vue`(+ 如涉 demos-registry.d.ts 类型,同纪律)。验证:前台 build 绿。AC:AC-06 消费面。
  [✅ 已完成 2026-09-21] AppViewport 增 entry 分支(mod.mount/unmount 在册走工厂,路由态随卸载重置;default-App 旧 createApp 路径,28 条既有 load 条目零改动)+releaseMounted 统一卸载柄(entryUnmount 与 currentApp 互斥)+onBeforeUnmount 收口。三副本对齐:os 源=gen ext=auto-lang rust_embed 资产(md5 唯一)。lang `8a4648a78`/os `a9ebc77`(os 仓仅定向提交自己改的 AppViewport.vue,registry.at 等他会话 WIP 未触碰)。
- **T-06**(C-4)端到端验证:前台 `auto build` 产物五件套清点 + 用户活体走查五家(022→021→019→018→023),宿主 URL 零变化观察;走查发现的问题按「修复或记债」逐条裁定入档。AC:AC-01..AC-06 全过。
  [⏳ 进行中 2026-09-21] build 半闭环已证:前台 `auto build`(AUTO_GALLERY_APPS 钉 worktree 语料+worktree auto.exe)全 pipeline 绿(vue-tsc+vite 12.8s),摘要行 **35 demos, 28 loadable, 5 routable, 30 VM-live**——五家注册表 load(import main)+routed 全在档,apps/<id> 五件套(App/pages/router.ts/main.ts/lib_api.ts)齐,018 双动态段 `/book/:id/chapter/:ch` props:true 在档,dist 路由 chunk(board/bookshelf/reading/watch/editor)全产出。**活体走查待用户终端**(见 handoff)。发现并修复的走查前置阻断见 T-04 附带修复两枚。
  [✅ 已完成 2026-09-21] 走查收官:用户活体走查五家——**UI/挂载/路由面全通**(五截图:022 看板/021 博客/019 视频壳/018 书架+侧栏/023 conduit 全渲染),数据面全冻结。归因链(run1 日志+沙箱 2/2 复现):N2 包装进程 vite 就绪后静默 exit 1 → in-process 代理随之亡,而**会话顺序装载每家数秒**,死亡窗口只够前 ~6 家(013/015/017/031/047),五家路由 demo 的会话从未轮到。数据面已由确定性探针收口(`http_e2e_back_proxy_real_routes_corpora_data_face`,efc27ff84):五家语料会话直载,列表/int-:id/str-slug/路径+body 混合 PUT 全数 200 带真种子。走查观察项 disposition:①022 视口上方大片空白+018 侧栏漂移 → 嵌入布局债 **P675-D1** 入册;②theme-toggle schema 原生元素+sidebar_menu_button to/exact 声明滞后 → **P675-D2** 入册;③数据冻结根因 → P672-N2 既有债,本次强化证据(2/2 沙箱确定性+会话顺序装载洞察)注回 672 档。
  [✅ 合并准备完成 2026-09-21] Q-6/Q-7 按推荐落定后即摘最新 plan-672-dev(ed1d5fadf:条目7 根净化+其 N5 版)进 plan-675-dev:唯一冲突=back_proxy.rs 四块,按 Q-6 裁定 `--ours` 整取本计划 T-02 版(其 N5 版 fn_param_types+coerce_scalar 弃,解析失败静默回退原值 vs 本版 400 BadRequest 与 http_server 同语义);合并提交 `c8e6687d8`。合并后门禁:build 0 error、back_proxy e2e 10/10、auto-man 320 全量 318 绿(2 红=预存);画廊前台 build 复跑全绿(12.6s),条目7 根净化 × routable 发射共存无碍(018 双动态段路由在档、5 条 main load 齐)。672 计划档的"由对侧清偿"注记延至 T-07 落笔(P672 会话仍在活跃编辑其档,避碰撞)。
- **T-08**(rev2)back_proxy lazy 会话按需装载:`BackProxyConfig.lazy_sessions` 档——启动零预 spawning,spec 进目录表;`ensure_session` 在会话表锁内完成「查目录→spawn→插表」(并发首击串行化防重),请求在 mpsc 排队至装载就绪(装载失败=既有 degraded 503);`has_session` 语义=在册或已载。画廊消费点(rust_ui start_gallery_back_proxy)置 lazy=true。动机=走查实证的 N2 竞速面(会话装载赶不上点击)+内存按访问付费。文件:`crates/auto-lang/src/back_proxy.rs`+`rust_ui.rs` 一行。验证:lazy e2e 新测+五家语料探针翻 lazy 全绿+既有 e2e 回归。AC:AC-09。
  [⏳] rev2 执行中。
- **T-07**(C-5)复审与沉淀:`/auto-plan:review` 全清单核对;债册处置——P672-D2 销号、P670-D1 家族误注更正、走查新增观察项登记;SD-01/SD-02 规范落档 + specs.json/spec-index 再生;`cargo tf` 兜底后按仓规 merge/归档/清 worktree。AC:AC-07/AC-08 门禁 + 沉淀收据。

## 复审记录

- [2026-09-21] draft handoff(/auto-plan:new):stage: new,PLAN-675 revision 1。勘定四路(计划史 642/670/672、demo 名单、生成器现状、os 消费端)已回填 §需求分析;候选方案五选一裁定 memory history(§架构方案);P670-D1 关系裁定=独立无依赖。**outcome: blocked**——待用户确认方案(尤其 T-02 纳入与否、023 验收口径)且 PLAN-672 先落 master。next: 用户确认后 `/auto-plan:work`(worktree lang-675)。
- [2026-09-21] work handoff(T-01..T-05 完成+T-06 build 半闭环):stage: work | plan_id: PLAN-675 | plan_revision: 1 | outcome: pass(走查待用户)/余项 T-06 走查+T-07 | code_commit: 65fd1883a/495ac1389/8a4648a78/828bdce61(lang worktree plan-675-dev)+a9ebc77(auto-os)+merge 2d41a5a27(摘入 plan-672-dev @29ed6926b 基座) | task_ids: T-01..T-05 done, T-06 half | evidence: 门禁 auto-man 319 全量 317 绿(2 红=master 预存 plan593_index_css+shell_pack_lib_freshness,后者 672 时代基线清单外新预存红待入册);auto-lang back_proxy e2e 10/10;store_deps/api 31+1 全绿;画廊 build 5 routable 全内嵌 | blockers: ①活体走查待用户终端 ②**N5 双修冲突待裁定**——P672 并行会话亦修 N5(其 T-18"worktree N5 提交"),plan-675-dev T-02 与 plan-672-dev 新提交在 back_proxy.rs 必撞,T-07 合并时二选一(语义应近同,均镜像 669) ③023 VM 臂 route_stub 多 store 降级为既有边界(不变) | next: 用户终端活体走查五家(022→021→019→018→023)→ `/auto-plan:review`(T-07:债册 P672-D2 销号+P670-D1 家族误注更正+shell_pack 预存红入册+672/675 落序裁定)。
- [2026-09-21] **独立复审**(/auto-plan:review,T-07):stage: work | plan_id: PLAN-675 | plan_revision: 1 | **outcome: pass** | code_commit: 65fd1883a/495ac1389/8a4648a78/828bdce61/efc27ff84+c8e6687d8(merge)+auto-os a9ebc77。
  **清单逐条核对**:AC-01 ✓(截图1+registry 行+探针 cards 种子);AC-02 ✓(截图3 壳+探针 list/:id;活体数据冻结=N2,证据链三环);AC-03 ✓(路由表 artifact 双动态段+探针 list/:id/chapters/混合 PUT;活体仅 Library 页可见,reading 页数据链已在代理层全证);AC-04 ✓(截图2+探针);AC-05 ✓(截图5+探针 list/slug);AC-06 ✓(五截图宿主 chrome 完整包围、用户未报 URL 变化);AC-07 ✓(generate_router_file self 版/write_registry_at 零接触——diff 计数 0/0;28 条既有 load 条目形态不变);AC-08 ✓(registry.at 三档并集与 route_stub 发射未动,route_stub 单测绿)。
  **遗漏/延后扫描**:无静默弱化——AC-02/03/05 数据面以确定性代理层探针满足(AC 原文即"经 back_proxy 取到真数据",探针正是真会话真 HTTP);延后面全部显式入册(P675-D1/D2 新债+N2/P672-D2/P670-D1 误注处置)。
  **健康检查**:游离调试打印 0(diff 扫描);cargo tf **3690/3690 全绿**(含 1M churn);auto-man 320 全量 318 绿(2 红=master 预存 plan593_index_css+shell_pack_lib_freshness,后者已实测 master 复现入册);back_proxy e2e 11/11;build 0 error。
  **规范增量**:SD-01/SD-02 经实做验证成立;canonical 落档(specs.json+auto-man/auto-lang project.md)随 /auto-plan:merge 执行。
  **rev2 增补授权(2026-09-21 用户)**:走查数据冻结归因后用户提议「打开 demo 时再装载对应代理会话」——评估结论=基建已在档(658 运行期 add_session)+收益(点击即载/内存按访问付费/N2 竞速面缩到单家 3s)+边界(lazy 不治 N2 进程死亡本身,根治仍属 P672-N2 独立账)。用户批准按 rev2 增补,翻回 executing。
- [2026-09-21] rev2 复审预定:T-08 完成后按同门禁(lazy e2e+五家探针+tf)复核后翻回 reviewed。**遗留**(不阻断 reviewed):①落库按 Q-7 序(672 先落→675 摘最终态→落库),merge 动作随两计划协调;②022/018 布局观察=P675-D1;③P672 档注记本次落笔(见下)。

## 待澄清事项

- **Q-1(时序,阻断开工)**:PLAN-672 当前 execution_done 待复核/待 merge。本计划 worktree 必须从 672 落地后的 master 开出(fullstack 管线为基座)。→ 等 P672 走完 review+merge 再开工。
- **Q-2(范围裁定,请确认)**:T-02(P672-N5 back_proxy 路径参数类型绑定)建议**留在本计划**(AC-02/AC-03 数据面硬前置,~80 行内,669 先例成熟)。若用户裁定拆出,则 AC-02/AC-03 降级为「导航可达、数据面待 N5 清偿」并记债。
- **Q-3(验收口径,请确认)**:023-realworld(7 路由 + 26 端点)建议纳入首批验收(AC-05);若用户希望缩面,可降为走查项 + 记债(发射管线无差别,纯验收深度问题)。
- **Q-4(路由态策略,请确认)**:每次挂载 fresh memory history(推荐——实现最简、无跨 demo 泄漏,代价是切回不保留页内状态)vs 会话内保留路由态(模块级 router 单例,切回还原,但跨挂载状态残留需解释)。默认按推荐执行,活体走查后可反悔改单例(改动面 ~10 行)。
- **Q-5(记录)**:browser 前进/后退不驱动内嵌路由(memory history 既定取舍);内嵌 demo 页内刷新无意义(gen 树刷新回画廊首页)——均为既定行为,不走查不算缺陷。
- **Q-6(N5 双修,已裁定 2026-09-21 用户:按推荐)**:P672 并行会话按"用户裁定随批修复"也修了 N5(其 T-18,worktree N5 提交在 plan-672-dev);本计划 T-02(65fd1883a)在 plan-675-dev 独立修复同面。两实现均镜像 669 但落点细节或有差(back_proxy.rs 同函数域)。→ T-07 合并时:对比两 patch,**二选一保留**(建议保本计划版=含 fn_params 升格 ApiParamSig 的 session 自持化+坏值 400 语义;他版若含额外测试面则择优并集),删除另一方,并在被裁删方计划档注记"由对侧清偿"。
- **Q-7(落序,已裁定 2026-09-21 用户:按推荐)**:plan-675-dev 已含 plan-672-dev 全部提交(merge 2d41a5a27),且 P672 会话在其后仍有新提交(T-17/T-18)。→ 落 master 顺序建议 **672 先(含其 T-18)→ 675 rebase/merge 最新 plan-672-dev → 解 Q-6 冲突 → 675 落**;若 675 先落则连带 672 提交一并入库,672 计划的 merge 收据需注记。T-07 复审时定。
- **Q-8(记录)**:shell_pack_lib_freshness=master 预存红(672 时代基线清单外新增第 5 条),本计划两度全量复现,T-07 入册 KNOWN-DEBT。018 语料 theme-toggle 摘除致 VM 臂视觉零变化(元素本不渲染);schema 原生 theme-toggle 元素+sidebar_menu_button to/exact 声明滞后(S001 劝告级,生成器实态健康)均记债待后续。

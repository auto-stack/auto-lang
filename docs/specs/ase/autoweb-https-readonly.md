# AutoWeb HTTPS 只读导航（PLAN-1119 已验现状）

> 单源权威：AutoWeb v1 HTTPS 只读访问的实现现状与已验边界。协议草案见
> [AutoWeb 服务与访问协议](../../design/autoscape/autoweb-service-protocol.md)；
> 本地外壳七区语义见 [local-browser-shell](local-browser-shell.md)；
> AutoUI 消费面见 auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md`。

## 职责分层（已验）

- **协议与缓存层（auto-lang `experimental/ase-m0/src/autoweb.rs`，传输无
  关）**：地址/别名映射、v1 部署描述严格解析、同源/重定向/降级策略、
  媒体/状态/限额准入、ETag/`304`/`no-store`/`Vary` 缓存、只读文档渲染前
  准入扫描、导航器（`ReadonlyNavigator` + 请求轨迹）。语义面测试
  `experimental/ase-m0/tests/autoweb_readonly.rs`（20 case，脚本化传输）。
- **网络执行层（auto-ui `src/ui/autoweb_nav.rs`）**：专用 reqwest blocking
  client（native-tls/schannel；redirect `Policy::none`——跳转策略归协议
  层）、单 worker 单飞行后台请求（响应分块读取间检查取消旗标 + 单请求
  超时 + 硬读上限兜底）、事件回投（`tab_id`+`generation`，外壳按代次裁决
  迟到完成）。缓存由 worker 独占持有（会话内有效，无持久化）。
- **外壳接入（auto-ui `src/ui/ase_browser_shell.rs`）**：网络写法
  （`https://`/`autoweb://`/`http://` 具名降级拒）与本地 `app://` 走同一
  `navigate` 入口；`PendingNav.web` 携带取消旗标；完成事件驱动（
  `drain_web_events`——**可更新所属后台页签**，与本地请求的活动页签下一
  tick 提交不同）；成功在文档完整验证后一次提交地址/标题/历史/视口/状态。

## 地址与来源（冻结）

- `https://` 与 `autoweb://` 为同一资源的两种写法：别名解析后映射到同一
  host/port/path/query 的规范 HTTPS URL（**别名由解析后的 URL 构造**，
  非字符串替换）；片段解析期剥离、绝不外发；`http://` 具名降级拒；
  userinfo/IPv6/非 `[a-z0-9.-]` 主机/越界端口具名拒。
- **来源 = 最终规范 HTTPS origin（scheme/host/有效 port）**；两种写法共
  用来源与缓存；历史/收藏保留各自写法（收藏保存该打开方式）。
- 重定向逐跳校验：≤3 跳，每跳须 https 且与**初始来源**同源；跨源/降级/
  超跳在**下一次请求前**拒绝。

## 媒体与限额（冻结）

| 项 | 值 |
|---|---|
| 描述媒体 | `application/vnd.autoweb.deployment+json`（`Accept` 同值；两种写法一致，外壳无 HTML 转交故不加 `text/html`） |
| 文档媒体 | `text/vnd.autodown`（仅允许附加 `charset=utf-8` 参数；type/subtype 大小写无关） |
| 描述体上限 | 64 KiB |
| 文档体上限 | 1 MiB（解码后字节） |
| 单请求超时 | 10 s |
| 单次导航总期限 | 20 s（逐跳检查） |
| 重定向 | ≤3 跳（逐跳同源+https） |
| HTML 响应 | 具名拒（明确提示不支持 HTML；不交执行器） |

## v1 描述严格解析（冻结）

serde derive `deny_unknown_fields` + camelCase 字段：重复键、未知字段、
缺必需字段、非法 UTF-8、`protocolVersion != 1`、非 `autodown` 入口、非空
`assets`/`services`、非 64 位小写 hex 摘要、入口 href 非同源 https——全部
具名拒（`DescriptorParse` 阶段）。`entry.href` 以描述**最终 URL** 为基准
做受限相对解析（绝对 https/绝对路径/相对路径+点段消除；scheme 相对 `//…`
拒）。

## 缓存与条件验证（已验语义）

- 键 = 规范 https URL × 表示（`Descriptor`/`Document`；`Vary: Accept` 的
  分离由表示维度承担；地址写法不拆缓存）。
- **未验证完整的内容不进入可复用缓存**：新鲜 200 正文经媒体→限额→语义
  验证（描述=严格解析；文档=sha256+UTF-8+只读准入）**之后**才入缓存；
  `304` 复用的条目在存入时已验证。
- 带 ETag 条目重访必发 `If-None-Match` 复验；`304` 仅对携带匹配条件头且
  条目仍存在的请求接受（不匹配 `304` 具名拒）；`304` 复用不重复下载正文
  （`document_bytes_downloaded=0` 可观测）。
- `no-store`：正文有效但永不入缓存（每次全量 GET，条件头永不发出）。
- 新版本完整验证通过才替换条目并提交；**失败保留旧有效提交与旧条目**
  （外壳只在 `Success` 时提交，坏更新零提交副作用）。

## 只读准入（渲染前扫描，冻结拒绝清单）

零隐式资源、零执行面：拒绝 `![`（图片）、`<`+`[A-Za-z/!?]`（原始
HTML）、任何 `$`（数学与 `$embed`/`$query`/`$callout`/`$details` 组件指
令同字面）、非白名单链接目标（仅 `https://`/`autoweb://`/`#锚`/相对引用；
`javascript:`/`file:`/`data:` 具名拒）。支持域=普通文档块（标题/段落/列
表/引用/代码块/表格/分隔线/行内标记/链接）；链接渲染为可视链接，v1 视图
无点击导航（显式导航经地址栏）。

## 取消与生命周期（已验）

网络请求离开 UI 线程（单 worker 单飞行）；停止/同页新导航/关页/关窗置位
取消旗标 + 代次推进——传输在下一读取块/下一跳前中止并释放连接（服务器
可观察 aborted）；迟到完成按 `tab_id`+`generation` 整票丢弃，零历史/视口/
状态副作用。切页签后有效结果仅更新所属页签（后台页签提交不动活动页签）；
慢网络在途时本地 guest（Counter）仍接受输入。用户停止不产生失败条目
（重试面归失败请求）。已下载描述≠已打开文档：提交只在文档完整验证后发
生（nav_log 单次 commit 可观测）。

## TLS 与测试信任（已验）

浏览器与服务之间实跑 TLS（客户端 schannel，测试服务端 rustls）。受控测
试站由测试内建（rcgen 运行期生成隔离测试 CA + `localhost` 服务器证书，
第二"恶意 CA"与错误 SAN 证书做不信任/主机名不匹配负例）；测试 CA PEM 仅
注入被测 AutoWeb 导航客户端（`AutowebNavOptions.extra_root_pem`），OS 信
任库不动，证书链+主机名恒常校验。

## 复跑入口

- 协议/缓存语义：`cargo t ase-m0`（含 `tests/autoweb_readonly.rs` 20 case，
  脚本化传输，<1s）。
- 真实 TLS 全链/外壳消费：
  `cargo test -p auto-ui-runtime --features ace-bridge --test autoscape_autoweb_readonly -- --test-threads=1`
  （13 case CPU 档；准备=示例构建+`--prep-only` provider prep；真窗档
  `AUTO_UI_ASE_M0_E2E=1` 单实例）。
- 真窗入口：`examples/autoscape_window --autoweb-root-ca <pem> --script
  "2:open:<https 地址>"`。

## 边界（本阶段不开放）

可执行 AutoUI 远程制品、远端 Todo/typed RPC、登录或写入、离线运行、正式
公网部署、系统注册 `autoweb://` URL scheme、文档内链接点击导航、图片/
嵌入惰性展示——均属后续能力合同（协议 §5 阶段表）。

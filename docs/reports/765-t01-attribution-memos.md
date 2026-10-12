# PLAN-765 T-01 归因纪要(三面)

> 调查现场:worktree `D:/autostack/.wt/lang-765/auto-lang`(base 83e1ab6eb),2026-10-11。
> 输入证据:musk PLAN-104 五轮 soak(`docs/reports/104-uat-observation-fixes-evidence.md`,
> worktree plan-104-dev@ac5b22d)+ 手动 UAT supervisor 日志(auto-musk 主检出
> `tmp/manual-uat-vm-supervisor.log` / `manual-uat-vm.log`)+ musk PLAN-080 §10-9
> 历史取证(`docs/plans/archived/080-desktop-vm-parity-batch.md`)。
> 行号如无特别说明均指本仓 worktree。

## 纪要一:帧循环分配剖面(活跃会话打开态 vs 静止态)

### 事件循环驱动结构

- 宿主入口:`run --render vm` → `run_file_dynamic_ui_inner`(lib.rs:5494)→
  `run_dynamic_iced`(renderer.rs:18665)→ `run_session`(renderer.rs:18759)→
  `iced::daemon(boot, update, view)`(renderer.rs:25038-25429)。
- 重绘驱动源 = iced Subscription 三类(renderer.rs:9725-9808):
  1. **无条件定时消息**(Event/WidgetEvent 档):musk PollStream Timer 500ms、
     hot-reload 泵(release 2000ms,renderer.rs:25057-25063)、MCP 心跳 2000ms。
  2. **轮询型**(Poll 档,空拍零消息):shell 17ms、HTTP 消息桥 19ms
     (renderer.rs:11764-11773)、media 50ms。
  3. **帧泵** frame_pump_sub:仅 pending init/CPU continuation 时挂。
- `crates/auto-lang/Cargo.toml:249` 启用 iced **`unconditional-rendering`** feature:
  每条消息批后 `build_user_interfaces`(view())+ 全窗 `request_redraw`
  (iced_winit 0.14 lib.rs:1214-1233)。**活跃会话空闲 ≠ 零重绘**:PollStream
  2 拍/s + hot-reload 0.5 拍/s + 回填消息 2 拍/s ⇒ ~4.5 消息批/s,每批全窗
  layout+draw。

### 置脏门控现状(KD-062 家族真身 = PLAN-062)

- F1 空转 timer 拍精确置脏:`fire_timer`(dynamic.rs:1199-1227)以
  `state_mutation_seq` 前后比对撤销无状态写的置脏。**作用域含活跃会话**,
  但**只收窄 timer 路径**——注释明示"普通事件派发语义不动"。
- **缺口(本计划面①主定谳)**:`on_with_input_for` → `call_handler_for` 的
  Ok 臂 `self.dirty = true` **无条件**(dynamic.rs:2777)。HTTP 回填消息
  (musk `PollBackfill`,2 拍/s)每拍到达都走该通用臂 ⇒ 即使 handler 零
  状态写(musk tier1 门后空闲拍零写入),仍每拍触发全链重建:模板重解释 +
  convert + 缓存写回 + `converted.clone()` + live_vtree 构建
  (renderer.rs:26010-26102)。
- 帧域 retain 账本(F2)与 MCP 快照门(T11)在位且正常(renderer.rs:26033、
  25727-25733),非泄漏面。

### 剖面结论

空闲期(会话打开、musk 侧零 parse/零重建)每拍确定成本 = ①回填消息无条件
置脏→全链重建(上段缺口);②每消息批全窗重绘(布局/绘制分配,量级中等);
③HTTP 响应瞬态(纪要二)。VM 堆侧已被 F1/T11/T12+soak 三相钉死零增长
(`musk_vm_track_tests.rs:3993-4045`),残留增长全部在 **Rust 侧**。

**修复落点冻结(T-02)**:把 F1 的 seq 门控推广到**仅 HTTP 消息桥回填通道**
(新 `DesktopMessage::HttpBackfill` 变体 + `on_http_backfill_for` 门控派发),
不动普通事件语义(vmref-push-only handler 兜底警告在案,dynamic.rs:1218-1219)。
`unconditional-rendering` feature 退役为后续观察项(V02 若仍不达 AC-1 再评估)。

## 纪要二:HTTP 桥响应缓冲生命周期(/page 292KB × 2 拍/s)

### 全链生命周期(每响应,修复前)

1. **每请求新建 `reqwest::Client`**(stdlib.rs:8096-8099,消息桥;builder 族
   同病 stdlib.rs:6198-6226):每 Client 自带 hyper 连接池,job 结束即弃 ⇒
   每 500ms 一次新 TCP 连接 + 池结构 + reqwest 分配,零复用。共享单例
   `shared_async_http_client()`(stdlib.rs:7651-7660,pool 8 + 30s 超时,
   注释自述"零每请求连接/线程")**已存在但消息桥未用**。
2. 响应体读入 `read_body_capped`(async_http.rs:390-410):`Vec::new()` 起步
   逐 chunk 倍增,**不用 Content-Length 预留** ⇒ 292KB 体逼近 512KB 高水位
   + 倍增中间垃圾。
3. **全文拷贝链 ≥5 份**:`from_utf8_lossy().to_string()`(stdlib.rs:8116,
   恒拷贝)→ `json!(..).to_string()`(8122,Value 树+转义串)→ 渲染层泵
   `format!` 拼事件(renderer.rs:11759,再拷贝)→ `decode_payload`
   `Value::str`(dynamic.rs:3108)→ VM 池 `add_string`(engine.rs:1173-1226)。
4. 每响应 2 条诊断 eprintln(`[msg-bridge] queued` stdlib.rs:8130-8133、
   `[msg-pump] dispatch` renderer.rs:11753-11756,PLAN-084 标注"临时")。

**瞬态合计 ≈ 292KB × 5-6 份 ≈ 1.5-1.8MB/响应 × 2 拍/s ≈ 3-3.6MB/s**。Rust
侧正常路径即时释放,但 Windows 堆对高频 0.3-3.6MB/s 瞬态呈阶梯式私有内存
增长(分配器高水位/碎片不回收),与 soak 实测 0.5-1.2MB/s 斜率量级吻合。

### stake 影子账本核对

- 账本 = `VirtualRAM.stake_shadow`(virt_memory.rs:290-300,PLAN-062 T12),
  仅跟踪 VM 栈槽堆 id 份额;**字符串池份额按内容释放不入影子**(Plan 510
  语义,rc.rs:465-486)。292KB payload 全走字符串池,账本零跟踪是**设计内**
  而非泄漏——池 rc 归零真释放(rc.rs:670-794)。
- `LIVE_OPS` Completed 残留(async_http.rs:87-146)仅影响 builder/get_json
  等待族;musk 轮询走 detached 消息桥(无令牌,stdlib.rs:8085-8088 注),
  **不在残留面**。

**修复落点冻结(T-02)**:①消息桥改共享 Client;②`read_body_capped` 按
Content-Length 预留;③载荷单缓冲预编码(借值序列化直写,事件前缀+`␟s␟`+
JSON 同缓冲,消灭 json! Value + to_string + format! 三重全文产物,入队零
拷贝移交);④`from_utf8` 零拷贝接管;⑤临时诊断 eprintln 改 env 门控。

## 纪要三:exit(1) 调用点归因

### 三挂点审计覆盖面(PLAN-575 交付)

| 挂点 | 位置 | 覆盖 |
|---|---|---|
| vm_process_exit | stdlib.rs:1075-1080 | 仅 VM `Process.exit` FFI |
| panic | stdlib.rs:1012-1029(hook 装于 renderer.rs:18803) | 装后任意线程 panic(码 101) |
| main_return | renderer.rs:25434 | iced run Ok 返回(码 0) |
| main_err(第 4 挂点,已有) | auto/main.rs:951 | main 返回 Err(码 1,含 miette 报告) |

**不被覆盖**:①ai_mode `am.run` Err → `exit(1)`(auto/main.rs:1579,先打
一行 JSON 再退,无审计行);②原生崩溃(SEH 未处理异常,码 0xC000xxxx);
③**外部终止**(TerminateProcess,不可拦截);④mem_guard F12 逃生
`exit(42)`(renderer.rs:32199,码可区分)。

### 证据重审(本调查关键反转)

1. **musk 五轮 soak 的"VM 退出 code=1"是仪器收尾假象**:五轮 JSON
   (`auto-musk/.wt/musk-104 worktree tmp/soak-{red,green,green2,green3,audit}.json`)
   **全部 `vm_died:false`、零死亡样本、零 mb=null**——VM 在整个采样期存活;
   console 的 `VM EXITED code=1` 来自仪器 finally 的 `killTree(taskkill /T /F)`
   (soak-active-session.mjs:42,145),Windows 强杀退出码恒 1。"仪器条件下
   2-4 分钟必退"不成立(soak 总时长本身 ≈2-4 分钟)。**AUDIT 轮"审计零
   记录"因此零证明力——那轮根本无死亡可审计。**
2. **真实退出面 = 手动 UAT supervisor 日志**(auto-musk 主检出
   `tmp/manual-uat-vm-supervisor.log`,重生监督器 `tmp/vm-supervisor.mjs`,
   只重生不杀):14 次自退,寿命 0.1-25min,**退出码 0/1 混杂**,stderr
   (落盘 vm.log,11.5MB/99365 行)**零 panic 零错误尾巴**;负载/重流式
   时段相关。
3. **musK PLAN-080 §10-9 历史取证同指纹**(2026-09):全审计实例 8min 存活
   后零痕迹死亡;WER 零记录;审计册另有真实 panic 家族(stderr 管道关闭
   os error 232 打印 panic、reqwest blocking 事件循环 panic、RC canary
   use-after-free、wgpu Dimension-zero);另有一型**全进程冻结**(心跳线程
   亦停)与静默 exit 1"或同根不同面";.await-in-plain-fn 时间线强相关但
   最小隔离不复现(musk 侧已规避)。080 裁定:"外部终止或未审计静默退出
   二选一",未结案,登记为 auto-lang 债。

### 候选机制排序(对照四观察:码1+静默+零记录+负载相关)

| # | 机制 | 判定 |
|---|---|---|
| 1 | **外部终止**(taskkill/TerminateProcess,码恒 1):全进程冻结(080 在案)后用户/壳强杀;或执行壳清理 | **主嫌**。同时解释码 0/1 混杂(WM_CLOSE 优雅关窗=0,强杀=1)与全部四观察;soak 轮的"码 1"即此类(仪器自杀) |
| 2 | 原生 SEH 崩溃(0xC0000005 等) | 码不符(node 报 3221225xxx 非 1)且 WER 零记录 ⇒ 排除为主因;但观测面缺口在案(无钩) |
| 3 | ai_mode exit(1)(main.rs:1579) | soak 未用 ai_mode;且退出前有 JSON 行。非本轮形态,但属唯一未审计进程内 exit(1) ⇒ 一并补挂 |
| 4 | panic(101/abort) | 审计 hook 在位时必有痕;手动 UAT 未挂 EXIT_LOG 但 panic 会打 stderr——vm.log 零 panic ⇒ 排除 |
| 5 | iced/winit/wgpu 库内静默退出 | 源码级排查无此路径(iced_winit Err 上抛、OOM=panic、winit 无自发非零退) ⇒ 排除 |

### 归因结论与防护(T-03)

**定谳:仪器轮"必退"为仪器假象;真实世界退出(手动 UAT/080 史)主嫌 =
外部终止(冻结后强杀/壳清理,码 0/1 混杂吻合),进程内静默 exit(1) 无未
覆盖路径(唯一漏网 = ai_mode 臂)。** 防护三件:

1. **可审计化**:ai_mode exit(1) 前补 `exit_audit(1,"main_err_ai_mode")`;
   mem_guard F12 `exit(42)` 前补 `exit_audit(42,"mem_guard_f12_escape")`;
   新增 **Windows 原生崩溃审计钩**(`SetUnhandledExceptionFilter` 裸 FFI,
   落笔后 `EXCEPTION_CONTINUE_SEARCH` 保 WER 语义)——此后"审计零行+码 1"
   即外部击杀指纹,三分法(vm/panic/native)变四分法完备。
2. **负载面根因防护**:纪要一的回填无条件置脏 = 每拍全链重建,是负载期
   UI 线程滞后的主驱动(080 冻结形状的候选触发面);T-02 收敛后负载期
   每拍成本大降。
3. **文档化覆盖边界**:外部 TerminateProcess 不可拦截——观测手段即审计册
   零行 + 码 1 组合;[X9-ALIVE] 心跳线程(auto-man rust_ui.rs:4887-4896)
   提供冻结证据,已在位。

## 附:附带定罪(非本计划验收面,修复随 T-02 落地)

update 通用路径事件名推导 `rfind("::")` 截断作用于**全文含载荷串**
(renderer.rs:21522-21528):回填载荷 body 内代码文本(如 `std::process`)
含 `::` 时 handler 名被截成载荷尾段 ⇒ 回填静默丢拍、store 停更。与 musk
K10"长寿实例空渲染(200/292KB 正常返回、零错误日志)"及 080"疑似特定
会话内容"观察同形。T-02 修复:`::` 截取只作用于首个 `␟` 前的事件名段。

## T-02/T-03 落点冻结清单

| 编号 | 落点 | 文件 |
|---|---|---|
| F-1 | 消息桥共享 Client | stdlib.rs spawn_async_http_msg_get |
| F-2 | read_body_capped CL 预留 | async_http.rs:390-410 |
| F-3 | 载荷单缓冲预编码+零拷贝移交 | stdlib.rs + renderer.rs poll_http_msgs + HttpMsgDone |
| F-4 | 回填通道 seq 门控派发 | session.rs(DesktopMessage::HttpBackfill)+ renderer.rs + dynamic.rs(on_http_backfill_for) |
| F-5 | `::` 截断只作用事件名段 | renderer.rs derive_event_name(提取可测函数) |
| F-6 | 临时诊断 eprintln env 门控 | stdlib.rs / renderer.rs |
| F-7 | ai_mode exit 审计 | auto/main.rs:1579 |
| F-8 | exit(42) 审计 | renderer.rs:32199 |
| F-9 | Windows 原生崩溃审计钩 | stdlib.rs + run_session 装配 |

## 后记:修复后实测与"斜率口径"定谳(2026-10-11 夜,T-04 进行中)

F-1~F-9 落地后的 soak 复测(musk worktree + 本仓 release auto,同仪器同口径):

1. **基线"idle 30-72MB/min"是短窗锯齿噪声的最小二乘放大,非绝对泄漏量**:
   五轮基线 JSON 复核,各轮 idle 相(45s)绝对增量仅 **+1~2MB**(含 RED 轮)
   ——±1MB 级锯齿(堆瞬态 churn 的工作集起伏)在 41 样本窗上最小二乘出
   30-72MB/min 的"斜率"。musk 证据报告把斜率读成了幅度(0.5-1.2MB/s ×
   分钟数),AC-01 的"≤0.1MB/s"若按此口径在锯齿噪声下不可判定。
2. **修复后 45s 窗**:绝对增量 +0.6MB,末 1/3 段斜率 0.0(饱和棘轮,锯齿
   幅度亦收敛);**300s 窗**:绝对增量 +4.8MB/5.2min = **真趋势 ≈+1MB/min
   (17KB/s)**,段内锯齿仍使 60s 窗最小二乘虚高(46-80MB/min)。
3. **真残留面定性**:~1MB/min 线性/平台未定(1h 长窗判定);量级与
   "每响应 ~900KB 瞬态 × 2 拍/s 的堆记账漂移"假设相容,远低于
   AC-1 数值门(6MB/min)。
4. **仪器轮"VM 必退"二次实证为收尾清理**:p765-short 轮(流式超时臂)
   VM 全程心跳连续、审计册零行,退出码 1 由仪器 finally `killTree`
   产生——与纪要三外部击杀指纹预测吻合;musk T-07 亦独立记录了同型
   "taskkill 宿主自毁"事故。
5. **仪器斜率单位 bug(移交 musk 修)**:`soak-active-session.mjs` 的
   `slopeMBPerMin` 对**已为分钟**的样本 t 再 ×60——报出的"MB/min"实为
   **MB/hour**(p765-1h 轮实测:脚本报 61.96,同窗绝对增量仅 +59.3MB/59.7min
   = 真 1.0MB/min,LS 无中生有 60 倍)。基线五轮"30-72MB/min"实为
   0.5-1.2MB/hour;修复:删去该函数的 `* 60`(或 t 改秒)。
6. **1h 长跑终判(V03,2026-10-12 00:05)**:idle 59.7min 绝对增量
   +59.3MB=**真趋势 1.0MB/min(0.017MB/s),线性不平台**(5min 段增量
   +2.7~+10.6MB 均值 ~5MB)——AC-1 数值门(≤0.1MB/s)6× 余量 PASS;
   残留 ~17KB/s 缓漏源未归因(候选:每响应仍余 3 份全文瞬态 ×2 拍/s 的
   堆记账漂移/重绘路径),登记后续杠杆。流式相位 95.2(脚本单位)=
   1.59MB/min,在基线带 [0.67,2.38]MB/min 内(样本方差大,7 样本噪声
   主导)。**退出面:vm_died=false 全程 60min 存活 + 审计册零行 +
   125 条连续心跳(62.5min)——外部击杀指纹之外的退出路径零发生**,
   收尾"VM EXITED code=1"再次为仪器清理(在 `written` 之后)。

## 后记⑦：r2 瞬态尖峰退出面归因——矩阵仪器 finally-taskkill 假象（2026-10-12，T-05）

### 呈报与复现背景

musk PLAN-104 T-03 四轮 bigpage 矩阵（2026-10-12 00:53–01:02，并行会话）呈报
"r1 修复面之外存在瞬态尖峰退出面"：①burst 形态（350×5KB/15s 消息分发入 VM
生命期，轮 1-3 死于爆发+列表轮询叠加窗）；②大快照形态（轮 4：合成预启动，
VM 唯一负载=一次 309KB autoui_snapshot 全树序列化，快照成功返回数秒内原生
exit(1)、零错误输出）。四轮全部使用 post-r1 release auto.exe（43a816b47，
00:49 构建——含 F-1~F-9 全部审计面）。

### 三通道零痕核查（复现前静态取证）

- **默认审计册**（`%LOCALAPPDATA%/auto-desktop/exit-audit.log`，AUTO_DESKTOP_EXIT_LOG
  未设时 r1 审计面的缺省落点——musk 四轮从未检查过该文件）：最后一条停在
  2026-10-11 22:05，四轮死亡时段（00:53–01:02）零条目 ⇒ panic(101)/main_err/
  native_crash(F-9 SEH)/vm_process_exit/mem_guard 五面全部未命中；
- **WER**：Application 日志与 ReportArchive 中 auto.exe 的 AppCrash 记录最近
  一次为 10-01（c0000409 fastfail 家族=PLAN-747 已修的 panic-in-panic），四轮
  时段零记录 ⇒ 非原生崩溃（若为 SEH，F-9 已装 + WER 应双痕）；10-12 00:36 的
  WER 簇经参数比对为 10-11 12:03 一次 BSOD（bugcheck 0x7E/c0000005）的延迟
  重报，机器在四轮时段无重启（System 日志 Kernel-Power 41 仅一条）；
- **vm.log**：四轮共用追加文件含 4 份启动 banner、仅一段连续 idle=1..51 的
  [POLL] 序列（轮 1-3 零 [POLL]），末尾干净截断零错误。

### 复现与因子判别（五臂实验，2026-10-12 01:57–02:25）

复现环境与四轮同构（musk worktree release musk.exe + release auto.exe +
同载荷 350 轮 261KB 页 + 同 spawn env）。判别台 = bigpage-matrix 的最小变体
（`p765-probe.mjs`，四臂 snap/nosnap/click/burst，错误捕获不触发收尾击杀，
全程被动观察；burst 臂把合成移入 VM 生命期且密度加严至 350 条/5s）：

| 臂 | 载荷 | 结果 |
|---|---|---|
| 原仪器复跑（run1/run2） | 快照+click | "VM EXITED code=1"（见下=收尾击杀） |
| snap（只快照不点击） | 309KB 快照 ×20+，8s 节奏 | **存活 180s** |
| click（点击但捕获错误继续观察） | click isError 后继续快照 | **存活 185s**（IAT 构建，审计册零行） |
| burst（合成入 VM 生命期 5s+列表轮询叠加） | 252 条 [POLL] 全窗 | **存活 181s** |
| run2（AUTOUI_MCP_DISABLE=1） | 心跳关闭 | 与 run1 同点收尾击杀（心跳非必要条件） |

**结论：两种"尖峰载荷"形态下 VM 全程存活，零原生退出。**

### 定谳：仪器 finally-taskkill 假象（r1 后记④/⑤同族，新位置复发）

bigpage-matrix.mjs 为 try/**finally 无 catch** 结构。每轮固定链路：
快照找到 entry → `autoui_action click`（本构建 click 要求 `value:"x,y"`，
musk 注释在案的已知怪癖）→ isError 抛出 → **finally 先执行**（killTree 砍
VM——`taskkill /T /F` 退出码恰为 1）→ `vm.on("exit")` 打出 "VM EXITED
code=1" → **此后**模块拒绝才打印错误栈。node ESM 顺序微实验钉死：finally
的输出先于未捕获拒绝的栈打印——控制台里 "VM EXITED code=1" 出现在 Error
trace 之前，正是收尾击杀而非进程自退的直接读据。

时间线复核全吻合：[POLL] 500ms/拍，idle=1..51 ≈ 25.5s 轮询窗 = VM 起后
~3s 开拍至 click 时刻（mcp-up+8s 首轮迭代即找到 entry——snap 臂实证
"has BIGPAGE: true" 从首个循环快照起恒真）；run1/run2 死点相同（idle=51）
because 击杀时机由仪器流程（click 抛错点）决定，与 VM 状态无关。轮 1-3
（旧形态合成在 VM 生命期内）的"死于爆发+列表轮询叠加窗"同构：击杀同样
发生在流程尾（click 抛错或 entry 检索超时后），爆发只是背景噪声。"快照
成功返回数秒内原生退出"实为"click 失败后收尾击杀"（8s 间隔在体感上读作
"数秒"）。与 r1 后记④ soak 仪器收尾假象、musk K14（taskkill 宿主自毁）
构成三例同族——**本机 VM 宿主的"负载相关静默必退"至今全部定谳为仪器/
驱动方击杀假象，无一例进程内真实退出**。

### r2 交付：F-10 ExitProcess IAT 追踪钩（观测面真缺口闭合）

归因过程中确认一个**真实的观测缺口**（非缺陷但属四分法边界）：依赖代码
直调 `ExitProcess(1)`（不经本仓任何审计 site、非 SEH）将零痕死亡——与外部
击杀不可区分。r2 落地 `install_exit_process_trace`（stdlib.rs，紧邻
native_crash_ffi；run_session 装配，env `P765_EXIT_TRACE=1` 门控）：遍历自身
模块 IAT，将 kernel32 的 ExitProcess/TerminateProcess 槽替换为追踪 trampoline
——落笔调用栈（模块基址相对偏移，release 构建配 line-tables 可符号化）后
转调原函数，只加观测不改语义。覆盖边界：外部 TerminateProcess（taskkill）
不经本进程 IAT，不在覆盖面（文档化）。单测：p765_iat_probe_child（子进程
装钩后裸 `std::process::exit(9)`）+ p765_iat_trace_writes_audit_and_preserves_
exit_code（审计行含 exitproc_trace/ExitProcess(9)/frames 偏移 + 退出码 9
原样保持）。防御性：PE 解析按 SizeOfImage 界内校验、无 INT 描述子跳过。

### musk T-03 解阻交接（仪器修复，非本仓面）

bigpage-matrix.mjs 两处修复即可闭 T-03：①click 补 `value:"x,y"`（配
include_bounds 快照取部件局部坐标——其注释③已勘明）；②收尾击杀与真实死亡
区分：finally 的 killTree 前置 `cleaningUp=true` 标记，`vm.on("exit")` 里
`cleaningUp ? "CLEANUP KILL" : "VM EXITED"`——消除 "VM EXITED code=1"
的误读源。修复后矩阵按本轮口径复跑（两形态预期零退出）。附带：本纪要
后记⑦证据链（三通道零痕核查 + 五臂判别 + 微实验）可直接支撑其 T-03
"退出面=仪器假象"的结案陈词。

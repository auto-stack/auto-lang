# auto-av 常驻在线热重载（PLAN-866）

> Status: active（实现=`core/src/av.rs` 安全点/换解析恢复 +
> `probe/src/resident.rs` 常驻驱动 + `av_probe_worker` resident 服务 +
> recipe `ac-av/resident-run-v1`；语料=ace-m1 `resident-online-866` H01-H03）。

## 常驻运行（resident run）

- 一个常驻运行 = **一条** guest 执行链（一次顶层入口调用）跨任意多 fuel 切片
  （`execute_program_steps` 首启 + `resume_program(Some(n))` 推进），**永不因编辑
  重入入口**（chain_entries 账目恒 1，重启实现可判别）。宿主以切片推进（默认
  4096 步/片，driver 固定 64）、可取消（CancelFlag→`Cancelled`）、可停止（宿主
  终态，区别于 guest 终局）。
- 切片粒度是宿主调度旋钮，**不得在 guest 语义中可观察**（切片不变式：任意粒度
  下终值与累计步数完全一致）。

## 安全点与在线安装

- 安全点 = **一条 guest 调用完成后、下一条 Call/CallClosure 派发前**的唯一边界
  （`resume_program_at_call_boundary`：派发前停车、步数回退中性）。循环回边不是
  安全点。armed 段持续执行 guest 工作直到真实调用边界，与宿主切片粒度无关。
- 安装时序：边界停车 → `session.commit`（828 单一发布点）→
  `resume_program_with_resolution` 换解析恢复（`Some(0)` 停车保留新 resolution）。
- **在途守卫（延迟安装）**：changed 函数属于在途帧集合（`PausedRun::frame_funcs`，
  def-id 剥 `d_` 前缀与源名比较）时**不 commit 不 swap**——候选留队、理由可观察
  （`deferred: ... in flight`），旧 body 跑到返回，新 body 只对下次顶层运行可见；
  链结束后再次 apply / 下次 run 生效。该守卫同时结构性排除在途帧 slot 布局失配
  （布局变更编辑在途函数永不接触新布局）。

## 可见规则（冻结）

- 安全点后的**新派发调用**看到新 revision；**在途调用**（安全点时刻已在执行的
  帧）整体留旧至返回；同一调用的一次执行内不得新旧混拼（帧代码在派发时经视图
  绑定，闭包 env 随创建帧同代；换解析必须整快照原子换）。未改函数跨修订按
  Arc 身份复用（slot 索引按 828 outline 稳定序可比）。
- def-id/outline 稳定性由 828 名集不变门保证；接口签名变更走 828 接口门，不属
  在线首例范围。

## 失败面与回收

- 候选编译/验证失败（prepare 拒）、安装失败（link/迟到/外来拒）→ 运行不中断、
  当前版不变、理由经 EditOutcome 可观察；guest trap/resource 按语言语义终止
  （与安装失败分列计数）。
- 回收：换版后 retired snapshot 无强持有即释放（帧 pin 的是代码对象 Arc，非整
  快照——Weak oracle）；旧代码对象的上界 = 828 内容史 HISTORY_CAP=32/函数 +
  max_frames 在途面；resident `MAX_RESIDENT_OLD_GENERATIONS=32` 为宿主持 pin 的
  守护闸。不新增内存上限（沿用 slot 8MiB/受管 32MiB/步数 1M/wall 按段 2s 与
  825 账本扣除）。

## 计量与验收

- 重载可见延迟：编辑入队 → 安全点生效（TestCase v2 口径 P90≤2000ms 硬门，
  P95/P99 诊断；样本不缩）。结构计数：安装集合、每站点代际、未改函数生产
  阶段=0、whole-module=0、旧代存活/峰值、生效步位。
- 负控（禁止替代，判红）：重启进程/重新运行（切片不变式+chain_entries）、
  暂停-恢复冒充（生效迭代号精确断言+无换版恢复负控）、只换 generation 标签
  （content digest 随代际同变）、切片边界换版（生效点锚定调用边界：不同粒度
  同界同值）、在途祖先帧换版（延迟守卫+旧值精确完成+下次 run 见新）。

## Recipe 与语料

- `ac-av/resident-run-v1`（append-only 三缝：auto-test-case contract 词汇
  `resident-slice{slices}`/`resident-apply`/`resident-finish` + 父侧判定
  （judge_resident_faces：chain_entries==1、unaffected_stages==0、
  whole_module==0）+ worker 服务）；父侧独立 oracle（worker 不见期望）。
- ace-m1 语料 H01-H03（连续运行 2100 / 在线换版 2608=46×7+254×9 / 双换版
  churn 3070=46×7+23×9+231×11；wrong-oracle 负控红）。852 binding I08 的 AV 列
  由此关闭（native 列 gap:ac-native-loader 保持）。

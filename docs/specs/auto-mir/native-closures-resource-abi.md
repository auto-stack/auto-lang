# Native 闭包资源 ABI 与故障原子性（PLAN-924）

> 状态：experimental（内部 native env/call/资源/版本/原子性合同；非 797
> 公共 AutoABI）

## env 布局与 ABI 身份（D-01 冻结）

- env = collections-runtime `Node { kind: CLOSURE }` 单块 arena 分配
  （header + `[Cell; len]` capture 序字段）：`type_fp[0]` = callee 机器
  code pointer（CreateClosure 发射的重定位绑定本 image）、`type_fp[1]` =
  签名身份（blake3 over layout descriptors）、`type_fp[2]` = 名义 owner
  身份、`disc` = 调用 pin 计、`epoch` = 已接受分派计数、`moved` =
  parked 标记。
- 闭包值 Cell `{ bits: env 地址, owned: 1 }`；`refs` 为 closure 值
  owner 计（share/clone 走现有 cell 语义）。MN-EVT 事件 kind=
  `closure`（创建/释放沿 919 通道）。
- body 机器签名 `(env: i64, params: *Cell.., out: *Cell)`；普通函数与
  前置 out-cell 约定不变；闭包 body import 携带 env 声明，跨单元 link
  准入逐位核对签名与 env 声明。

## seam 与故障注入（R02 闭包部分）

- 新 seam **S9 env-node**（env 单块存储，含 header+字段，独立计数）；
  923 的 S1..S8 计数与语义不变（字段深拷贝落在其物理分配 seam：S2 数据/
  S5 clone-work/S6-S7 text）。
- 入口沿用 `mn_coll_fail_nth_seam(seam, n)` 与 wrapper knobs；closures
  recipe 允许 seam 1..=9，collections recipe 保持 1..=8。
- 注入/预算/取消错误分态与 923 相同（INJECTED 25 带 MN-COLL-INJECT
  seam=/nth= 事实行；RESOURCE 22；CANCELLED 26）；env 准入拒绝
  （foreign/stale/错签名）为 INVALID 23 + `MN-COLL-CLOSURE refusal=N`
  事实行，被调计数保持 0。

## 原子性与清理

- 两段式事务：prepare 全部 fallible 工作先做（staging 不入 registry），
  commit 只发布；任一失败逆序无分配回滚，源逻辑值/责任/旧 Closure/
  代际不变（Move 源的杀责发生在 commit 之后的生成代码里，失败路径天然
  不杀）。Move+Clone 混合且失败落在后一字段时前一字段不误杀（staging
  retain 语义）。
- 同进程 fault 清除后重试正确；probe（持源/旧 Closure 的同 runtime 状态
  探针）与 source PE 两腿证据并存。
- 最后 owner/pin 后 env/字段/backing/text/arena 全归基线（live=0/
  bytes=0/invalid=0/arena_live=0）；abort 帧扫后终结 parked env，trap 后
  清理不变量保持。

## 版本与消费者

- `mn.collections.runtime/3` → `/4`（env 面 + S9；旧 COFF/manifest 随
  runtime 内容身份变化正确 STALE，不承诺旧工件命中）。collections
  cell/3、layout/codegen 不变；923 全部非闭包覆盖在 /4 下重验（R02 完整
  聚合的前提）。
- 观察解码：MN-EVT kind 词表新增 `closure`（native observe 与父侧
  resource_events/guards 解码同步）；MN-COLL 摘要行不变（live/bytes 已含
  env 节点）。

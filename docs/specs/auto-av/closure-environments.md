# 类型化闭包环境与捕获生命周期（av.closures/1）

> 实验性 typed 执行 profile；来源 PLAN-823。PLAN-865 另接入下述显式源码 producer；不改变本文件冻结的 AVAD 身份或运行时合同。

## 身份与 wire

`av.closures/1` 使用独立身份：profile id 6、AVAD wire v5、类型 tag 17（`TY_CLOSURE`）、语义版本 1/1/1、producer `plan823/av-closures`。`CREATE_CLOSURE`/`CALL_CLOSURE`/`ENV_GET` opcode 为 `0x32`/`0x33`/`0x34`。PLAN-824 已交付的 `av.dynamic/1` 保留 v4/5/tag 16/`0x2F`–`0x31`；两 profile 的 wire 族互斥（closures/dynamic tag 集仅在各自族内可载），交叉 owner 工件在准入门拒绝。整合前的 v4/id5/tag16/`0x2F`–`0x31` 闭包登记为历史，不作为现行身份。

## 环境所有权与调用

环境是 owned aggregate，永不 Copy：move 捕获杀死源槽，显式 clone 按字段深克隆并独立拥有（委托 `av::clone_owned_value` 单源实现），闭包 CLONE 产出独立 env 与新句柄。创建者返回后合法 owned 闭包仍可在新调用帧执行；可变环境字段经共享 List 句柄实现。调用核验签名与 env-store 句柄；trap/取消沿全出口清理，环境字段恰好释放一次。

逃逸捕获仅 move 或显式 clone；借用捕获限静态可证非逃逸的同步调用（borrow 条目=StrView share，源槽全函数钉住，返回/入容器/跨未知调用在 verify 前门拒绝）。捕获构造为 prepare/commit 两相：可失败分配/克隆先于任何转移；提交前失败仅清理暂存、未提交源保留，提交后失败清理新 env 而非复活源。父 env 字段转移仅专用 `MoveFromEnv`（moved 标志位图；别名 Move 洒洗字段所属在 `mir.capture.env-alias` 拒绝），`ENV_GET` 目标槽界限在 verify 期检查。

## 失败分态与预算

构造/克隆/记账的分配失败为 Infra（attempt 计数），越界/失效句柄/错型/代际为运行期受控拒绝并断言危险目标操作计数 0；OOM/预算/取消/infra 与 invalid/unsupported/trap 分态。env 字节计入 slot_bytes 单行（值数据 8MiB 口径），worker 峰值由父侧采样硬门。typed fixture、MIR→ABC→AV 与新进程恢复入口保持支持；PLAN-865 增加编译源码并接入同一 closures session 的 producer。

## 源码绑定（PLAN-865）

显式 auto.source.closures.draft profile 见[源码闭包规范](../auto-ac/source-closures.md)。它在集合源码面上加入函数类型、lambda、捕获分析和间接调用，并降到现有 av.closures/1 环境与调用合同；不新增 AVAD 版本、profile id、类型 tag 或闭包 opcode。

源码分析在 lowering 前生成有序 Move/Borrow 捕获模式。Copy 捕获仍复制；拥有值需由 move lambda 转入环境；借用仅限可静态证明同步且不逃逸的 Str，映射为 StrView。源码 f.clone() 使用既有闭包深复制路径，生成独立 env 与句柄，不改变捕获列表或顺序。env 声明与 fn: 链接期精确相等检查保持原合同。借用逃逸、共享可变捕获和不支持的捕获类型在源码编译阶段拒绝。

K01、K02、K03、K05 已经由源码编译进入现有闭包 session 执行。K04 仍是借用逃逸编译期拒绝；K06 以 source.closure-shared-mutable 拒绝并保留 R5-O owner，本次不关闭 C-19。命名函数项值、闭包字段/List 元素、async、持久化、跨线程、native closure ABI 与集合方法 HOF 仍在本 profile 之外。

## 缓存与恢复

代码缓存与活环境分域：缓存仅持久化代码及可持久依赖（A23 层），活帧/borrow/宿主地址不入缓存；新 worker 从同 store 重验身份（producer/owner/entry/profile/types-fp/语义版本/source-snapshot）、payload、entry 与 snapshot 后执行，好 hash 不替代内部深验。恢复与交叉身份规则见 [缓存合同](cache-and-conformance.md)；与 824 的交叉正反控（wire 身份、pack/freeze 任意深度拒绝、合法值正控）为常驻回归面。

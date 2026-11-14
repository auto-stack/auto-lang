# ASE M0 无窗口加载（headless）现状记录

> 来源：PLAN-1021（2026-11-14 交付，交付提交见计划 §9）。本文件只记录**已验证**的
> 能力：两个已注册 app 身份的严格解析，与经**公开** ACE 包链的一次冷执行及错误
> 分态；未交付能力一律不视为已支持。实现位于 `experimental/ase-m0/`（独立 Rust
> workspace，非语言/ACE workspace 成员；仅 path 依赖 ACE 公开 crate，零 ACE 源改动）。

## 支持域与身份（AC-01）

`app://<identity>/`：scheme 仅 `app`，authority 仅已注册身份（当前 `auto.demo.a` /
`auto.demo.b`），路由仅根 `/`；userinfo/port/query/fragment、大写身份、空身份均具名
拒绝（`uri`）。身份来自宿主元数据目录 `registry.toml`（实验描述，非公共作者语法），
URI 不用于拼接文件路径。未注册身份（`authz`）、未注册版本（`authz`）、声明的额外
宿主能力（M0 无任何额外能力，`authz`）在读源与 dispatch 之前拒绝；注册版本格式
`[A-Za-z0-9.+_-]+`、包名 `[a-z][a-z0-9_-]*` 由 registry 结构校验把关。URI 不是执行
任意文件的授权。

## 源闭包与根包含（AC-02）

注册项 = 身份/版本/ACE 包名/根（相对元数据目录）/入口模块/源文件清单。读取在
`registry.toml` 目录内做有界根检查：根或源名含 `..`/绝对前缀/多段路径
（`source-escape`）、注册目录下任意组件是 reparse 点/junction/symlink
（`source-reparse`）、注册件缺失或非常规文件（`source-missing`）均具名拒绝，
拒绝先于任何内容读取（宿主私有路径不被读）。**这是有界根检查，不是沙箱**。

## 公开执行链与结果观测（AC-03/AC-04）

一次执行 = `compile_package_product`（普通源前门/verify，TextRecord 域，仅
`ac.source.modules.packages/v1` 能力）→ `PackageAvTarget::cold` → `run_value`，全程
仅公开入口：无 legacy VM、无手写 HIR、不访问私有 VM 字段。每次调用为独立冷执行：
A/B 实测返回 11/22；编辑 A 源为 12 不影响 B（22）；换根注册/重排注册序不影响解析
（值跟随注册闭包；包名参与绑定 `input_hash`，故同源不同包名身份不同）。坏源在编译
阶段拒绝，修复后可真实重试。运行报告含消费源 sha256（前 16 hex）与包
`input_hash`——返回常量的 stub 无法伪造来源身份。

## 错误分态与 dispatch 计数（AC-05）

错误 = Phase（resolve/source/compile/link/execute）× Kind
（uri/authz/registry/source-escape/source-reparse/source-missing/source-io/
compile/resource/link/execute）。所有读源/dispatch 前的拒绝 dispatch=0；公开入口
预算耗尽（诊断码 `source.resource`）分类为 `resource`，不改判语法错或成功；编译期
其他诊断分类为 `compile`；冷链接/执行失败分别为 `link`/`execute`。CLI 成功/拒绝
退出码 0/1，输出含 dispatch 观测。

## 入口

- CLI：`cargo run --manifest-path experimental/ase-m0/Cargo.toml -- --app
  app://auto.demo.a/ --artifact-root <dir> [--registry <dir>] [--version <v>]`
  （`--registry` 默认随包 fixtures）。
- 测试：`cargo test --manifest-path experimental/ase-m0/Cargo.toml --test headless`
  （V-01..V-05；junction 负控 cfg(windows)）＋ uri 单测。
- 固定组：`cargo t ase-m0`（显式 opt-in；不在 daily、`cargo t all` 或 ACE 并集）。

## 边界

无 UI/后台服务/沙箱/安装器/通用 daemon/持久 AppSession/磁盘零编译缓存；无 warm
body-edit（声明/主体改动经 packages lane 重编译，沿 960 边界）；重复执行是一次次
冷执行，运行后 AV store 文件 owner 释放（store 目录可删）。ASE 只做发现/编排与错误
投影，不复制 ACE 解析器/执行器，不加 Core→ASE 反依赖。1022 起的产品/窗口链走
969/E0 等正式门面；本实验的低层路径不升级为长期产品 API。上游 VM 臂不可运行等
Gallery 现状见 [m0-readiness.md](m0-readiness.md)。

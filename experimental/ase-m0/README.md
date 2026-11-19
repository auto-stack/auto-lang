# ASE M0 — 应用解析与无窗口 AV 加载实验（PLAN-1021）

有界一次执行研究：严格解析两个已注册 `app://` 身份（`auto.demo.a` / `auto.demo.b`），
在允许根内读取已登记源闭包，经 **公开** ACE 包链
（`compile_package_product` → `PackageAvTarget::cold` → `run_value`）一次冷执行并真实返回
11/22；A 编辑为 12 不影响 B；坏源按解析/编译/执行/资源分类拒绝，修复后可重试。

边界：无 UI/后台服务/沙箱/安装器/持久会话/磁盘零编译缓存/warm body-edit；本目录是
独立 workspace（不加入语言/ACE workspace），只消费 ACE 公开 crate，不改 ACE 私有执行。
URI 不是执行任意文件的授权——只有显式注册的本地可信程序可加载。根包含检查是有界根
检查，不称完整沙箱。

CLI（实验入口）：

```
cargo run --manifest-path experimental/ase-m0/Cargo.toml -- \
  --app app://auto.demo.a/ --artifact-root <dir> [--registry <dir>] [--version <v>]
```

`--registry` 默认为随包 fixtures 目录。测试：`cargo test --manifest-path
experimental/ase-m0/Cargo.toml --test headless`（固定组 `cargo t ase-m0`）。

## PLAN-1022：两页普通 Node 文字源（UI 页面输入）

`fixtures/text-a/main.at` 与 `fixtures/text-b/main.at` 是 ASE M0 文字浏览器
（PLAN-1022）的两份**独立普通 Node 页面输入**：中文页（`TextPageA`，字面量叶
+ `headline_a()` 函数文本叶）与 ASCII 页（`SecondTextRoot`，不同合法根名/
函数名，同构双叶）。它们不是 `registry.toml` 冷链 app（M0 能力域无 UI 能力、
入口须返回 i32），不经 `run_app` 执行；UI 宿主（auto-ui `ase_m0_frame`，组内
兄弟树显式路径）经 979/980 冻结的普通 Node 兼容面装载并取 AV 真值文本。
源闭包=单文件自包含；换合法根/函数名及改文字仍工作（auto-ui 侧
`ase_m0_text` 目标 V-02 反硬编码 oracle）。

## PLAN-1023：两份交互 Counter 源（会话宿主输入）

`fixtures/counter-a/main.at` 与 `fixtures/counter-b/main.at` 是 ASE M0 交互
会话（PLAN-1023）的两份**独立普通 Node 应用输入**：同 Schema（无参
`msg { Inc, Dec, Reset }` + `model count int = 0` + `on` 复合赋值），不同
合法根名（`CounterA`/`CounterB`）与文字——反硬编码 oracle 输入。按钮声明序
两源一致（0=`-` Dec、1=Reset、2=`+` Inc）。它们同 text-a/b 一样不经
`run_app`/registry 冷链执行；UI 侧会话宿主（auto-ui `ase_m0_sessions`，
组内兄弟树显式路径）经 979/980/981 冻结的公共面装载：`count` 经已交 int
状态槽显示（显示=只读镜像，业务权威在会话引擎），事件=981 事件 A 无参
unit 面（带参/载荷近邻在边界具名拒）；同 app 双开=两次独立装载（双引擎，
模型不合并）。不为显示引入 f-string 文本叶（W1 已知边界）。

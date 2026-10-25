# PLAN-832 T-05 独立 Man workspace / Core 安装验证

冻结实现：Man `2419e57e22b5539726416fe7d44e982096dcf63f`；lang `087c40c9e92796b2ccf1dc391a547547f8f4c981`；UI/down 固定版本同 T-01。

## 1. 独立 workspace 与锁

- Man 根 9 成员、default=`auto`，`.cargo/config.toml` 提供 t/tc/ti 本地 CI 入口；根 `Cargo.lock` 已受管，SHA256=`d545ea649539404a12876057c3084465f88a8b833313bf7c5118b8e367228936`。
- `cargo check --locked`（runtime/client/五个管理 core）通过。
- `cargo build --locked -p auto --no-default-features --features core` 通过；参考 auto.exe SHA256=`44f54e41b2532f1e457f72f6b8b76b8b0c1d8c996b50ec03fcfc42b02a753b96`。
- normal/build 与 normal/build/dev 闭包均无 UI、旧 auto-lang/auto-man compat 或 iced，且包含 lang-core/runtime/client/protocol/execution：[t05-core-tree.txt](t05-core-tree.txt)、[t05-core-dev-tree.txt](t05-core-dev-tree.txt)。

## 2. 管理测试与资源

- 管理库 scoped tests：**252 passed / 0 failed / 4 ignored**：6+45+118+14+14+17+38；Core CLI：**18 passed / 0 failed**。未运行整档。
- 五组外部 consumer：**23 passed / 0 failed**，且各 `verify-deps.sh` PASS（man-core 3、package-lock 4、resolver 7、toolchain 5、assets 4）。
- 补入 T-04 未闭资源：`crates/autoui-skill` 编译期 wizard 模板、199 个 pac.at 真实语料和 project-model 反例。资源审计 [t05-resource-audit.json.txt](t05-resource-audit.json.txt)：**262 same / 3 hygiene / 0 unexpected-drift**；hygiene 仅三个 pac.at EOF 空行。
- 语料测试先因未提交只见 7 文件而红，提交真实输入后 118/122 runtime（4 ignored）通过；未降低 140 的最低语料门槛。

## 3. Core 安装与事务

`scripts/man-install.py` 提供 install/verify：

- 从当前 Man/固定语言源构建，metadata 收集 normal/build/dev 路径闭包；Core 不复制 UI 实现、auto-lang compat、shell-pack 或 auto-ui-tool。
- SDK 清单改写到包内兄弟；缺失 optional path 依赖做受管投影并记录 edits；SDK 根保留第三方 workspace.dependencies、去除 Core 不需要的 iced patch，离线重生成锁。
- staging 唯一标记→全部文件 hash/profile/来源校验→受管交换；陌生非空目录拒绝，旧安装保留于具名 backup，失败仅清本次 staging/恢复旧根。
- 最终安装 [t05-core-install-manifest.json.txt](t05-core-install-manifest.json.txt)：**643 files**；安装态 metadata=29 packages、66 条本地 path 全部在安装根内；`cargo check --locked`（安装 SDK 的 auto Core）通过。

实际命令（清空 AUTO_*根/target 覆盖）：

- 安装 auto `--version`、Unicode+空格路径脚本及两个透传参数：输出 `man-core-ok` 和 `3`。
- `auto new demo`：真实生成 `demo/pac.at` 与 `demo/demo.at`（模板实际布局，非虚构 src/main.at）。
- `auto trans … python`：真实 Python 产物非空。
- `stdlib inspect --format json`：合法 JSON，当前 `status=partial`、合同 exit 3。
- `auto ui list` / `auto bp list`：均非零并具名要求 full profile/ui-extra，未静态 fallback。

## 4. 调试与边界

- Core 首次构建缺 `autoui-skill` 三个 include_str 输入：已补源并验证。
- 安装源曾遇 optional UI path、workspace.dependencies、feature 逗号、资产目录通配、iced patch 与投影锁问题；安装器逐项修正后由独立 metadata/path/locked check 闭合，未靠主检出兜底。
- 现有 lang-core、runtime 和部分旧 fixture 警告按来源保留记录；未宣称全仓零警告。Man 根 default-run 无效键已移除，导入/提交卫生检查通过。
- full UI 工具发现、生成/build/run、服务/watch 和三仓组合仍归 T-06；fresh clone/重定位/升级故障矩阵归 T-07；merge 统一门 pending。

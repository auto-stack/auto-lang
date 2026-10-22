# F-815-T05-R1-01 Unix 平台实测证据（R5 修复轮，2026-10-22）

R5 复审要求："关闭需声明支持的 Unix 环境对实际冻结源码执行具名真停/
自然 EOF/有界清理测试及必要编译，绑定依赖/目标/命令/结果。"

## 环境（本机 WSL）

| 项 | 值 |
|---|---|
| 发行版 | Ubuntu-22.04.5 LTS（WSL2，`wsl -l -v` 确认） |
| 工具链 | rustc 1.97.1 (8bab26f4 2026-07-14) / cargo 1.97.1 (c980f4866)（`~/.cargo/bin`，登录 shell 可见） |
| 系统库 | libxkbcommon/libxkbcommon-x11/libwayland（dev 头+so 在，pkg-config xkbcommon ok——iced/winit Linux 链编译前提） |
| 源码 | Windows worktree `D:/autostack/.wt/lang-815/auto-lang` 经 `/mnt/d` 只读消费；修复提交 `c6fdaaaf1ecc2799d5ad225db3922a121c90c383`（测试时为同内容工作树，随后原样提交） |
| 网络 | 本仓库外环境事实（非源码问题）：WSL `~/.gitconfig` 带 `http.proxy=localhost:10809`（Windows 侧代理，WSL2 内不可达且 cargo 会读它）；用 `CARGO_HTTP_PROXY=`（空覆盖直连）+阿里云 sparse 镜像 env 替换拉取依赖。不改任何用户配置文件。 |

## 命令与结果

```
cd /mnt/d/autostack/.wt/lang-815/auto-lang
CARGO_HTTP_PROXY= \
CARGO_SOURCE_CRATES_IO_REPLACE_WITH=aliyun \
CARGO_SOURCE_ALIYUN_REGISTRY="sparse+https://mirrors.aliyun.com/crates.io-index/" \
cargo test --locked -p auto-ui-tools --lib unix_reader_truly_stops \
  --target-dir ~/plan815-unix-target
```

- 编译：`--locked`（依赖=工作区 Cargo.lock 冻结），lib 测试目标全量编译
  成功（含 auto-ui-runtime default=ui-iced 的 Linux winit/iced 链），
  ~1m42s（20 核，warm registry）。
- 结果（首轮）：`unix_reader_truly_stops_when_non_owned_process_holds_pipe ... ok`
  （1 passed; 0 failed，0.32s）——**非 owned 持管道**（`sh -c 'sleep 30 &
  exit 0'` 后台进程持有 piped stdout 写端，无数据无 EOF）下 reader 在
  停止标志处 5s 内真停 + `Stopped` 具名；echo 正控 `NaturalEof`。
- 补充面（同环境二次运行，`-- proc_host::tests::unix_reader
  proc_host::tests::captured_drain proc_host::tests::vm_session`）：
  **4/4 ok**（含 R5 新捕获排水的跨平台 sh 形态取消/自然 EOF 双例、
  vm 会话上下文每请求替换与致命捕获两单测）。
- 完整原始输出存档于本文件同目录 `wsl-unix-test.log.txt`（两轮拼接）。

## 边界与披露

- 本轮 Linux 面只跑 proc_host 具名测试（R1-01/R5-P1 的不变量所在），
  未整档跑 Linux 全库——整档归 merge 冻结门（裁定不变）。
- `real_tree_cancel_input_forward_and_reader_join`（R1 轮既有测试）使用
  Windows 命令（findstr/cmd/ping）且未 cfg 门控——Linux 面此前从未执行
  （无 Linux CI 跑该 crate lib），本轮 Linux 运行会红；这是**预存测试
  可移植性缺口**（非产品缺陷、非本轮引入），披露不修（修复属测试卫生
  跟进；SLOT_LOCK 毒化级联的其他两红均由此引发）。
- unix_reader/captured_drain 两测试已双平台可移植（Windows PeekNamedPipe
  臂 + Unix poll 臂各自原生命令形态）。

# sccache 加速编译（可选）

本项目支持用 [sccache](https://github.com/mozilla/sccache) 缓存 rustc 编译产物，加速 clean build。
它是**可选**的：没装 sccache 也能正常 `cargo build`（wrapper 会自动透传给 rustc）。

## 为什么需要手动开启

`cargo` 的 `rustc-wrapper` 配置**不支持**按平台区分，也没有“未安装就回退”的机制。
因此本项目没有在 `.cargo/config.toml` 里写死 wrapper，而是改用环境变量 `RUSTC_WRAPPER`
（它的优先级高于 config）配合一个跨平台的探测脚本：

- 脚本逻辑：PATH 里有 `sccache` 就代理给它，否则直接调用 rustc。
- 所以无论装没装 sccache，都不会报错。

## 开启方式（在 shell profile 里加一行）

仓库自带的 wrapper 脚本路径是 `scripts/sccache-wrap.sh`（Linux/macOS）和
`sccache-wrap.cmd`（Windows）。把对应的绝对路径设给 `RUSTC_WRAPPER` 即可。

### Linux / macOS（bash / zsh）

在 `~/.bashrc` 或 `~/.zshrc` 末尾加：

```sh
export RUSTC_WRAPPER="/path/to/auto-lang/scripts/sccache-wrap.sh"
```

把 `/path/to/auto-lang` 换成本仓库在你机器上的绝对路径。

### Windows（PowerShell）

在 PowerShell profile（`$PROFILE`，可用 `notepad $PROFILE` 打开）里加：

```powershell
$env:RUSTC_WRAPPER = "D:\path\to\auto-lang\scripts\sccache-wrap.cmd"
```

把 `D:\path\to\auto-lang` 换成本仓库在你机器上的绝对路径。

## 验证

设置后新开一个终端，运行：

```
cargo build
sccache --show-stats    # 已装 sccache 时能看到缓存命中统计
```

## 临时禁用

不想用 sccache 时，不必改 profile，直接：

```
RUSTC_WRAPPER= cargo build          # Linux/macOS
$env:RUSTC_WRAPPER=""; cargo build  # Windows PowerShell
```

（`RUSTC_WRAPPER` 设为空字符串会让 cargo 退回普通 build，不经过任何 wrapper。）

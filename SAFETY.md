# 跨仓库文件操作安全规范

> 本文档由一次真实事故催生：2026-08-02，用 `robocopy /MIR` 清理 git worktree
> 时，因 worktree 内含指向主仓库的循环 junction，导致两个仓库的 `.git` 和
> auto-ai 全部工作区被清空。本规范旨在**永久防止此类事故**。

## 1. 事故根因（务必理解）

`D:/autostack/` 下的仓库之间存在 **junction/symlink**，用于跨仓库 path 依赖：

- `auto-lang/.worktrees/<name>/auto-lang/` → 指回 `auto-lang/` 自身（循环！）
- `auto-lang/.worktrees/<name>/auto-ai/` → 指向 `auto-ai/`
- `auto-ai/crates/<crate>/` 的 Cargo.toml 用 `path = "../../../auto-lang/crates/..."`

**致命组合**：破坏性文件操作（`robocopy /MIR`、`rm -rf`、`rsync --delete`）
+ 含 junction 的目录 = **跟随链接连锁删除**。robocopy 把 junction 当真实目录
镜像清空，于是被链接指向的目标也被清空。

## 2. 永久禁令（NEVER）

以下操作**禁止**在 `D:/autostack/` 下任何含 `.worktrees/`、`target/`、
`crates/` 的目录上执行，除非已通过 §3 的预检：

| 禁止 | 原因 | 安全替代 |
|---|---|---|
| `robocopy <src> <dest> /MIR` | 跟随 junction 镜像清空 | `git worktree remove` / 先删 junction |
| `rm -rf <dir>/` 含 junction | 跟随链接删目标 | 先 `rmdir` junction，或用 §4 脚本 |
| `rsync -a --delete <src>/ <dest>/` | 同 robocopy | 同上 |
| `Remove-Item -Recurse -Force` | PowerShell 也会跟随 | 同上 |
| `find <dir> -delete` | 跟随 symlink | `find -type l -prune` 先排除 |

## 3. 操作前预检（MUST）

在 `D:/autostack/` 下执行任何**删除/清空/镜像**操作前，**必须**先运行：

```bash
# 预检：扫描目标目录及其父目录链上的 junction/symlink
bash D:/autostack/auto-ai/scripts/check-junctions.sh <目标目录>
```

如果输出任何 junction/symlink，**停止操作**，改用安全替代方案。

## 4. git worktree 的安全清理

清理 worktree 时，**永远用 git 原生命令**，不要用文件系统操作：

```bash
# ✅ 正确：git 自己管理 worktree
git worktree remove <path>      # 安全，git 知道 junction 边界

# 如果 remove 因长路径失败（Windows）：
#   1. 先 cd 进 worktree，删除其内部的 junction（它们是 worktree 自己建的）
#   2. 再 git worktree remove --force
#   3. 绝不要用 robocopy /MIR 或 rm -rf 清整个 worktree 目录

# 清理 git 的 worktree 记录
git worktree prune --verbose
```

## 5. 跨仓库 path 依赖的 junction 创建/识别

worktree 之所以需要 junction，是因为 a2r 的 `rust/Cargo.toml` 用相对 path
依赖指向 `auto-lang/crates/`。识别方法：

```bash
# Windows: 列出目录下的 junction
dir /AL <目录>

# Git Bash: 查找 symlink/junction
find <目录> -maxdepth 2 -type l    # symlink
# junction 在 Git Bash 里显示为目录，需用 Windows cmd 的 dir /AL 确认
```

**铁律**：任何 `git worktree` 的目录里如果出现 `auto-lang/`、`auto-ai/`、
`auto-shell/` 等同名子目录（与 `D:/autostack/` 下的仓库同名），**那几乎
必然是 junction**，绝不能对其执行破坏性操作。

## 6. .gitignore 配套

worktree 内的 junction 应被 `.gitignore` 忽略，避免误提交。规则**必须锚定
根目录**（前导 `/`），否则会误伤 `crates/auto-lang/` 等正常子目录：

```gitignore
# ✅ 正确：锚定根目录，只忽略 worktree 根的 junction
/auto-lang/
/auto-ai/

# ❌ 错误：不锚定，会匹配 crates/auto-lang/ 导致该目录文件无法 git add
auto-lang/
```

## 7. 事故应急

如果已经发生 junction 连锁删除：

1. **立即停止所有文件操作**，不要试图"补救"（可能扩大损失）
2. 检查损失范围：哪些目录的 `.git` 丢了、哪些工作区空了
3. 从远程 `git clone` 恢复（参考本次事故的恢复记录）
4. 本地未 push 的提交如果 `.git` 已丢，**无法恢复**——这是为什么
   完成工作后应及时 push

---

**本规范的适用范围**：`D:/autostack/` 下所有仓库（auto-lang、auto-ai、
auto-shell、auto-musk 等）。任何 agent/人在此目录树下执行文件操作前，
都应知晓本规范。

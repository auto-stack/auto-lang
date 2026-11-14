# ACE 自有标准输出：公共输出能力、Windows Unicode 控制台与重定向（native-stdio）

> 状态：PLAN-1041 交付稿（worktree 预备；merge 时正式沉淀）。
> 单源：`stdlib/auto/platform/io.at`（公共算法）+ 平台操作（1039/1040 增量）
> + a2x 运行时桩。本稿描述已交付行为；证据锚点见 PLAN-1041 §0/§9。
> 取代关系：`ace-host-services.md` §2 的"console 公共输出四端分账"仍描述
> 旧 `print/eprint/say` 画像；本能力是**新增公共输出 API**，不改旧拼写。

## 1. 公共 API（io 包公共面；冻结）

| 拼写 | 语义 |
|---|---|
| `out_stdout() int` / `out_stderr() int` | 标准流句柄（OS 侧即时验证；0/-1 → -8(6)） |
| `out_kind(h) int` | 1=Windows 文本控制台；2=字节 sink（文件/管道/Linux 流）；负数=错误 |
| `out_text(h, s) int` | 文本输出，无隐式换行；**进度单位=已确认 UTF-8 字节**（控制台只计完整标量）；成功返回==`s.len_bytes()` |
| `out_bytes(h, s) int` | 原始 UTF-8 字节到字节 sink；**文本控制台显式拒绝（-13）**；重定向逐字节原样（NUL 是内容；无 BOM/CRLF/编码变化） |
| `out_last_error() int` | 最后一个失败原语的原始 OS 错误码（GetLastError/errno） |

错误码：-1 未授权、-2 无效句柄、-8 OS 失败（原始码经 out_last_error）、
-9 预算（65536/调用、1048576/run——947/1039 冻结值）、-13 控制台字节拒绝、
-14 受控 WriteZero。EINTR（errno 4）同片有界重试（连续 64 次零进度→-8(4)）；
EAGAIN（11）立即返错不忙等；write 成功 0 且有剩余→有界重试 64 次→-14。
**分类冻结**：GetConsoleMode TRUE→1；FALSE 且 GetLastError==5→-8(5)（真实
权限错误上浮）；FALSE 其他→2。句柄有效性在取得步（不吞 6）。不改 code page。

## 2. 实现分账（六路径 + 原 A2C；2026-11-14 交付态）

| 路径 | 状态 | 证据/边界 |
|---|---|---|
| AV/native | implemented | Win32 backend（1039 增量 console_kind/console_write16）；AV console lane run_platform；OUT-01/03 捕获 oracle |
| AC | implemented | COFF/ELF shim 同操作（kernel32 GetConsoleMode/WriteConsoleW + Linux write）；真实 PE/WSL ELF 运行；导入闭包审计 |
| A2R | implemented | 生成工程 Rust runtime 桩（`a2x_*`；extern system WriteConsoleW）；rustc 真跑 |
| A2PY | implemented | Python 端口（PEP 528 控制台路径；buffer.write 字节 sink）；真 OS Python 真跑 |
| 原 A2C | implemented | C runtime 桩（static inline；cl 真跑）；与 AV/C 共享 C 来源的装配位就绪 |
| AV/C、AV/Rust | pending | 具名阻塞：包装配面拒绝 use.rs（binding.rs "belongs to the single-module source surface"）+ 967 F0 已纳帧仅单 &str→i32/bool/Option；解锁=967 包级 foreign 准入+富帧 |

**共享算法**：UTF-8→UTF-16LE 转换、分块（4096 UTF-8 字节/块，提交块不拆代理对，
OS 停在代理对内时保留剩余单元续发、只确认完整标量）、EINTR/WriteZero 循环全部
在 `io.at` Auto 源（char_at/to_codepoint + 数值算术实现编解码）；AV/AC/a2x
只提供"必要原语"（平台操作/运行时桩）。内存区做 UTF-16 暂存（8192 字节），
无需通用堆。

**旧语义不动**：`print/eprint/say` 保持 947 console/1 画像逐字节不变（本包
不触碰）；旧 p947 语料全绿。

## 3. 加载与覆盖机制

io 包按 1043 平铺装配消费：`io.at` + `io.at.at`/`io.c.at`/`io.rs.at`/`io.py.at`
候选（候选当前为薄层——provider 差异化适配位随 967 富帧交付）；`#in sys is
current` 目标轴与 provider 轴按 1043 合同（选择负控/未选不绑定见 V-LOAD）。
控制台/重定向判别是**运行时** kind 检查（同一程序两路），不是编译期选择。

## 4. 平台操作增量（platform.io/1）

- `auto.platform.io.console_kind.v1(h)`：分类（§1 冻结值）；能力 `platform.io.write`。
- `auto.platform.io.console_write16.v1(h, region, unit_off, unit_len)`：从 OS 内存
  区读 UTF-16LE 单元经 WriteConsoleW 写控制台；返回单位数（真实 OS 部分进度）；
  字节预算按 2×units 计入 65536/1048576；混合来源校验（流句柄+内存区）。
- Linux backend：kind 恒 2；write16 恒 -7（受控拒绝，绝不静默回退）。

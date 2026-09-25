# shell 跳转列表 Recent 集成（SHAddToRecentDocs shim）

> PLAN-701 供⑥（2026-09-25）。auto-edit PLAN-014 T-01 裁定 (b) 用户
> 确认改道件（2026-09-24）——内核零 shell 面四层勘定（源/二进制/注册
> 表/动态全零，probe_jumplist_report）的清偿。

## 契约

`shell_add_recent(path) -> bool`（native id 9914，catalog 名
`auto.shell.add_recent`，intrinsic 裸名直呼）：

- **调用形**：`SHAddToRecentDocs(SHARD_PATHW, path.encode_utf16()+NUL)`
  一调用（shell32 直 FFI，零新依赖）。效果=打开的文件进 shell Recent
  项（任务栏跳转列表「最近」）。
- **返回**：true=调用已发（Windows）；false=空路径（不发调用）/
  **非 Windows no-op**（恒定可链接——a2r 轨同形）。
- **零注册表关联面**：本件不触碰 HKCR/Classes——跳转列表 Recent 面
  不依赖文件关联注册（PLAN-014 T-01 勘定：pac `opens` 是 auto 桌面壳
  内部注册）。
- **非目标**：ICustomDestinationList 自定义任务类（014 T-01 裁定成
  文）。
- **测试边界**：真调用冒烟属下游 E2E/手工面——测试进程调用会污染用
  户真实 Recent 列表；单测只钉空路径 false 形 + 三面在册 grep。

## 三面同步

VM shim（`vm/native.rs` shim_shell_add_recent）+ catalog（9914）+
a2r/merged 臂（`vm_builtin_host_call` → `vm::native::shell_add_recent`
——实现体单源，非 Windows 形由实现函数内部 cfg 承载）。

## 消费位

auto-edit PLAN-014 T-02「消费 shim」：recents 落盘挂点
（SessionSave/OpenPath 维护位）直调，使打开文件进壳 Recent。

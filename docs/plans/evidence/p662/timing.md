# PLAN-662 计时证据（2026-09-20，lang-662 worktree 二进制 d5ec6d7b7 构建）

场景：`cd .wt/lang-662/auto-os/ui-gallery && auto run -r vm` 零 env（T-01 平铺探测命中组内 auto-lang），
计时 = 命令发出 → 日志 "AutoUI MCP: first state sync in view()"。

| 档 | 耗时 | 日志 |
|---|---|---|
| 基线（master fe88a3240 语义，658 时代实测） | ~83s（77s 串行扫描 + 3s VM 编译 + ~1s proxy） | /tmp/vm_gallery_ok.log（16:52:16→16:53:33 扫描段） |
| 冷启动·并行（默认核数） | **22s** | /tmp/662_cold.log：Gallery rows: 36 demos (36 scanned, 0 from disk cache) |
| 冷启动·串行（AUTO_GALLERY_SCAN_JOBS=1） | ~62s（02:02:49→02:03:51） | /tmp/662_serial.log：36 scanned, 0 from cache |
| 二次启动（缓存命中） | **5s** | /tmp/662_warm.log：Gallery rows: 36 demos (0 scanned, 36 from disk cache) |

## 串行/并行产物字节一致性（T-03 安全守卫）

并行冷跑产物快照（registry.at + AppViewport.vm.at + demos/ 76 文件）与清缓存后
串行冷跑再生产物 **diff 全空**（diff -q/-rq 逐文件 IDENTICAL）——并行默认档确认。

## 020 显示修复（T-04/T-05）

- 修复前：020_before_658.png（底部控制条被裁成窄缝、曲库表格黑块、进度条缩成点）
- 修复后：p662_020_fixed.png（播放控制条完整可见：歌曲信息+五控制按钮+进度条 0:30/4:18；393 曲库正常；is_playing 可交互）
- T-05 裁定=选项 B（020 语料零改动）：§5.5 判定标准达成；flex-wrap 两行换行登记 Plan 412 已知边界（KNOWN-DEBT P662-D1）

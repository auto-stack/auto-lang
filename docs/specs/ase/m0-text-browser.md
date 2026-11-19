# ASE M0 两页文字浏览器（同宿主 frame）已验现状

> 来源：PLAN-1022（2026-11-19 交付，复审/交付身份见计划 §9，归档于
> `docs/plans/archive/1022-ase-m0-text-browser-and-embedding.md`）。本文件只记录
> **已验证**能力与边界；未交付面一律不视为已支持。UI 侧消费合同见 auto-ui
> [ase-m0-frame-consumption](../../../../auto-ui/docs/specs/runtime/ase-m0-frame-consumption.md)
> （同组兄弟树/同仓消费）。

## 支持域（已验）

一个原生验证窗口内轮换两份**独立普通 Node 页面**：打开A/打开B、重新打开、关闭、
错误行与当前地址；A/B 内容由独立 Node 源经 ACE AV 真值产生（字面量叶 + 普通函数
返回文本叶；中文/ASCII 各一），经 980 MIN-A-UI 公共 ui980 车道（真实 UI-FFI）
提交至**同一宿主** frame。页面输入为 `experimental/ase-m0/fixtures/text-{a,b}/main.at`
（单文件自包含；非 `registry.toml` 冷链 app——M0 能力域无 UI 能力、入口须返回
i32），UI 宿主以显式路径消费，不复制权威。换合法根名/函数名/改文字仍工作
（反硬编码 oracle 在 auto-ui `ase_m0_text` 目标常驻）。

## 导航代次与所有权（已验）

导航代次在**被受理**的指令（打开/重开/关闭）上推进；装载票据挂载核代次——
陈旧结果整票拒绝（不 open、零 session、零 UI 效果、不覆盖新页）。轮换与重开
先 `close_session` 释放旧 owner 再开新代（不留存活 session）；关闭后 frame 为空，
重开重执行并重新提交显示模型（业务状态延续）。未知 app key 具名拒：零装载、
零执行、代次不动；坏源装载期具名拒且不半挂载，修复后同入口真实重开。
适配器不安装 980 全局 BRIDGE 槽（避免覆盖全局桥遗留旧 owner）。

## 兼容与验证边界

- 1021 无窗口冷加载链（`cargo t ase-m0` 组，V-01..V-05 含 11/22/12 oracle）
  在 1022 树复验 PASS；本链不改 ACE 源、不改 1021 历史冷执行路径。
- 已验 resize/焦点为 OS 级注入档（真实 SetWindowPos/SetForegroundWindow 到真窗，
  持续渲染、内容保持 AV 真值）；真人鼠标/键盘走查为显式 pending 的人工档。
- 页面为静态面（无 model/msg）：页内交互、输入组件、多窗口、3D、后台服务、
  保留状态均不在本域（1023 交互会话承接页内事件接线）。
- HOST 已关闭 session 记录随轮次累积为 980 既有 late 事件靶设计（存活 owner
  结构性单槽）；长期驻留宿主的记录裁剪义务归 980 owner。

## 入口

- UI 宿主/测试/排窗命令与 recipe 冻结于 PLAN-1022 §9（auto-ui
  `ase_m0_text_window` 示例 + `ase_m0_text` 测试目标，排窗
  `AUTO_UI_ASE_M0_E2E=1` 单实例；fixture 根可经 `ASE_M0_TEXT_ROOT` 覆盖）。

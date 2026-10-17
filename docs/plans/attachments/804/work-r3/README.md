# work-r3 附件说明

- `input-state-control.py`：F804-R2-02 要求的子进程退出输入残留反控
  （运行前三键清断言 → 子进程跑 dnd_e2e → 退出后立即与 +250ms 双采样，
  任一残留即 FAIL）。
- 运行方式：`python input-state-control.py <804 worktree 根>`（需
  AUTO 键态清洁的安静桌面；首次会编译 native-dnd 档）。
- 本目录曾保留一份修正前（守卫 RIGHTDOWN/RIGHTUP 常量颠倒 bug 期）的
  争用轮 log——该轮 `FAIL tests: exit=101` 即该自产残留所致，根因已在
  21d5f9c01 修正；为免误读为终态证据已撤下，反控以脚本在复审/门禁时
  现场运行为准。

# 042-two-inputs-child

Plan 483 最小复现 example:**条件渲染的子 widget 内多 input** 的焦点/键盘
投递缺陷载体(上游需求:auto-musk `docs/designs/011`,musk 登录页实测发现)。

## 形态(镜像 auto-musk src/front/login.at + app.at)

- 根 widget `App`:`if store.authed != true { LoginChild } else { text "in" }`
- 子 widget `LoginChild`:两 input(placeholder "Enter username" /
  "Enter password",各自 oninput marker handler)+ `if store.error` 条件
  文本块 + 提交按钮(admin/admin → `AuthStore.SetAuthed(true)`)
- 单例 store `AuthStore`:authed/error

## 复现步骤(修复前,VM 轨,真人键盘)

```bash
auto run -r vm            # 或 auto run --render=vm
```

1. 点击 "Enter username" 框,输入 `admin`(此时正常:单焦点、单投递);
2. **按 Tab 键**切换到 password 框(触发链:text_input 不捕获 Tab →
   `__focus_prompt` → `focus("prompt_input")` → 修复前两框同 Id 全置焦)
   → **复现:两框同时焦点环、光标双闪**;
3. 输入 `admin`(5 键)→ **复现:username 框同步追加,变 "adminadmin";
   状态双污染 user="adminadmin" / pass="admin"**;
4. (顺带)Login 按钮因 user 已被污染而判定失败。

自动化等价断言(无需真键盘):renderer.rs `line_edit_tests::p483_*`
(iced_test,`--features iced-layout-tests`);MCP 驱动:
`autoui_type` 到第二 input 断言仅 pass 变化。

> 历史注记:Plan 483 T1 曾以「视觉取证」记录复现,后经 MCP 状态真值复核
> 确认为视觉模型诱导性幻觉并撤除;复现证据以上述无头测试与 musk 真人
> 实测(上游 011 文档)为准。

## 预期(修复后)

- 点击 password 框 → 仅 password 有焦点环,user 值不变;
- 在 password 输入 admin → 仅 pass 变化;user=="admin" && pass=="admin"
  点 Login → 根条件切换显示 "in"。

## 对照组

- `003-converter`:根级双输入(无条件、无子 widget)——修复前后均正常。

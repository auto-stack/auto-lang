# PLAN-764 验证记录

2026-10-11，Category A，仅官网素材与说明，无 Rust 修改，无 Cargo 测试或 docs_gen。

- 原图 byte-for-byte 与作者提供文件一致，3840×2419，1,003,230 bytes，SHA-256 f1fed8656c2f44f15c1a46bf8488af3561d985ec558518874acdcc061fed3d1c；dist 复制品同 hash。
- npm --prefix website run build：成功，230.63s。已存在 Auto/Autodown/specs 高亮 fallback、bundle size 提示，本次没有修改这些系统。
- AUTO_WEBSITE_TEST_PORT=4225 CI=1 npm --prefix website run test:e2e -- tests/v05-copy.spec.ts tests/apps-introduction.spec.ts --grep（AutoEdit captures / AutoEdit 双语介绍 / 五宽度双主题 / 双语发布页留空保留）--workers=2 --reporter=line：8/8 pass，1.1min。
- 三张 AutoEdit 原图加载，SHOT-03 尺寸、原图链接、Enter 打开、Escape 关闭及焦点回返通过。双语 route/outline/locale/无 Vue 错误和业务请求通过。
- 360/390/768/1024/1440 × EN/ZH × 深/浅主题无横向溢出。新增原图正常呈现，旧两图和其他产品待拍保留。
- 截图证据：p764-comparison/zh-autoedit-comparison-{1440,390}.png。初次证据截图取得图像 metadata 后异步解码尚未绘制；等待 decode 与渲染帧后重新拍摄并目视复核，未修改产品代码。
- 当前截图指南过期“757 两图尚未合入”已纠正；历史 757 的诊断和来源目录字节保留。新图不冒用旧 backend/build identity。
- spec-index.py 成功，26 projects，索引语义无变动。SD-01 和模块回写 hash 冻结 p764-spec-delta.json。
- 同会话复审限制：未另开 agent，按真实图像、提交 diff、hash、既有测试断言和页面截图重建结论。
- 本地预览：http://127.0.0.1:4218/zh/apps/autoedit/ 。Codex 浏览器打开请求因当前 chat 隐藏而 queued，不能声称已在用户前台显示。

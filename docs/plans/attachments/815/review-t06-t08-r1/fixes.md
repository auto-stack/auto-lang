# PLAN-815 T-06..T-08 提前 R1 修复轮交接（2026-10-22）

修复提交 `d039f23995cb7763787b86cd27f4144627d69d3d`（code，12 文件
+767/-179）+ 本 docs 提交（无生产 diff）。与 T-05 R6 两项（已在
`9291a7ee3` 修复并送 R7）同批统一交回后的本批六项 closure：

| Finding | 修复与永久反例 | 当期结果 |
|---|---|---|
| F-815-T06-R1-01（P2 VM 忽略 front 声明） | `util::request_front_dir`（节点形式优先，镜像 parse_workspace_path，`./` 归一）+`PacSource::declared_front_dir`；贯穿 run_vm entry/CWD/`AUTO_SOURCE_DIRS` 种子、run_vue 的 vm-server 种子、tool_ops rust UI 判定、jet/ark/vscode `from_workspace_with_request`（build_front/assemble_front，legacy 探测链与默认 src/front 正控保持） | 回放 profile-paths：`front_decl=./ui/front` 的 VM run **会话真实呈现**（MCP listening+first state sync，原立即 "Frontend entry not found: src/front/app.at"）；vue 正控 17 文件保持；entry 错误携带声明路径 |
| F-815-T06-R1-02（P2 Ark run 假完成） | `assemble_hap` 共享抽取（build/run 同源：wrapper 检查+owned assembleHap+产物 oracle）；托管 run 真实链=构建→hdc 门（缺=具名）→`hdc list targets` owned 捕获探测（新 `run_capture_probe` 助手=R6 捕获契约可复用：非自然结束 kill+具名/非零退出具名/取消哨兵）→`hdc -t <dev> install -r`→`hdc shell aa start`（bundle/ability 读生成产物 AppScope/app.json5、module.json5）；legacy CLI 手工指导 Ok 冻结 | 回放 profile-paths ark 例：RunOk **failed** 具名 "Hvigor wrapper not found … (no build was executed)"（原 completed/110ms/手工打印） |
| F-815-T07-R1-01（P2 假 build 成功 oracle） | lifecycle build oracle 正负分面：成功面=`BuildOk completed+error=None+dist/index.html 真实内容+dist/assets JS 束+引用核对`（任一不满足即失败）；新负控 `lifecycle_build_install_failure_is_named_failed_no_dist`：请求 `pkg:"pnpm"`（raw 单源）+stub PM exit7 PATH→断言具名 failed+无 dist+stub 实被调用 | 回放 lifecycle-oracle：安装确定失败（4 次 install exit7）时成功面测试 **exit101**（原 exit0 PASS）；无 dist 残留；重串行正负 3/3 |
| F-815-T07-R1-02（P2 stderr 不排水堵死） | 辅助器重写：stderr 专职排水线程（收集日志）+stdout 通道化 reader+`next_response_bounded`（30s 界，超时 kill owned 树+panic 带日志尾巴）+Drop 可靠清理 | 回放 lifecycle-noisy-log：131072B 噪声+安装失败 **1.19s 收口**（原 12s 外层截止未完成）；静默对照同步收口 |
| F-815-T07-R1-03（P2 闭包门漏判） | verify-deps.sh **deny-by-default** 重写：closure 内全部 `auto-*` 默认拒绝、白名单精确四包（auto-tool-protocol/auto-project-model/auto-val/auto-atom=实际数据域闭包）；修原 `auto-x `+` v` 双空格匹配缺陷 | 回放 closure-negative：无注入正控 PASS；**auto-ui-dialect 注入 exit1**（连带 auto-ast/lex/parser-api/diagnostics 全具名）、**auto-bindgen 注入 exit1**（直接具名）；cargo tree exit0 确认真实入图 |
| F-815-T07-R1-04（P3 fixture 卫生） | tool-protocol-consumer lib.rs rustfmt+import 全量 `#[cfg(test)]` 门拢+未用 Budgets/ErrorKind/test_request 死码处理（test_request 实为在用——恢复为 cfg(test) 项）；lifecycle 未用 import 清理 | 两 fixture 目标 **0 告警**、格式相等、协议 consumer 5/5 保持 |

## 附带收口

- **777 stale_baseline 受管重采样**（T-07 表列待办）：对最终提交
  `d039f2399` 干净树 `--refresh-baseline`（base_revision=d039f23995cb、
  declared_edges=121 不变——libc 为 cfg(unix) 无新声明边、text_scan
  250/0/94 保持）；strict 复跑 **pass violations=0 grandfathered=3**
  （在案三条不变）。
- T-08 handoff 声明校正：指纹（源 d039f2399/二进制 385ffef8）+新增
  "run 前置与执行语义"（ark 真实链/前置缺失具名 failed/front 声明贯穿），
  与实现逐项对账。

## 当期验证账

- 回放：profile-paths（VM 呈现/ark 具名/vue 正控）+lifecycle-oracle
  （oracle 翻转）+lifecycle-noisy-log（1.19s 收口）+closure-negative
  （双注入具名拒）全闭；产物在 fix-evidence/。
- scoped：lifecycle 重串行 **3/3**（含新负控）、tool_process 21/21+
  重串行 8/8、lib 175/179（4 预存在案）、legacy 2/2、consumer 5/5、
  闭包门 PASS、fmt 全净、fixture 零告警；WSL Linux pkg/proc_host 3/3
  保持。
- 测试时序裁定遵守：未整档；merge 门 pending。T-07 完整 777 并集外的
  792/fresh/locked 发行、资源审计与 V-01..04 全矩阵仍为待办（本轮未做，
  不冒称）。

# PLAN-729 T-08 验证报告（门禁 + AC/SD 绑定）

- 最终代码 revision：worktree `D:/autostack/.wt/lang-729/auto-lang` 分支 `plan-729-dev`
  @ 8eecd9f0d（T-01 67e194661 → T-02 861962d99 → T-03 500aa757d → T-04 f1bd17655 →
  T-05 4f35bd1ce → T-06 7b1738d10 → T-07 8eecd9f0d + 本报告/规范稿提交）
- 基线：master `e5b068bd0a`（实施起点；727 landed@6eb396e5a 在其内）

## 1. 门禁记录（命令 → 结果）

| 门禁 | 命令 | 结果 |
|---|---|---|
| 快速 | `cargo check -p auto-lang` / `-p auto-man --lib` / `-p a2r-std` | ✅ 0 error；新文件 0 警告（仓库存量警告不变） |
| scoped 日常 | `cargo t plan729` | ✅ 19/19 |
| a2r-std | `cargo test -p a2r-std -- --test-threads=1` | ✅ 76+7+6+0（1 ignored 存量） |
| auto-man | `cargo test -p auto-man api_gen:: -- --test-threads=1` | ✅ 33+1（含 3 新文件分支测试） |
| VM 真 TCP | `cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729` | ✅ 9/9 |
| 生成服务实编 | `cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan729 -- --test-threads=1` | ✅ 1/1（真实生成产物 cargo build + wire） |
| **裸 `cargo t`** | `cargo t --no-fail-fast` | 5025 测，5011 过，**14 红** |
| `cargo tv` | `cargo tv` | ✅ 162/162 |
| `cargo tt` | `cargo tt` | 1062 trans 例绿（见 §2 预存 flake） |
| `cargo th` | `cargo th` | 30+/…（见 §2 预存 2 红） |
| 格式 | `rustfmt --check`（新文件） | ✅ 清洁（已格式化） |
| 调试输出扫描 | 新增代码无 eprintln/println 残留（测试内 eprintln 为 SKIP/诊断记录面，与既有测试惯例一致） | ✅ |

**tf 未跑**：批量回归档（fix-test-tiering 2026-09-30 裁定非 per-plan 门禁；merge 到期后
由 `/auto-plan:regress` 主检出单实例执行）。

## 2. 预存红逐名对照（零新增确定性红）

- 裸 `cargo t` 14 红（本分支）vs master `e5b068bd0a` 同命令 14 红——**名称逐一相同**：
  musk_vm_track p053_1×2 / p053_4 / p053_6 / p054_t1 / p054_t4（6）、plan606 gallery
  029（1）、projector_counter（1）、desktop bus/surface（2）、ash_leak_probe（1）、
  queue_coverage_drift_fence（1）、kitchen_sink_page_in_sync（1）、schema_drift_fence（1）。
  本分支 5025 测 = 基线 5006 + 19 新 plan729（全绿）。
- `cargo th` 2 红在 master 基线同命令同样复现：back_proxy corpora data face
  （`/api/books/:id/chapters` 字面 `:id` 400——测试 fixture 侧，与 PLAN-729 的
  back_proxy 改动无关：该请求不含 FileResponse 面）+ plan707 relay 帧时序
  （4f123a50e 在案四红之一 flake）。
- `cargo tt` 曾见 `lock_serializes` / `merged_api_warning` 闪红——PLAN-716 收据在案
  基线 solo 复现预存；复跑 tt 时 1062 trans 例绿。
- `cargo check -p auto-lang --no-default-features` 的 4 个 frame_bench 引用错误为
  master 预存（PLAN-716 的 `crate::ui::frame_bench` 未门控引用）；本计划未改依赖/feature，
  条件门禁不触发。
- `cargo check -p auto-man`（bin target "main not found"）：main.rs 预存空壳；lib 门禁
  （`--lib`）清洁。

## 3. AC 绑定（可观察交付 → 证据）

| AC | 证据 |
|---|---|
| AC-01 公共声明/VM+两 facade 同形/真实 api.at 二进制返回 | http.at/http.vm.at 声明；shim 9936 + ApiBody::File + 声明门；trans lowering + a2r_std 转发壳；VM e2e basic/12MiB hash + 生成服务实编 e2e（非 JSON/base64）；plain-int 反例绿（method_and_int_guard） |
| AC-02 GET/HEAD/单区间/条件子集 | §6.1 双端 wire 表全行（protocol 报告）：零文件不变 1（basic）、u64 无截断（range u64 大值 416）、If-Range 失配完整重下（VM+生成）、字节窗精确断言 |
| AC-03 root 限制/头值安全/同句柄 | T-01 平台探针（junction 伪装/设备句柄/rename 钉住/截断 EOF）+ 单元 junction 403 + e2e 路径安全族（traversal/编码/双重编码/junction 404/403，无主机路径泄露）+ auth 先于打开（打开在 transport，晚于 owner 全部段）+ 截断→发送失败帧（单元+探针） |
| AC-04 有界/不占 owner | owner 零 I/O（构造零 I/O + serve 在 transport/生成 handler 的 tokio 上下文）；N+Q+1 队满 503+Retry-After（quota e2e + 时间戳证据）；排队期零句柄（queued 探针）；块/队列峰值=pump 读驱动 ≤2 块公式；慢下载时 plain 端点 200（quota 恢复段）；无每请求线程/runtime（pump/watchdog 任务） |
| AC-05 恰一次回收/半关闭 | finish hook 恰一次（单元 Completed 单条）；断连 active→0（slow_client e2e + 单元 client_gone）；取消→server 资源退出（727 互通取消臂）；headers 后故障不假成功（截断发送失败帧）；不 poll 的 body 由独立 watchdog 收口（idle 覆写旋钮在 serve 面就绪；60s 默认）；排队到期出队（30s 准备期，quota e2e） |
| AC-06 真实生成 handler/互通/不支持形态 | 生成服务实编 e2e（真实产物 cargo build + wire 全绿）；转译失败不 fallback（api_gen 测试：诊断 500，无模板断言）；TS Response 不 `.json()`（to_ts_type + return 路径）；IPC/merged/back-proxy/legacy 明确拒绝（tauri Err 诊断生成测试 + back_proxy 501 e2e 面代码 + legacy 500 诊断 + Server.static 占位不变）；727 五态互通（完整/206 续传/失配 200/416 保旧/取消）双端；JSON/SSE/媒体 scoped 回归（http_server 42 + vm::ffi::http 44 + api:: 30 + api_gen 33 绿） |

## 4. SD 绑定（规范稿就位）

| SD | 文件 | 状态 |
|---|---|---|
| SD-01 add | `docs/specs/stdlib/design/http-server-files.md` | worktree 全文就位（§1-7：公共面/options/协议/打开/配额/矩阵/单源） |
| SD-02 modify | `stdlib/design/http-server.md` §12 | ✅（支持面+过时 ureq 注记清理） |
| SD-03 modify | `stdlib/design/backend-assembly.md` + `stdlib/project.md` | ✅（文件单源+两代 axum 投影+不支持形态） |
| SD-04 modify | `stdlib/design/http-handler-async-lifecycle.md` | ✅（文件 body scope 代持+两期限+FS 收口） |
| SD-05 modify | `a2r-std/project.md` | ✅（server_file 模块定位：构造面+纯决策，不启动服务器） |
| SD-06 add | `auto-lang/trans/design/http-file-response-lowering.md` | ✅（构造/类型映射/生成消费/门） |
| SD-07 modify | `auto-lang/trans/overview.md` + `auto-man/project.md` | ✅（发射/生成分支现状行） |

canonical ledger（`.autoos/specs.json`）与 specs 索引刷新按范式归 **merge** 技能。

## 5. 平台覆盖

- Windows 11（本机）：全部 wire/单元/生成 e2e（junction 实链；真 symlink 无 dev-mode
  → 环境缺项如实记录——std 同一 `is_symlink` 判定路径覆盖 reparse 类）。
- Linux：CI `http-e2e-ci.yml`（push/PR）跑同一 e2e 族（lstat/no-follow 语义）+ 文档
  声明的平台限制（决策报告 §2.5）。

## 6. 遗留/债候选

- 生成服务 e2e 首编冷态 ~14min（共享 worktree target 后 ~10s）——CI 冷缓存成本记录在案。
- idle watchdog 的 60s 默认未在 wire 级演练（时长成本）；收口语义经 idle 覆写旋钮 +
  watchdog 代码路径审查 + 断连/取消 e2e 覆盖。如需 wire 级长时验证，归后续批量档。
- `cargo th` 的 back_proxy corpora 红为测试 fixture 侧字面 `:id`（预存，建议归
  KNOWN-DEBT 或修复计划）。
- shutdown drain 的文件 body 窗语义沿 serve_with 现状（未做专项 wire 演练——scope
  收口臂与 SSE 同形已由共用路径覆盖）。

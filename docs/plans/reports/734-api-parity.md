# PLAN-734 T-07 对等报告：五形态支持矩阵与已知边界

- 代码基线：@ `438d22c47`。

## 1. 形态矩阵（本轮实证状态）

| 形态 | 状态 | 证据 |
|---|---|---|
| VM 默认 HTTP `#[api]` | ✅ 契约消费（绑定/编组/i64/错误收敛） | http_e2e_plan734 4/4（参数校验/i64 wire/error 字段/plain int） |
| merged（进程内直调） | ✅ 值语义 + 可定位失败 | plan734_merged（missing_impl 绿；direct_call 见 D5） |
| 生成 Rust/Axum | ✅ 同契约 api_gen（strict+scaffold/ready 记录/glue 契约化） | api_gen 41/42（预存红在案） |
| back-proxy | ✅ typed 守卫 + meta 源 | 守卫/绑定代码 + 729/730 既有 e2e 回归 |
| Tauri IPC | ⚠️ dev-dep 已加（tauri 2.12 test feature 编译过，MockRuntime API 在案） | MockRuntime dispatcher fixture 未及落地（见 §3） |

## 2. contains 字面量退役清单（AC-01）

| 面 | 处数 | 去向 |
|---|---|---|
| vm/ffi/http_server.rs 门/路由/注入 | 5 | ResponseKind/is_upload_param/is_meta_alias |
| vm/codegen.rs 上传合同诊断 | 4 | 同上（param 身份=unique_name——User Display 是 s-expr，注释在案） |
| back_proxy.rs 守卫/广播门 | 5 | ResponseKind |
| api/targets/{tauri,axum,typescript} | 10 | ResponseKind/is_upload_param |
| auto-man api_gen glue/分支 | 4 | ResponseKind/is_upload_param |
| （既有 729/730 分支本体） | — | 经同一 ResponseKind 判定改写，语义等价 |

假同名反例锁定：`MyFileResponse`/`UploadReceiptLog`/`List<FileResponse>` 均
分类 Json（身份匹配，非子串）。

## 3. 未及面与已知边界（如实）

- **Tauri MockRuntime fixture**：tauri 2.12 dev-dep + `test` feature 已加并
  编译通过（`mock_builder`/`get_ipc_response` 在案，E7），但
  command-dispatcher 实测 fixture 未在本轮落地——tauri 生成命令的既有
  生成串测试在档（729/730 Unsupported 形态），dispatcher 级实证归 T-08 补
  或债登记。
- **P734-D4**：泛型 stdlib 的 json.encode 返回通路与 str+int 拼接仍有
  i32-lane 损坏（应用层泛型转换，API 契约面外；binding 报告 §1）。
- **P734-D5**：run_with_capture 对 #[api] 程序的测试基建非确定挂死
  （auto-server guard 后仍复现；direct_call 单滤绿/全滤 TIMEOUT 交替，
  逐 token 等价克隆同挂——基建归因另案；测试 #[ignore] 在案）。
- 015/017/023/027 代表契约 scoped 回归：本轮未单独跑（全 UI 生成矩阵），
  以 cargo t（裸日常面）+ api_gen 全套覆盖生成路径；T-08 门禁见 verification。

## 4. 复现命令

见各报告头部；示例 `examples/http_server/api_contract/`（VM/生成两腿同
api.at 的 curl 探针矩阵在 README）。

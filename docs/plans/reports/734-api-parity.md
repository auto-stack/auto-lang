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
| （既有 729/730 分支本体） | 8 | R1 F-1 修复后全部经 ResponseKind/is_upload_param 判定改写（此前 8 生产位点残留被复审捕获——api_gen ×6 + typescript ×2；语义等价，假同名反例自此闭合） |

假同名反例锁定：`MyFileResponse`/`UploadReceiptLog`/`List<FileResponse>` 均
分类 Json（身份匹配，非子串）。

## 3. 未及面与已知边界（如实）

- ~~Tauri MockRuntime fixture~~ **R1 F-3 已清偿**：dispatcher 实测落地
  （`plan734_tauri_mock_dispatcher_invokes_generated_command`——mock_builder
  + generate_handler! + get_ipc_response 全链，Echo 业务值往返断言）；
  配套 fixture 锁测试绑定 TauriGenerator 当前输出与提交的 fixture 逐字一致。
  注意：tauri dev-dep 必须 `default-features = false, features = ["test"]`——
  默认 features 引入 wry/WebView2Loader 使测试二进制在本机
  STATUS_ENTRYPOINT_NOT_FOUND（R1 实证）。
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

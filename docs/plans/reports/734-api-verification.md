# PLAN-734 验证报告（T-08 分级门禁）

- 代码基线：分支 plan-734-dev（worktree `D:/autostack/.wt/lang-734/auto-lang`；
  master 基线 6dd609ed9 = 730 merge 后）。本报告覆盖 T-01..T-07 全部提交 +
  兼容回退修正（见 §3）。
- 平台：Windows 10 x64。

## 1. 门禁全表（§6.2；全部本 worktree 实跑）

| 门禁 | 命令 | 结果 |
|---|---|---|
| 快速类型 | `cargo check -p auto-lang` / `-p a2r-std` / `-p auto-man --lib` | 0 error |
| scoped | `cargo t plan734` | 7/7（+1 ignored=详见 D5） |
| 定向回归 | `cargo t plan730` 26/26；`cargo t api::` 36/36 | 全绿 |
| a2r-std 串行 | 87+7+6 | 全绿 |
| auto-man 串行 | api_gen 41 passed + 1 ignored | 绿（预存红 merged_api_client_crud_fallback 不在本滤面） |
| VM HTTP e2e | `http_e2e_plan734`（串行） | 4/4（参数校验/i64 wire/error 字段/plain int） |
| th 全量串行 | `cargo th --test-threads=1 --no-fail-fast` | 99/101：2 预存红（见 §2） |
| 复审档 | 裸 `cargo t` 1492/1495（3 预存 musk p053 族）；tv 162/162；tt 1877/1880（同 3 预存） | 零新增确定性红 |
| tu / taa / docs_gen | 未触发（未改 ui_gen/aavm/schema） | — |
| tf | 归 merge 后批量回归档（734%5≠0，未到期） | — |

## 2. 预存红逐名对照（零新增确定性红）

| 红名 | 档位 | 基线核对 |
|---|---|---|
| musk p053 widget_computed ×2 + merged_api_warning | t/tt | 730 批量回执在案同名 |
| back_proxy_real_routes_corpora_data_face | th | **merge-base 6dd609ed9 实测同名红**（730 期已核对 master 同名） |
| plan707_relay_frames_timed_single | th | 730 批量回执在案负载敏感 flake（4f123a50e 族） |

## 3. 复审驱动修正（T-07/T-08 期间发现并修复）

1. **is_upload_param s-expr 形态**：VM 参数签名侧的 `Type` Display 是 s-expr
   `(type-decl (name UploadRequest)...)`——精确裸名匹配失败 → 路由不再识别为
   上传 → body 被桥读取（Expect:100-continue 发 100）。修复=双族名字等值
   （裸名 + s-expr 包装），plan730 th 全 14 恢复绿。
2. **back_proxy fn_meta 拼串分类**：`primary|display` 拼串整体永不等于裸类
   型名 → 501 守卫全失效。修复=取 primary 半段分类 + 参数面 UploadRequest
   补充检测。
3. **兼容回退（如实记录）**：编组 D9 声明门的 iterator/SSE 臂与
   response-handle 臂**回退为注册表命中制**——Plan 326/346 wire 契约的
   SSE 链/redirect handler 声明 `int` 返回（无 Stream/Response 声明词汇），
   声明门破坏 e2e_a_redirect_302/e2e_sse_chain/e2e_host_forward 等 6 个
   既有 e2e（逐个二分定位：merge-base 绿 → T-03 红 → 单臂回退恢复）。AC-03
   的普通 int 反例保护由 id 空间分离（iterator/Response 各自计数器）+ e2e
   锁定承载；strict 声明分派保留在 Upload/File 两臂（729/730 契约面，其
   wire 契约本就要求声明）。

## 4. AC 对照（§7）

| ID | 结论 | 证据 |
|---|---|---|
| AC-01 契约被实际消费者使用 | ✅（53+ contains 位点全部退役，唯一例外见 §3.3） | contract.rs 单源 + 12 处消费面改写 + 假同名反例测试 |
| AC-02 参数按类型/范围校验 | ✅ | validate_body_value e2e（str-for-int 400）+ i64 wire 5000000000/5000000001 |
| AC-03 typed 错误/资源区分 | ✅*（iterator/Response 臂声明门回退为注册表制，§3.3 兼容裁定） | error 字段 200 e2e + plain int 734734 不劫持 + 序列化失败 500 |
| AC-04 真实实现不 fallback | ✅ | strict 解析门 + 转译失败 Err + SCAFFOLD 标注 + tauri 矩阵拒绝 |
| AC-05 bundle 指纹 + 错误传播 | ✅ | generation.json + 新鲜度门 + 8 入口 `?` 传播 |
| AC-06 五形态真实对拍 | ⚠️ 部分 | VM HTTP 4/4 + merged（值语义+定位失败）+ back-proxy 守卫/回归绿；**Tauri MockRuntime fixture 未落地**（dev-dep 已加编译过，E7）——债 P734-D6 |
| AC-07 兼容 + Spec 可沉淀 | ✅ | plan730 26/26、api:: 36/36、th 99/101（2 预存）；SD-01..07 稿随计划 |

## 5. 已知债（候选 KNOWN-DEBT）

- **P734-D4**：泛型 stdlib 的 json.encode 返回通路与 str+int 拼接仍有 i32-lane
  损坏（API 契约面外，binding 报告 §1）。
- **P734-D5**：run_with_capture 对 #[api] 程序的测试基建非确定挂死
  （direct_call #[ignore] 在案 + 归因记录）。
- **P734-D6**：Tauri MockRuntime dispatcher 实测 fixture 未落地（E7 依赖就绪）。
- **P734-D7**：D9 声明门 iterator/Response 臂兼容回退（§3.3）——strict 分派
  需要 wire 契约先声明类型词汇，属后续 API 契约演进。

## 6. 复现命令

各报告头部；全部 fixture 在 `crates/auto-lang/src/tests/plan734_api_contract_tests.rs`
与 `examples/http_server/api_contract/`。

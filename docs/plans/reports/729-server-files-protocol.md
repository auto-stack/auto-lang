# PLAN-729 T-06 协议/根目录/兼容回归报告

- worktree `D:/autostack/.wt/lang-729/auto-lang` @ 7b1738d10（后续 T-07 修正在案）
- 命令：`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729`（VM 腿，9/9 绿）
  + `cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan729 -- --test-threads=1`（生成腿，1/1 绿）
- fixture：确定字节（文本/NUL+非 UTF-8/零字节/12MiB 模式生成 blake3 对拍/0..=199 窗口）。

## 1. §6.1 双端 wire 表逐行结果

| 族 | VM 默认 HTTP | 生成 Rust（axum 0.7 实编） |
|---|---|---|
| 基本：文本 GET | ✅ 200/CT=text/plain/CL=17/Accept-Ranges/Last-Modified/无默认 ETag | ✅ 同形（hello generated） |
| 基本：NUL+非 UTF-8 | ✅ 字节保真（CT=octet-stream） | ✅ 同 |
| 基本：零字节 | ✅ 200/CL=0（非 1） | ✅ 同 |
| 基本：HEAD | ✅ body=0/CL=GET 表示长/Range 忽略/CORS 追加 | ✅ 同（get(h).head(h) 链式） |
| 12MiB | ✅ CL=12582912/blake3 hash 一致/HEAD CL 一致 | —（生成腿以 200B 窗口矩阵覆盖同断言面） |
| Range bounded 10-19 | ✅ 206/Content-Range bytes 10-19/200/CL=10/字节窗 | ✅ 同 |
| Range open 190- | ✅ 206 到 EOF | —（纯决策层已表驱动） |
| Range suffix -7 | ✅ 206 尾窗 | — |
| Range end 越 EOF 90-99999 | ✅ 206 截至字节 90-199/200 | — |
| Range suffix>len -5000 | ✅ 206 整文件 0-199/200 | — |
| Range -0 | ✅ 416 + bytes */200 + 空体 | ✅ 416 |
| Range 起点=EOF 200- | ✅ 416 | — |
| Range 多区间/坏单位/坏语法 | ✅ 忽略→200 全量（4 形） | — |
| Range u64 大值 | ✅ 416（不读数 GiB） | — |
| 条件 If-None-Match 命中 | ✅ 304/无 body/ETag 回显 | ✅ 304 |
| 条件 弱比较 W/v1 | ✅ 304（If-None-Match 允许） | — |
| 条件 失配/通配 | ✅ 200 / 304 | — |
| 条件 If-Match 失配/弱形态/无 etag | ✅ 412×3 | ✅ 412（失配） |
| 条件 If-Modified-Since 未来/过去 | ✅ 304 / 200 | — |
| 条件先于 Range | ✅ 304 | — |
| If-Range etag 匹配/失配/弱 | ✅ 206 / 200 全量 / 200 全量 | ✅（失配 200 全量） |
| 路径 ../（URL 编码） | ✅ 403（不泄露主机路径） | ✅ 403 |
| 路径 绝对（%2F） | ✅ 403 | — |
| 路径 双重编码 | ✅ 404/403（单次解码后字面 %2e 不匹配磁盘） | — |
| 路径 目录/缺失/深缺失 | ✅ 404×3（多段路由命中后走文件路径） | ✅ 缺失 404 |
| 路径 junction 中间段 | ✅ 403（walk 检出，不读到 ESCAPED；mklink /J 实链） | — |
| 方法 POST 文件注解 | ✅ 405 + Allow: GET, HEAD | —（生成期诊断+405 handler，api_gen 单测） |
| 声明文件返回 int | ✅ 500 诊断（不 JSON 200） | — |
| 普通 int | ✅ JSON 200（撞号不误判） | — |

注：304 的 Content-Length 被 hyper 剥离（wire 现实；表示长度语义由 200/HEAD 承载，决策报告 §4 注记）。

## 2. 兼容回归

- 既有媒体 Range（`ui::media_service::parse_range`）与普通 JSON/SSE：未改语义；
  `cargo t http_server`（42 绿）+ `cargo t vm::ffi::http`（44 绿）+ `cargo t api::`（30 绿）
  + `cargo test -p auto-man api_gen::`（33 绿，含 3 个新文件分支测试）。
- a2r golden：`32_plan729/001_http_file_response`（trans lowering/类型映射冻结）；`cargo tt`
  1062 trans 例绿（唯一红=预存 flake `lock_serializes`/`merged_api_warning`，PLAN-716 收据在案基线）。

## 3. 环境记录

- Windows 11（win32 10.0.26200）：junction 实链验证 ✓；真 symlink 无 dev-mode 权限（SKIP 记录，
  std 同一 `is_symlink` 判定路径覆盖 reparse 类）；Linux 腿由 CI（http-e2e-ci.yml）覆盖 lstat 语义。
- e2e 端口：VM 腿 18950-18958（18731-18775 已被既有 http e2e 占用）+ 生成腿 18960。

# PLAN-736 T-07 报告：受控负载与资源曲线（AC-03/05/06/08, SD-01/02/03/06）

> 环境（T-01 冻结，未事后放宽）：Windows 11 26200 · x64 · 20 逻辑核 · 32,581 MiB ·
> 单机单实例**顺序**执行（不与他负载并跑）· release fixture（`cargo build -p auto
> --release` + 生成轨 `cargo run --release`，冷编译时间排除）·
> 探针 `scripts/http-service-probe.py`（python 3.14 标准库，全部有界并发+deadline）·
> 后端=deployment fixture（proxy_service profile, 18500；超载门专用 32/8 帽子版 18501）·
> 日期 2026-10-08 · 原始 JSONL 全文见本报告各表。

## VM 轨（进程内 AutoVM）

### json（16 并发 × 10,000；零意外非 2xx；p95≤1s）——**3/3 PASS**

| 轮 | 请求数 | 非预期 | p50 | p95 | p99 | RPS |
|---|---|---|---|---|---|---|
| 0 | 10000 | 0 | 2.6ms | 12.8ms | 25.3ms | 4554.5 |
| 1 | 10000 | 0 | 2.5ms | 3.9ms | 4.7ms | 6142.5 |
| 2 | 10000 | 0 | 2.6ms | 4.0ms | 4.9ms | 6012.0 |

### overload（conn cap=32 / inflight cap=8；压 128 占位连接+64 burst）——**3/3 PASS**

| 轮 | 占位 | burst | served | cap 关闭 | 真错误 | 恢复 |
|---|---|---|---|---|---|---|
| 0-2 | 128 | 64 | 0 | 64/64（解析前关闭，合同语义） | 0 | 0.02s ≤5s |

### mixed（2 SSE 挂住 + 2 慢下载 + 2 上传 + 8 JSON ×60s）——**PASS**

json 8134 req p95 24.4ms · health 283 req p95 23.5ms（独立额度表）·
uploads 379（唯一名 201）· downloads 566 · errors 0。

### resources（同进程混合/取消周期；漂移≤64MiB、峰值≤512MiB）——**PASS**

idle RSS（MiB）：42.4→45.3→50.2→56.7→57.5→59.6（**6/10 周期有效样本**——4 次
OpenProcess 瞬断未采样；判定在周期 2..6 集合成立）· 漂移 **14.3MiB** ≤64 ·
峰值 **59.6MiB** ≤512 · 余量充足（峰值上限的 12%）。
观察：漂移单调缓升 ~2-3MiB/周期，未见平台化——量级远低于合同阈值，登记为
观察项（ allocator 面推断，非泄漏判据；后续批量回归可复核）。

## 生成轨（axum release fixture）

### json——**3/3 PASS**

| 轮 | 请求数 | 非预期 | p50 | p95 | p99 | RPS |
|---|---|---|---|---|---|---|
| 0-2 | 10000×3 | 0 | ~2.4ms | ~3.9ms | ~4.7ms | ~6200 |

### overload——**3/3 PASS**（64/64 cap 关闭、0 真错、恢复 0.03s）

### mixed——**PASS**

json 8351 req p95 23.9ms · health 287 req p95 22.6ms · uploads 380 ·
downloads 573 · sse_holds 2（订阅腿满时长保持）· errors 0。

### resources——**PASS（10/10 周期全采）**

idle RSS：14.5→14.3→13.4→15.0→12.5→12.5→12.7→12.7→13.0→15.2 ·
漂移 **2.7MiB** · 峰值 **15.2MiB**。

## 探针修正记录（诚实面）

1. Windows cap-close 形态=WinError 10053/10054/10061 族——归 `closed_by_cap`
   （合同 §5.2"连接满解析前关闭，不承诺 503"），非真错误。
2. 上传腿唯一文件名（730 create-only；409=预期冲突不计错）。
3. rust 轨 SSE 订阅腿超时归 `sse_hold`（无发布者=734 支持面边界，连接保持满
   时长即载荷目的）。
4. mixed 的 PASS 判据=json/health p95 与零意外错误；上传/下载腿计数为吞吐
   证据（hash 一致性由 T-06 代理 fixture 与 730 契约测试承载）。

## 合同边界重申

小型参考服务本机门——不证明任意应用吞吐或公网安全；CPU/native 无强抢占；
本报告不冒称未测面（跨 OS 实机/多层代理/直接公网）已验证。

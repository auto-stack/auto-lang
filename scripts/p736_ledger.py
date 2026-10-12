"""PLAN-736 merge ledger projection: specs.json P736-1/2.

Mirrors the P685 entry schema exactly (validated offline projection precedent);
indent-1 serialization verified by diff (existing bytes must not churn).
Source of truth: docs/specs/stdlib/design/http-service-deployment.md 等
（SD-01..06 已随 a80f35984 落 canonical）。
"""
import json

SPEC = ".autoos/specs.json"
ARCHIVE_PATH = "docs/plans/archive/736-http-service-deployment-and-operations.md"

with open(SPEC, encoding="utf-8") as f:
    data = json.load(f)

sec_design = sec_review = None
for i, sec in enumerate(data["sections"]):
    ids = {it.get("id") for it in sec.get("items", [])}
    if "P735-1" in ids:
        sec_design = i
    if "P735-2" in ids:
        sec_review = i
assert sec_design is not None and sec_review is not None, "anchor sections not found"

existing = {it.get("id") for sec in data["sections"] for it in sec.get("items", [])}
assert "P736-1" not in existing and "P736-2" not in existing, "P736 entries already present"

p736_1 = {
    "id": "P736-1",
    "title": "http-service-deployment-and-operations — HTTP 服务部署基线：独立启动、配置与资源预算、观测和关闭验证（SD-01..06）",
    "content": (
        "交付等级=本机开发+受单层反向代理保护的 HTTP 服务（736 不宣称直接公网/内置 TLS）。"
        "契约=docs/specs/stdlib/design/http-service-deployment.md（SD-01 新件，a80f35984 沉淀）："
        "①入口 `auto service --server vm|rust --http-config`（server-only 无 UI；`auto serve` "
        "让位 Plan269 daemon）；②配置严格 JSON（deny_unknown_fields/重复/越界指名非零退出）+"
        "development/proxy_service 双 profile 缺省+每字段来源记录+effective_config_hash（与 734 "
        "generation hash 分离）；③预算：连接 cap（满=解析前关闭+conn_rejected，不承诺 503）/"
        "inflight 503+Retry-After/permit 持至 body 终态（生成轨经 response extensions）/"
        "头 64KiB+100/慢头 10s/body 10MiB+10s 语义由配置冻结，file/upload 期限归 729/730 专属；"
        "④策略纯函数单源（http_service_config）：CORS preflight/精确 origin/Vary/"
        "*+credentials 拒、Host 精确表（hostname-only，X-Forwarded-Host 不放宽）、单层可信代理"
        "（peer 精确命中才消费单段 XFF/XFP，覆盖语义）、有界限速器（桶容量 4096 默认+TTL，满表"
        "新身份保守 429）；⑤观测：身份四元组（instance_id/bound/profile/config_hash）三消费点"
        "同源、/__auto 控制面（live/ready 开放、snapshot/shutdown 仅 loopback）、JSONL 有界 sink"
        "（队满丢弃+log_dropped）、请求终态恰一次（scope finalize 单点收敛）；⑥关闭：Starting→"
        "Ready→Draining→Stopped，shutdown 端点与 Ctrl+C 同一 watch，排空超期强制收口，端口"
        "释放重绑；⑦支持等级：Windows 实测+单 nginx 1.31.6+测试 CA；未测面（跨 OS/多层代理/"
        "直接公网/CPU 抢占）逐项披露。生成轨边界：route-A 门内普通 JSON 端点桩体（P670-D1 域）、"
        "转译 POST 自动广播不在 734 支持面（上传/下载 729/730 胶水=真实体）。实现="
        "http_service_config.rs+http_service_observability.rs+auto-man/http_service.rs+"
        "auto service 命令+VM 桥层策略链+生成轨 SERVICE 运行环（hyper-util）+rust_ui health "
        "探针。负载门双轨四类全 PASS（json p95≤24ms/超载 64 连接 cap 关闭恢复 0.02s/混合 "
        "IO 零错/资源漂移≤14.3MiB 峰值≤59.6MiB）；P729-D1 drain 窗在途文件体 e2e 补齐。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-736"],
    "status": "published",
    "created": "2026-10-08",
}
p736_2 = {
    "id": "P736-2",
    "title": "http-service-deployment-and-operations 复审与合并收据",
    "content": (
        "review R1 needs_fix→R2 pass（同会话复审已声明——裁定全部工件重建，独立性限制随档）。"
        "R1 findings（R2 收口）：R2 脏面=rustfmt 覆盖层 byte-identical 实证+verification 报告"
        "入库（1336b3025）；R1 簿记修正（父侧身份核对措辞）。R1→R5 登记 KNOWN-DEBT（R3 "
        "eprintln 原始 path 出 sink 合同外、R4 生成轨 cosmetic、R5 ledger 通道）。门禁：裸 "
        "cargo t --no-fail-fast 失败集=master 基线逐项一致（16 预存族，零新增确定性红）；th "
        "2 红=在案预存；tv 162/162；plan736 11/11（含 P729-D1 真 TCP drain e2e）。实机：双轨"
        "真实 nginx 1.31.6 代理 PASS（vm 全矩阵含 SSE 帧经 TLS 代理；rust upload/download/"
        "Range 真实体，SSE/ping 桩体按 734 支持面边界记录）；双轨负载四门 PASS（release "
        "fixture，raw 全录 reports/736-http-load.md）。合并：两次 rebase（并发移动 885efdefe→"
        "a0e50b614）后 ff-only，master tip=a80f35984；代码 tip 864dccf41 vs 复审绑定 6156c3de3 "
        "range-diff 全 `=`。ledger=validated offline projection（p685 先例）P736-1/2 本条。"
        "债：P736-R1..R5；观察项：698 SSE 订阅会话首播后 ~1-2s 收口（帧竞速）、VM resources "
        "RSS 单调缓升（59.6MiB，远低阈值）。"
    ),
    "file": ARCHIVE_PATH,
    "related": ["PLAN-736"],
    "status": "published",
    "created": "2026-10-08",
}
data["sections"][sec_design]["items"].append(p736_1)
data["sections"][sec_review]["items"].append(p736_2)

with open(SPEC, "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=1)
    f.write("\n")

print("specs.json sections:", sec_design, sec_review,
      "| items now:", sum(len(s.get("items", [])) for s in data["sections"]))

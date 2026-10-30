"""Map the frozen PLAN-890 source items and historical test obligations.

This is an intake map, not proof of execution or final physical package layout.
The feasibility task freezes package names and cuts; later migration compares
these original identities rather than regenerating a smaller obligation set.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def source_owner(path: str, symbol: str) -> tuple[str, str]:
    if "/auto-man/" in path:
        return "man", "consumer-glue; delegate service implementation to Web"
    if "/auto-ui/" in path:
        return "ui", "consumer-glue; use Web HTTP exits, retain non-HTTP behavior"
    if path.endswith("/http_service_config.rs"):
        runtime = ("RateBucket", "RateLimiter", "RateDecision", "ServiceRuntime", "ACTIVE_SERVICE_CONFIG",
                   "SERVICE_POLICY_ACTIVE", "active_slot", "set_active_service_config",
                   "take_active_service_config", "set_service_policy_active", "service_policy_active")
        if any(name in symbol for name in runtime):
            return "web", "service runtime, admission state or active configuration assembly"
        return "web-contract", "pure configuration/validation/policy; shared single-source contract"
    if any(part in path for part in ("/http_file_service.rs", "/http_upload_service.rs",
                                    "/http_service_observability.rs", "/http_transport.rs")):
        return "web", "framework protocol, lifecycle or service implementation"
    if "/auto-api-gen/src/" in path:
        if not path.endswith("/api_gen.rs"):
            return "language-generation", "existing shared generation/source/sidecar owner; no Web back-edge"
        generic = ("transpile", "post_process", "qualify", "code_region", "replace_in_code",
                   "borrow_", "parse_", "try_full_parse", "extract_api_lenient", "promote_api_types",
                   "primary_type_name", "extract_db_fn", "append_", "strip_collection", "balance_",
                   "utf8_len", "fix_borrowed", "auto_type_to", "rust_field_ident", "to_pascal_case",
                   "matching_paren", "split_top_level", "take_ascii_ident", "module_call_shape",
                   "fnv1a", "source_snapshot", "freshness", "bind_workspace_lock",
                   "generation_consumer_identity", "test_parse", "test_extract", "a2r_env_lock")
        if "tauri" in symbol:
            return "language-generation", "non-HTTP Tauri generator/control; retain single shared helpers"
        if any(part in symbol for part in generic):
            return "language-generation", "shared semantic generation helper or its control"
        if symbol in ("generate_api", "tests"):
            return "language-generation", "dispatch facade or test container; nested items have separate owners"
        return "web", "HTTP server/client generation, resources or corresponding control"
    if "/api/targets/axum.rs" in path or "/api/targets/typescript.rs" in path:
        return "web", "generated HTTP host/client implementation; language extraction stays below this owner"
    if path.endswith("/axum_adapter.rs"):
        return "language-adapter", "language extern/heap/extractor adapter; HTTP policy delegates through the host port"
    if "/http_server/server_model.rs" in path:
        if symbol in ("ApiRequest", "ApiBody", "ApiReply", "FileReplySeed") or symbol.startswith("<ApiRequest>"):
            return "web-contract", "owned request/reply contract; remove Axum body before independent consumption"
        return "language-adapter", "VM value conversion; no copied VM/AST/store"
    if "/http_server/server_argbind.rs" in path or "/http_server/server_dispatch.rs" in path:
        return "language-adapter", "VM task/value/argument execution; Web policy must be extracted via a narrow port"
    if "/http_server/server_route.rs" in path:
        if any(part in symbol for part in ("param_sig", "return_type", "return_shadow", "async_return", "ApiParamSig")):
            return "language-adapter", "compiler-published metadata, stored once and read through the host boundary"
        return "web", "HTTP route matching or its framework control"
    if "/http_server/server_scope.rs" in path:
        return "web", "request scope/lifetime policy; Core resource cleanup is an explicit callback"
    if path.endswith("/http_server.rs"):
        if any(part in symbol for part in ("API_PARAM_SIGS", "API_RETURN", "API_ASYNC", "fn_is_api_async")):
            return "language-adapter", "compiler metadata single source"
        if "spawn_outcome_reporter" in symbol or "panic_payload_text" in symbol or "plan755_outcome" in symbol:
            return "language", "shared diagnostic helper; actual 888 overlap, serial write window"
        return "web", "HTTP service composition/policy; VM execution calls go through the language adapter"
    if "/a2r-std/" in path:
        return "language", "existing standard-library facade/value/resource contract; framework executor moves to Web"
    if "/crates/auto-lang/src/tests/" in path:
        domain = ("plan729_http_server_file", "plan730_http_upload", "plan736_http_service", "back_proxy")
        if any(part in path for part in domain):
            return "web", "framework behavioral fixture/control; retain original assertion and source identity"
        return "language", "language/resource/client or cross-module regression; preserve consumer coverage"
    if "/crates/auto-lang-core/src/" in path:
        return "language", "language compiler/VM/stdlib or shared SDK; replace only concrete framework coupling"
    raise ValueError(f"source has no intake owner: {path}::{symbol}")


def obligation_owner(identity: str) -> str:
    namespace = identity.split("\t", 1)[-1]
    if namespace.startswith(("http_file_service::", "http_upload_service::", "http_service_config::",
                             "http_service_observability::", "vm::ffi::http_transport::")):
        return "web"
    if namespace.startswith("vm::ffi::http_server::"):
        if any(part in namespace for part in ("plan755_outcome", "nv_to_json")):
            return "language"
        return "web"
    if any(part in namespace for part in ("plan729_http_server_file", "plan730_http_upload",
                                         "plan736_http_service", "back_proxy_tests")):
        return "web"
    return "language"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--items", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    source = json.loads(args.items.read_text(encoding="utf-8-sig"))
    rows = []
    macros = []
    for file in source["files"]:
        path = Path(file["path"])
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        for ordinal, item in enumerate(file["items"]):
            owner, reason = source_owner(file["path"], item["symbol"])
            row = dict(item, source=file["path"], source_sha256=digest, ordinal=ordinal,
                       target_owner=owner, reason=reason, status="intake-not-migrated")
            rows.append(row)
            if item["kind"] == "macro_unexpanded":
                macros.append({"source": file["path"], "symbol": item["symbol"], "owner": owner,
                               "boundary": "preserve macro input; expansion is not runtime case discovery"})
    if not rows:
        raise ValueError("zero source items")
    obligations = []
    for tier in ("language", "tt", "tb", "taa", "ta", "t3"):
        manifest = args.root / "scripts" / f"full-regression-{tier}-ids.txt"
        lines = [line for line in manifest.read_text(encoding="utf-8-sig").splitlines()
                 if line.strip() and not line.startswith("#")]
        if not lines or len(set(lines)) != len(lines):
            raise ValueError(f"empty or duplicate identities in {tier}")
        digest = hashlib.sha256(manifest.read_bytes()).hexdigest()
        for identity in lines:
            if "\t" not in identity:
                raise ValueError(f"malformed identity in {tier}: {identity}")
            obligations.append(dict(tier=tier, identity=identity, target_owner=obligation_owner(identity),
                                    original_manifest_sha256=digest, executed=False,
                                    status="historical-obligation-not-current-runtime-enumeration"))
    payload = dict(plan="PLAN-890", revision=2, scope="frozen-intake-map-not-delivery-or-test-pass",
                   source_items=rows, macro_boundaries=macros, historical_obligations=obligations,
                   source_counts=dict(Counter(row["target_owner"] for row in rows)),
                   obligation_counts=dict(Counter(row["target_owner"] for row in obligations)),
                   unknown_owner=0, executed=0)
    args.out.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"source_items": len(rows), "source_counts": payload["source_counts"],
                      "historical_tier_rows": len(obligations), "obligation_counts": payload["obligation_counts"],
                      "macro_boundaries": len(macros), "unknown_owner": 0, "executed": 0}))


if __name__ == "__main__":
    main()

# nav-handler-routing (canary, GREEN)

Validates that `nav("/path")` in an `on`-block handler body generates
`router.push("/path")` in the Vue output, enabling multi-page navigation.

Status: **GREEN** — `auto build` + `vue-tsc` pass. Handler generates:
```js
function GoDetail(): void {
  router.push('/detail');
  emit('GoDetail')
}
```

Key changes:
- `token.rs`: Removed `nav` from keyword list (lexed as Ident, not TokenKind::Nav)
- `ts_adapter.rs`: `Stmt::Expr(Expr::NavCall)` → `router.push(path)` at statement level
- `ts_adapter.rs`: `stmts_have_router_nav` detects NavCall → triggers `useRouter` import

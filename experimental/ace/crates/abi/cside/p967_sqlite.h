// p967_sqlite.h — PLAN-967 T-15 frozen REAL-LIBRARY C provider header.
// The annotations ARE the import metadata; the `dep` line pins the
// SQLite amalgamation by CONTENT (the staged sqlite3.h must declare
// exactly this version — anything else is a named rejection).
// auto-ffi-c provider p967_sqlite/1 target=x86_64-pc-windows-msvc
// auto-ffi-c dep sqlite =3.46.0
#ifndef P967_SQLITE_H
#define P967_SQLITE_H
#include <stdint.h>
// Open (or create) the provider's single connection. `path` is a view
// (":memory:" or a file path). Embedded NUL -> named `nul-path`;
// sqlite open failure -> named `open-failed`. `out` = 1 on success.
// auto-ffi-c fn p967_open form=fallible errors=open-failed:-1,nul-path:-3,too-long:-4
extern int32_t p967_open(const uint8_t* path, uint64_t len, int32_t* out);
// Run SQL text against the open connection. `out` = sqlite3_changes.
// Not open -> `not-open`; sqlite error -> `sql-error`.
// auto-ffi-c fn p967_exec form=fallible errors=not-open:-1,sql-error:-2
extern int32_t p967_exec(const uint8_t* sql, uint64_t len, int32_t* out);
// Prepare+step one query; `out` = column 0 of the first row as int.
// auto-ffi-c fn p967_query_i32 form=fallible errors=not-open:-1,sql-error:-2
extern int32_t p967_query_i32(const uint8_t* sql, uint64_t len, int32_t* out);
// Close the connection (the ack view is the frame; its bytes are
// ignored). Double close -> `not-open`; sqlite close error -> named.
// auto-ffi-c fn p967_close form=fallible errors=not-open:-1,close-error:-2
extern int32_t p967_close(const uint8_t* ack, uint64_t len, int32_t* out);
#endif

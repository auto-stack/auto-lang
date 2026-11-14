// p967_sqlite.c — PLAN-967 T-15 frozen REAL-LIBRARY C provider
// implementation. The pinned amalgamation (sqlite3.c/sqlite3.h staged
// beside this file by the build helper) is the REAL SQLite; this file
// only adapts the sqlite3 open/exec/prepare/step/close line onto the
// frozen view frames. No CRT headers — the object build stays inside
// the freestanding include domain (byte loops, not memcpy/memchr).
#include "p967_sqlite.h"
#include "sqlite3.h"

static sqlite3* g_db = 0;

static int p967_view_has_nul(const uint8_t* p, uint64_t n) {
    for (uint64_t i = 0; i < n; i++) {
        if (p[i] == 0) {
            return 1;
        }
    }
    return 0;
}

static void p967_copy(char* d, const uint8_t* s, uint64_t n) {
    for (uint64_t i = 0; i < n; i++) {
        d[i] = (char)s[i];
    }
}

int32_t p967_open(const uint8_t* path, uint64_t len, int32_t* out) {
    char buf[1024];
    *out = 0;
    if (len >= sizeof(buf)) {
        return -4; /* too-long */
    }
    if (p967_view_has_nul(path, len)) {
        return -3; /* nul-path: the C-string boundary does not accept
                      embedded NUL — a NAMED rejection, never a silent
                      truncation (V-NATIVE) */
    }
    p967_copy(buf, path, len);
    buf[len] = 0;
    if (g_db) {
        sqlite3_close(g_db);
        g_db = 0;
    }
    if (sqlite3_open_v2(buf, &g_db, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE, 0) != SQLITE_OK) {
        if (g_db) {
            sqlite3_close(g_db);
            g_db = 0;
        }
        return -1; /* open-failed */
    }
    *out = 1;
    return 0;
}

int32_t p967_exec(const uint8_t* sql, uint64_t len, int32_t* out) {
    char buf[4096];
    *out = 0;
    if (!g_db) {
        return -1; /* not-open */
    }
    if (len >= sizeof(buf)) {
        return -2; /* sql-error: the statement cannot be framed */
    }
    p967_copy(buf, sql, len);
    buf[len] = 0;
    char* err = 0;
    int rc = sqlite3_exec(g_db, buf, 0, 0, &err);
    if (err) {
        sqlite3_free(err);
    }
    if (rc != SQLITE_OK) {
        return -2; /* sql-error */
    }
    *out = sqlite3_changes(g_db);
    return 0;
}

int32_t p967_query_i32(const uint8_t* sql, uint64_t len, int32_t* out) {
    char buf[4096];
    *out = 0;
    if (!g_db) {
        return -1; /* not-open */
    }
    if (len >= sizeof(buf)) {
        return -2; /* sql-error */
    }
    p967_copy(buf, sql, len);
    buf[len] = 0;
    sqlite3_stmt* st = 0;
    if (sqlite3_prepare_v2(g_db, buf, -1, &st, 0) != SQLITE_OK || st == 0) {
        return -2; /* sql-error */
    }
    int v = 0;
    if (sqlite3_step(st) == SQLITE_ROW) {
        v = sqlite3_column_int(st, 0);
    }
    sqlite3_finalize(st);
    *out = v;
    return 0;
}

int32_t p967_close(const uint8_t* ack, uint64_t len, int32_t* out) {
    (void)ack;
    (void)len;
    *out = 0;
    if (!g_db) {
        return -1; /* not-open */
    }
    if (sqlite3_close(g_db) != SQLITE_OK) {
        g_db = 0;
        return -2; /* close-error */
    }
    g_db = 0;
    *out = 1;
    return 0;
}

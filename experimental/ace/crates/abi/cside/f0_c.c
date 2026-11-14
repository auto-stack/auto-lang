// f0_c.c — PLAN-967 T-10 frozen C provider implementation.
#include "f0_c.h"
#include <stdint.h>
int32_t f0c_scaled_len(const uint8_t* data, uint64_t len) {
    (void)data;
    return (int32_t)(len * 3);
}
int32_t f0c_point_sum(const f0c_point* p) { return p->x + p->y; }
int32_t f0c_status_of(const uint8_t* data, uint64_t len, int32_t* out) {
    if (len == 0) { return F0C_EMPTY; }
    *out = (int32_t)data[0];
    return F0C_OK;
}
int32_t f0c_constants_probe(void) { return F0C_MAX; }
// The handle VALUE is a small i32-domain id (the frozen handle lane is
// i64 with checked i64<->i32 conversion — real 64-bit pointers would
// trip the wide-range transport; the provider encodes slot ids).
struct f0c_counter { int32_t value; int32_t live; };
static struct f0c_counter g_slots[2] = { { 0, 0 }, { 0, 0 } };
static f0c_counter* slot(int32_t id) {
    if (id < 1 || id > 2) { return 0; }
    return &g_slots[id - 1];
}
f0c_counter* f0c_counter_create(int32_t initial) {
    for (int32_t i = 0; i < 2; i++) {
        if (!g_slots[i].live) {
            g_slots[i].value = initial;
            g_slots[i].live = 1;
            return (f0c_counter*)(intptr_t)(i + 1);
        }
    }
    return 0;
}
int32_t f0c_counter_get(const f0c_counter* c, int32_t* out) {
    f0c_counter* s = slot((int32_t)(intptr_t)c);
    if (!s || !s->live) { return -1; }
    *out = s->value;
    return 0;
}
int32_t f0c_counter_destroy(f0c_counter* c, int32_t* out) {
    f0c_counter* s = slot((int32_t)(intptr_t)c);
    if (!s || !s->live) { return -1; }
    s->live = 0;
    s->value = -1;
    *out = 0;
    return 0;
}
int32_t f0c_apply(f0c_binop op, long long ctx, int32_t a, int32_t b, int32_t* out) {
    if (!op) { return -1; }
    long long r1 = op(ctx, a);
    long long r2 = op(ctx, b);
    /* stage markers: 1000=body ran, r1/r2 raw */
    if (r1 == 999999) { *out = 1000; return 0; }
    *out = (int32_t)(r1 + r2);
    return 0;
}
int32_t f0c_fold(f0c_binop cb, long long ctx_seed, int32_t n, int32_t* out) {
    if (!cb || n < 0 || n > 64) { return -2; }
    long long sum = 0;
    for (int32_t i = 0; i < n; i++) {
        long long r = cb(ctx_seed, i);
        if (r < -2147483648LL || r > 2147483647LL) { return -2; }
        sum += r;
        if (sum < -2147483648LL || sum > 2147483647LL) { return -2; }
    }
    *out = (int32_t)sum;
    return 0;
}

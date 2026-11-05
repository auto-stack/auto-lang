#include "source_foreign938.h"
#include <limits.h>
#ifndef BODY_BIAS
#define BODY_BIAS 0
#endif
static uint32_t trace_value = 0;
static int32_t call_count = 0;
unsigned char provider_ctx = 0x5a;
static int32_t bits_to_i32(uint32_t value) {
    union { uint32_t bits; int32_t signed_value; } result;
    result.bits = value;
    return result.signed_value;
}
int32_t provider_add(int32_t a, int32_t b) {
    ++call_count;
    return bits_to_i32((uint32_t)a + (uint32_t)b + (uint32_t)BODY_BIAS);
}
uint8_t provider_not(uint8_t value) {
    ++call_count;
    return value ? 0 : 1;
}
int32_t provider_div(void *ctx, int32_t a, int32_t b, int32_t *out) {
    ++call_count;
    if (ctx != &provider_ctx) return -3;
    if (b == 0) return -1;
    if (a == INT32_MIN && b == -1) return -2;
    *out = a / b;
    return 0;
}
int32_t provider_tick(int32_t value) {
    ++call_count;
    trace_value = trace_value * 10u + (uint32_t)value;
    return value;
}
int32_t provider_trace(void) { ++call_count; return bits_to_i32(trace_value); }
int32_t provider_calls(void) { ++call_count; return call_count; }
int32_t provider_mix(int32_t a, uint8_t b, int32_t c, uint8_t d, int32_t e) {
    ++call_count;
    return bits_to_i32((uint32_t)a + 10u * b + 100u * (uint32_t)c + 1000u * d + 10000u * (uint32_t)e);
}
void provider_reset(void) { trace_value = 0; call_count = 0; }

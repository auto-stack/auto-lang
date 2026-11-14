// f0_c.c — PLAN-967 T-10 frozen C provider implementation.
#include "f0_c.h"
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

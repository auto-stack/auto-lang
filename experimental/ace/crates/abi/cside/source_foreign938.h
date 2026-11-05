#ifndef PROVIDER_938_H
#define PROVIDER_938_H
#include <stdint.h>
int32_t provider_add(int32_t a, int32_t b);
uint8_t provider_not(uint8_t value);
int32_t provider_div(void *ctx, int32_t a, int32_t b, int32_t *out);
int32_t provider_tick(int32_t value);
int32_t provider_trace(void);
int32_t provider_calls(void);
int32_t provider_mix(int32_t a, uint8_t b, int32_t c, uint8_t d, int32_t e);
void provider_reset(void);
extern unsigned char provider_ctx;
#endif

/* Independent C source/call site, never generated from Auto/HIR/MIR. The
   parent alone owns the expected result. This program returns observed data. */
#include "source_foreign938.h"
#include <stdio.h>
#ifndef SOURCE938_SCENARIO
#define SOURCE938_SCENARIO 0
#endif
int main(void) {
#if SOURCE938_SCENARIO == 0
    return provider_add(20,22);
#elif SOURCE938_SCENARIO == 1
    if (provider_not(0)) return 42;
    return 1;
#elif SOURCE938_SCENARIO == 2
    if (provider_not(1)) return 1;
    return 42;
#elif SOURCE938_SCENARIO >= 3 && SOURCE938_SCENARIO <= 5
    int32_t value; /* Intentionally uninitialized: never read on failure. */
#if SOURCE938_SCENARIO == 3
    int32_t status = provider_div(&provider_ctx,84,2,&value);
#elif SOURCE938_SCENARIO == 4
    int32_t status = provider_div(&provider_ctx,84,0,&value);
#else
    int32_t status = provider_div(&provider_ctx,INT32_MIN,-1,&value);
#endif
    if (status != 0) {
        fprintf(stderr,"foreign.status=%d\n",status);
        return 70;
    }
    return value;
#elif SOURCE938_SCENARIO == 6
    /* C argument order is unspecified. Match the declared Auto source order
       explicitly rather than borrowing its compiler's ordering by accident. */
    int32_t left = provider_tick(2);
    int32_t right = provider_tick(3);
    int32_t sum = provider_add(left,right);
    int32_t trace = provider_trace();
    int32_t calls = provider_calls();
    return sum*1000+trace*10+calls;
#elif SOURCE938_SCENARIO == 7
    volatile int selected = 0;
    if (selected && provider_not(0)) return 99;
    return provider_calls();
#elif SOURCE938_SCENARIO == 8
    volatile int selected = 1;
    if (selected || provider_not(0)) return provider_calls();
    return 99;
#elif SOURCE938_SCENARIO == 9
    return provider_mix(1,1,2,0,3);
#elif SOURCE938_SCENARIO == 10
    int32_t right = provider_tick(3);
    int32_t left = provider_tick(2);
    int32_t sum = provider_add(left,right);
    int32_t trace = provider_trace();
    return sum+trace*10;
#else
#error unreviewed source certification scenario
#endif
}

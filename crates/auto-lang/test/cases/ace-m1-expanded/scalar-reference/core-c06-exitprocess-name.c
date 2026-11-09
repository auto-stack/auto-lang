/* PLAN-941 independently authored C11 scalar reference; no generated code.
 * int64 arithmetic gives a separate implementation of checked i32 operations. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <limits.h>
void r_trap(const char *name){printf("trap:%s\n",name);exit(0);}
int32_t r_checked(int64_t value){if(value<INT32_MIN||value>INT32_MAX){r_trap("Overflow");}return (int32_t)value;}
int32_t r_add(int32_t a,int32_t b){return r_checked((int64_t)a+(int64_t)b);}
int32_t r_sub(int32_t a,int32_t b){return r_checked((int64_t)a-(int64_t)b);}
int32_t r_mul(int32_t a,int32_t b){return r_checked((int64_t)a*(int64_t)b);}
int32_t r_div(int32_t a,int32_t b){if(b==0){r_trap("DivByZero");}if(a==INT32_MIN&&b==-1){r_trap("DivOverflow");}return a/b;}
int32_t r_rem(int32_t a,int32_t b){if(b==0){r_trap("RemByZero");}if(a==INT32_MIN&&b==-1){r_trap("RemOverflow");}return a%b;}
int32_t r_i(int32_t x){return x;}
int32_t r_min(void){return INT32_MIN;}int32_t r_max(void){return INT32_MAX;}
int32_t trace_a(void){fputs("mark_a\n",stderr);return 1;}int32_t trace_b(void){fputs("mark_b\n",stderr);return 2;}
int32_t user_exit_process(void){return 4;} int32_t calc(void){return user_exit_process();}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%" PRId32 "\n",calc());return 0;}

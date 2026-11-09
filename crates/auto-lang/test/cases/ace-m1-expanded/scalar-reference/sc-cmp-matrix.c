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
int32_t calc(void){int32_t a=r_sub(0,7);
if(r_div(a,3)!=-2){return 1;}if(r_rem(a,3)!=-1){return 2;}
if(r_div(7,-3)!=-2){return 3;}if(r_rem(7,-3)!=1){return 4;}
if(r_div(-7,-3)!=2){return 5;}if(r_rem(-7,-3)!=-1){return 6;}
if(!(r_min()<r_max())){return 7;}if(r_min()>=r_max()){return 8;}
if(!(r_min()<=r_min())){return 9;}if(!(r_max()>r_min())){return 10;}
bool t=true,f=false;if(t==f){return 11;}if(!(t!=f)){return 12;}
if(!(t==true&&f==false)){return 13;}if(r_i(1)==r_i(2)||r_i(2)!=r_i(2)){return 14;}
if(!(r_add(1,r_mul(2,3))==7)){return 15;}
if(!(r_add(2,3)<6&&r_i(4)>r_i(2))){return 16;}
if(f||t&&f){return 17;}if(!((r_i(1)<r_i(2))==t)){return 18;}
int32_t neg=r_sub(0,r_add(a,3));if(neg!=4){return 19;}
int32_t nn=r_sub(0,-5);if(nn!=5){return 20;}
if((!(!t))!=t){return 21;}if(!(r_rem(10,4)==2&&r_div(9,2)==4)){return 22;}return 0;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%" PRId32 "\n",calc());return 0;}

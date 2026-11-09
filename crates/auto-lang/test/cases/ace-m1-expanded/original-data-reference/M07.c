/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int n;} R;
static int down(int k){if(k<=0){return 42;}return down(k-1);}
static int vdown(const R *r,int k){if(k<=0){return r->n;}return vdown(r,k-1);}
static int calc(void){R r={42};return down(9)+vdown(&r,9);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

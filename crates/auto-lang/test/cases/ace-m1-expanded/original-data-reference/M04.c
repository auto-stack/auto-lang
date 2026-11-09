/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int v;} P;
static int mixed(const P *ro,P *a,int k,P *b){a->v+=k;b->v+=2*k;return ro->v*100+a->v*10+b->v;}
static int calc(void){P ro={1},a={2},b={3};int value=mixed(&ro,&a,1,&b);if(ro.v*100+a.v*10+b.v-value!=0){return 60;}if(ro.v!=1){return 70+ro.v;}return value;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

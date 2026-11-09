/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int v;} P;
static int mix(const P *self,P *q,int d){q->v+=d;return self->v*10+q->v;}
static int calc(void){P p={7},q={2};int value=mix(&p,&q,3);if(p.v!=7){return 90+p.v;}if(q.v!=5){return 80+q.v;}return value;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

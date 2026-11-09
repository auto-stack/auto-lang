/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int value;} Inner;typedef struct{Inner inner;int extra;} Outer;
static int calc(void){Outer p={{3},2};p.inner.value=7;return p.inner.value+p.extra;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

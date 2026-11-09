/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int left,right;} Pair;
static int calc(void){Pair p={1,2};Pair replacement={p.right,p.left};p=replacement;return p.left*10+p.right;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

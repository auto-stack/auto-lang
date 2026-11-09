/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int pick_i32(int n,int seed){if(n<=0)return seed;return pick_i32(n-1,seed);}
int value(void){return pick_i32(2,40);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

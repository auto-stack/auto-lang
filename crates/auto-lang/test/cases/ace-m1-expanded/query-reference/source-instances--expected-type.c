/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int make_i32(void){return 0;}
int value(void){int x=make_i32();return x+42;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

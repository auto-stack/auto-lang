/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int choose_i32(int a,int b,int flag){if(flag)return a;return b;}
int choose_bool(int a,int b,int flag){if(flag)return a;return b;}
int value(void){if(choose_bool(1,0,1))return choose_i32(40,42,0);return 0;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

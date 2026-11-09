/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int choose_i32(int a,int b,int flag){if(flag)return a;return b;}
int choose_bool(int a,int b,int flag){if(flag)return a;return b;}
int wrap_i32(int a,int b){return choose_i32(a,b,1);}
int wrap_bool(int a,int b){return choose_bool(a,b,1);}
int value(void){if(wrap_bool(0,1))return 1;return wrap_i32(40,42);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

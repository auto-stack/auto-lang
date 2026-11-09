/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int pick(int a,int b){if(b)return a;return 0;}
int value(void){int b=1;int a=7;return pick(a,b);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

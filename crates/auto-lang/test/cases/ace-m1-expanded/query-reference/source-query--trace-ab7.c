/* Independent PLAN-941 query/instance reference. */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>
int mark_a(void){fputs("mark_a\n",stderr);return 1;}
int mark_b(void){fputs("mark_b\n",stderr);return 2;}
int sub(int a,int b){return a*3+b*2;}
int value(void){int x=mark_a();int b=mark_b();int a=x;return sub(a,b);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);printf("value:%d\n",value());return 0;}

/* Source evaluation order is b then a; bind computed values by parameter name. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
static int mark_b(void){fputs("mark_b\n",stderr);return 2;}
static int mark_a(void){fputs("mark_a\n",stderr);return 1;}
static int f(int a,int b){return a*10+b;}
int main(void){
    _setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);
    int b=mark_b()+1;int a=mark_a()*2;
    printf("value:%d\n",f(a,b));return 0;
}

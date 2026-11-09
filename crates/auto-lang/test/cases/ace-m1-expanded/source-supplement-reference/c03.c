/* Independent original call graph, using C functions. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
static int leaf(int value){return value*3;}
static int mid(int value){return leaf(value)+1;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",mid(20));return 0;}

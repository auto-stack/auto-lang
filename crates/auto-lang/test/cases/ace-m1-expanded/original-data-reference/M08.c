/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int value;} Counter;
static Counter counter_new(int value){Counter c={value};return c;}
static int counter_read(const Counter *c){return c->value;}
static int calc(void){Counter x=counter_new(42);return counter_read(&x);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

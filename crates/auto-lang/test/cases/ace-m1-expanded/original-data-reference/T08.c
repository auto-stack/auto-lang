/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
static int scalar_len(const char *text){int count=0;const unsigned char *p=(const unsigned char*)text;while(*p){if((*p&0xc0)!=0x80){++count;}++p;}return count;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);const char *text="\xe4\xb8\xad";volatile int start=0,end=2;
if(start<0 || start>end || end>scalar_len(text)){puts("trap:TextIndex");return 0;}printf("value:%d\n",end-start);return 0;}

/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
static int scalar_len(const char *text){int count=0;const unsigned char *p=(const unsigned char*)text;while(*p){if((*p&0xc0)!=0x80){++count;}++p;}return count;}
static int calc(void){const char *text="A\xe4\xb8\xad\xf0\x9f\x98\x80";const char *selected=text+1;return scalar_len(selected)*10+(int)strlen(selected);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

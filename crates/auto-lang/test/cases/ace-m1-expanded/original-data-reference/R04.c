/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
static int scalar_len(const char *text){int count=0;const unsigned char *p=(const unsigned char*)text;while(*p){if((*p&0xc0)!=0x80){++count;}++p;}return count;}
typedef struct{char name[16];int code;} Item;
static int calc(void){Item a={"hello",1};Item b=a;memcpy(a.name,"x",2);return scalar_len(b.name)*10+scalar_len(a.name);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

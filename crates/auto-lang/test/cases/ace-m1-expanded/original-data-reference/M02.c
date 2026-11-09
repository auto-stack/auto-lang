/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
static int scalar_len(const char *text){int count=0;const unsigned char *p=(const unsigned char*)text;while(*p){if((*p&0xc0)!=0x80){++count;}++p;}return count;}
typedef struct{char text[16];int kind,line,at,len;} Token;
static Token ident(const char *text,int line,int at){Token t={"",1,0,0,0};memcpy(t.text,text,strlen(text)+1);t.line=line;t.at=at;t.len=scalar_len(text);return t;}
static int weigh(Token t){return t.len+t.line+t.at;}
static int calc(void){Token a=ident("keyword",3,12);if(a.kind!=1){return 100;}if(a.len!=7){return 101;}Token b=a;b.at=40;if(a.at!=12){return 102;}return weigh(b)+scalar_len(a.text);}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

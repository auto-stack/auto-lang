/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
static int calc(void){const unsigned char utf8[]={0xf0,0x9f,0x98,0x80};return (int)(((utf8[0]&7u)<<18)|((utf8[1]&63u)<<12)|((utf8[2]&63u)<<6)|(utf8[3]&63u));}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}

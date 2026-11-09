/* Independent PLAN-941 C11 source reference, no generated headers. */
#include <stdint.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
void binary(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);}
size_t scalars(const unsigned char *s,size_t n){size_t count=0;for(size_t i=0;i<n;++i){if((s[i]&0xc0u)!=0x80u){++count;}}return count;}
int main(void){binary();const unsigned char a[]={97,98},tail[]={0xe4,0xb8,0xad};unsigned char b[2],c[5];memcpy(b,a,2);memcpy(c,b,2);memcpy(c+2,tail,3);printf("value:%zu\n",scalars(a,2)*10+scalars(c,5));return 0;}

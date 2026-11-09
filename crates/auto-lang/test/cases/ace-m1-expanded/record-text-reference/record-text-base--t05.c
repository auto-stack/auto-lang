/* Independent PLAN-941 C11 source reference, no generated headers. */
#include <stdint.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
void binary(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);}
int main(void){binary();const unsigned char a[]={65,0,66},one[]={65},other[]={65,0,66};size_t n=sizeof a,m=sizeof one,k=sizeof other;
if(n==m&&memcmp(a,one,m)==0){printf("value:1\n");return 0;}if(n!=k||memcmp(a,other,n)!=0){printf("value:2\n");return 0;}printf("value:42\n");return 0;}

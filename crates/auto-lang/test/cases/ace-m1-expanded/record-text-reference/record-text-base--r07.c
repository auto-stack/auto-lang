/* Independent PLAN-941 C11 source reference, no generated headers. */
#include <stdint.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
void binary(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);}
typedef struct{int32_t value;} Counter;int32_t bump(Counter *x,int32_t d){x->value+=d;return x->value;}
int main(void){binary();Counter x={20};int32_t now=bump(&x,1);printf("value:%" PRId32 "\n",now+x.value);return 0;}

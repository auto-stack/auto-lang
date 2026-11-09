/* Independent PLAN-941 C11 source reference, no generated headers. */
#include <stdint.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <io.h>
void binary(void){_setmode(_fileno(stdout),_O_BINARY);_setmode(_fileno(stderr),_O_BINARY);}
typedef struct{int32_t value;} Counter;Counter counter_new(int32_t n){Counter c={n};return c;}int32_t counter_read(const Counter *c){return c->value;}
int main(void){binary();Counter x=counter_new(42);printf("value:%" PRId32 "\n",counter_read(&x));return 0;}

/* Independent wide division avoids undefined signed i32 overflow. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
int main(void){
    _setmode(_fileno(stdout),_O_BINARY);
    volatile int64_t dividend=INT32_MIN, divisor=-1;
    int64_t quotient=dividend/divisor;
    if(quotient>INT32_MAX || quotient<INT32_MIN){puts("trap:DivOverflow");}
    else{printf("value:%" PRId64 "\n",quotient);}
    return 0;
}

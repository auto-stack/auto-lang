#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    int32_t snap = 20;      /* copy taken at evaluation point */
    int32_t now = 21;        /* the callee writes back 20+1 */
    printf("value:%d\n", snap * 10 + now);
    return 0;
}

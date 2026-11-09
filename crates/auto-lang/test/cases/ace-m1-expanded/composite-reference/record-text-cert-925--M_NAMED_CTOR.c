#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    int32_t a = 1; int32_t b = 2; /* by name */
    printf("value:%d\n", a * 10 + b);
    return 0;
}

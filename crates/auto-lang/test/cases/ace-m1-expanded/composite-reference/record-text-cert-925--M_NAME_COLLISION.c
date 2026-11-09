#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    int32_t c = 1, g = 2;
    printf("value:%d\n", c * 100 + (g + 1) * 10 + 7);
    return 0;
}

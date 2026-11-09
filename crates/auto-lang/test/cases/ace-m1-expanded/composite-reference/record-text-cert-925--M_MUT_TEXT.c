#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", 4); /* "ab"+"ab" = 4 chars */
    return 0;
}

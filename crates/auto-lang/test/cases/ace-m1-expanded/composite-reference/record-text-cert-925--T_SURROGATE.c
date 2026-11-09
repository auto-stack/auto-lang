#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    uint32_t cp = 55296u;
    if (cp >= 0xD800u && cp <= 0xDFFFu) { printf("trap:InvalidScalar\n"); return 0; }
    printf("value:99\n");
    return 0;
}

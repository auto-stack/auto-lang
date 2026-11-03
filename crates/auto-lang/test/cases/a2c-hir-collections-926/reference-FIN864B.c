/* reference-FIN864B.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

static int32_t scalars(const char *s) {
    int32_t n = 0;
    for (const unsigned char *p = (const unsigned char *)s; *p; p++)
        if ((*p & 0xC0) != 0x80) n++;
    return n;
}

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    /* ("ab",1) then ("cde",2): len*10 + code each */
    printf("value:%d\n", scalars("ab") * 10 + 1 + scalars("cde") * 10 + 2);
    return 0;
}

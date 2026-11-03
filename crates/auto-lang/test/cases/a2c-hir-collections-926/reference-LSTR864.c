/* reference-LSTR864.c - hand-written std-only reference (PLAN-926); an
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
        if ((*p & 0xC0) != 0x80) n++;          /* count non-continuation bytes */
    return n;
}

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    /* removed "hello"; get(0) = "中" */
    printf("value:%d\n", scalars("hello") * 10 + scalars("\xe4\xb8\xad"));
    return 0;
}

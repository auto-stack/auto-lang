/* reference-V05.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

static int32_t name_len(const char *s) { int n = 0; while (*s++) n++; return n; }

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    /* Token.Data(Item("hello", 37)) */
    printf("value:%d\n", name_len("hello") + 37);
    return 0;
}

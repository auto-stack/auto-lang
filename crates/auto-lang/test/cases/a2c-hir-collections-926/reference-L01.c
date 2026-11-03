/* reference-L01.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int xs[2];
    _setmode(_fileno(stdout), _O_BINARY);
    xs[0] = 20;
    xs[1] = 22;
    printf("value:%d\n", xs[0] + xs[1]);
    return 0;
}

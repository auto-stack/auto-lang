/* reference-L06.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    /* xs has one element; get(1) is out of bounds */
    printf("trap:ListIndex\n");
    return 0;
}

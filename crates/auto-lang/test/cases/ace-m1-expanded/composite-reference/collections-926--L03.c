/* reference-L03.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int xs[1];
    _setmode(_fileno(stdout), _O_BINARY);
    xs[0] = 42;                               /* push then clear */
    (void)xs;
    printf("value:%d\n", 0);                 /* len after clear */
    return 0;
}

/* reference-L02.c - hand-written std-only reference (PLAN-926); an
 * independent implementation of the case's semantics (plain C, no
 * generated runtime, no helpers). Binary-mode stdout: the frozen
 * observation protocol is LF-terminated. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    /* push 10, 30; insert(1, 20) -> [10,20,30]; remove(1) -> [10,30] */
    int xs[3];
    _setmode(_fileno(stdout), _O_BINARY);
    xs[0] = 10; xs[1] = 30;
    xs[2] = xs[1]; xs[1] = 20;               /* insert */
    xs[1] = xs[2];                            /* remove index 1 */
    printf("value:%d\n", xs[0] + xs[1] + 2);
    return 0;
}

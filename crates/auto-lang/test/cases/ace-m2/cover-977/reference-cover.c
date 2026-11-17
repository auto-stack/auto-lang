/* PLAN-977 cover-977 A2C certify leg: the HAND-WRITTEN independent
 * reference for the public source `fn main() int { return 2 + 3 * 4 }`
 * (expected value 14, hand-computed). Compiled and run separately from
 * the emitted program; both must print `value:14\n` (hex
 * 76616c75653a31340a). The value is recomputed here in C (2 + 3 * 4),
 * never generated from the transpiler output; stdout is binary-mode so
 * the newline is a single 0x0a byte on Windows (the same discipline as
 * the PLAN-941 reference-Q4-001.c). */
#include <stdio.h>
#include <fcntl.h>
#include <io.h>

int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", 2 + 3 * 4);
    return 0;
}

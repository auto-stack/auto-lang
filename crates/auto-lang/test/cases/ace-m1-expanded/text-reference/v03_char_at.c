/* Independent PLAN-941 reference; no emitted headers/runtime/source parser.
 * All byte arrays here are finite, valid UTF-8 data. Length includes NUL. */
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
static void value(int n) { printf("value:%d\n", n); }
static void trap(const char *name) { printf("trap:%s\n", name); }
static int scalars(const unsigned char *s, size_t n) {
    int count = 0;
    for (size_t i = 0; i < n; ++i) if ((s[i] & 0xc0u) != 0x80u) ++count;
    return count;
}
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    const uint32_t s[] = {104,233,108,108,111}; int index = 1; if (index < 0 || index >= 5) trap("TextIndex"); else value((int)s[index]);
    return 0;
}

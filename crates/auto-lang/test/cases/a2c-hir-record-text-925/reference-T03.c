#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
/* hand-written certification reference (PLAN-925 T-08/T-09): value:23
 * a="ab"; b=a.clone(); c=b.move; c=c+"中" -> a.len()*10 + c.len()
 * len counts Unicode SCALARS: a=2, c="ab中"=3 -> 2*10+3 = 23 */
static size_t scalar_len(const uint8_t* p, size_t n) {
    size_t s = 0;
    for (size_t i = 0; i < n; i++) {
        if ((p[i] & 0xC0u) != 0x80u) s++; /* count non-continuation bytes */
    }
    return s;
}
int main(void) {
    static const uint8_t c[] = { 'a', 'b', 0xE4, 0xB8, 0xAD }; /* "ab中" */
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", (int)(2 * 10 + (int)scalar_len(c, sizeof c)));
    return 0;
}

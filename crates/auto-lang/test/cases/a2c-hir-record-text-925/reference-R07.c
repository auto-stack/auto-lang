#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
/* hand-written certification reference (PLAN-925 T-08): value:42 */
static int bump(int32_t* x, int32_t delta) {
    x[0] = x[0] + delta;
    return x[0];
}
int main(void) {
    int32_t x = 20;
    int32_t now = bump(&x, 1);
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", now + x);
    return 0;
}

#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
/* hand-written certification reference (PLAN-925 T-08): value:42
 * static ctor + view read + owned Str len: 40 + 1 + 1 */
static int32_t counter_new(int32_t n) { return n; }
static int32_t counter_read(const int32_t* self) { return self[0]; }
int main(void) {
    int32_t c = counter_new(40);
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", counter_read(&c) + 1 + 1);
    return 0;
}

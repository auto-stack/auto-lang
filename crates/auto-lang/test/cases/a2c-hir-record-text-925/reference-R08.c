#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
/* hand-written certification reference (PLAN-925 T-08): value:42 */
static int32_t read_value(const int32_t* self) { return self[0]; }
int main(void) {
    int32_t c = 42;
    _setmode(_fileno(stdout), _O_BINARY);
    printf("value:%d\n", read_value(&c));
    return 0;
}

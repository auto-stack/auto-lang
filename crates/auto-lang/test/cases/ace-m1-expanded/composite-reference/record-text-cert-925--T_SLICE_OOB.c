#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    _setmode(_fileno(stdout), _O_BINARY);
    int64_t n = 5; int64_t end = 6; /* h é l l o */
    if (end > n) { printf("trap:TextIndex\n"); return 0; }
    printf("value:99\n");
    return 0;
}
